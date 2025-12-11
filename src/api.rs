use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub struct Client {
    http: reqwest::Client,
    base_url: String,
    api_key: String,
}

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

    async fn get_setting(&self, key: &str) -> Result<Value> {
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

    /// Get security settings (IPS, ad blocking, DNS filtering)
    pub async fn get_security_settings(&self) -> Result<Value> {
        self.get_setting("ips").await
    }

    async fn get_rest(&self, endpoint: &str) -> Result<Value> {
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

    async fn get_v2(&self, endpoint: &str) -> Result<Value> {
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

    /// Get Teleport VPN settings
    pub async fn get_vpn_teleport(&self) -> Result<Value> {
        self.get_setting("teleport").await
    }

    /// Get Site-to-Site VPN settings
    pub async fn get_vpn_site_to_site(&self) -> Result<Value> {
        self.get_setting("magic_site_to_site_vpn").await
    }

    /// Get VPN servers (WireGuard, OpenVPN)
    pub async fn get_vpn_servers(&self) -> Result<Value> {
        // Try multiple endpoints and combine results
        let wg = self.get_rest("wg").await.unwrap_or(Value::Array(vec![]));
        let openvpn = self.get_setting("openvpn").await.ok();

        let mut result = serde_json::json!({
            "wireguard": wg,
        });
        if let Some(ovpn) = openvpn {
            result["openvpn"] = ovpn;
        }
        Ok(result)
    }

    /// Get VPN clients (remote site IPsec)
    pub async fn get_vpn_clients(&self) -> Result<Value> {
        // Return empty array if endpoint fails (no clients configured)
        self.get_rest("remotesiteipsec")
            .await
            .or_else(|_| Ok(Value::Array(vec![])))
    }

    /// Get all networks (LANs, VLANs, VPN)
    pub async fn get_networks(&self) -> Result<Value> {
        self.get_rest("networkconf").await
    }

    /// Get WiFi/WLAN configurations
    pub async fn get_wifi(&self) -> Result<Value> {
        self.get_rest("wlanconf").await
    }

    async fn get_stat(&self, endpoint: &str) -> Result<Value> {
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

    /// Get UniFi devices (APs, switches, gateways)
    pub async fn get_devices(&self) -> Result<Value> {
        self.get_stat("device").await
    }

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
}
