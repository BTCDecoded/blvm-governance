//! Governance module: unified CLI via #[module] macro.

use blvm_node::module::ipc::protocol::{EventMessage, ModuleMessage};
use blvm_sdk::module::prelude::*;
use blvm_sdk_macros::module;
use std::sync::Arc;

use crate::economic_nodes::EconomicNodeRegistry;
use crate::proposals::ProposalStore;
use crate::webhook::GovernanceWebhookClient;

/// Governance module: CLI + event handlers in one struct.
#[derive(Clone)]
pub struct GovernanceModule {
    pub proposal_store: Arc<ProposalStore>,
    pub webhook_client: Arc<GovernanceWebhookClient>,
    pub economic_nodes: Arc<EconomicNodeRegistry>,
}

#[module]
impl GovernanceModule {
    #[on_event(GovernanceProposalCreated, GovernanceProposalVoted, GovernanceProposalMerged, EconomicNodeRegistered, EconomicNodeVeto, NewBlock)]
    async fn on_governance_event(&self, event: &EventMessage, ctx: &InvocationContext) -> Result<(), ModuleError> {
        let msg = ModuleMessage::Event(event.clone());
        let api = ctx.node_api().expect("node_api required");
        if let Err(e) = self.webhook_client.handle_event(&msg, api.as_ref()).await {
            tracing::warn!("Error handling event in webhook client: {}", e);
        }
        if let Err(e) = self.economic_nodes.handle_event(&msg, api.as_ref()).await {
            tracing::warn!("Error handling event in economic node registry: {}", e);
        }
        if let Err(e) = self.proposal_store.handle_event(&msg) {
            tracing::warn!("Error handling event in proposal store: {}", e);
        }
        Ok(())
    }

    /// List governance proposals (stored when node publishes GovernanceProposal* events).
    #[command]
    fn list_proposals(&self, _ctx: &InvocationContext) -> Result<String, ModuleError> {
        let proposals = self
            .proposal_store
            .load_proposals()
            .map_err(|e| ModuleError::Other(e.to_string()))?;
        if proposals.is_empty() {
            return Ok("No proposals yet.\n\
                Proposals appear when the node publishes GovernanceProposalCreated events.".into());
        }
        let mut out = format!("Proposals ({}):\n", proposals.len());
        for (i, p) in proposals.iter().enumerate() {
            let status = match &p.status {
                crate::proposals::ProposalStatus::Created => "open",
                crate::proposals::ProposalStatus::Merged => "merged",
            };
            out.push_str(&format!(
                "  {}. {} | {}#{} | {} | {} votes\n",
                i + 1, p.proposal_id, p.repository, p.pr_number, status, p.votes.len(),
            ));
        }
        Ok(out)
    }

    /// Send a test webhook payload to verify configuration.
    #[command]
    fn webhook_test(&self, _ctx: &InvocationContext) -> Result<String, ModuleError> {
        let data_dir = std::env::var("DATA_DIR").unwrap_or_else(|_| "data/modules/blvm-governance".into());
        let config_path = std::path::Path::new(&data_dir).join("config.toml");
        let config = crate::GovernanceConfig::load(&config_path).unwrap_or_default();
        let Some(url) = &config.webhook_url else {
            return Ok("Webhook not configured (governance.webhook_url). Set in config.toml.".into());
        };
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|e| ModuleError::Other(e.to_string()))?;
        let payload = serde_json::json!({
            "event": "test",
            "source": "blvm-governance",
            "timestamp": std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs(),
        });
        let res = client.post(url).json(&payload).send();
        match res {
            Ok(r) if r.status().is_success() => Ok(format!("Webhook test OK: {} {}", r.status(), url)),
            Ok(r) => Ok(format!("Webhook returned {}: {}", r.status(), url)),
            Err(e) => Ok(format!("Webhook test failed: {} - {}", url, e)),
        }
    }

    /// Show module status.
    #[command]
    fn status(&self, _ctx: &InvocationContext) -> Result<String, ModuleError> {
        let data_dir = std::env::var("DATA_DIR").unwrap_or_else(|_| "data/modules/blvm-governance".into());
        let config_path = std::path::Path::new(&data_dir).join("config.toml");
        let config = crate::GovernanceConfig::load(&config_path).unwrap_or_default();
        Ok(format!(
            "blvm-governance module\n\
             Config: {}\n\
             Webhook URL: {}",
            config_path.display(),
            config.webhook_url.as_deref().unwrap_or("(not set)")
        ))
    }
}

impl GovernanceModule {
    /// Handle node events: webhook, economic nodes, proposal store.
    pub async fn handle_event(
        &self,
        event: &blvm_node::module::ipc::protocol::ModuleMessage,
        node_api: &dyn blvm_node::module::traits::NodeAPI,
    ) -> Result<(), blvm_node::module::traits::ModuleError> {
        self.webhook_client
            .handle_event(event, node_api)
            .await
            .map_err(|e| blvm_node::module::traits::ModuleError::Other(e.to_string()))?;
        self.economic_nodes
            .handle_event(event, node_api)
            .await
            .map_err(|e| blvm_node::module::traits::ModuleError::Other(e.to_string()))?;
        self.proposal_store
            .handle_event(event)
            .map_err(|e| blvm_node::module::traits::ModuleError::Other(e.to_string()))?;
        Ok(())
    }
}
