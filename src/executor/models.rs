use async_trait::async_trait;
use serde_derive::Deserialize;
use serde_derive::Serialize;

use crate::errors;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlockRequest {
    pub target: Target,
    pub reason: String,
    #[serde(default)]
    pub ttl: u64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnblockRequest {
    pub target: Target,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Target {
    pub ip: Option<String>,
    pub ua: Option<String>,
}

#[async_trait]
pub trait Executor {
    async fn ban(
        &self,
        block_request: BlockRequest,
        analyzer_id: String,
    ) -> Result<(), errors::ServerError>;
    async fn unban(&self, unblock_request: UnblockRequest) -> Result<(), errors::ServerError>;
}
