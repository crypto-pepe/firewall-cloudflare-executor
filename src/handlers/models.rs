use crate::errors;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminRequest {
    pub dry_run: Option<bool>,
    pub log_level: Option<String>,
}

pub fn wrap_err(e: anyhow::Error) -> errors::ServerError {
    return errors::ServerError::WrappedErr {
        cause: format!("cause : {}", e),
    };
}
