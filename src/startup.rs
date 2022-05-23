use crate::cloudflare_client::CloudflareClient;
use crate::configuration;
use crate::handlers;

use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use bb8::ManageConnection;
use bb8_diesel::{DieselConnection, DieselConnectionManager};
use diesel::PgConnection;
use std::net::TcpListener;
use std::sync::atomic::AtomicBool;
use tracing::info;
use tracing_actix_web::TracingLogger;
use tracing_subscriber::fmt::Formatter;
use tracing_subscriber::reload::Handle;
use tracing_subscriber::EnvFilter;

pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    pub async fn build(
        configuration: configuration::Settings,
        log_level_handle: Handle<EnvFilter, Formatter>,
    ) -> Result<Self, anyhow::Error> {
        let cloudflare_client = configuration.cloudflare.client();
        let db_conn_string = configuration.db.pg_conn_string();
        let pg_mgr = DieselConnectionManager::<DieselConnection<PgConnection>>::new(db_conn_string);
        let pool = bb8::Pool::builder().build(pg_mgr).await?;
        let server_addr = configuration.server.get_address();
        let listener = TcpListener::bind(&server_addr)?;

        let port = listener.local_addr()?.port();
        let server = run(listener, cloudflare_client, pool, log_level_handle).await?;
        info!("server is running on: {:?}", server_addr);
        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}
async fn run<T: ManageConnection>(
    listener: TcpListener,
    client: CloudflareClient,
    db_pool: bb8::Pool<T>,
    log_level_handle: Handle<EnvFilter, Formatter>,
) -> Result<Server, std::io::Error> {
    let is_dry = web::Data::new(AtomicBool::new(false));
    let log_level = web::Data::new(log_level_handle);

    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/healthcheck", web::get().to(handlers::healthcheck))
            .route("/api/bans", web::post().to(handlers::ban))
            .route("/api/bans", web::delete().to(handlers::unban))
            .route("/api/dry", web::post().to(handlers::config))
            .app_data(client.clone())
            .app_data(db_pool.clone())
            .app_data(is_dry.clone())
            .app_data(log_level.clone())
    })
    .listen(listener)?
    .run();
    Ok(server)
}
