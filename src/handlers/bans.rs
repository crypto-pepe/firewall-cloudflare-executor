use crate::errors;
use crate::executor;
use crate::executor::Executor;
use crate::handlers;
use actix_web::{web, HttpRequest, HttpResponse};
use std::sync::atomic::{AtomicBool, Ordering};

pub async fn ban_according_to_mode(
    http_req: HttpRequest,
    req: web::Json<executor::models::BlockRequest>,
    op_service: web::Data<executor::ExecutorService>,
    dry_service: web::Data<executor::ExecutorServiceDry>,
    is_dry: web::Data<AtomicBool>,
) -> HttpResponse {
    let block_request = req.0;
    let restriction_result: Result<(), errors::ServerError>;
    let analyzer_id = get_x_analyzer_id_header(&http_req);

    let analyzer_id = match analyzer_id {
        Some(analyzer_id) => analyzer_id,
        None => {
            return HttpResponse::BadRequest()
                .json(handlers::models::ExecutorResponse::no_analyzer_id());
        }
    };
    if is_dry.load(Ordering::Relaxed) {
        restriction_result = dry_service
            .ban(block_request, String::from(analyzer_id))
            .await;
    } else {
        restriction_result = op_service
            .ban(block_request, String::from(analyzer_id))
            .await;
    }
    match restriction_result {
        Ok(()) => HttpResponse::NoContent().finish(),
        Err(e) => e.into(),
    }
}

pub async fn unban_according_to_mode(
    req: web::Json<executor::models::UnblockRequest>,
    op_service: web::Data<executor::ExecutorService>,
    dry_service: web::Data<executor::ExecutorServiceDry>,
    is_dry: web::Data<AtomicBool>,
) -> HttpResponse {
    let unblock_request = req.0;
    let restriction_result: Result<(), errors::ServerError>;

    if is_dry.load(Ordering::Relaxed) {
        restriction_result = dry_service.unban(unblock_request).await;
    } else {
        restriction_result = op_service.unban(unblock_request).await;
    }
    match restriction_result {
        Ok(()) => HttpResponse::NoContent().finish(),
        Err(e) => e.into(),
    }
}

fn get_x_analyzer_id_header(req: &HttpRequest) -> Option<&str> {
    req.headers().get("X-Analyzer-Id")?.to_str().ok()
}
