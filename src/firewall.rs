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

        let mut body = serde_json::Map::new();
        // Required defaults that UniFi expects
        body.insert("src_networkconf_type".into(), Value::String("NETv4".into()));
        body.insert("dst_networkconf_type".into(), Value::String("NETv4".into()));
        body.insert("src_networkconf_id".into(), Value::String(String::new()));
        body.insert("dst_networkconf_id".into(), Value::String(String::new()));
        body.insert("src_mac_address".into(), Value::String(String::new()));
        body.insert("src_firewallgroup_ids".into(), Value::Array(vec![]));
        body.insert("dst_firewallgroup_ids".into(), Value::Array(vec![]));
        body.insert("icmp_typename".into(), Value::String(String::new()));
        body.insert("ipsec".into(), Value::String(String::new()));
        body.insert("logging".into(), Value::Bool(false));
        body.insert("protocol_match_excepted".into(), Value::Bool(false));
        body.insert("state_established".into(), Value::Bool(false));
        body.insert("state_invalid".into(), Value::Bool(false));
        body.insert("state_new".into(), Value::Bool(false));
        body.insert("state_related".into(), Value::Bool(false));
        body.insert("setting_preference".into(), Value::String("manual".into()));
        // Caller-provided fields override defaults
        body.extend(rule.iter().map(|(k, v)| (k.clone(), v.clone())));

        let resp = self
            .http
            .post(&url)
            .header("X-API-Key", &self.api_key)
            .json(&body)
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

    /// Update a firewall rule by ID
    pub async fn update_firewall_rule(
        &self,
        id: &str,
        fields: &serde_json::Map<String, Value>,
    ) -> Result<Value> {
        let url = format!(
            "{}/proxy/network/api/s/default/rest/firewallrule/{}",
            self.base_url, id
        );

        let resp = self
            .http
            .put(&url)
            .header("X-API-Key", &self.api_key)
            .json(fields)
            .send()
            .await
            .context("Failed to update firewall rule")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Failed to update firewall rule ({}): {}", status, body);
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
