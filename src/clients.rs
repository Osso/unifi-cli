use anyhow::{Context, Result};
use serde_json::Value;

use crate::api::Client;

impl Client {
    /// Get online clients
    pub async fn get_clients_online(&self) -> Result<Value> {
        self.get_stat("sta").await
    }

    /// Get all known clients
    pub async fn get_clients_all(&self) -> Result<Value> {
        self.get_rest("user").await
    }

    /// Get offline clients (all known minus online)
    pub async fn get_clients_offline(&self) -> Result<Value> {
        let all = self.get_clients_all().await?;
        let online = self.get_clients_online().await?;

        let online_macs: std::collections::HashSet<String> = online
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|c| c.get("mac").and_then(|m| m.as_str()).map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let offline: Vec<Value> = all
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter(|c| {
                        c.get("mac")
                            .and_then(|m| m.as_str())
                            .map(|mac| !online_macs.contains(mac))
                            .unwrap_or(true)
                    })
                    .cloned()
                    .collect()
            })
            .unwrap_or_default();

        Ok(Value::Array(offline))
    }

    /// Kick a client by MAC address (forces reconnect)
    pub async fn kick_client(&self, mac: &str) -> Result<()> {
        let url = format!("{}/proxy/network/api/s/default/cmd/stamgr", self.base_url);

        let resp = self
            .http
            .post(&url)
            .header("X-API-Key", &self.api_key)
            .json(&serde_json::json!({"cmd": "kick-sta", "mac": mac}))
            .send()
            .await
            .context("Failed to kick client")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Failed to kick client ({}): {}", status, body);
        }

        Ok(())
    }
}
