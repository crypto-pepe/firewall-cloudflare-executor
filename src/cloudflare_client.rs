use std::ops::Add;

use crate::errors;
use crate::models;
use anyhow::Result;
use reqwest::{header, Client};
use tracing::{error, info};

#[derive(Clone)]
pub struct CloudflareClient {
    c: Client,
    base_api_url: String,
    zone_id: String,
    account_id: String,
}

impl CloudflareClient {
    pub fn new(base_api_url: String, token: String, account_id: String, zone_id: String) -> Self {
        let mut hmap = header::HeaderMap::new();
        hmap.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(["Bearer", token.as_str()].join(" ").as_str()).unwrap(),
        );
        Self {
            c: Client::builder().default_headers(hmap).build().unwrap(),
            base_api_url,
            zone_id,
            account_id,
        }
    }

    pub async fn restrict_ip_by_zone(
        &self,
        ip: &str,
        restriction_type: models::RestrictionType,
    ) -> Result<()> {
        let path = format!("zones/{}/firewall/access_rules/rules", self.zone_id);
        let req = serde_json::to_string(&models::AccessRuleRequest {
            mode: restriction_type.to_string(),
            configuration: models::Configuration {
                target: "ip".to_string(),
                value: ip.to_string(),
            },
        })?;

        info!("Will ban {:?}, zone_id: {:?}", ip, self.zone_id);

        let resp = self
            .c
            .post(self.base_api_url.to_owned().add(path.as_str()))
            .body(req)
            .send()
            .await?
            .json::<models::AccessRuleResponse>()
            .await?;
        if !resp.success {
            error!("Request was sent, but CloudFlare responded with unsuccess");
            return Err(errors::ServerError::Unsuccessfull { info: resp.errors }.into());
        };
        Ok(())
    }

    pub async fn restrict_ip_global(
        &self,
        ip: &str,
        restriction_type: models::RestrictionType,
    ) -> Result<()> {
        let path = format!("accounts/{}/firewall/access_rules/rules", self.account_id);
        let req = serde_json::to_string(&models::AccessRuleRequest {
            mode: restriction_type.to_string(),
            configuration: models::Configuration {
                target: "ip".to_string(),
                value: ip.to_string(),
            },
        })?;

        info!("Will ban {:?} globally", ip);

        let resp = self
            .c
            .post(self.base_api_url.to_owned().add(path.as_str()))
            .body(req)
            .send()
            .await?
            .json::<models::AccessRuleResponse>()
            .await?;
        if !resp.success {
            error!("Request was sent, but CloudFlare responded with unsuccess");
            return Err(errors::ServerError::Unsuccessfull { info: resp.errors }.into());
        };
        Ok(())
    }

    pub async fn restrict_user_agent(
        &self,
        user_agent: &str,
        restriction_type: models::RestrictionType,
    ) -> Result<()> {
        let path = format!("zones/{}/firewall/ua_rules", self.zone_id);
        let req = serde_json::to_string(&models::AccessRuleRequest {
            mode: restriction_type.to_string(),
            configuration: models::Configuration {
                target: "ua".to_string(),
                value: user_agent.to_string(),
            },
        })?;

        info!("Will ban {:?} globally", user_agent);

        let resp = self
            .c
            .post(self.base_api_url.to_owned().add(path.as_str()))
            .body(req)
            .send()
            .await?
            .json::<models::AccessRuleResponse>()
            .await?;
        if !resp.success {
            error!("Request was sent, but CloudFlare responded with unsuccess");
            return Err(errors::ServerError::Unsuccessfull { info: resp.errors }.into());
        };
        Ok(())
    }
}
