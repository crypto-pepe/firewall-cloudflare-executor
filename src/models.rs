use crate::{errors::ServerError, schema::filters, schema::nongratas};
use chrono::{DateTime, Utc};

use diesel::Queryable;
use serde::Serialize;
use std::{net::IpAddr, str::FromStr};
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

#[derive(PartialEq, diesel_derive_enum::DbEnum, Debug, Clone)]
#[DieselType = "Filter_type"]
#[DbValueStyle = "SCREAMING_SNAKE_CASE"]
pub enum FilterType {
    IP,
    UserAgent,
    IPUserAgent,
    Unset,
}

impl ToString for FilterType {
    fn to_string(&self) -> String {
        match self {
            FilterType::IP => String::from("ip"),
            FilterType::UserAgent => String::from("user_agent"),
            FilterType::IPUserAgent => String::from("ip_user_agent"),
            FilterType::Unset => String::from("unset"),
        }
    }
}

const SEPARATOR: &str = " and ";

#[derive(Insertable, Queryable, Clone, Debug)]
#[table_name = "filters"]
pub struct Filter {
    pub id: String,
    pub rule_id: String,
    pub filter_type: FilterType,
    pub expression: String,
}

impl Filter {
    pub fn from_expression(id: String, expr: String) -> Self {
        Self {
            id,
            rule_id: "".into(),
            filter_type: FilterType::Unset,
            expression: expr,
        }
    }

    pub fn new(ip: Option<IpAddr>, ua: Option<String>) -> Result<Self, ServerError> {
        let mut expression = vec![];
        let filter_type: FilterType;

        if let Some(ua) = ua.clone() {
            if ua.is_empty() {
                return Err(ServerError::BadRequest("Empty 'user_agent' field".into()));
            }
        }

        match (ua, ip) {
            (None, None) => {
                return Err(ServerError::BadRequest(
                    "Empty fields, at least one field is required: 'ip', 'user_agent'".into(),
                ));
            }
            (Some(ua), None) => {
                expression.push(format!("(http.user_agent eq \"{}\")", ua));
                filter_type = FilterType::UserAgent;
            }
            (None, Some(ip)) => {
                expression.push(format!("(ip.src eq {})", ip));
                filter_type = FilterType::IP;
            }
            (Some(ua), Some(ip)) => {
                expression.push(format!("(ip.src eq {}", ip));
                expression.push(format!("http.user_agent eq \"{}\")", ua));
                filter_type = FilterType::IPUserAgent;
            }
        }

        let expression = expression.join(SEPARATOR);

        Ok(Self {
            id: "".into(),
            rule_id: "".into(),
            filter_type,
            expression,
        })
    }

    pub fn append(&mut self, to_append: Filter) -> Result<(), ServerError> {
        if self.filter_type.to_string() == to_append.filter_type.to_string() {
            self.expression = format!(
                "{} or {}",
                self.expression.trim(),
                to_append.expression.trim()
            );
            return Ok(());
        }
        Err(ServerError::WrongFilter)
    }
    pub fn trim_expression(&mut self, trim_filter: Filter) -> Result<(), ServerError> {
        if self.filter_type.to_string() == trim_filter.filter_type.to_string() {
            if self.expression.contains("or") {
                let trim_expression = format!("or {}", trim_filter.expression.trim());
                self.expression = self.expression.replace(&trim_expression.trim(), "");
            }
            self.expression = self.expression.replace(&*trim_filter.expression.trim(), "");
            self.expression = self
                .expression
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ");
            return Ok(());
        }
        Err(ServerError::WrongFilter)
    }
    pub fn already_includes_filter(&mut self, new_filter: Filter) -> Result<bool, ServerError> {
        if self.filter_type.to_string() == new_filter.filter_type.to_string() {
            let expression = format!("{}", new_filter.expression.trim());
            let contains = self.expression.contains(expression.as_str());
            let equals = self.expression == expression;

            return Ok(contains || equals);
        }
        Err(ServerError::WrongFilter)
    }
    pub fn is_empty(self) -> bool {
        self.expression.is_empty()
    }
}

#[derive(Insertable, Identifiable, Queryable, Debug, Clone)]
#[diesel(primary_key(id))]
#[table_name = "nongratas"]
pub struct Nongrata {
    #[diesel(deserialize_as = "i64")]
    pub id: Option<i64>,
    pub filter_id: String,
    pub reason: String,
    pub restriction_value: String,
    pub restriction_type: String,
    pub expires_at: DateTime<Utc>,
    pub is_global: bool,
    pub analyzer_id: String,
}

