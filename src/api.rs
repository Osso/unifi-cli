use anyhow::{Context, Result};
use serde_json::Value;

pub struct Client {
    pub(crate) http: reqwest::Client,
    pub(crate) base_url: String,
    pub(crate) api_key: String,
}

impl Client {
    pub fn new(host: &str, api_key: &str) -> Result<Self> {
        let http = reqwest::Client::builder()
            .danger_accept_invalid_certs(true) // UDM uses self-signed certs
            .build()?;

        let base_url = if host.starts_with("http") {
            host.to_string()
        } else {
            format!("https://{}", host)
        };

        Ok(Self {
            http,
            base_url,
            api_key: api_key.to_string(),
        })
    }

    pub(crate) async fn get_rest(&self, endpoint: &str) -> Result<Value> {
        let url = format!(
            "{}/proxy/network/api/s/default/rest/{}",
            self.base_url, endpoint
        );

        let resp = self
            .http
            .get(&url)
            .header("X-API-Key", &self.api_key)
            .send()
            .await
            .context("Failed to fetch data")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Failed to get {} ({}): {}", endpoint, status, body);
        }

        let body: Value = resp.json().await?;
        Ok(body.get("data").cloned().unwrap_or(Value::Array(vec![])))
    }

    pub(crate) async fn get_v2(&self, endpoint: &str) -> Result<Value> {
        let url = format!(
            "{}/proxy/network/v2/api/site/default/{}",
            self.base_url, endpoint
        );

        let resp = self
            .http
            .get(&url)
            .header("X-API-Key", &self.api_key)
            .send()
            .await
            .context("Failed to fetch data")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Failed to get {} ({}): {}", endpoint, status, body);
        }

        resp.json().await.context("Failed to parse response")
    }

    pub(crate) async fn get_setting(&self, key: &str) -> Result<Value> {
        let url = format!(
            "{}/proxy/network/api/s/default/rest/setting/{}",
            self.base_url, key
        );

        let resp = self
            .http
            .get(&url)
            .header("X-API-Key", &self.api_key)
            .send()
            .await
            .context("Failed to fetch setting")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Failed to get setting ({}): {}", status, body);
        }

        let body: Value = resp.json().await?;

        if let Some(data) = body.get("data").and_then(|d| d.as_array()) {
            if let Some(first) = data.first() {
                return Ok(first.clone());
            }
        }

        anyhow::bail!("Setting '{}' not found", key)
    }

    pub(crate) async fn get_stat(&self, endpoint: &str) -> Result<Value> {
        let url = format!(
            "{}/proxy/network/api/s/default/stat/{}",
            self.base_url, endpoint
        );

        let resp = self
            .http
            .get(&url)
            .header("X-API-Key", &self.api_key)
            .send()
            .await
            .context("Failed to fetch data")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Failed to get {} ({}): {}", endpoint, status, body);
        }

        let body: Value = resp.json().await?;
        Ok(body.get("data").cloned().unwrap_or(Value::Array(vec![])))
    }
}
