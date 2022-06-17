use crate::pool::DbConn;
use crate::schema;
use crate::{cloudflare_client::CloudflareClient, errors::ServerError};

use chrono::Utc;
use diesel::prelude::*;
use diesel::r2d2::Pool;
use diesel::r2d2::PooledConnection;
use futures::future::join_all;
use std::time::Duration;
use tokio::{task, time};

#[derive(Clone)]
pub struct Invalidator {
    cloudflare_client: CloudflareClient,
    db_pool: Pool<DbConn>,
    timeout: Duration,
}

impl Invalidator {
    pub fn new(
        cloudflare_client: CloudflareClient,
        db_pool: Pool<DbConn>,
        timeout: Duration,
    ) -> Self {
        Self {
            cloudflare_client,
            db_pool,
            timeout,
        }
    }
    pub async fn run(self) -> Result<(), ServerError> {
        let invalidation_handle = task::spawn(async move {
            let mut interval = time::interval(self.timeout);
            loop {
                interval.tick().await;
                self.clone().invalidate().await?;
            }
        });
        invalidation_handle
            .await
            .map_err(|e| ServerError::from(anyhow::anyhow!(e)))?
    }
    pub async fn run_untill_stopped(self) -> Result<(), ServerError> {
        self.run().await
    }
    pub async fn invalidate(self) -> Result<(), ServerError> {
        let conn: PooledConnection<DbConn> = self
            .db_pool
            .get()
            .map_err(|e| ServerError::PoolError(e.to_string()))?;
        let rule_ids = schema::nongratas::table
            .filter(
                schema::nongratas::expires_at.le(chrono::DateTime::<Utc>::from_utc(
                    chrono::NaiveDateTime::from_timestamp(
                        chrono::offset::Utc::now().timestamp(),
                        0,
                    ),
                    Utc,
                )),
            )
            .select(schema::nongratas::rule_id)
            .load::<String>(&*conn)
            .map_err(ServerError::from)?;
        let handlers = rule_ids
            .iter()
            .map(|id| self.cloudflare_client.delete_block_rule(id.clone()));
        let handlers = join_all(handlers).await;
        handlers
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
