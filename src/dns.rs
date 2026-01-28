use anyhow::{Context, Result};
use serde_json::Value;

use crate::api::Client;

impl Client {
    /// Get static DNS records
    pub async fn get_dns_records(&self) -> Result<Value> {
        self.get_v2("static-dns").await
    }

    /// Create a static DNS record (A record)
    pub async fn create_dns_record(&self, key: &str, value: &str) -> Result<Value> {
        let url = format!(
            "{}/proxy/network/v2/api/site/default/static-dns",
            self.base_url
        );

        let body = serde_json::json!({
            "key": key,
            "value": value,
            "record_type": "A",
            "enabled": true
        });

        let resp = self
            .http
            .post(&url)
            .header("X-API-Key", &self.api_key)
            .json(&body)
            .send()
            .await
            .context("Failed to create DNS record")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Failed to create DNS record ({}): {}", status, body);
        }

        resp.json().await.context("Failed to parse response")
    }

    /// Delete a static DNS record by ID
    pub async fn delete_dns_record(&self, id: &str) -> Result<()> {
        let url = format!(
            "{}/proxy/network/v2/api/site/default/static-dns/{}",
            self.base_url, id
        );

        let resp = self
            .http
            .delete(&url)
            .header("X-API-Key", &self.api_key)
            .send()
            .await
            .context("Failed to delete DNS record")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Failed to delete DNS record ({}): {}", status, body);
        }

        Ok(())
    }
}
