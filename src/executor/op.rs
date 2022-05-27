use crate::cloudflare_client;
use crate::errors;
use crate::executor::*;
use crate::handlers;
use crate::models;
use crate::models::Nongrata;
use crate::schema;
use crate::schema::nongratas::restriction_value;

use bb8::Pool;
use bb8::PooledConnection;
use bb8_diesel::{DieselConnection, DieselConnectionManager};
use chrono::Utc;
use diesel::prelude::*;

use tracing::info;

#[derive(Clone)]
pub struct ExecutorService {
    client: cloudflare_client::CloudflareClient,
    db_pool: Pool<DieselConnectionManager<DieselConnection<PgConnection>>>,
}

impl ExecutorService {
    pub fn new(
        client: cloudflare_client::CloudflareClient,
        db_pool: Pool<DieselConnectionManager<DieselConnection<PgConnection>>>,
    ) -> Self {
        Self { client, db_pool }
    }
    pub async fn ban(&self, block_request: BlockRequest) -> Option<errors::ServerError> {
        info!("Incoming request:{:?}", block_request.clone());

        let conn: PooledConnection<DieselConnectionManager<DieselConnection<PgConnection>>> =
            self.db_pool.get().await.unwrap();
        let mut rule_id = String::from("");
        let rule = block_request.clone();
        let restriction_type = models::RestrictionType::Block;
        let firewall_rule = match models::form_firewall_rule_expression(
            rule.target.ip.clone(),
            rule.target.ua.clone(),
        ) {
            Some(r) => r,
            None => return Some(errors::ServerError::EmptyRequest),
        };
        let mut nongrata = Nongrata::new(
            block_request.reason.clone(),
            rule_id,
            chrono::DateTime::<Utc>::from_utc(
                chrono::NaiveDateTime::from_timestamp(block_request.ttl, 0),
                Utc,
            ),
            restriction_type.to_string(),
            firewall_rule,
            true,
        );
        rule_id = match self
            .client
            .restrict_rule(
                rule.target.ip,
                rule.target.ua,
                models::RestrictionType::Block,
            )
            .await
        {
            Ok(rule_id) => rule_id,
            Err(e) => return Some(handlers::models::wrap_err(e)),
        };
        nongrata.rule_id = rule_id;
        diesel::insert_into(schema::nongratas::table)
            .values(nongrata)
            .execute(&*conn)
            .unwrap();
        None
    }

    pub async fn unban(&self, unblock_request: UnblockRequest) -> Option<errors::ServerError> {
        info!("Incoming request:{:?}", unblock_request.clone());

        let conn: PooledConnection<DieselConnectionManager<DieselConnection<PgConnection>>> =
            self.db_pool.get().await.unwrap();
        let rule = unblock_request.clone();
        let firewall_rule = match models::form_firewall_rule_expression(
            rule.target.ip.clone(),
            rule.target.ua.clone(),
        ) {
            Some(r) => r,
            None => return Some(errors::ServerError::EmptyRequest),
        };
        let rule_id = schema::nongratas::table
            .filter(schema::nongratas::restriction_value.eq(firewall_rule.clone()))
            .select(schema::nongratas::rule_id)
            .first::<String>(&*conn)
            .unwrap();
        if let Err(e) = self
            .client
            .restrict_rule(
                rule.target.ip,
                rule.target.ua,
                models::RestrictionType::Unblock(rule_id),
            )
            .await
        {
            return Some(handlers::models::wrap_err(e));
        };
        diesel::delete(schema::nongratas::table.filter(restriction_value.eq(firewall_rule)))
            .execute(&*conn)
            .unwrap();
        None
    }
}
