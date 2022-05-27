use crate::errors;
use crate::executor;
use actix_web::{web, HttpResponse};
use std::sync::atomic::{AtomicBool, Ordering};

pub async fn ban_according_to_mode(
    req: web::Json<executor::models::BlockRequest>,
    op_service: web::Data<executor::ExecutorService>,
    dry_service: web::Data<executor::ExecutorServiceDry>,
    is_dry: web::Data<AtomicBool>,
) -> HttpResponse {
    let block_request = req.0;
    let restriction_result: Option<errors::ServerError>;

    if is_dry.load(Ordering::Relaxed) {
        restriction_result = dry_service.ban(block_request).await;
    } else {
        restriction_result = op_service.ban(block_request).await;
    }
    match restriction_result {
        Some(res) => res.into(),
        None => HttpResponse::NoContent().finish(),
    }
}

pub async fn unban_according_to_mode(
    req: web::Json<executor::models::UnblockRequest>,
    op_service: web::Data<executor::ExecutorService>,
    dry_service: web::Data<executor::ExecutorServiceDry>,
    is_dry: web::Data<AtomicBool>,
) -> HttpResponse {
    let unblock_request = req.0;
    let restriction_result: Option<errors::ServerError>;

    if is_dry.load(Ordering::Relaxed) {
        restriction_result = dry_service.unban(unblock_request).await;
    } else {
        restriction_result = op_service.unban(unblock_request).await;
    }
    match restriction_result {
        Some(res) => res.into(),
        None => HttpResponse::NoContent().finish(),
    }
}
