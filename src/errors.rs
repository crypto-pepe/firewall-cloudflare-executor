use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("CF responded unsuccessfull: {info:?})")]
    Unsuccessfull { info: Vec<String> },
    #[error("Request body overflow)")]
    Overflow,
    #[error("Request body overflow)")]
    WrappedErr { cause: String },
}

impl actix_web::error::ResponseError for ServerError {}
