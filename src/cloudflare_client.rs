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
        filter_id: String,
        restriction_type: models::RestrictionType,
    ) -> Result<String> {
        let req = vec![model::CreateRuleRequest {
            action: restriction_type.to_string(),
            filter: model::Filter { id: filter_id },
        }];
        let path = format!("zones/{}/firewall/rules", self.zone_id);

        let resp = self
            .http_client
            .post(format!("{}{}", self.base_api_url, path))
            .json(&req)
            .send()
            .await?
            .json::<model::CloudflareResponse>()
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

    pub async fn create_filter(
        &self,
        filter: models::Filter,
    ) -> Result<String, errors::ServerError> {
        let req = vec![model::CreateFilterRequest::from(filter)];
        let path = format!("zones/{}/filters", self.zone_id);

        let resp = self
            .http_client
            .post(format!("{}{}", self.base_api_url, path))
            .json(&req)
            .send()
            .await?
            .json::<model::CloudflareResponse>()
            .await?;

        if !resp.success {
            error!("Request was sent, but CloudFlare responded with unsuccess");
            return Err(errors::ServerError::Unsuccessfull {
                errors: resp.errors.into_iter().map(|v| v.message).collect(),
            });
        };

        let filters = resp.result.ok_or::<ServerError>(ServerError::WrappedErr {
            cause: "bad response".to_string(),
        })?;

        let filter = filters
            .first()
            .ok_or::<ServerError>(ServerError::WrappedErr {
                cause: "bad response".to_string(),
            })?;
        Ok(filter.id.clone())
    }

    pub async fn update_filter(
        &self,
        filter: models::Filter,
    ) -> Result<String, errors::ServerError> {
        let req = vec![model::UpdateFilterRequest::from(filter)];
        info!("{:?}", req);
        let path = format!("zones/{}/filters", self.zone_id);

        let resp = self
            .http_client
            .put(format!("{}{}", self.base_api_url, path))
            .json(&req)
            .send()
            .await?
            .json::<model::CloudflareResponse>()
            .await?;

        if !resp.success {
            error!("Request was sent, but CloudFlare responded with unsuccess");
            return Err(errors::ServerError::Unsuccessfull {
                errors: resp.errors.into_iter().map(|v| v.message).collect(),
            });
        };

        let filters = resp.result.ok_or::<ServerError>(ServerError::WrappedErr {
            cause: "bad response".to_string(),
        })?;

        let filter = filters
            .first()
            .ok_or::<ServerError>(ServerError::WrappedErr {
                cause: "bad response".to_string(),
            })?;
        Ok(filter.id.clone())
    }

    #[tracing::instrument()]
    pub async fn delete_block_rule(&self, rule_id: String) -> Result<(), ServerError> {
        info!("Will delete rule id {}", rule_id);

        let path = format!("zones/{}/firewall/rules/{}", self.zone_id, rule_id);

        let resp = self
            .http_client
            .delete(format!("{}{}", self.base_api_url, path))
            .query(&[("delete_filter_if_unused", true)])
            .send()
            .await?;
        let resp = resp.json::<model::CloudflareResponseSingle>().await?;
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
    use crate::models;
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize)]
    pub(super) struct CloudflareResponse {
        pub success: bool,
        pub result: Option<Vec<Object>>,
        pub errors: Vec<Error>,
    }
    #[derive(Deserialize)]
    pub(super) struct CloudflareResponseSingle {
        pub success: bool,
        pub errors: Vec<Error>,
    }

    #[derive(Deserialize)]
    pub(super) struct Object {
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
    pub struct Filter {
        pub id: String,
    }

    #[derive(Serialize)]
    pub struct CreateFilterRequest {
        pub expression: String,
        pub description: String,
    }

    impl From<models::Filter> for CreateFilterRequest {
        fn from(filter: models::Filter) -> Self {
            Self {
                description: filter.filter_type.to_string(),
                expression: filter.expression,
            }
        }
    }

    #[derive(Serialize, Debug)]
    pub struct UpdateFilterRequest {
        pub id: String,
        pub expression: String,
    }

    impl From<models::Filter> for UpdateFilterRequest {
        fn from(filter: models::Filter) -> Self {
            Self {
                id: filter.id,
                expression: filter.expression,
            }
        }
    }
}
