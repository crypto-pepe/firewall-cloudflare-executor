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

    pub async fn restrict_rule(
        &self,
        ip: Option<String>,
        ua: Option<String>,
        restriction_type: models::RestrictionType,
    ) -> Result<String> {
        let expr = models::form_firewall_rule_expression(ip.as_ref(), ua.as_ref())
            .ok_or(errors::ServerError::EmptyRequest)?;
        info!(
            "Will {}: {}\n globally",
            match restriction_type {
                models::RestrictionType::Unblock(_) => "unblock",
                _ => "block",
            },
            expr,
        );

        let req = serde_json::to_string(&models::FirewallRuleRequest {
            action: restriction_type.to_string(),
            filter: models::Filter { expression: expr },
        })?;
        let path = format!("zones/{}/firewall/rules", self.zone_id);

        let builder = match restriction_type {
            models::RestrictionType::Unblock(_) => self
                .http_client
                .delete(self.base_api_url.to_owned().add(path.as_str())),
            _ => self
                .http_client
                .post(self.base_api_url.to_owned().add(path.as_str())),
        };
        let resp = builder
            .json(&req)
            .send()
            .await?
            .json::<models::FirewallRuleResponse>()
            .await?;
        if !resp.success {
            error!("Request was sent, but CloudFlare responded with unsuccess");
            return Err(errors::ServerError::Unsuccessfull { info: resp.errors }.into());
        };
        let value = resp.result.first().ok_or::<ServerError>(
            ServerError::WrappedErr {
                cause: "bad response".to_string(),
            }
            .into(),
        )?;
        Ok(value.id.clone())
    }
}
