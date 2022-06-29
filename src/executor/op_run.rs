use crate::cloudflare_client;
use crate::errors;
use crate::errors::ServerError;
use crate::executor::models::Executor;
use crate::executor::*;
use crate::models;
use crate::models::Nongrata;
use crate::pool::DbConn;
use crate::schema;

use async_trait::async_trait;
use chrono::Utc;
use diesel::prelude::*;
use diesel::r2d2::Pool;
use diesel::r2d2::PooledConnection;

use futures::future::try_join_all;
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
        let restriction_type = models::RestrictionType::Block;
        let firewall_rule =
            models::form_firewall_rule_expression(rule.target.ip, rule.target.user_agent)
                .ok_or(errors::ServerError::MissingTarget)?;
        let rule_id = schema::nongratas::table
            .filter(schema::nongratas::restriction_value.eq(&firewall_rule))
            .select(schema::nongratas::rule_id)
            .load::<String>(&*conn)
            .map_err(ServerError::from)?;
        if !rule_id.is_empty() {
            let target = schema::nongratas::table
                .filter(schema::nongratas::restriction_value.eq(&firewall_rule));
            diesel::update(target)
                .set(
                    schema::nongratas::expires_at.eq(chrono::DateTime::<Utc>::from_utc(
                        chrono::NaiveDateTime::from_timestamp(
                            rule.ttl as i64 + chrono::offset::Utc::now().timestamp(),
                            0,
                        ),
                        Utc,
                    )),
                )
                .execute(&*conn)
                .map_err(ServerError::from)?;
        } else {
            let rule_id = self
                .client
                .create_block_rule(firewall_rule.clone(), models::RestrictionType::Block)
                .await
                .map_err(ServerError::from)?;

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
                .map_err(ServerError::from)?;
        }
        Ok(())
    }

    async fn unban(&self, unblock_request: UnblockRequest) -> Result<(), errors::ServerError> {
        info!("Incoming request:{:?}", unblock_request);

        let conn: PooledConnection<DbConn> = self
            .db_pool
            .get()
            .map_err(|e| ServerError::PoolError(e.to_string()))?;

        let rule = unblock_request;
        let firewall_rule =
            models::form_firewall_rule_expression(rule.target.ip, rule.target.user_agent)
                .ok_or(errors::ServerError::MissingTarget)?;
        let rule_ids = schema::nongratas::table
            .filter(schema::nongratas::restriction_value.eq(firewall_rule.clone()))
            .select(schema::nongratas::rule_id)
            .load::<String>(&*conn)
            .map_err(ServerError::from)?;
        let handles = rule_ids
            .iter()
            .map(|id| self.client.delete_block_rule(id.clone()));
        let handles = try_join_all(handles).await?;
        handles
            .iter()
            .zip(rule_ids.clone().iter())
            .try_for_each(|(_, id)| {
                diesel::delete(schema::nongratas::table.filter(schema::nongratas::rule_id.eq(id)))
                    .execute(&*conn)
                    .map(|_| ())
                    .map_err(ServerError::from)
            })
    }
}
