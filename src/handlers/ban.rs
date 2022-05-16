use crate::cloudflare_client::CloudflareClient;
use crate::errors;
use crate::models;
use crate::models::Nongrata;
use crate::schema;

use actix_web::{web, HttpResponse};
use bb8::Pool;
use bb8::PooledConnection;
use bb8_diesel::DieselConnectionManager;
use chrono::Utc;
use diesel::prelude::*;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use tracing::{error, info};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Root {
    pub target: Vec<Target>,
    pub reason: String,
    pub ttl: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Target {
    #[serde(rename = "type")]
    pub type_field: String,
    pub value: String,
}

pub async fn ban(
    req: web::Json<Root>,
    client: web::Data<CloudflareClient>,
    pool: web::Data<Pool<DieselConnectionManager<PgConnection>>>,
) -> Result<HttpResponse, errors::ServerError> {
    info!("Incoming request:{:?}", req.clone());

    let conn: PooledConnection<DieselConnectionManager<PgConnection>> = pool.get().await.unwrap();

    for t in &req.target {
        let rule = t.clone();
        let nongrata = Nongrata::from_req(
            rule,
            req.reason.clone(),
            chrono::DateTime::<Utc>::from_utc(
                chrono::NaiveDateTime::from_timestamp(req.ttl, 0),
                Utc,
            ),
            true,
        );
        match t.type_field.as_str() {
            "ip" => {
                client
                    .restrict_ip_global(t.value.as_str(), models::RestrictionType::Block)
                    .await
                    .map_err(wrap_err)?;
            }
            "user-agent" => {
                client
                    .restrict_user_agent(t.value.as_str(), models::RestrictionType::Block)
                    .await
                    .map_err(wrap_err)?;
            }
            _ => {
                error!("Unknown restriction type");
                return Ok(HttpResponse::BadRequest().finish());
            }
        }
        diesel::insert_into(schema::nongratas::table)
            .values(nongrata)
            .execute(&*conn)
            .unwrap();
    }

    Ok(HttpResponse::NoContent().finish())
}

fn wrap_err(e: anyhow::Error) -> errors::ServerError {
    return errors::ServerError::WrappedErr {
        cause: format!("cause : {}", e),
    };
}
