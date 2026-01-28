use anyhow::Result;
use serde_json::Value;

use crate::api::Client;

impl Client {
    /// Get WiFi/WLAN configurations
    pub async fn get_wifi(&self) -> Result<Value> {
        self.get_rest("wlanconf").await
    }
}
