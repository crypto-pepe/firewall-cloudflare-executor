use std::net::IpAddr;

use async_trait::async_trait;
use diesel::r2d2::PooledConnection;
use serde_derive::Deserialize;
use serde_derive::Serialize;

use crate::errors::ServerError;
use crate::models::Filter;
use crate::pool::DbConn;

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
    pub ip: Option<IpAddr>,
    pub user_agent: Option<String>,
}

#[async_trait]
pub trait Executor {
    async fn ban(
        &self,
        block_request: BlockRequest,
        analyzer_id: String,
    ) -> Result<(), ServerError>;
    async fn unban(&self, unblock_request: UnblockRequest) -> Result<(), ServerError>;
    async fn create_rule(
        &self,
        block_request: BlockRequest,
        filter: Filter,
        analyzer_id: String,
    ) -> Result<(), ServerError>;
    async fn find_filter(&self, filter: Filter) -> Result<Vec<Filter>, ServerError>;
    async fn create_filter(&self, filter: Filter) -> Result<String, ServerError>;
    async fn update_filter(
        &self,
        block_request: BlockRequest,
        old_filter: Filter,
        new_filter: Filter,
        analyzer_id: String,
    ) -> Result<(), ServerError>;
}
