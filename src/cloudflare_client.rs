use std::ops::Add;

use crate::errors;
use crate::errors::ServerError;
use crate::models;
use anyhow::Result;
use reqwest::{header, Client};
use tracing::{error, info};

#[derive(Clone)]
pub struct CloudflareClient {
    http_client: Client,
    base_api_url: String,
    zone_id: String,
}

impl CloudflareClient {
    pub fn new(base_api_url: String, token: String, zone_id: String) -> Self {
        let mut hmap = header::HeaderMap::new();
        hmap.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(["Bearer", token.as_str()].join(" ").as_str())
                .expect("can't initialize client: token problem"),
        );
        Self {
            http_client: Client::builder()
                .default_headers(hmap)
                .build()
                .expect("can't initialize client"),
            base_api_url,
            zone_id,
        }
    }

    pub async fn create_block_rule(
        &self,
        expr: String,
        restriction_type: models::RestrictionType,
    ) -> Result<String> {
        info!("Will block globally: {}", expr);

        let req = serde_json::to_string(&models::FirewallRuleRequest {
            action: restriction_type.to_string(),
            filter: models::Filter { expression: expr },
        })?;
        let path = format!("zones/{}/firewall/rules", self.zone_id);

        let resp = self
            .http_client
            .post(self.base_api_url.to_owned().add(path.as_str()))
            .body(req)
            .send()
            .await?
            .json::<models::FirewallRuleResponse>()
            .await?;
        if !resp.success {
            error!("Request was sent, but CloudFlare responded with unsuccess");
            return Err(errors::ServerError::Unsuccessfull { info: resp.errors }.into());
        };
        let value = match resp.result.first() {
            Some(v) => v,
            None => {
                return Err(ServerError::WrappedErr {
                    cause: String::from("bad response"),
                }
                .into())
            }
        };
        Ok(value.id.clone())
    }
    pub async fn delete_block_rule(&self, rule_id: String) -> Result<(), ServerError> {
        info!("Will delete rule id {}: ttl reached", rule_id);
        let path = format!("zones/{}/firewall/rules/{}", self.zone_id, rule_id);

        let resp = self
            .http_client
            .delete(self.base_api_url.to_owned().add(path.as_str()))
            .send()
            .await
            .map_err(|e| errors::wrap_err(e.into()))?
            .json::<models::FirewallRuleResponse>()
            .await
            .map_err(|e| errors::wrap_err(e.into()))?;
        if !resp.success {
            error!("Request was sent, but CloudFlare responded with unsuccess");
            return Err(errors::ServerError::Unsuccessfull { info: resp.errors });
        };
        Ok(())
    }
}
