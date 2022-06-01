use crate::errors;
use crate::models;
use crate::schema;
use crate::{cloudflare_client::CloudflareClient, errors::ServerError};

use bb8::Pool;
use bb8::PooledConnection;
use chrono::Utc;
use diesel::prelude::*;
use futures::future::join_all;
use std::time::Duration;
use tokio::{task, time};

#[derive(Clone)]
pub struct Invalidator {
    cloudflare_client: CloudflareClient,
    db_pool: Pool<models::DbConn>,
}

impl Invalidator {
    pub fn new(cloudflare_client: CloudflareClient, db_pool: Pool<models::DbConn>) -> Self {
        Self {
            cloudflare_client,
            db_pool,
        }
    }
    pub async fn run(self) -> Result<(), ServerError> {
        let forever = task::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(1));
            loop {
                interval.tick().await;
                self.clone().invalidate().await?;
            }
        });
        forever.await.map_err(|e| errors::wrap_err(e.into()))?
    }
    pub async fn run_invalidator_untill_stopped(self) -> Result<(), ServerError> {
        self.run().await
    }
    pub async fn invalidate(self) -> Result<(), ServerError> {
        let conn: PooledConnection<models::DbConn> = self
            .db_pool
            .get()
            .await
            .map_err(|e| errors::wrap_err(e.into()))?;
        let rule_ids = schema::nongratas::table
            .filter(
                schema::nongratas::expires_at.ge(chrono::DateTime::<Utc>::from_utc(
                    chrono::NaiveDateTime::from_timestamp(
                        chrono::offset::Utc::now().timestamp(),
                        0,
                    ),
                    Utc,
                )),
            )
            .select(schema::nongratas::rule_id)
            .load::<String>(&*conn)
            .map_err(|e| errors::wrap_err(e.into()))?;
        let handlers = rule_ids
            .iter()
            .map(|id| self.cloudflare_client.delete_block_rule(id.clone()));
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
