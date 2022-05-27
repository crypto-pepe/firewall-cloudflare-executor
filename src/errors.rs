use actix_web::HttpResponse;
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
}

impl From<ServerError> for HttpResponse {
    fn from(v: ServerError) -> Self {
        match v {
            ServerError::Unsuccessfull { info } => HttpResponse::BadGateway().json(info),
            ServerError::Overflow => HttpResponse::PayloadTooLarge().finish(),
            ServerError::WrappedErr { cause } => HttpResponse::InternalServerError().json(cause),
            ServerError::EmptyRequest => HttpResponse::Ok().finish(),
        }
    }
}

impl actix_web::error::ResponseError for ServerError {}
