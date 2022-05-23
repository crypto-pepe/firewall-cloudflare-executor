use crate::cloudflare_client::CloudflareClient;
use crate::errors;
use crate::handlers;
use crate::models;
use crate::schema;
use crate::schema::nongratas::restriction_value;

use actix_web::{web, HttpResponse};
use bb8::Pool;
use bb8::PooledConnection;
use bb8_diesel::DieselConnectionManager;
use diesel::prelude::*;
use tracing::{error, info};

use super::UnblockRequest;

pub async fn unban(
    req: web::Json<UnblockRequest>,
    client: web::Data<CloudflareClient>,
    pool: web::Data<Pool<DieselConnectionManager<PgConnection>>>,
    is_dry: web::Data<bool>,
) -> Result<HttpResponse, errors::ServerError> {
    if *is_dry.get_ref() {
        return unban_dry(req).await;
    }
    return unban_op(req, client, pool).await;
}

pub async fn unban_dry(
    req: web::Json<UnblockRequest>,
) -> Result<HttpResponse, errors::ServerError> {
    info!("Incoming request:{:?}", req.clone());
    match req.target.ip.clone() {
        Some(ip) => match req.target.ua.clone() {
            Some(ua) => info!("gonna unban {:?}{:?}", ua, ip),
            None => info!("gonna unban {:?} by IP", ip),
        },
        None => match req.target.ua.clone() {
            Some(ua) => info!("gonna unban {:?}", ua),
            None => {
                error!("Empty request");
                return Ok(HttpResponse::BadRequest().finish());
            }
        },
    }
    Ok(HttpResponse::NoContent().finish())
}

pub async fn unban_op(
    req: web::Json<UnblockRequest>,
    client: web::Data<CloudflareClient>,
    pool: web::Data<Pool<DieselConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, errors::ServerError> {
    info!("Incoming request:{:?}", req.clone());

    let conn: PooledConnection<DieselConnectionManager<PgConnection>> = pool.get().await.unwrap();
    let rule = req.clone();
    let firewall_rule =
        match models::form_firewall_rule_expression(rule.target.ip.clone(), rule.target.ua.clone())
        {
            Some(r) => r,
            None => return Err(errors::ServerError::EmptyRequest),
        };
    let rule_id = schema::nongratas::table
        .filter(schema::nongratas::restriction_value.eq(firewall_rule.clone()))
        .select(schema::nongratas::rule_id)
        .first::<String>(&*conn)
        .unwrap();

    client
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
