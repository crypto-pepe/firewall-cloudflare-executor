use crate::errors;
use crate::executor;
use crate::executor::Executor;
use crate::handlers;
use actix_web::{web, HttpRequest, HttpResponse};
use std::sync::atomic::{AtomicBool, Ordering};

const ANALYZER_ID_HEADER: &str = "X-Analyzer-Id";

pub async fn ban_according_to_mode(
    http_req: HttpRequest,
    req: web::Json<executor::models::BlockRequest>,
    op_service: web::Data<executor::ExecutorService>,
    dry_run_service: web::Data<executor::ExecutorServiceDryRun>,
    is_dry_run: web::Data<AtomicBool>,
) -> HttpResponse {
    let block_request = req.0;
    if block_request.ttl == 0 {
        return errors::ServerError::MissingTTL.into();
    }
    let restriction_result: Result<(), errors::ServerError>;
    let analyzer_id = get_x_analyzer_id_header(&http_req);

    let analyzer_id = match analyzer_id {
        Some(analyzer_id) => {
            if analyzer_id.is_empty() {
                return handlers::models::bad_request(format!(
                    "Empty {} header",
                    ANALYZER_ID_HEADER
                ));
            }
            analyzer_id
        }
        None => {
            return handlers::models::bad_request(format!("Empty {} header", ANALYZER_ID_HEADER));
        }
    };
    if is_dry_run.load(Ordering::Relaxed) {
        restriction_result = dry_run_service.ban(block_request, analyzer_id.into()).await;
    } else {
        restriction_result = op_service.ban(block_request, analyzer_id.into()).await;
    }
    match restriction_result {
        Ok(()) => HttpResponse::NoContent().finish(),
        Err(e) => e.into(),
    }
}

pub async fn unban_according_to_mode(
    req: web::Json<executor::models::UnblockRequest>,
    op_service: web::Data<executor::ExecutorService>,
    dry_run_service: web::Data<executor::ExecutorServiceDryRun>,
    is_dry_run: web::Data<AtomicBool>,
) -> HttpResponse {
    let unblock_request = req.0;
    let restriction_result: Result<(), errors::ServerError>;

    if is_dry_run.load(Ordering::Relaxed) {
        restriction_result = dry_run_service.unban(unblock_request).await;
    } else {
        restriction_result = op_service.unban(unblock_request).await;
    }
    match restriction_result {
        Ok(()) => HttpResponse::NoContent().finish(),
        Err(e) => e.into(),
    }
}

fn get_x_analyzer_id_header(req: &HttpRequest) -> Option<&str> {
    req.headers().get(ANALYZER_ID_HEADER)?.to_str().ok()
}
