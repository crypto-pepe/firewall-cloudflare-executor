use actix_http::StatusCode;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminRequest {
    pub dry_run: Option<bool>,
    pub log_level: Option<String>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ExecutorResponse {
    pub code: u16,
    pub reason: String,
    pub details: Details,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Details {
    pub target: String,
}

impl ExecutorResponse {
    pub fn no_analyzer_id() -> Self {
        Self {
            code: StatusCode::BAD_REQUEST.as_u16(),
            reason: String::from(
                "Provided request does not match the constraints: empty analyzer-id header",
            ),
            details: Details {
                target: String::from(""),
            },
        }
    }
    pub fn no_target() -> Self {
        Self {
            code: StatusCode::BAD_REQUEST.as_u16(),
            reason: String::from("Provided request does not match the constraints"),
            details: Details {
                target: String::from("This field is required"),
            },
        }
    }
}
