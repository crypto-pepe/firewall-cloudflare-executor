use config::ConfigError;
use pepe_config;
use serde::{Deserialize, Serialize};

use crate::cloudflare_client::CloudflareClient;

#[derive(Serialize, Deserialize, Clone)]
pub struct Settings {
    pub server: ServerConfig,
    pub cloudflare: CloudflareClientConfig,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CloudflareClientConfig {
    base_url: String,
    account_id: String,
    zone_id: String,
    token: String,
}

pub fn get_config(path: &str) -> Result<Settings, ConfigError> {
    pepe_config::load(path, config::FileFormat::Json)
}

impl CloudflareClientConfig {
    pub fn client(self) -> CloudflareClient {
        CloudflareClient::new(self.base_url, self.token, self.account_id, self.zone_id)
    }
}
