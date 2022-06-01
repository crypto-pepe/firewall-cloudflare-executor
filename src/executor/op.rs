use crate::cloudflare_client;
use crate::errors;
use crate::executor::models::Executor;
use crate::executor::*;
use crate::models;
use crate::models::Nongrata;
use crate::schema;

use async_trait::async_trait;
use bb8::Pool;
use bb8::PooledConnection;
use chrono::Utc;
use diesel::prelude::*;

use tracing::info;

#[derive(Clone)]
pub struct ExecutorService {
    client: cloudflare_client::CloudflareClient,
    db_pool: Pool<models::DbConn>,
}

impl ExecutorService {
    pub fn new(client: cloudflare_client::CloudflareClient, db_pool: Pool<models::DbConn>) -> Self {
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
        info!("Incoming request:{:?}", block_request.clone());

        let conn: PooledConnection<models::DbConn> = self
            .db_pool
            .get()
            .await
            .map_err(|e| errors::wrap_err(e.into()))?;
        let rule = block_request.clone();
        let restriction_type = models::RestrictionType::Block;
        let firewall_rule = match models::form_firewall_rule_expression(
            rule.target.ip.as_ref(),
            rule.target.ua.as_ref(),
        ) {
            Some(r) => r,
            None => return Err(errors::ServerError::EmptyRequest),
        };

        let rule_id = self
            .client
            .create_block_rule(firewall_rule.clone(), models::RestrictionType::Block)
            .await
            .map_err(errors::wrap_err)?;
        let nongrata = Nongrata::new(
            block_request.reason.clone(),
            rule_id,
            chrono::DateTime::<Utc>::from_utc(
                chrono::NaiveDateTime::from_timestamp(
                    block_request.ttl + chrono::offset::Utc::now().timestamp(),
                    0,
                ),
                Utc,
            ),
            restriction_type.to_string(),
            firewall_rule,
            true,
            analyzer_id,
        );
        if let Err(e) = diesel::insert_into(schema::nongratas::table)
            .values(nongrata)
            .execute(&*conn)
        {
            return Err(errors::wrap_err(e.into()));
        }
        Ok(())
    }

    async fn unban(&self, unblock_request: UnblockRequest) -> Result<(), errors::ServerError> {
        info!("Incoming request:{:?}", unblock_request);

        let conn: PooledConnection<models::DbConn> = self
            .db_pool
            .get()
            .await
            .map_err(|e| errors::wrap_err(e.into()))?;
        let rule = unblock_request;
        let firewall_rule = match models::form_firewall_rule_expression(
            rule.target.ip.as_ref(),
            rule.target.ua.as_ref(),
        ) {
            Some(r) => r,
            None => return Err(errors::ServerError::EmptyRequest),
        };
        let rule_ids = schema::nongratas::table
            .filter(schema::nongratas::restriction_value.eq(firewall_rule.clone()))
            .select(schema::nongratas::rule_id)
            .load::<String>(&*conn)
            .map_err(|e| errors::wrap_err(e.into()))?;
        for id in rule_ids {
            if let Err(e) = self.client.delete_block_rule(id.clone()).await {
                return Err(errors::wrap_err(e.into()));
            };
            if let Err(e) =
                diesel::delete(schema::nongratas::table.filter(schema::nongratas::rule_id.eq(id)))
                    .execute(&*conn)
            {
                return Err(errors::wrap_err(e.into()));
            }
        }
        Ok(())
    }
}
