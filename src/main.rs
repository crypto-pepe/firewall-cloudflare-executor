#[macro_use]
extern crate diesel;

pub mod cloudflare_client;
pub mod configuration;
pub mod errors;
pub mod handlers;
pub mod models;
pub mod schema;
pub mod startup;

use std::fmt::{Debug, Display};
use tokio::task::JoinError;
use tracing::{error, info};

const DEFAULT_CFG_PATH: &str = "./config.yaml";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let configuration =
        configuration::get_config(DEFAULT_CFG_PATH).expect("Failed to read configuration.");
    let application = startup::Application::build(configuration.clone()).await?;
    let application_task = tokio::spawn(application.run_until_stopped());
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
        }
        Ok(Err(e)) => {
            error!("{} failed with: {}", task_name, e);
        }
        Err(e) => {
            error!("{} failed to complete with: {}", task_name, e);
        }
    }
}
