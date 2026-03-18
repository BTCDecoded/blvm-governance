//! Governance module configuration.
//!
//! Loaded from config.toml in module data dir. Node overrides via [modules.governance] and
//! MODULE_CONFIG_* env vars.

use blvm_sdk_macros::config;
use serde::{Deserialize, Serialize};

/// Governance module configuration.
///
/// Config file: `config.toml` in module data dir.
/// Node override: `[modules.governance]` or `[modules.blvm-governance]` in node config.
/// Env override: `MODULE_CONFIG_WEBHOOK_URL`, `MODULE_CONFIG_NODE_ID`.
#[config(name = "governance")]
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct GovernanceConfig {
    /// Webhook URL for governance events (e.g. https://governance.example.com/webhook)
    #[serde(default)]
    #[config_env]
    pub webhook_url: Option<String>,

    /// Node identifier for webhook events
    #[serde(default)]
    #[config_env]
    pub node_id: Option<String>,

    /// Webhook secret for HMAC signing.
    #[serde(default)]
    pub webhook_secret: Option<String>,
    /// Retry count for failed webhook deliveries.
    #[serde(default = "default_webhook_retry_count")]
    pub webhook_retry_count: u32,
    /// Governance tier: "maintainer" | "contributor".
    #[serde(default)]
    pub governance_tier: Option<String>,
}

fn default_webhook_retry_count() -> u32 {
    3
}

blvm_sdk::impl_module_config!(GovernanceConfig);

impl GovernanceConfig {
    /// Convert to ModuleContext config map for webhook client compatibility.
    pub fn to_context_map(&self) -> std::collections::HashMap<String, String> {
        let mut m = std::collections::HashMap::new();
        if let Some(ref url) = self.webhook_url {
            m.insert("governance.webhook_url".to_string(), url.clone());
        }
        if let Some(ref id) = self.node_id {
            m.insert("governance.node_id".to_string(), id.clone());
        }
        m
    }
}
