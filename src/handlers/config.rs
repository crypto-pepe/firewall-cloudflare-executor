use crate::handlers;

use actix_web::{web, HttpResponse};
use std::sync::Mutex;
use tracing::warn;
use tracing_subscriber::fmt::Formatter;
use tracing_subscriber::reload::Handle;
use tracing_subscriber::EnvFilter;

pub async fn config(
    is_dry_run: web::Data<Mutex<bool>>,
    log_level: web::Data<Handle<EnvFilter, Formatter>>,
    q: web::Query<handlers::models::AdminRequest>,
) -> HttpResponse {
    let dry_run_status = is_dry_run.lock();
    match dry_run_status {
        Ok(mut s) => {
            if let Some(dry_run) = q.dry_run {
                *s = dry_run;
            }
            warn!(
                "Alert! Dry-run mode is now {}",
                if *s { "ON" } else { "OFF" }
            );
        }
        Err(_) => return HttpResponse::BadRequest().finish(),
    };
    if let Some(log_lvl) = q.log_level.clone() {
        if log_level.modify(|e| *e = EnvFilter::new(log_lvl)).is_err() {
            return HttpResponse::InternalServerError().finish();
        }
    }

    HttpResponse::Ok().finish()
}
