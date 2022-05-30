use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminRequest {
    pub dry_run: Option<bool>,
    pub log_level: Option<String>,
}
