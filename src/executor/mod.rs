pub mod dry;
pub mod models;
pub mod op;

use std::sync::atomic::{AtomicBool, Ordering};

pub use dry::*;
pub use models::*;
pub use op::*;

use crate::errors;
use actix_web::{web, HttpResponse};

pub async fn ban_according_to_mode(
    req: web::Json<models::BlockRequest>,
    op_service: web::Data<ExecutorService>,
    dry_service: web::Data<ExecutorServiceDry>,
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
        Some(res) => res.form_http_response(),
        None => HttpResponse::NoContent().finish(),
    }
}

pub async fn unban_according_to_mode(
    req: web::Json<models::UnblockRequest>,
    op_service: web::Data<ExecutorService>,
    dry_service: web::Data<ExecutorServiceDry>,
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
        Some(res) => res.form_http_response(),
        None => HttpResponse::NoContent().finish(),
    }
}
