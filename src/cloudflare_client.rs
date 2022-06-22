use crate::errors;
use crate::errors::ServerError;
use crate::models;
use anyhow::Result;
use reqwest::{header, Client};
use tracing::{error, info};

#[derive(Clone, Debug)]
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
            format!("Bearer {}", token.as_str())
                .parse()
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
    #[tracing::instrument()]
    pub async fn create_block_rule(
        &self,
        expr: String,
        restriction_type: models::RestrictionType,
    ) -> Result<String> {
        info!("Will block globally: {}", expr);

        let req = vec![models::FirewallRuleRequest {
            action: restriction_type.to_string(),
            filter: models::Filter { expression: expr },
        }];
        let path = format!("zones/{}/firewall/rules", self.zone_id);

        let resp = self
            .http_client
            .post(format!("{}{}", self.base_api_url, path))
            .json(&req)
            .send()
            .await?
            .json::<models::RuleResponse>()
            .await?;
        if !resp.success {
            error!("Request was sent, but CloudFlare responded with unsuccess");
            return Err(errors::ServerError::Unsuccessfull {
                cause: resp.errors.into_iter().map(|v| v.message).collect(),
            }
            .into());
        };
        let result_response = resp.result.ok_or::<ServerError>(ServerError::WrappedErr {
            cause: "bad response".to_string(),
        })?;
        let result_response =
            result_response
                .first()
                .ok_or::<ServerError>(ServerError::WrappedErr {
                    cause: "bad response".to_string(),
                })?;
        Ok(result_response.id.clone())
    }
    #[tracing::instrument()]
    pub async fn delete_block_rule(&self, rule_id: String) -> Result<(), ServerError> {
        info!("Will delete rule id {}: ttl reached", rule_id);
        let path = format!("zones/{}/firewall/rules/{}", self.zone_id, rule_id);

        let resp = self
            .http_client
            .delete(format!("{}{}", self.base_api_url, path))
            .send()
            .await?;
        let resp = resp.json::<models::DeleteRuleResponse>().await?;
        if !resp.success {
            error!("Request was sent, but CloudFlare responded with unsuccess");
            return Err(errors::ServerError::Unsuccessfull {
                cause: resp.errors.into_iter().map(|v| v.message).collect(),
            });
        };
        Ok(())
    }
}
