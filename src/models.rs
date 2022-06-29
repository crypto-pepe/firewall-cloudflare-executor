use crate::{errors::ServerError, schema::nongratas};

use chrono::{DateTime, Utc};
use serde::Serialize;
use std::{net::Ipv4Addr, str::FromStr};
use strum_macros::Display;

#[derive(Debug, Display, PartialEq, Serialize)]
#[strum(serialize_all = "lowercase")]
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
    pub analyzer_id: String,
}

impl Nongrata {
    pub fn new(
        reason: String,
        rule_id: String,
        ttl: DateTime<Utc>,
        restriction_type: String,
        restriction_value: String,
        is_global: bool,
        analyzer_id: String,
    ) -> Self {
        Self {
            rule_id,
            restriction_type,
            reason,
            restriction_value,
            expires_at: ttl,
            is_global,
            analyzer_id,
        }
    }
}

const SEPARATOR: &str = " and ";

pub fn form_firewall_rule_expression(
    ip: Option<Ipv4Addr>,
    ua: Option<String>,
) -> Result<String, ServerError> {
    let mut ss = vec![];

    if ua.is_none() && ip.is_none() {
        return Err(ServerError::BadRequest(
            "Empty fields, at least one field is required: 'ip', 'user_agent'".into(),
        ));
    }

    if let Some(ua) = ua {
        if !ua.is_empty() {
            ss.push(format!("http.user_agent eq \"{}\"", ua));
        } else {
            return Err(ServerError::BadRequest("Empty 'user_agent' field".into()));
        }
    }

    if let Some(ip) = ip {
        ss.push(format!("ip.src eq {}", ip));
    }

    Ok(ss.join(SEPARATOR))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_form_firewall_rule_expression() {
        assert_eq!(
            form_firewall_rule_expression(
                Some(Ipv4Addr::from_str("192.168.0.1").unwrap()),
                Some(String::from(
                    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)"
                ))
            ).unwrap(),
            String::from("http.user_agent eq \"Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)\" and ip.src eq 192.168.0.1")
        );

        assert_eq!(
            form_firewall_rule_expression(Some(Ipv4Addr::from_str("192.168.0.1").unwrap()), None,)
                .unwrap(),
            String::from("ip.src eq 192.168.0.1")
        );

        assert_eq!(
            form_firewall_rule_expression(
                None,
                Some(String::from(
                    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)"
                ))
            )
            .unwrap(),
            String::from("http.user_agent eq \"Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)\"")
        );
    }
}
