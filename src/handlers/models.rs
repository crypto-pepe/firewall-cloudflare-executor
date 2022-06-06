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
    #[serde(skip_serializing_if = "String::is_empty")]
    pub target: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub ttl: String,
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
                ttl: String::from(""),
            },
        }
    }
    pub fn no_target() -> Self {
        Self {
            code: StatusCode::BAD_REQUEST.as_u16(),
            reason: String::from("Provided request does not match the constraints"),
            details: Details {
                target: String::from("This field is required"),
                ttl: String::from(""),
            },
        }
    }
    pub fn no_ttl() -> Self {
        Self {
            code: StatusCode::BAD_REQUEST.as_u16(),
            reason: String::from("Provided request does not match the constraints"),
            details: Details {
                target: String::from(""),
                ttl: String::from("This field is required"),
            },
        }
    }
    pub fn wrong_log_level() -> Self {
        Self {
            code: StatusCode::BAD_REQUEST.as_u16(),
            reason: String::from("Log level is incorrect"),
            details: Details {
                target: String::from(""),
                ttl: String::from(""),
            },
        }
    }
    pub fn no_dry_run_status() -> Self {
        Self {
            code: StatusCode::BAD_REQUEST.as_u16(),
            reason: String::from("Dry run status is incorrect"),
            details: Details {
                target: String::from(""),
                ttl: String::from(""),
            },
        }
    }
}
