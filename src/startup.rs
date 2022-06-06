use crate::cloudflare_client::CloudflareClient;
use crate::configuration;
use crate::executor;
use crate::handlers;
use crate::handlers::bans;
use crate::models;

use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use bb8::Pool;
use std::net::TcpListener;
use std::sync::atomic::AtomicBool;
use tracing::info;
use tracing_actix_web::TracingLogger;
use tracing_subscriber::fmt::Formatter;
use tracing_subscriber::reload::Handle;
use tracing_subscriber::EnvFilter;

pub struct Application {
    server: Server,
}

impl Application {
    pub async fn build(
        configuration: configuration::Settings,
        log_level_handle: Handle<EnvFilter, Formatter>,
        cloudflare_client: CloudflareClient,
        pool: Pool<models::DbConn>,
    ) -> Result<Self, anyhow::Error> {
        let server_addr = configuration.server.get_address();
        let listener = TcpListener::bind(&server_addr)?;

        let executor_service_op = executor::ExecutorService::new(cloudflare_client, pool);
        let executor_service_dry = executor::ExecutorServiceDry::new();
        let server = run(
            listener,
            log_level_handle,
            executor_service_op.clone(),
            executor_service_dry.clone(),
        )
        .await?;
        info!("server is running on: {:?}", server_addr);
        Ok(Self { server })
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}
async fn run(
    listener: TcpListener,
    log_level_handle: Handle<EnvFilter, Formatter>,
    executor_service_op: executor::ExecutorService,
    executor_service_dry: executor::ExecutorServiceDry,
) -> Result<Server, std::io::Error> {
    let is_dry = web::Data::new(AtomicBool::new(false));

    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/healthcheck", web::get().to(handlers::healthcheck))
            .route("/api/config", web::post().to(handlers::config))
            .route("/api/bans", web::post().to(bans::ban_according_to_mode))
            .route("/api/bans", web::delete().to(bans::unban_according_to_mode))
            .app_data(web::Data::new(log_level_handle.clone()))
            .app_data(web::Data::new(executor_service_op.clone()))
            .app_data(web::Data::new(executor_service_dry.clone()))
            .app_data(web::Data::new(is_dry.clone()))
    })
    .listen(listener)?
    .run();
    Ok(server)
}
