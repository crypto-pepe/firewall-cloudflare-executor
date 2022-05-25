use crate::cloudflare_client;
use crate::errors;
use crate::handlers;
use crate::models;
use crate::models::Nongrata;
use crate::schema;
use crate::schema::nongratas::restriction_value;

use actix_web::{web, HttpResponse};
use anyhow::Result;
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
    pub async fn ban(
        &self,
        req: web::Json<handlers::models::BlockRequest>,
    ) -> Result<HttpResponse, errors::ServerError> {
        info!("Incoming request:{:?}", req.clone());

        let conn: PooledConnection<DieselConnectionManager<DieselConnection<PgConnection>>> =
            self.db_pool.get().await.unwrap();
        let mut rule_id = String::from("");
        let rule = req.clone();
        let restriction_type = models::RestrictionType::Block;
        let firewall_rule = match models::form_firewall_rule_expression(
            rule.target.ip.clone(),
            rule.target.ua.clone(),
        ) {
            Some(r) => r,
            None => return Err(errors::ServerError::EmptyRequest),
        };
        let mut nongrata = Nongrata::new(
            req.reason.clone(),
            rule_id,
            chrono::DateTime::<Utc>::from_utc(
                chrono::NaiveDateTime::from_timestamp(req.ttl, 0),
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
            Err(e) => return Err(handlers::models::wrap_err(e)),
        };
        nongrata.rule_id = rule_id;
        diesel::insert_into(schema::nongratas::table)
            .values(nongrata)
            .execute(&*conn)
            .unwrap();
        Ok(HttpResponse::NoContent().finish())
    }

    pub async fn unban(
        &self,
        req: web::Json<handlers::models::UnblockRequest>,
    ) -> Result<HttpResponse, errors::ServerError> {
        info!("Incoming request:{:?}", req.clone());

        let conn: PooledConnection<DieselConnectionManager<DieselConnection<PgConnection>>> =
            self.db_pool.get().await.unwrap();
        let rule = req.clone();
        let firewall_rule = match models::form_firewall_rule_expression(
            rule.target.ip.clone(),
            rule.target.ua.clone(),
        ) {
            Some(r) => r,
            None => return Err(errors::ServerError::EmptyRequest),
        };
        let rule_id = schema::nongratas::table
            .filter(schema::nongratas::restriction_value.eq(firewall_rule.clone()))
            .select(schema::nongratas::rule_id)
            .first::<String>(&*conn)
            .unwrap();
        self.client
            .restrict_rule(
                rule.target.ip,
                rule.target.ua,
                models::RestrictionType::Unblock(rule_id),
            )
            .await
            .map_err(handlers::models::wrap_err)?;
        diesel::delete(schema::nongratas::table.filter(restriction_value.eq(firewall_rule)))
            .execute(&*conn)
            .unwrap();
        Ok(HttpResponse::NoContent().finish())
    }
}
