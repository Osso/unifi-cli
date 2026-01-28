use anyhow::Result;
use serde_json::Value;

use crate::api::Client;

impl Client {
    /// Get security settings (IPS, ad blocking, DNS filtering)
    pub async fn get_security_settings(&self) -> Result<Value> {
        self.get_setting("ips").await
    }
}
