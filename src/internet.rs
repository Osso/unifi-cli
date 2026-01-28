use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::api::Client;

#[derive(Debug, Serialize, Deserialize)]
pub struct DnsSettings {
    pub mode: String,
    pub dns1: Option<String>,
    pub dns2: Option<String>,
    pub mode_ipv6: String,
    pub dns1_ipv6: Option<String>,
    pub dns2_ipv6: Option<String>,
}

impl Client {
    async fn get_wan_network(&self) -> Result<Value> {
        let url = format!(
            "{}/proxy/network/api/s/default/rest/networkconf",
            self.base_url
        );

        let resp = self
            .http
            .get(&url)
            .header("X-API-Key", &self.api_key)
            .send()
            .await
            .context("Failed to fetch network config")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Failed to get network config ({}): {}", status, body);
        }

        let body: Value = resp.json().await?;

        if let Some(data) = body.get("data").and_then(|d| d.as_array()) {
            for network in data {
                let purpose = network.get("purpose").and_then(|p| p.as_str());
                if purpose == Some("wan") {
                    return Ok(network.clone());
                }
            }
        }

        anyhow::bail!("No WAN network found")
    }

    /// Get all WAN settings
    pub async fn get_wan_settings(&self) -> Result<Value> {
        self.get_wan_network().await
    }

    /// Get DNS settings from internet/WAN configuration
    pub async fn get_dns_settings(&self) -> Result<DnsSettings> {
        let network = self.get_wan_network().await?;

        let get_str = |key: &str| {
            network
                .get(key)
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
        };

        Ok(DnsSettings {
            mode: get_str("wan_dns_preference").unwrap_or_else(|| "auto".to_string()),
            dns1: get_str("wan_dns1"),
            dns2: get_str("wan_dns2"),
            mode_ipv6: get_str("wan_ipv6_dns_preference").unwrap_or_else(|| "auto".to_string()),
            dns1_ipv6: get_str("wan_ipv6_dns1"),
            dns2_ipv6: get_str("wan_ipv6_dns2"),
        })
    }
}
