#[macro_use]
extern crate diesel;

pub mod cloudflare_client;
pub mod configuration;
pub mod errors;
pub mod executor;
pub mod handlers;
pub mod invalidator;
pub mod models;
pub mod pool;
pub mod schema;
pub mod startup;
pub mod telemetry;

use std::process;

use tracing::{error, info};

#[tokio::main]
async fn main() {
    tracing::info!("start application");

    let configuration = configuration::get_config(configuration::DEFAULT_CFG_PATH)
        .expect("Failed to read configuration.");
    let (subscriber, log_filter_handler) = telemetry::get_subscriber(&configuration.clone());
    let cloudflare_client = configuration.clone().cloudflare.client();
    let pool = pool::get_db_pool(configuration.db.clone().pg_conn_string())
        .await
        .expect("failed to create pool");
    let application = startup::Application::build(
        configuration.clone(),
        log_filter_handler.clone(),
        cloudflare_client.clone(),
        pool.clone(),
    )
    .await
    .expect("Failed to build application");
    let invalidator = invalidator::Invalidator::new(
        cloudflare_client,
        pool,
        configuration.cloudflare.invalidation_timeout.into(),
    );
    telemetry::init_subscriber(subscriber);

    info!("cloudflare-executor is up!");
    let server_task = tokio::spawn(application.run_until_stopped());
    let invalidator_task = tokio::spawn(invalidator.run_untill_stopped());

    tokio::select! {
        server_exit = server_task => match server_exit {
            Err(e) => {
                error!("Cloudflare-executor failed with {}", e);
                process::exit(1);
            }
            Ok(Ok(())) => process::exit(0),
            Ok(Err(e))  => {
                error!("Cloudflare-executor failed with {}", e);
                process::exit(2);
            }
        },
        invalidator_exit = invalidator_task => match invalidator_exit{
            Err(e) => {
                error!("Cloudflare-invalidator failed with {}", e);
                process::exit(1);
            }
            Ok(Ok(()))  => process::exit(0),
            Ok(Err(e))  => {
                error!("Cloudflare-invalidator failed with {}", e);
                process::exit(2);
            }
        }
    };
}
