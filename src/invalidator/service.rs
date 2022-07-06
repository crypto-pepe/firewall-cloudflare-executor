use crate::pool::DbConn;
use crate::{cloudflare_client::CloudflareClient, errors::ServerError};
use crate::{models, schema};

use chrono::Utc;
use diesel::prelude::*;
use diesel::r2d2::Pool;
use diesel::r2d2::PooledConnection;
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

        // select expired entries
        let nongratas = schema::nongratas::table
            .filter(
                schema::nongratas::expires_at.le(chrono::DateTime::<Utc>::from_utc(
                    chrono::NaiveDateTime::from_timestamp(
                        chrono::offset::Utc::now().timestamp(),
                        0,
                    ),
                    Utc,
                )),
            )
            .load::<models::Nongrata>(&*conn)
            .map_err(ServerError::from)?;

        for target in nongratas {
            // construct desired raw trim filter
            let mut trim_filter = models::Filter::from_expression(
                target.clone().filter_id,
                target.clone().restriction_value,
            );

            // then get existing filter
            let mut filter = schema::filters::table
                .filter(schema::filters::id.eq(target.clone().filter_id))
                .select(schema::filters::all_columns)
                .first::<models::Filter>(&*conn)
                .map_err(ServerError::from)?;
            trim_filter.filter_type = filter.clone().filter_type;

            // then delete nongrata's entry
            diesel::delete(
                schema::nongratas::table.filter(schema::nongratas::id.eq(target.id.unwrap())),
            )
            .execute(&*conn)
            .map_err(ServerError::from)?;

            filter.trim_expression(trim_filter)?;
            if !filter.clone().is_empty() {
                // then update cf filter
                self.cloudflare_client.update_filter(filter.clone()).await?;

                // then update filter's entry
                let filter_entry =
                    schema::filters::table.filter(schema::filters::id.eq(target.filter_id));
                diesel::update(filter_entry)
                    .set(schema::filters::expression.eq(filter.expression))
                    .execute(&*conn)
                    .map_err(ServerError::from)?;
            } else {
                // then delete rule & filter
                self.cloudflare_client
                    .delete_block_rule(filter.rule_id)
                    .await?;

                // then update filter's entry
                let filter_entry =
                    schema::filters::table.filter(schema::filters::id.eq(target.filter_id));
                diesel::delete(filter_entry)
                    .execute(&*conn)
                    .map_err(ServerError::from)?;
            }
        }

        Ok(())
    }
}
