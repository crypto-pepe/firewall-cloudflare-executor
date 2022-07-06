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
use tracing::{info, warn};

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

        let _conn: PooledConnection<DbConn> = self
            .db_pool
            .get()
            .map_err(|e| ServerError::PoolError(e.to_string()))?;

        let rule = block_request.clone();
        let new_filter = &mut Filter::new(rule.target.ip, rule.target.user_agent)?;

        let existing_filter = self.find_filter(new_filter.clone()).await?;

        match existing_filter {
            Some(existing_filter) => {
                self.update_filter(
                    block_request.clone(),
                    existing_filter.clone(),
                    new_filter.clone(),
                    analyzer_id,
                )
                .await?;
            }
            None => {
                // Create CF and local filter
                self.create_filter(new_filter).await?;

                // Create CF and nongrata rule
                return self
                    .create_rule(block_request.clone(), new_filter.clone(), analyzer_id)
                    .await;
            }
        }

        Ok(())
    }

    async fn find_filter(&self, filter: Filter) -> Result<Option<Filter>, ServerError> {
        let conn: PooledConnection<DbConn> = self
            .db_pool
            .get()
            .map_err(|e| ServerError::PoolError(e.to_string()))?;
        schema::filters::table
            .filter(schema::filters::filter_type.eq(&filter.filter_type))
            .select(schema::filters::table::all_columns())
            .first::<Filter>(&*conn)
            .optional()
            .map_err(ServerError::from)
    }

    async fn create_filter(&self, filter: &mut Filter) -> Result<(), ServerError> {
        let conn: PooledConnection<DbConn> = self
            .db_pool
            .get()
            .map_err(|e| ServerError::PoolError(e.to_string()))?;
        // Create CF filter
        filter.id = self
            .client
            .create_filter(filter.clone())
            .await
            .map_err(ServerError::from)?;

        // Create new local filter
        diesel::insert_into(schema::filters::table)
            .values(filter.clone())
            .execute(&*conn)
            .map_err(ServerError::from)?;

        Ok(())
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
            filter.expression,
            restriction_type.to_string(),
            true,
            analyzer_id,
        );

        // Set filter CF rule id
        let target = schema::filters::table.filter(schema::filters::id.eq(&filter.id));
        diesel::update(target)
            .set(schema::filters::rule_id.eq(rule_id))
            .execute(&*conn)
            .map_err(ServerError::from)?;

        // Create new nongrata
        diesel::insert_into(schema::nongratas::table)
            .values(nongrata)
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
        warn!("{} {}", old_filter.expression, new_filter.expression);
        if old_filter.already_includes_filter(new_filter.clone())? {
            // find existing nongrata
            let existing_nongrata = schema::nongratas::table
                .filter(schema::nongratas::restriction_value.ilike(new_filter.expression))
                .first::<models::Nongrata>(&*conn)
                .map_err(ServerError::from)?;

            let target = schema::nongratas::table
                .filter(schema::nongratas::id.eq(existing_nongrata.id.unwrap()));

            // update entry
            diesel::update(target)
                .set((
                    schema::nongratas::reason.eq(block_request.clone().reason),
                    schema::nongratas::analyzer_id.eq(analyzer_id),
                    schema::nongratas::expires_at.eq(chrono::DateTime::<Utc>::from_utc(
                        chrono::NaiveDateTime::from_timestamp(
                            block_request.ttl as i64 + chrono::offset::Utc::now().timestamp(),
                            0,
                        ),
                        Utc,
                    )),
                ))
                .execute(&*conn)
                .map_err(ServerError::from)?;

            return Ok(());
        }

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
            new_filter.expression,
            restriction_type.to_string(),
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

        let trim_filter =
            Filter::new(unblock_request.target.ip, unblock_request.target.user_agent)?;

        // then get existing filter
        let filter = self.find_filter(trim_filter.clone()).await?;
        match filter {
            Some(mut filter) => {
                // find existing rule
                let nongrata_id = schema::nongratas::table
                    .filter(schema::nongratas::restriction_value.ilike(filter.clone().expression))
                    .select(schema::nongratas::id)
                    .first::<i64>(&*conn)
                    .map_err(ServerError::from)?;

                filter.trim_expression(trim_filter.clone())?;

                if !filter.clone().is_empty() {
                    // then update cf filter
                    self.client.update_filter(filter.clone()).await?;

                    // then update filter's entry
                    let filter_entry =
                        schema::filters::table.filter(schema::filters::id.eq(filter.clone().id));
                    diesel::update(filter_entry)
                        .set(schema::filters::expression.eq(filter.clone().expression))
                        .execute(&*conn)
                        .map_err(ServerError::from)?;
                } else {
                    // then delete rule & filter
                    self.client
                        .delete_block_rule(filter.clone().rule_id)
                        .await?;

                    // then update filter's entry
                    let filter_entry =
                        schema::filters::table.filter(schema::filters::id.eq(filter.clone().id));
                    diesel::delete(filter_entry)
                        .execute(&*conn)
                        .map_err(ServerError::from)?;
                }

                // then delete nongrata's entry
                diesel::delete(
                    schema::nongratas::table.filter(schema::nongratas::id.eq(nongrata_id)),
                )
                .execute(&*conn)
                .map_err(ServerError::from)?;
            }
            None => return Err(ServerError::WrongFilter),
        }

        Ok(())
    }
}
