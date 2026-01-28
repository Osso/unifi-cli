use anyhow::Result;
use serde_json::Value;

use crate::api::Client;

impl Client {
    /// Get all networks (LANs, VLANs, VPN)
    pub async fn get_networks(&self) -> Result<Value> {
        self.get_rest("networkconf").await
    }
}
