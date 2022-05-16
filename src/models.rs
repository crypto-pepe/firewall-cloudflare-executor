use crate::handlers;
use crate::schema::nongratas;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use strum_macros::Display;

#[derive(Serialize)]
pub struct AccessRuleRequest {
    pub mode: String,
    pub configuration: Configuration,
}
#[derive(Serialize)]
pub struct Configuration {
    pub target: String,
    pub value: String,
}
#[derive(Deserialize)]
pub struct AccessRuleResponse {
    pub success: bool,
    pub errors: Vec<String>,
}

#[derive(Debug, Display, PartialEq)]
pub enum RestrictionType {
    Block,
    Challenge,
    Whitelist,
}

impl FromStr for RestrictionType {
    type Err = ();
    fn from_str(input: &str) -> Result<RestrictionType, Self::Err> {
        match input {
            "block" => Ok(RestrictionType::Block),
            "challenge" => Ok(RestrictionType::Challenge),
            "whitelist" => Ok(RestrictionType::Whitelist),
            _ => Err(()),
        }
    }
}

#[derive(Insertable)]
pub struct Nongrata {
    pub reason: String,
    pub restriction_type: String,
    pub restriction_value: String,
    pub expires_at: DateTime<Utc>,
    pub is_global: bool,
}

impl Nongrata {
    pub fn from_req(
        req: handlers::Target,
        reason: String,
        ttl: DateTime<Utc>,
        is_global: bool,
    ) -> Self {
        Self {
            restriction_type: req.type_field,
            reason: reason,
            restriction_value: req.value,
            expires_at: ttl,
            is_global: is_global,
        }
    }
}
