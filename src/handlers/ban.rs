use crate::cloudflare_client::CloudflareClient;
use crate::errors;
use crate::models;

use actix_web::{web, HttpResponse};
use serde_derive::Deserialize;
use serde_derive::Serialize;

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
) -> Result<HttpResponse, errors::ServerError> {
    for t in &req.target {
        match t.type_field.as_str() {
            "ip" => client
                .restrict_ip_global(t.value.as_str(), models::RestrictionType::Block)
                .await
                .map_err(wrap_err)?,
            "user-agent" => client
                .restrict_user_agent(t.value.as_str(), models::RestrictionType::Block)
                .await
                .map_err(wrap_err)?,
            _ => return Ok(HttpResponse::BadRequest().finish()),
        }
    }
    Ok(HttpResponse::NoContent().finish())
}

fn wrap_err(e: anyhow::Error) -> errors::ServerError {
    return errors::ServerError::WrappedErr {
        cause: format!("cause : {}", e),
    };
}
