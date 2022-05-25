use crate::errors;
use crate::handlers;

use actix_web::{web, HttpResponse};

use tracing::{error, info};

#[derive(Clone)]
pub struct ExecutorServiceDry {}
impl ExecutorServiceDry {
    pub fn new() -> Self {
        Self {}
    }
    pub async fn ban(
        &self,
        req: web::Json<handlers::models::BlockRequest>,
    ) -> Result<HttpResponse, errors::ServerError> {
        info!("Incoming request:{:?}", req.clone());
        match req.target.ip.clone() {
            Some(ip) => match req.target.ua.clone() {
                Some(ua) => info!("gonna ban {:?}{:?}", ua, ip),
                None => info!("gonna ban {:?} by IP", ip),
            },
            None => match req.target.ua.clone() {
                Some(ua) => info!("gonna ban {:?}", ua),
                None => {
                    error!("Empty request");
                    return Ok(HttpResponse::BadRequest().finish());
                }
            },
        }
        Ok(HttpResponse::NoContent().finish())
    }
    pub async fn unban(
        &self,
        req: web::Json<handlers::models::UnblockRequest>,
    ) -> Result<HttpResponse, errors::ServerError> {
        info!("Incoming request:{:?}", req.clone());
        match req.target.ip.clone() {
            Some(ip) => match req.target.ua.clone() {
                Some(ua) => info!("gonna unban {:?}{:?}", ua, ip),
                None => info!("gonna unban {:?} by IP", ip),
            },
            None => match req.target.ua.clone() {
                Some(ua) => info!("gonna unban {:?}", ua),
                None => {
                    error!("Empty request");
                    return Ok(HttpResponse::BadRequest().finish());
                }
            },
        }
        Ok(HttpResponse::NoContent().finish())
    }
}
