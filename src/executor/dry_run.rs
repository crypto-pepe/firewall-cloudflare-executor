use crate::errors;
use crate::errors::ServerError;
use crate::executor::*;
use crate::models;

use async_trait::async_trait;
use tracing::info;

#[derive(Clone, Default)]
pub struct ExecutorServiceDryRun {}
impl ExecutorServiceDryRun {
    pub fn new() -> Self {
        Self {}
    }
}
#[async_trait]
impl Executor for ExecutorServiceDryRun {
    async fn ban(
        &self,
        block_request: BlockRequest,
        analyzer_id: String,
    ) -> Result<(), errors::ServerError> {
        info!("Incoming request:{:?}", block_request);
        let ua = block_request.target.user_agent;
        let ip = block_request.target.ip;
        if block_request.ttl == 0 {
            return Err(errors::ServerError::MissingTTL);
        }
        let rule = models::form_firewall_rule_expression(ip, ua);
        rule.clone().ok_or(ServerError::MissingTarget)?;
        info!(
            "Going to apply BAN rule: {:?}\n Analyzer: {:?}",
            rule, analyzer_id,
        );
        return Ok(());
    }
    async fn unban(&self, unblock_request: UnblockRequest) -> Result<(), errors::ServerError> {
        info!("Incoming request:{:?}", unblock_request);
        let ua = unblock_request.target.user_agent;
        let ip = unblock_request.target.ip;
        let rule = models::form_firewall_rule_expression(ip, ua);
        rule.clone().ok_or(ServerError::MissingTarget)?;
        info!("Going apply UNBAN rule: {:?}", rule);
        return Ok(());
    }
}
