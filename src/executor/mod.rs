pub mod dry;
pub mod op;

use std::sync::atomic::{AtomicBool, Ordering};

pub use dry::*;
pub use op::*;

use crate::errors;
use crate::handlers::models;
use actix_web::{web, HttpResponse};

pub async fn ban_according_to_mode(
    req: web::Json<models::BlockRequest>,
    op_service: web::Data<ExecutorService>,
    dry_service: web::Data<ExecutorServiceDry>,
    is_dry: web::Data<AtomicBool>,
) -> Result<HttpResponse, errors::ServerError> {
    if is_dry.load(Ordering::Relaxed) {
        return dry_service.ban(req).await;
    }
    return op_service.ban(req).await;
}
