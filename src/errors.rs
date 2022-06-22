use crate::handlers;

use actix_web::HttpResponse;
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("CF responded unsuccessfull: {errors:?})")]
    Unsuccessfull { errors: Vec<String> },
    #[error("Request body overflow")]
    Overflow,
    #[error("HTTP client error: {0}")]
    ClientError(#[from] reqwest::Error),
    #[error("Wrapped error: {cause:?}")]
    WrappedErr { cause: String },
    #[error("Missing target")]
    MissingTarget,
    #[error("Missing TTL")]
    MissingTTL,
    #[error("Missing log_level")]
    WrongLogLevel,
    #[error("Missing dry run status")]
    MissingDryRunStatus,
    #[error("PoolError: {0}")]
    PoolError(String),
    #[error("DB error: {0}")]
    DBError(#[from] diesel::result::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl From<ServerError> for HttpResponse {
    fn from(v: ServerError) -> Self {
        match v {
            ServerError::Unsuccessfull { errors } => HttpResponse::BadGateway().json(errors),
            ServerError::Overflow => HttpResponse::PayloadTooLarge().finish(),
            ServerError::WrappedErr { cause } => HttpResponse::InternalServerError().json(cause),
            ServerError::PoolError(cause) => HttpResponse::InternalServerError().json(cause),
            ServerError::ClientError(source) => {
                HttpResponse::InternalServerError().json(source.to_string())
            }
            ServerError::MissingTarget => {
                HttpResponse::BadRequest().json(handlers::models::ExecutorResponse::no_target())
            }
            ServerError::MissingTTL => {
                HttpResponse::BadRequest().json(handlers::models::ExecutorResponse::no_ttl())
            }
            ServerError::WrongLogLevel => HttpResponse::BadRequest()
                .json(handlers::models::ExecutorResponse::wrong_log_level()),
            ServerError::MissingDryRunStatus => HttpResponse::BadRequest()
                .json(handlers::models::ExecutorResponse::no_dry_run_status()),
            ServerError::DBError(source) => {
                HttpResponse::InternalServerError().json(source.to_string())
            }
            ServerError::Other(source) => HttpResponse::InternalServerError().json(json!({
                "reason": source.to_string()
            })),
        }
    }
}

impl actix_web::error::ResponseError for ServerError {}
