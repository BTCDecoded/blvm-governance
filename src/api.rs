//! Module API for inter-module communication
//!
//! Exposes governance data to other modules via call_module.

use blvm_node::module::ipc::protocol::EventPayload;
use blvm_node::module::inter_module::api::ModuleAPI;
use blvm_node::module::traits::{EventType, ModuleError, NodeAPI};
use std::sync::Arc;

/// Governance module API for other modules
pub struct GovernanceModuleApi {
    proposal_store: Arc<crate::proposals::ProposalStore>,
    economic_nodes: Arc<crate::economic_nodes::EconomicNodeRegistry>,
    webhook_url: Option<String>,
    node_api: Arc<dyn NodeAPI>,
}

impl GovernanceModuleApi {
    pub fn new(
        proposal_store: Arc<crate::proposals::ProposalStore>,
        economic_nodes: Arc<crate::economic_nodes::EconomicNodeRegistry>,
        webhook_url: Option<String>,
        node_api: Arc<dyn NodeAPI>,
    ) -> Self {
        Self {
            proposal_store,
            economic_nodes,
            webhook_url,
            node_api,
        }
    }
}

#[async_trait::async_trait]
impl ModuleAPI for GovernanceModuleApi {
    async fn handle_request(
        &self,
        method: &str,
        params: &[u8],
        _caller_module_id: &str,
    ) -> Result<Vec<u8>, ModuleError> {
        match method {
            "get_proposals" => {
                let proposals = self.proposal_store.load_proposals().map_err(|e| {
                    ModuleError::OperationError(format!("Failed to load proposals: {}", e))
                })?;
                serde_json::to_vec(&proposals).map_err(|e| {
                    ModuleError::OperationError(format!("Serialization error: {}", e))
                })
            }
            "get_economic_nodes" => {
                let nodes = self.economic_nodes.list_nodes().await;
                let json_nodes: Vec<serde_json::Value> = nodes
                    .into_iter()
                    .map(|n| {
                        serde_json::json!({
                            "node_id": hex::encode(n.node_id),
                            "hashpower_percentage": n.hashpower_percentage,
                            "economic_activity_percentage": n.economic_activity_percentage,
                            "registered_at": n.registered_at,
                            "last_seen": n.last_seen,
                            "veto_count": n.veto_count,
                        })
                    })
                    .collect();
                serde_json::to_vec(&json_nodes).map_err(|e| {
                    ModuleError::OperationError(format!("Serialization error: {}", e))
                })
            }
            "get_webhook_status" => {
                let status = serde_json::json!({
                    "enabled": self.webhook_url.is_some(),
                    "url": self.webhook_url.as_deref(),
                });
                serde_json::to_vec(&status).map_err(|e| {
                    ModuleError::OperationError(format!("Serialization error: {}", e))
                })
            }
            "create_proposal" => {
                let params_json: serde_json::Value = serde_json::from_slice(params)
                    .unwrap_or(serde_json::json!({}));
                let proposal_id = params_json
                    .get("proposal_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        ModuleError::OperationError(
                            "create_proposal requires proposal_id (string)".to_string(),
                        )
                    })?
                    .to_string();
                let repository = params_json
                    .get("repository")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let pr_number = params_json
                    .get("pr_number")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let tier = params_json
                    .get("tier")
                    .and_then(|v| v.as_str())
                    .unwrap_or("standard")
                    .to_string();
                let payload = EventPayload::GovernanceProposalCreated {
                    proposal_id: proposal_id.clone(),
                    repository: repository.clone(),
                    pr_number,
                    tier: tier.clone(),
                };
                self.node_api
                    .publish_event(EventType::GovernanceProposalCreated, payload)
                    .await
                    .map_err(|e| {
                        ModuleError::OperationError(format!("Failed to publish proposal: {}", e))
                    })?;
                serde_json::to_vec(&serde_json::json!({
                    "ok": true,
                    "proposal_id": proposal_id,
                    "repository": repository,
                    "pr_number": pr_number,
                    "tier": tier
                }))
                .map_err(|e| {
                    ModuleError::OperationError(format!("Serialization error: {}", e))
                })
            }
            "record_proposal_vote" => {
                let params_json: serde_json::Value = serde_json::from_slice(params)
                    .unwrap_or(serde_json::json!({}));
                let proposal_id = params_json
                    .get("proposal_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        ModuleError::OperationError(
                            "record_proposal_vote requires proposal_id (string)".to_string(),
                        )
                    })?
                    .to_string();
                let voter = params_json
                    .get("voter")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let vote = params_json
                    .get("vote")
                    .and_then(|v| v.as_str())
                    .unwrap_or("abstain")
                    .to_string();
                let payload = EventPayload::GovernanceProposalVoted {
                    proposal_id: proposal_id.clone(),
                    voter: voter.clone(),
                    vote: vote.clone(),
                };
                self.node_api
                    .publish_event(EventType::GovernanceProposalVoted, payload)
                    .await
                    .map_err(|e| {
                        ModuleError::OperationError(format!("Failed to publish proposal vote: {}", e))
                    })?;
                serde_json::to_vec(&serde_json::json!({
                    "ok": true,
                    "proposal_id": proposal_id,
                    "voter": voter,
                    "vote": vote
                }))
                .map_err(|e| {
                    ModuleError::OperationError(format!("Serialization error: {}", e))
                })
            }
            "record_proposal_merged" => {
                let params_json: serde_json::Value = serde_json::from_slice(params)
                    .unwrap_or(serde_json::json!({}));
                let proposal_id = params_json
                    .get("proposal_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        ModuleError::OperationError(
                            "record_proposal_merged requires proposal_id (string)".to_string(),
                        )
                    })?
                    .to_string();
                let repository = params_json
                    .get("repository")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let pr_number = params_json
                    .get("pr_number")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let payload = EventPayload::GovernanceProposalMerged {
                    proposal_id: proposal_id.clone(),
                    repository: repository.clone(),
                    pr_number,
                };
                self.node_api
                    .publish_event(EventType::GovernanceProposalMerged, payload)
                    .await
                    .map_err(|e| {
                        ModuleError::OperationError(format!("Failed to publish proposal merged: {}", e))
                    })?;
                serde_json::to_vec(&serde_json::json!({
                    "ok": true,
                    "proposal_id": proposal_id,
                    "repository": repository,
                    "pr_number": pr_number
                }))
                .map_err(|e| {
                    ModuleError::OperationError(format!("Serialization error: {}", e))
                })
            }
            _ => Err(ModuleError::OperationError(format!("Unknown method: {}", method))),
        }
    }

    fn list_methods(&self) -> Vec<String> {
        vec![
            "get_proposals".to_string(),
            "get_economic_nodes".to_string(),
            "get_webhook_status".to_string(),
            "create_proposal".to_string(),
            "record_proposal_vote".to_string(),
            "record_proposal_merged".to_string(),
        ]
    }

    fn api_version(&self) -> u32 {
        1
    }
}
