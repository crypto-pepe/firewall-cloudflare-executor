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
