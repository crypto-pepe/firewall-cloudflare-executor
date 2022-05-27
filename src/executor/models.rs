use serde_derive::Deserialize;
use serde_derive::Serialize;
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlockRequest {
    pub target: Target,
    pub reason: String,
    pub ttl: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnblockRequest {
    pub target: Target,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlockResponse {
    pub type_field: String,
    pub value: String,
    pub reason: String,
    pub ttl: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Target {
    pub ip: Option<String>,
    pub ua: Option<String>,
}
