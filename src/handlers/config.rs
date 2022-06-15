use crate::errors;
use crate::handlers;
use actix_web::{web, HttpResponse};
use tokio::sync::Mutex;
use tracing::warn;
use tracing_subscriber::fmt::Formatter;
use tracing_subscriber::reload::Handle;
use tracing_subscriber::EnvFilter;

pub async fn config(
    is_dry_run: web::Data<Mutex<bool>>,
    log_level: web::Data<Handle<EnvFilter, Formatter>>,
    q: web::Query<handlers::models::AdminRequest>,
) -> HttpResponse {
    let mut dry_run = is_dry_run.lock().await;
    if let Some(dry_run_switch) = q.dry_run {
        *dry_run = dry_run_switch;
    }
    warn!(
        "Dry-run mode is now {}",
        if *dry_run { "ON" } else { "OFF" }
    );
    if let Some(log_lvl) = q.log_level.clone() {
        if log_level.modify(|e| *e = EnvFilter::new(log_lvl)).is_err() {
            return errors::ServerError::WrongLogLevel.into();
        }
    }

    HttpResponse::Ok().finish()
}
