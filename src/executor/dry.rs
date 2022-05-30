use crate::errors;
use crate::errors::ServerError;
use crate::executor::*;
use crate::models;

use async_trait::async_trait;
use tracing::info;

#[derive(Clone)]
pub struct ExecutorServiceDry {}
impl ExecutorServiceDry {
    pub fn new() -> Self {
        Self {}
    }
}
#[async_trait]
impl Executor for ExecutorServiceDry {
    async fn ban(&self, block_request: BlockRequest) -> Result<(), errors::ServerError> {
        info!("Incoming request:{:?}", block_request);
        let ua = block_request.target.ua;
        let ip = block_request.target.ip;
        let rule = models::form_firewall_rule_expression(ip.as_ref(), ua.as_ref());
        if rule.is_none() {
            return Err(ServerError::EmptyRequest);
        }
        info!("gonna apply BAN rule: {:?}", Some(rule));
        return Ok(());
    }
    async fn unban(&self, unblock_request: UnblockRequest) -> Result<(), errors::ServerError> {
        info!("Incoming request:{:?}", unblock_request);
        let ua = unblock_request.target.ua;
        let ip = unblock_request.target.ip;
        let rule = models::form_firewall_rule_expression(ip.as_ref(), ua.as_ref());
        if rule.is_none() {
            return Err(ServerError::EmptyRequest);
        }
        info!("gonna apply UNBAN rule: {:?}", Some(rule));
        return Ok(());
    }
}

impl Default for ExecutorServiceDry {
    fn default() -> Self {
        Self::new()
    }
}
