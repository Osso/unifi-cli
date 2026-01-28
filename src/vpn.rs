use anyhow::Result;
use serde_json::Value;

use crate::api::Client;

impl Client {
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
        self.get_rest("remotesiteipsec")
            .await
            .or_else(|_| Ok(Value::Array(vec![])))
    }
}
