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
            ServerError::Overflow => HttpResponse::PayloadTooLarge().finish(),

            ServerError::Unsuccessfull { errors } => {
                handlers::models::internal(errors.into_iter().collect::<String>())
            }
            ServerError::WrappedErr { cause } => handlers::models::internal(cause),
            ServerError::PoolError(cause) => handlers::models::internal(cause),
            ServerError::ClientError(source) => handlers::models::internal(source.to_string()),
            ServerError::DBError(source) => handlers::models::internal(source.to_string()),
            ServerError::Other(source) => handlers::models::internal(source.to_string()),

            ServerError::MissingTarget => handlers::models::bad_request("Missing target"),
            ServerError::MissingTTL => handlers::models::bad_request("Missing TTL"),
            ServerError::WrongLogLevel => handlers::models::bad_request("Wrong log level"),
            ServerError::MissingDryRunStatus => {
                handlers::models::bad_request("Missing dry run status")
            }
            ServerError::BadIP => handlers::models::bad_request("Bad IP"),
            ServerError::BadRequest(reason) => handlers::models::bad_request(reason),
        }
    }
}

impl actix_web::error::ResponseError for ServerError {}
