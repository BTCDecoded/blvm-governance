//! Governance module configuration.
//!
//! Loaded from config.toml in module data dir. Node overrides via [modules.blvm-governance]
//! and MODULE_CONFIG_* env vars.

use blvm_sdk_macros::config;
use serde::{Deserialize, Serialize};

/// Governance module configuration.
///
/// Config file: `config.toml` in module data dir.
/// Node override: `[modules.blvm-governance]` in node config.
/// Env override: `MODULE_CONFIG_WEBHOOK_URL`, `MODULE_CONFIG_NODE_ID`, `MODULE_CONFIG_WEBHOOK_RETRY_COUNT`, `MODULE_CONFIG_LISTEN`, `MODULE_CONFIG_COMMONS_URL`.
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

    /// Posts per event. Absent posts once. A set count is the total, and the client stops after a success.
    #[serde(default)]
    #[config_env]
    pub webhook_retry_count: Option<u32>,

    /// Bind address for the proxy. Unset means this process does not listen.
    #[serde(default)]
    #[config_env]
    pub listen: Option<String>,

    /// blvm-commons base URL. Block posts are forwarded to `{commons_url}/webhooks/block`.
    #[serde(default)]
    #[config_env]
    pub commons_url: Option<String>,
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
        if let Some(count) = self.webhook_retry_count {
            m.insert(
                "governance.webhook_retry_count".to_string(),
                count.to_string(),
            );
        }
        if let Some(ref listen) = self.listen {
            m.insert("governance.listen".to_string(), listen.clone());
        }
        if let Some(ref commons) = self.commons_url {
            m.insert("governance.commons_url".to_string(), commons.clone());
        }
        m
    }
}

#[cfg(test)]
mod retry_env_tests {
    use super::GovernanceConfig;
    use std::sync::Mutex;

    static LOCK: Mutex<()> = Mutex::new(());
    const ENV_KEY: &str = "MODULE_CONFIG_WEBHOOK_RETRY_COUNT";

    fn set_env(value: &str) {
        unsafe { std::env::set_var(ENV_KEY, value) }
    }

    fn unset_env() {
        unsafe { std::env::remove_var(ENV_KEY) }
    }

    #[test]
    fn omitted_retry_stays_unset_and_env_beats_the_file() {
        let _guard = LOCK.lock().unwrap_or_else(|err| err.into_inner());
        let previous = std::env::var(ENV_KEY).ok();
        let dir = std::env::temp_dir().join(format!("governance-retry-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("config.toml");

        unset_env();
        std::fs::write(&path, "node_id = \"n1\"\n").unwrap();
        let omitted = GovernanceConfig::load(&path).unwrap();
        assert!(omitted.webhook_retry_count.is_none());

        std::fs::write(&path, "webhook_retry_count = 2\n").unwrap();
        set_env("5");
        let overlaid = GovernanceConfig::load(&path).unwrap();
        assert_eq!(overlaid.webhook_retry_count, Some(5));

        match previous {
            Some(value) => set_env(&value),
            None => unset_env(),
        }
        let _ = std::fs::remove_dir_all(&dir);
    }
}
