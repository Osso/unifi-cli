use anyhow::Result;
use serde_json::Value;

use crate::api::Client;

impl Client {
    /// Get UniFi devices (APs, switches, gateways)
    pub async fn get_devices(&self) -> Result<Value> {
        self.get_stat("device").await
    }
}
