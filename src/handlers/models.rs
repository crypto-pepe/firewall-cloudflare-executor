use actix_http::StatusCode;
use actix_web::HttpResponse;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdminRequest {
    pub dry_run: Option<bool>,
    pub log_level: Option<String>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ExecutorResponse {
    pub code: u16,
    pub reason: String,
    pub details: Option<ErrorDetails>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorDetails {
    pub target: Option<String>,
    pub ttl: Option<String>,
}

pub fn internal(reason: impl AsRef<str>) -> HttpResponse {
    HttpResponse::InternalServerError().json(ExecutorResponse {
        code: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
        reason: reason.as_ref().into(),
        details: None,
    })
}

pub fn bad_request(reason: impl AsRef<str>) -> HttpResponse {
    HttpResponse::BadRequest().json(ExecutorResponse {
        code: StatusCode::BAD_REQUEST.as_u16(),
        reason: reason.as_ref().into(),
        details: None,
    })
}
