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

    async fn fetch(&self, url: &str, context: &str) -> Result<reqwest::Response> {
        let resp = self
            .http
            .get(url)
            .header("X-API-Key", &self.api_key)
            .send()
            .await
            .context(format!("Failed to fetch {context}"))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Failed to get {context} ({status}): {body}");
        }

        Ok(resp)
    }

    fn extract_data(body: Value) -> Value {
        body.get("data").cloned().unwrap_or(Value::Array(vec![]))
    }

    pub(crate) async fn get_rest(&self, endpoint: &str) -> Result<Value> {
        let url = format!(
            "{}/proxy/network/api/s/default/rest/{}",
            self.base_url, endpoint
        );
        let body: Value = self.fetch(&url, endpoint).await?.json().await?;
        Ok(Self::extract_data(body))
    }

    pub(crate) async fn get_v2(&self, endpoint: &str) -> Result<Value> {
        let url = format!(
            "{}/proxy/network/v2/api/site/default/{}",
            self.base_url, endpoint
        );
        self.fetch(&url, endpoint)
            .await?
            .json()
            .await
            .context("Failed to parse response")
    }

    pub(crate) async fn get_setting(&self, key: &str) -> Result<Value> {
        let url = format!(
            "{}/proxy/network/api/s/default/rest/setting/{}",
            self.base_url, key
        );
        let body: Value = self
            .fetch(&url, &format!("setting {key}"))
            .await?
            .json()
            .await?;

        body.get("data")
            .and_then(|d| d.as_array())
            .and_then(|a| a.first().cloned())
            .ok_or_else(|| anyhow::anyhow!("Setting '{}' not found", key))
    }

    pub(crate) async fn get_stat(&self, endpoint: &str) -> Result<Value> {
        let url = format!(
            "{}/proxy/network/api/s/default/stat/{}",
            self.base_url, endpoint
        );
        let body: Value = self.fetch(&url, endpoint).await?.json().await?;
        Ok(Self::extract_data(body))
    }
}
