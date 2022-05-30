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

const SEPARATOR: &str = " and ";

pub fn form_firewall_rule_expression(ip: Option<&String>, ua: Option<&String>) -> Option<String> {
    let mut ss = vec![];

    if ua.is_none() && ip.is_none() {
        return None;
    }
    if let Some(ua) = ua {
        ss.push(format!("http.user_agent eq \"{}\"", ua));
    }
    if let Some(ip) = ip {
        ss.push(format!("ip.src eq {}", ip));
    }
    Some(ss.join(SEPARATOR))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_form_firewall_rule_expression() {
        assert_eq!(
            form_firewall_rule_expression(
                Some(&String::from("192.168.0.1")),
                Some(&String::from(
                    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)"
                ))
            ),
            Some(String::from("http.user_agent eq \"Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)\" and ip.src eq 192.168.0.1"))
        );

        assert_eq!(
            form_firewall_rule_expression(Some(&String::from("192.168.0.1")), None,),
            Some(String::from("ip.src eq 192.168.0.1"))
        );

        assert_eq!(
            form_firewall_rule_expression(
                None,
                Some(&String::from(
                    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)"
                ))
            ),
            Some(String::from(
                "http.user_agent eq \"Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)\""
            ))
        );
        
    }
}
