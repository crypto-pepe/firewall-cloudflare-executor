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

#[derive(Insertable, Queryable, Clone)]
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
                expression.push(format!("http.user_agent eq \"{}\"", ua));
                filter_type = FilterType::UserAgent;
            }
            (None, Some(ip)) => {
                expression.push(format!("ip.src eq {}", ip));
                filter_type = FilterType::IP;
            }
            (Some(ua), Some(ip)) => {
                expression.push(format!("ip.src eq {}", ip));
                expression.push(format!("http.user_agent eq \"{}\"", ua));
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
            self.expression = format!("{} or ({})", self.expression, to_append.expression);
            return Ok(());
        }
        Err(ServerError::WrongFilter)
    }
    pub fn trim_expression(&mut self, trim_filter: Filter) -> Result<(), ServerError> {
        if self.filter_type.to_string() == trim_filter.filter_type.to_string() {
            let trim_expression = format!("or ({})", trim_filter.expression);
            self.expression = self.expression.replace(&trim_expression, "");
            return Ok(());
        }
        Err(ServerError::WrongFilter)
    }
}

#[derive(Insertable, Queryable, Debug, Clone)]
#[table_name = "nongratas"]
pub struct Nongrata {
    pub id: i64,
    pub filter_id: String,
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
        filter_id: String,
        ttl: DateTime<Utc>,
        restriction_type: String,
        restriction_value: String,
        is_global: bool,
        analyzer_id: String,
    ) -> Self {
        Self {
            id: 0,
            reason,
            filter_id,
            restriction_type,
            restriction_value,
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
            String::from("http.user_agent eq \"Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)\" and ip.src eq 192.168.0.1")
        );

        assert_eq!(
            Filter::new(Some(IpAddr::from_str("192.168.0.1").unwrap()), None,)
                .unwrap()
                .expression,
            String::from("ip.src eq 192.168.0.1")
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
            String::from("http.user_agent eq \"Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)\"")
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
            String::from("http.user_agent eq \"Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)\" and ip.src eq 192.168.0.1 or (http.user_agent eq \"SOME_USER_AGENT\" and ip.src eq 1.1.1.1)")
        );
    }
}
