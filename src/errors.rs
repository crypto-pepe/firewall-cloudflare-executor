use crate::handlers;

use actix_web::HttpResponse;
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("CF responded unsuccessfull: {info:?})")]
    Unsuccessfull { info: Vec<String> },
    #[error("Request body overflow")]
    Overflow,
    #[error("Wrapped err: {cause:?}")]
    WrappedErr { cause: String },
    #[error("Empty request")]
    EmptyRequest,
    #[error("PoolError: {0}")]
    PoolError(String),
    #[error("DB error: {0}")]
    DBError(#[from] diesel::result::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error)
}

impl From<ServerError> for HttpResponse {
    fn from(v: ServerError) -> Self {
        match v {
            ServerError::Unsuccessfull { info } => HttpResponse::BadGateway().json(info),
            ServerError::Overflow => HttpResponse::PayloadTooLarge().finish(),
            ServerError::WrappedErr { cause } => HttpResponse::InternalServerError().json(cause),
            ServerError::EmptyRequest => {
                HttpResponse::Ok().json(handlers::models::ExecutorResponse::no_target())
            }
            ServerError::PoolError(cause) => HttpResponse::InternalServerError().json(cause),
            ServerError::DBError(source) => HttpResponse::InternalServerError().json(source.to_string()),
            ServerError::Other(source) => HttpResponse::InternalServerError().json(json!({
                "reason": source.to_string()
            })),
        }
    }
}

impl actix_web::error::ResponseError for ServerError {}

pub fn wrap_err(e: anyhow::Error) -> ServerError {
    return ServerError::WrappedErr {
        cause: format!("cause : {}", e),
    };
}

pub fn wrap_client_err(e: anyhow::Error) -> ServerError {
    return ServerError::Unsuccessfull {
        info: vec![format!("cause : {}", e)],
    };
}
