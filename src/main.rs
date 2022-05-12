pub mod cloudflare_client;
pub mod configuration;
pub mod errors;
pub mod handlers;
pub mod models;
pub mod startup;

use std::fmt::{Debug, Display};
use tokio::task::JoinError;

const DEFAULT_CFG_PATH: &str = "./config.yaml";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let configuration =
        configuration::get_config(DEFAULT_CFG_PATH).expect("Failed to read configuration.");

    let application = startup::Application::build(configuration.clone()).await?;
    let application_task = tokio::spawn(application.run_until_stopped());

    tokio::select! {
        o = application_task => report_exit("API", o),
    };

    Ok(())
}

fn report_exit(task_name: &str, outcome: Result<Result<(), impl Debug + Display>, JoinError>) {
    match outcome {
        Ok(Ok(())) => {
            pepe_log::info!("{} has exited", task_name);
        }
        Ok(Err(e)) => {
            pepe_log::error!("{} failed with: {}", task_name, e);
        }
        Err(e) => {
            pepe_log::error!("{} failed to complete with: {}", task_name, e);
        }
    }
}
