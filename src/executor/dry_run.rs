use crate::errors;
use crate::errors::ServerError;
use crate::executor::models::{BlockRequest, Executor, UnblockRequest};
use crate::models;
use crate::models::Filter;

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
        let filter = models::Filter::new(ip, ua)?;
        info!(
            "Going to apply BAN rule: {:?}\n Analyzer: {:?}",
            filter.expression, analyzer_id,
        );
        return Ok(());
    }
    async fn unban(&self, unblock_request: UnblockRequest) -> Result<(), errors::ServerError> {
        info!("Incoming request:{:?}", unblock_request);
        let ua = unblock_request.target.user_agent;
        let ip = unblock_request.target.ip;
        let filter = models::Filter::new(ip, ua)?;
        info!("Going apply UNBAN rule: {:?}", filter.expression);
        return Ok(());
    }

    async fn create_filter(&self, _filter: &mut Filter) -> Result<(), ServerError> {
        Ok(())
    }

    async fn create_rule(
        &self,
        _block_request: BlockRequest,
        _filter: Filter,
        _analyzer_id: String,
    ) -> Result<(), ServerError> {
        Ok(())
    }

    async fn update_filter(
        &self,
        _block_request: BlockRequest,
        _old_filter: Filter,
        _new_filter: Filter,
        _analyzer_id: String,
    ) -> Result<(), ServerError> {
        Ok(())
    }
    async fn find_filter(&self, _filter: Filter) -> Result<Option<Filter>, ServerError> {
        Ok(None)
    }
}
