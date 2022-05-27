use crate::errors;
use crate::errors::ServerError;
use crate::executor::*;

use tracing::{error, info};

#[derive(Clone)]
pub struct ExecutorServiceDry {}
impl ExecutorServiceDry {
    pub fn new() -> Self {
        Self {}
    }
    pub async fn ban(&self, block_request: BlockRequest) -> Option<errors::ServerError> {
        info!("Incoming request:{:?}", block_request);
        match block_request.target.ip.clone() {
            Some(ip) => match block_request.target.ua {
                Some(ua) => info!("gonna ban {:?}{:?}", ua, ip),
                None => info!("gonna ban {:?} by IP", ip),
            },
            None => match block_request.target.ua {
                Some(ua) => info!("gonna ban {:?}", ua),
                None => {
                    error!("Empty request");
                    return Some(ServerError::EmptyRequest);
                }
            },
        }
        None
    }
    pub async fn unban(&self, unblock_request: UnblockRequest) -> Option<errors::ServerError> {
        info!("Incoming request:{:?}", unblock_request);
        match unblock_request.target.ip.clone() {
            Some(ip) => match unblock_request.target.ua {
                Some(ua) => info!("gonna unban {:?}{:?}", ua, ip),
                None => info!("gonna unban {:?} by IP", ip),
            },
            None => match unblock_request.target.ua {
                Some(ua) => info!("gonna unban {:?}", ua),
                None => {
                    error!("Empty request");
                    return Some(ServerError::EmptyRequest);
                }
            },
        }
        None
    }
}

impl Default for ExecutorServiceDry {
    fn default() -> Self {
        Self::new()
    }
}
