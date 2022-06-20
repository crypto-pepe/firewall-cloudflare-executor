use std::sync::atomic::AtomicBool;

use crate::errors;
use crate::handlers;
use actix_web::{web, HttpResponse};
use std::sync::atomic::Ordering;
use tracing::warn;
use tracing_subscriber::fmt::Formatter;
use tracing_subscriber::reload::Handle;
use tracing_subscriber::EnvFilter;

pub async fn config(
    is_dry_run: web::Data<AtomicBool>,
    log_level: web::Data<Handle<EnvFilter, Formatter>>,
    req: web::Json<handlers::models::AdminRequest>,
) -> HttpResponse {
    let dry_run = is_dry_run.get_ref();
    if let Some(dry_run_switch) = req.dry_run {
        dry_run.store(dry_run_switch, Ordering::Relaxed)
    }
    warn!(
        "Dry-run mode is now {}",
        if dry_run.load(Ordering::Relaxed) {
            "ON"
        } else {
            "OFF"
        }
    );
    if let Some(log_lvl) = req.log_level.clone() {
        if log_level.modify(|e| *e = EnvFilter::new(log_lvl)).is_err() {
            return errors::ServerError::WrongLogLevel.into();
        }
    }

    HttpResponse::Ok().finish()
}
