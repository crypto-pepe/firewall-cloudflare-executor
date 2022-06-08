use crate::cloudflare_client;
use crate::errors;
use crate::errors::ServerError;
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
use futures::future::join_all;

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
            .map_err(|e| ServerError::PoolError(e.to_string()))?;

        let rule = block_request.clone();
        let restriction_type = models::RestrictionType::Block;
        let firewall_rule =
            models::form_firewall_rule_expression(rule.target.ip.as_ref(), rule.target.ua.as_ref())
                .ok_or(errors::ServerError::MissingTarget)?;

        let rule_id = self
            .client
            .create_block_rule(firewall_rule.clone(), models::RestrictionType::Block)
            .await
            .map_err(|e| ServerError::from(e))?;
        if block_request.ttl == 0 {
            return Err(errors::ServerError::MissingTTL);
        }

        let nongrata = Nongrata::new(
            block_request.reason.clone(),
            rule_id,
            chrono::DateTime::<Utc>::from_utc(
                chrono::NaiveDateTime::from_timestamp(
                    block_request.ttl as i64 + chrono::offset::Utc::now().timestamp(),
                    0,
                ),
                Utc,
            ),
            restriction_type.to_string(),
            firewall_rule,
            true,
            analyzer_id,
        );
        diesel::insert_into(schema::nongratas::table)
            .values(nongrata)
            .execute(&*conn)
            .map_err(|e| ServerError::from(e))?;

        Ok(())
    }

    async fn unban(&self, unblock_request: UnblockRequest) -> Result<(), errors::ServerError> {
        info!("Incoming request:{:?}", unblock_request);

        let conn: PooledConnection<models::DbConn> = self
            .db_pool
            .get()
            .await
            .map_err(|e| ServerError::PoolError(e.to_string()))?;

        let rule = unblock_request;
        let firewall_rule =
            models::form_firewall_rule_expression(rule.target.ip.as_ref(), rule.target.ua.as_ref())
                .ok_or(errors::ServerError::MissingTarget)?;
        let rule_ids = schema::nongratas::table
            .filter(schema::nongratas::restriction_value.eq(firewall_rule.clone()))
            .select(schema::nongratas::rule_id)
            .load::<String>(&*conn)
            .map_err(|e| errors::wrap_err(e.into()))?;
        let handlers = rule_ids
            .iter()
            .map(|id| self.client.delete_block_rule(id.clone()));
        let handlers_iter = join_all(handlers).await;
        handlers_iter
            .iter()
            .zip(rule_ids.clone().iter())
            .try_for_each(|(_, id)| {
                diesel::delete(schema::nongratas::table.filter(schema::nongratas::rule_id.eq(id)))
                    .execute(&*conn)
                    .map(|_| ())
                    .map_err(|e| errors::wrap_err(e.into()))
            })
    }
}
