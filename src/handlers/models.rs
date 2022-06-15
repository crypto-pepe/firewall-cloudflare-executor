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
    pub details: ErrorDetails,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorDetails {
    pub target: Option<String>,
    pub ttl: Option<String>,
}

impl ExecutorResponse {
    pub fn no_analyzer_id() -> Self {
        Self {
            code: StatusCode::BAD_REQUEST.as_u16(),
            reason: String::from(
                "Provided request does not match the constraints: empty analyzer-id header",
            ),
            details: ErrorDetails {
                target: None,
                ttl: None,
            },
        }
    }
    pub fn no_target() -> Self {
        Self {
            code: StatusCode::BAD_REQUEST.as_u16(),
            reason: String::from("Provided request does not match the constraints"),
            details: ErrorDetails {
                target: Some(String::from("This field is required")),
                ttl: None,
            },
        }
    }
    pub fn no_ttl() -> Self {
        Self {
            code: StatusCode::BAD_REQUEST.as_u16(),
            reason: String::from("Provided request does not match the constraints"),
            details: ErrorDetails {
                target: None,
                ttl: Some(String::from("This field is required")),
            },
        }
    }
    pub fn wrong_log_level() -> Self {
        Self {
            code: StatusCode::BAD_REQUEST.as_u16(),
            reason: String::from("Log level is incorrect"),
            details: ErrorDetails {
                target: None,
                ttl: None,
            },
        }
    }
    pub fn no_dry_run_status() -> Self {
        Self {
            code: StatusCode::BAD_REQUEST.as_u16(),
            reason: String::from("Dry run status is incorrect"),
            details: ErrorDetails {
                target: None,
                ttl: None,
            },
        }
    }
}
