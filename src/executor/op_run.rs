use crate::cloudflare_client;
use crate::errors;
use crate::errors::ServerError;
use crate::executor::models::Executor;
use crate::executor::*;
use crate::models;
use crate::models::{Filter, Nongrata};
use crate::pool::DbConn;
use crate::schema;

use async_trait::async_trait;
use chrono::Utc;
use diesel::prelude::*;
use diesel::r2d2::Pool;
use diesel::r2d2::PooledConnection;
use tracing::info;

#[derive(Clone)]
pub struct ExecutorService {
    client: cloudflare_client::CloudflareClient,
    db_pool: Pool<DbConn>,
}

impl ExecutorService {
    pub fn new(client: cloudflare_client::CloudflareClient, db_pool: Pool<DbConn>) -> Self {
        Self { client, db_pool }
    }
}

#[async_trait]
impl Executor for ExecutorService {
    async fn ban(
        &self,
        block_request: BlockRequest,
        analyzer_id: String,
    ) -> Result<(), errors::ServerError> {
        info!("Incoming request:{:?}", block_request);

        let conn: PooledConnection<DbConn> = self
            .db_pool
            .get()
            .map_err(|e| ServerError::PoolError(e.to_string()))?;

        let rule = block_request.clone();
        let mut new_filter = Filter::new(rule.target.ip, rule.target.user_agent)?;

        let existing_filters = self.find_filter(new_filter.clone()).await?;
        let existing_filter = existing_filters.first();

        match existing_filter {
            Some(existing_filter) => {
                self.update_filter(
                    block_request.clone(),
                    existing_filter.clone(),
                    new_filter,
                    analyzer_id,
                )
                .await?;
            }
            None => {
                // Create CF and local filter
                let filter_id = self.create_filter(new_filter.clone()).await?;

                new_filter.id = filter_id;

                // Create CF and nongrata rule
                return self
                    .create_rule(block_request.clone(), new_filter, analyzer_id)
                    .await;
            }
        }

        Ok(())
    }

    async fn find_filter(&self, filter: Filter) -> Result<Vec<Filter>, ServerError> {
        let conn: PooledConnection<DbConn> = self
            .db_pool
            .get()
            .map_err(|e| ServerError::PoolError(e.to_string()))?;
        schema::filters::table
            .filter(schema::filters::filter_type.eq(&filter.filter_type))
            .select(schema::filters::table::all_columns())
            .load::<Filter>(&*conn)
            .map_err(ServerError::from)
    }

    async fn create_filter(&self, filter: Filter) -> Result<String, ServerError> {
        let conn: PooledConnection<DbConn> = self
            .db_pool
            .get()
            .map_err(|e| ServerError::PoolError(e.to_string()))?;
        // Create CF filter
        let filter_id = self
            .client
            .create_filter(filter.clone())
            .await
            .map_err(ServerError::from)?;

        // Create new local filter
        diesel::insert_into(schema::filters::table)
            .values(filter)
            .execute(&*conn)
            .map_err(ServerError::from)?;

        Ok(filter_id)
    }

    async fn create_rule(
        &self,
        block_request: BlockRequest,
        filter: Filter,
        analyzer_id: String,
    ) -> Result<(), ServerError> {
        let conn: PooledConnection<DbConn> = self
            .db_pool
            .get()
            .map_err(|e| ServerError::PoolError(e.to_string()))?;
        let restriction_type = models::RestrictionType::Block;

        // Create CF rule
        let rule_id = self
            .client
            .create_block_rule(filter.id.clone(), models::RestrictionType::Block)
            .await
            .map_err(ServerError::from)?;

        let nongrata = Nongrata::new(
            block_request.reason.clone(),
            filter.id.clone(),
            chrono::DateTime::<Utc>::from_utc(
                chrono::NaiveDateTime::from_timestamp(
                    block_request.ttl as i64 + chrono::offset::Utc::now().timestamp(),
                    0,
                ),
                Utc,
            ),
            restriction_type.to_string(),
            filter.expression,
            true,
            analyzer_id,
        );

        // Create new nongrata
        diesel::insert_into(schema::nongratas::table)
            .values(nongrata)
            .execute(&*conn)
            .map_err(ServerError::from)?;

        // Set filter CF rule id
        let target = schema::filters::table.filter(schema::filters::id.eq(&filter.id));
        diesel::update(target)
            .set(schema::filters::rule_id.eq(rule_id))
            .execute(&*conn)
            .map_err(ServerError::from)?;
        Ok(())
    }

    async fn update_filter(
        &self,
        block_request: BlockRequest,
        mut old_filter: Filter,
        new_filter: Filter,
        analyzer_id: String,
    ) -> Result<(), ServerError> {
        let conn: PooledConnection<DbConn> = self
            .db_pool
            .get()
            .map_err(|e| ServerError::PoolError(e.to_string()))?;
        let restriction_type = models::RestrictionType::Block;
        old_filter.append(new_filter.clone())?;

        // Update CF filter
        self.client
            .update_filter(old_filter.clone())
            .await
            .map_err(ServerError::from)?;

        // Update local filter expression
        let target = schema::filters::table.filter(schema::filters::id.eq(&old_filter.id));
        diesel::update(target)
            .set(schema::filters::expression.eq(old_filter.expression))
            .execute(&*conn)
            .map_err(ServerError::from)?;

        // Create new nongrata based on new_filter in order to be able to select further
        let nongrata = Nongrata::new(
            block_request.reason.clone(),
            old_filter.id,
            chrono::DateTime::<Utc>::from_utc(
                chrono::NaiveDateTime::from_timestamp(
                    block_request.ttl as i64 + chrono::offset::Utc::now().timestamp(),
                    0,
                ),
                Utc,
            ),
            restriction_type.to_string(),
            new_filter.expression,
            true,
            analyzer_id,
        );

        // Insert nongrata
        diesel::insert_into(schema::nongratas::table)
            .values(nongrata)
            .execute(&*conn)
            .map_err(ServerError::from)?;

        Ok(())
    }

    async fn unban(&self, unblock_request: UnblockRequest) -> Result<(), errors::ServerError> {
        info!("Incoming request:{:?}", unblock_request);

        let conn: PooledConnection<DbConn> = self
            .db_pool
            .get()
            .map_err(|e| ServerError::PoolError(e.to_string()))?;

        // Get rule ID by pattern matching
        let trim_filter =
            Filter::new(unblock_request.target.ip, unblock_request.target.user_agent)?;

        let expression_filter = format!("%({})%", trim_filter.expression);

        let mut filter = schema::filters::table
            .filter(schema::filters::expression.ilike(expression_filter))
            .select(schema::filters::all_columns)
            .first::<models::Filter>(&*conn)
            .map_err(ServerError::from)?;

        // then trim
        filter.trim_expression(trim_filter.clone());

        // then update CF
        self.client.update_filter(filter.clone()).await?;

        // and update local filter
        let target = schema::filters::table.filter(schema::filters::id.eq(&filter.id));
        diesel::update(target)
            .set(schema::filters::expression.eq(filter.clone().expression))
            .execute(&*conn)
            .map_err(ServerError::from)?;

        // then delete nongrata entry
        diesel::delete(
            schema::nongratas::table
                .filter(schema::nongratas::restriction_value.eq(trim_filter.clone().expression)),
        )
        .execute(&*conn)
        .map_err(ServerError::from)?;

        Ok(())
    }
}
