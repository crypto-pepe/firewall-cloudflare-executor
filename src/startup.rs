use crate::cloudflare_client::CloudflareClient;
use crate::configuration;
use crate::handlers;
use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;
pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    pub async fn build(configuration: configuration::Settings) -> Result<Self, anyhow::Error> {
        let cloudflare_client = configuration.cloudflare.client();

        let address = format!(
            "{}:{}",
            configuration.server.host, configuration.server.port
        );
        let listener = TcpListener::bind(&address)?;
        let port = listener.local_addr().unwrap().port();
        let server = run(listener, cloudflare_client).await?;
        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}
async fn run(listener: TcpListener, client: CloudflareClient) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/healthcheck", web::get().to(handlers::healthcheck))
            .route("/api/ban", web::post().to(handlers::ban))
            .app_data(client.clone())
    })
    .listen(listener)?
    .run();
    Ok(server)
}
