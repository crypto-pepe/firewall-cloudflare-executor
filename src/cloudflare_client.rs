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

        let req = vec![model::CreateRuleRequest {
            action: restriction_type.to_string(),
            filter: model::Filter { expression: expr },
        }];
        let path = format!("zones/{}/firewall/rules", self.zone_id);

        let resp = self
            .http_client
            .post(format!("{}{}", self.base_api_url, path))
            .json(&req)
            .send()
            .await?
            .json::<model::CreateRuleResponse>()
            .await?;
        if !resp.success {
            error!("Request was sent, but CloudFlare responded with unsuccess");
            return Err(errors::ServerError::Unsuccessfull {
                errors: resp.errors.into_iter().map(|v| v.message).collect(),
            }
            .into());
        };
        let rules = resp.result.ok_or::<ServerError>(ServerError::WrappedErr {
            cause: "bad response".to_string(),
        })?;
        let rule = rules
            .first()
            .ok_or::<ServerError>(ServerError::WrappedErr {
                cause: "bad response".to_string(),
            })?;
        Ok(rule.id.clone())
    }

    #[tracing::instrument()]
    pub async fn delete_block_rule(&self, rule_id: String) -> Result<(), ServerError> {
        info!("Will delete rule id {}", rule_id);

        let req = model::DeleteRuleRequest {
            delete_filter_if_unused: true,
        };
        let path = format!("zones/{}/firewall/rules/{}", self.zone_id, rule_id);

        let resp = self
            .http_client
            .delete(format!("{}{}", self.base_api_url, path))
            .json(&req)
            .query(&[("delete_filter_if_unused", true)])
            .send()
            .await?;
        let resp = resp.json::<model::DeleteRuleResponse>().await?;
        if !resp.success {
            error!("Request was sent, but CloudFlare responded with unsuccess");
            return Err(errors::ServerError::Unsuccessfull {
                errors: resp.errors.into_iter().map(|v| v.message).collect(),
            });
        };
        Ok(())
    }
}

mod model {
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize)]
    pub(super) struct CreateRuleResponse {
        pub success: bool,
        pub result: Option<Vec<Rule>>,
        pub errors: Vec<Error>,
    }

    #[derive(Deserialize)]
    pub(super) struct DeleteRuleResponse {
        pub success: bool,
        pub errors: Vec<Error>,
    }

    #[derive(Deserialize)]
    pub(super) struct Rule {
        pub id: String,
    }

    #[derive(Deserialize)]
    pub(super) struct Error {
        pub message: String,
    }
    #[derive(Serialize)]
    pub struct CreateRuleRequest {
        pub action: String,
        pub filter: Filter,
    }

    #[derive(Serialize)]
    pub struct DeleteRuleRequest {
        pub delete_filter_if_unused: bool,
    }

    #[derive(Serialize)]
    pub struct Filter {
        pub expression: String,
    }
}
