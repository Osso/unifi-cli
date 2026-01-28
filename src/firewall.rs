use anyhow::{Context, Result};
use serde_json::Value;

use crate::api::Client;

impl Client {
    /// Get firewall rules
    pub async fn get_firewall_rules(&self) -> Result<Value> {
        self.get_rest("firewallrule").await
    }

    /// Get firewall groups
    pub async fn get_firewall_groups(&self) -> Result<Value> {
        self.get_rest("firewallgroup").await
    }

    /// Get traffic rules
    pub async fn get_traffic_rules(&self) -> Result<Value> {
        self.get_v2("trafficrules").await
    }

    /// Create a firewall rule
    pub async fn create_firewall_rule(
        &self,
        rule: &serde_json::Map<String, Value>,
    ) -> Result<Value> {
        let url = format!(
            "{}/proxy/network/api/s/default/rest/firewallrule",
            self.base_url
        );

        let resp = self
            .http
            .post(&url)
            .header("X-API-Key", &self.api_key)
            .json(rule)
            .send()
            .await
            .context("Failed to create firewall rule")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Failed to create firewall rule ({}): {}", status, body);
        }

        let body: Value = resp.json().await?;
        Ok(body
            .get("data")
            .and_then(|d| d.as_array())
            .and_then(|a| a.first())
            .cloned()
            .unwrap_or(body))
    }

    /// Delete a firewall rule by ID
    pub async fn delete_firewall_rule(&self, id: &str) -> Result<()> {
        let url = format!(
            "{}/proxy/network/api/s/default/rest/firewallrule/{}",
            self.base_url, id
        );

        let resp = self
            .http
            .delete(&url)
            .header("X-API-Key", &self.api_key)
            .send()
            .await
            .context("Failed to delete firewall rule")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Failed to delete firewall rule ({}): {}", status, body);
        }

        Ok(())
    }
}