impl Nongrata {
    pub fn new(
        reason: String,
        filter_id: String,
        ttl: DateTime<Utc>,
        restriction_value: String,
        restriction_type: String,
        is_global: bool,
        analyzer_id: String,
    ) -> Self {
        Self {
            id: None,
            reason,
            filter_id,
            restriction_value,
            restriction_type,
            expires_at: ttl,
            is_global,
            analyzer_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_form_firewall_filter_expression() {
        assert_eq!(
            Filter::new(
                Some(IpAddr::from_str("192.168.0.1").unwrap()),
                Some(String::from(
                    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)"
                ))
            ).unwrap().expression,
            String::from("(ip.src eq 192.168.0.1 and http.user_agent eq \"Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)\")")
        );

        assert_eq!(
            Filter::new(Some(IpAddr::from_str("192.168.0.1").unwrap()), None,)
                .unwrap()
                .expression,
            String::from("(ip.src eq 192.168.0.1)")
        );

        assert_eq!(
            Filter::new(
                None,
                Some(String::from(
                    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)"
                ))
            )
            .unwrap()
            .expression,
            String::from(
                "(http.user_agent eq \"Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)\")"
            )
        );
    }

    #[test]
    fn test_append_filter() {
        let mut filter = Filter::new(
            Some(IpAddr::from_str("192.168.0.1").unwrap()),
            Some(String::from(
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)",
            )),
        )
        .unwrap();
        let filter2 = Filter::new(
            Some(IpAddr::from_str("1.1.1.1").unwrap()),
            Some(String::from("SOME_USER_AGENT")),
        )
        .unwrap();

        filter.append(filter2).expect("");

        assert_eq!(
            filter.expression,
            String::from("(ip.src eq 192.168.0.1 and http.user_agent eq \"Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)\") or (ip.src eq 1.1.1.1 and http.user_agent eq \"SOME_USER_AGENT\")")
        );
    }
    #[test]
    fn test_trim_filter() {
        let mut filter = Filter::new(
            Some(IpAddr::from_str("192.168.0.1").unwrap()),
            Some(String::from("Chrome-ua")),
        )
        .unwrap();
        let filter2 = Filter::new(
            Some(IpAddr::from_str("1.1.1.1").unwrap()),
            Some(String::from("SOME_USER_AGENT")),
        )
        .unwrap();
        let mut filter_ip = Filter::new(Some(IpAddr::from_str("1.1.1.1").unwrap()), None).unwrap();
        let filter_ip2 = Filter::new(Some(IpAddr::from_str("5.5.5.5").unwrap()), None).unwrap();
        let mut filter_long =
            Filter::new(Some(IpAddr::from_str("2.3.4.5").unwrap()), None).unwrap();

        filter.append(filter2.clone()).expect("");
        filter.trim_expression(filter2).expect("");
        filter_ip.append(filter_ip2.clone()).expect("");
        filter_ip.trim_expression(filter_ip2.clone()).expect("");
        filter_long.append(filter_ip.clone()).expect("");
        filter_long.append(filter_ip2.clone()).expect("");
        filter_long.append(filter_ip.clone()).expect("");
        filter_long.trim_expression(filter_ip2).expect("");

        assert_eq!(
            filter.expression,
            String::from("(ip.src eq 192.168.0.1 and http.user_agent eq \"Chrome-ua\")")
        );
        assert_eq!(filter_ip.expression, String::from("(ip.src eq 1.1.1.1)"));
        assert_eq!(
            filter_long.expression,
            String::from("(ip.src eq 2.3.4.5) or (ip.src eq 1.1.1.1) or (ip.src eq 1.1.1.1)")
        );
    }

    #[test]
    fn test_already_contains() {
        let mut filter = Filter::new(
            Some(IpAddr::from_str("192.168.0.1").unwrap()),
            Some(String::from(
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)",
            )),
        )
        .unwrap();
        let filter2 = Filter::new(
            Some(IpAddr::from_str("1.1.1.1").unwrap()),
            Some(String::from("SOME_USER_AGENT")),
        )
        .unwrap();

        filter.append(filter2.clone()).expect("");

        assert_eq!(filter.already_includes_filter(filter2).unwrap(), true);
    }
}
