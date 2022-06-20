use crate::cloudflare_client::CloudflareClient;
use pepe_config::{self, DurationString};
use serde::{Deserialize, Serialize};

pub const DEFAULT_CFG_PATH: &str = include_str!("../config.yaml");

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Settings {
    pub server: ServerConfig,
    pub cloudflare: CloudflareClientConfig,
    pub db: DbConfig,
    pub tracing: TracingConfig,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub dry_run: bool,
}
impl ServerConfig {
    pub fn get_address(self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CloudflareClientConfig {
    pub base_url: String,
    pub account_id: String,
    pub zone_id: String,
    pub token: String,
    pub invalidation_timeout: DurationString,
}

impl CloudflareClientConfig {
    pub fn client(self) -> CloudflareClient {
        CloudflareClient::new(self.base_url, self.token, self.zone_id)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DbConfig {
    user: String,
    password: String,
    db: String,
    host: String,
    port: String,
}
impl DbConfig {
    pub fn pg_conn_string(self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.user, self.password, self.host, self.port, self.db
        )
    }
}
pub fn get_config(path: &str) -> Result<Settings, pepe_config::ConfigError> {
    pepe_config::load(path, pepe_config::FileFormat::Yaml)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TracingConfig {
    pub svc_name: String,
    pub jaeger_endpoint: Option<String>,
}
