use crate::schema::nongratas;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use strum_macros::Display;

#[derive(Serialize)]
pub struct FirewallRuleRequest {
    pub action: String,
    pub filter: Filter,
}

#[derive(Serialize)]
pub struct Filter {
    pub expression: String,
}

#[derive(Deserialize)]
pub struct FirewallRuleResponse {
    pub success: bool,
    pub result: Vec<ResultResp>,
    pub errors: Vec<String>,
}

#[derive(Deserialize)]
pub struct ResultResp {
    pub id: String,
}
#[derive(Debug, Display, PartialEq)]
pub enum RestrictionType {
    Block,
    Challenge,
    Whitelist,
    Unblock(String),
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

#[derive(Insertable, Queryable, Debug)]
pub struct Nongrata {
    pub rule_id: String,
    pub reason: String,
    pub restriction_type: String,
    pub restriction_value: String,
    pub expires_at: DateTime<Utc>,
    pub is_global: bool,
}

impl Nongrata {
    pub fn new(
        reason: String,
        rule_id: String,
        ttl: DateTime<Utc>,
        restriction_type: String,
        restriction_value: String,
        is_global: bool,
    ) -> Self {
        Self {
            rule_id,
            restriction_type,
            reason,
            restriction_value,
            expires_at: ttl,
            is_global,
        }
    }
}

pub fn form_firewall_rule_expression(ip: Option<String>, ua: Option<String>) -> Option<String> {
    match ip {
        Some(ip) => match ua {
            Some(ua) => Some(format!(
                "http.user_agent eq \"{}\" and ip.src eq {}",
                ua, ip
            )),
            None => Some(format!("(ip.src eq {})", ip)),
        },
        None => ua.map(|ua| format!("(http.user_agent eq \"{}\")", ua)),
    }
}
