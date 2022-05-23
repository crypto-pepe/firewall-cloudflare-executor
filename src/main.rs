#[macro_use]
extern crate diesel;

pub mod cloudflare_client;
pub mod configuration;
pub mod errors;
pub mod handlers;
pub mod models;
pub mod schema;
pub mod startup;
pub mod telemetry;

use std::{
    fmt::{Debug, Display},
    process,
};
use tokio::task::JoinError;
use tracing::{error, info};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing::info!("start application");

    let configuration = configuration::get_config(configuration::DEFAULT_CFG_PATH)
        .expect("Failed to read configuration.");
    let (subscriber, log_filter_handler) = telemetry::get_subscriber(&configuration);
    let application =
        startup::Application::build(configuration.clone(), log_filter_handler.clone()).await?;
    let application_task = tokio::spawn(application.run_until_stopped());
    telemetry::init_subscriber(subscriber);
    info!("cloudflare-executor is up!");
    tokio::select! {
        o = application_task => report_exit("API", o),
    };
    Ok(())
}

fn report_exit(task_name: &str, outcome: Result<Result<(), impl Debug + Display>, JoinError>) {
    match outcome {
        Ok(Ok(())) => {
            error!("{} has exited", task_name);
            process::exit(0)
        }
        Ok(Err(e)) => {
            error!("{} failed with: {}", task_name, e);
            process::exit(1)
        }
        Err(e) => {
            error!("{} failed to complete with: {}", task_name, e);
            process::exit(1)
        }
    }
}
