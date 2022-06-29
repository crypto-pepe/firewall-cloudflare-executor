use crate::handlers;

use actix_web::HttpResponse;
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
    #[error("Target IP is invalid")]
    BadIP,
    #[error("Provided request does not match the constraints: {0}")]
    BadRequest(String),
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
            ServerError::Unsuccessfull { errors } => HttpResponse::InternalServerError().json(
                handlers::models::ExecutorResponse::internal(
                    errors.into_iter().collect::<String>(),
                ),
            ),
            ServerError::Overflow => HttpResponse::PayloadTooLarge().finish(),
            ServerError::WrappedErr { cause } => HttpResponse::InternalServerError()
                .json(handlers::models::ExecutorResponse::internal(cause)),
            ServerError::PoolError(cause) => HttpResponse::InternalServerError()
                .json(handlers::models::ExecutorResponse::internal(cause)),
            ServerError::ClientError(source) => HttpResponse::InternalServerError().json(
                handlers::models::ExecutorResponse::internal(source.to_string()),
            ),
            ServerError::MissingTarget => HttpResponse::BadRequest().json(
                handlers::models::ExecutorResponse::bad_request("Missing target"),
            ),
            ServerError::MissingTTL => HttpResponse::BadRequest().json(
                handlers::models::ExecutorResponse::bad_request("Missing TTL"),
            ),
            ServerError::WrongLogLevel => HttpResponse::BadRequest().json(
                handlers::models::ExecutorResponse::bad_request("Wrong log level"),
            ),
            ServerError::MissingDryRunStatus => HttpResponse::BadRequest().json(
                handlers::models::ExecutorResponse::bad_request("Missing dry run status"),
            ),
            ServerError::DBError(source) => HttpResponse::InternalServerError().json(
                handlers::models::ExecutorResponse::internal(source.to_string()),
            ),
            ServerError::BadIP => HttpResponse::BadRequest()
                .json(handlers::models::ExecutorResponse::bad_request("Bad IP")),
            ServerError::BadRequest(reason) => HttpResponse::BadRequest()
                .json(handlers::models::ExecutorResponse::bad_request(reason)),
            ServerError::Other(source) => HttpResponse::InternalServerError().json(
                handlers::models::ExecutorResponse::internal(source.to_string()),
            ),
        }
    }
}

impl actix_web::error::ResponseError for ServerError {}
