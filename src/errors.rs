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

impl actix_web::error::ResponseError for ServerError {}
