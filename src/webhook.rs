//! Governance webhook client

use crate::error::GovernanceError;
use blvm_node::module::ipc::protocol::EventPayload;
use blvm_node::module::ipc::protocol::ModuleMessage;
use blvm_node::module::traits::NodeAPI;
use blvm_node::module::EventType;
use reqwest::Client;
use tracing::{debug, info, warn};

/// Governance webhook client
pub struct GovernanceWebhookClient {
    client: Client,
    webhook_url: Option<String>,
    node_id: Option<String>,
    enabled: bool,
}

impl GovernanceWebhookClient {
    /// Whether the webhook is configured and enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Webhook URL if configured.
    pub fn webhook_url(&self) -> Option<&str> {
        self.webhook_url.as_deref()
    }

    /// Node ID if configured.
    pub fn node_id(&self) -> Option<&str> {
        self.node_id.as_deref()
    }

    /// Create a new webhook client
    pub async fn new(
        ctx: &blvm_node::module::traits::ModuleContext,
    ) -> Result<Self, GovernanceError> {
        let webhook_url = ctx.get_config("governance.webhook_url").cloned();
        let node_id = ctx.get_config("governance.node_id").cloned();
        let enabled = webhook_url.is_some();

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|e| {
                GovernanceError::WebhookError(format!("Failed to create HTTP client: {}", e))
            })?;

        if enabled {
            info!(
                "Governance webhook client initialized: {}",
                webhook_url.as_ref().unwrap()
            );
        } else {
            debug!("Governance webhook client disabled (no URL configured)");
        }

        Ok(Self {
            client,
            webhook_url,
            node_id,
            enabled,
        })
    }

    /// Handle an event from the node
    pub async fn handle_event(
        &self,
        event: &ModuleMessage,
        node_api: &dyn NodeAPI,
    ) -> Result<(), GovernanceError> {
        if !self.enabled {
            return Ok(());
        }

        match event {
            ModuleMessage::Event(event_msg) => {
                match event_msg.event_type {
                    EventType::NewBlock => {
                        if let EventPayload::NewBlock { block_hash, height } = &event_msg.payload {
                            // Get block data
                            if let Ok(Some(block)) = node_api.get_block(block_hash).await {
                                self.notify_block(&block, *height, node_api).await?;
                            }
                        }
                    }
                    EventType::GovernanceProposalCreated => {
                        if let EventPayload::GovernanceProposalCreated {
                            proposal_id,
                            repository,
                            pr_number,
                            tier,
                        } = &event_msg.payload
                        {
                            info!(
                                "Governance proposal created: id={}, repository={}, pr={}, tier={}",
                                proposal_id, repository, pr_number, tier
                            );
                            self.notify_governance_event(
                                "proposal_created",
                                serde_json::json!({
                                    "proposal_id": proposal_id,
                                    "repository": repository,
                                    "pr_number": pr_number,
                                    "tier": tier,
                                }),
                                node_api,
                            )
                            .await?;
                        }
                    }
                    EventType::GovernanceProposalVoted => {
                        if let EventPayload::GovernanceProposalVoted {
                            proposal_id,
                            voter,
                            vote,
                        } = &event_msg.payload
                        {
                            info!(
                                "Governance proposal voted: id={}, voter={}, vote={}",
                                proposal_id, voter, vote
                            );
                            self.notify_governance_event(
                                "proposal_voted",
                                serde_json::json!({
                                    "proposal_id": proposal_id,
                                    "voter": voter,
                                    "vote": vote,
                                }),
                                node_api,
                            )
                            .await?;
                        }
                    }
                    EventType::GovernanceProposalMerged => {
                        if let EventPayload::GovernanceProposalMerged {
                            proposal_id,
                            repository,
                            pr_number,
                        } = &event_msg.payload
                        {
                            info!(
                                "Governance proposal merged: id={}, repository={}, pr={}",
                                proposal_id, repository, pr_number
                            );
                            self.notify_governance_event(
                                "proposal_merged",
                                serde_json::json!({
                                    "proposal_id": proposal_id,
                                    "repository": repository,
                                    "pr_number": pr_number,
                                }),
                                node_api,
                            )
                            .await?;
                        }
                    }
                    _ => {
                        // Ignore other events
                    }
                }
            }
            _ => {
                // Not an event message
            }
        }

        Ok(())
    }

    /// Notify governance app about a governance event
    async fn notify_governance_event(
        &self,
        event_type: &str,
        data: serde_json::Value,
        node_api: &dyn NodeAPI,
    ) -> Result<(), GovernanceError> {
        if !self.enabled {
            return Ok(());
        }

        let url = self.webhook_url.as_ref().unwrap();

        // Prepare payload
        let payload = serde_json::json!({
            "event_type": event_type,
            "data": data,
            "node_id": self.node_id.as_deref(),
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        });

        // Send webhook and publish WebhookSent/WebhookFailed
        match self.client.post(url).json(&payload).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    debug!(
                        "Governance webhook sent successfully: event_type={}",
                        event_type
                    );
                    let payload = EventPayload::WebhookSent {
                        webhook_url: url.clone(),
                        event_type: event_type.to_string(),
                        success: true,
                    };
                    let _ = node_api.publish_event(EventType::WebhookSent, payload).await;
                } else {
                    warn!(
                        "Governance webhook returned error status {} for event_type={}",
                        response.status(),
                        event_type
                    );
                    let payload = EventPayload::WebhookFailed {
                        webhook_url: url.clone(),
                        event_type: event_type.to_string(),
                        error: format!("HTTP {}", response.status()),
                    };
                    let _ = node_api.publish_event(EventType::WebhookFailed, payload).await;
                }
            }
            Err(e) => {
                warn!(
                    "Failed to send governance webhook for event_type={}: {}",
                    event_type, e
                );
                let payload = EventPayload::WebhookFailed {
                    webhook_url: url.clone(),
                    event_type: event_type.to_string(),
                    error: e.to_string(),
                };
                let _ = node_api.publish_event(EventType::WebhookFailed, payload).await;
            }
        }

        Ok(())
    }

    /// Notify governance app about a new block
    async fn notify_block(
        &self,
        block: &blvm_protocol::Block,
        height: u64,
        node_api: &dyn NodeAPI,
    ) -> Result<(), GovernanceError> {
        if !self.enabled {
            return Ok(());
        }

        let url = self.webhook_url.as_ref().unwrap();

        // Calculate block hash
        let block_hash = self.calculate_block_hash(block);

        // Serialize block to JSON
        let block_json = serde_json::to_value(block).map_err(|e| {
            GovernanceError::WebhookError(format!("Failed to serialize block: {}", e))
        })?;

        // Prepare payload
        let payload = serde_json::json!({
            "block_hash": hex::encode(block_hash),
            "block_height": height as i32,
            "block": block_json,
            "contributor_id": self.node_id.as_deref(),
        });

        let event_type = "block";

        // Send webhook and publish WebhookSent/WebhookFailed
        match self.client.post(url).json(&payload).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    debug!(
                        "Governance webhook sent successfully for block {} at height {}",
                        hex::encode(block_hash),
                        height
                    );
                    let payload = EventPayload::WebhookSent {
                        webhook_url: url.clone(),
                        event_type: event_type.to_string(),
                        success: true,
                    };
                    let _ = node_api.publish_event(EventType::WebhookSent, payload).await;
                } else {
                    warn!(
                        "Governance webhook returned error status {} for block {} at height {}",
                        response.status(),
                        hex::encode(block_hash),
                        height
                    );
                    let payload = EventPayload::WebhookFailed {
                        webhook_url: url.clone(),
                        event_type: event_type.to_string(),
                        error: format!("HTTP {}", response.status()),
                    };
                    let _ = node_api.publish_event(EventType::WebhookFailed, payload).await;
                }
            }
            Err(e) => {
                warn!(
                    "Failed to send governance webhook for block {} at height {}: {}",
                    hex::encode(block_hash),
                    height,
                    e
                );
                let payload = EventPayload::WebhookFailed {
                    webhook_url: url.clone(),
                    event_type: event_type.to_string(),
                    error: e.to_string(),
                };
                let _ = node_api.publish_event(EventType::WebhookFailed, payload).await;
            }
        }

        Ok(())
    }

    /// Calculate block hash (double SHA256 of block header)
    fn calculate_block_hash(&self, block: &blvm_protocol::Block) -> [u8; 32] {
        use sha2::{Digest, Sha256};

        // Serialize block header
        let mut header_data = Vec::new();
        header_data.extend_from_slice(&(block.header.version as u32).to_le_bytes());
        header_data.extend_from_slice(&block.header.prev_block_hash);
        header_data.extend_from_slice(&block.header.merkle_root);
        header_data.extend_from_slice(&block.header.timestamp.to_le_bytes());
        header_data.extend_from_slice(&block.header.bits.to_le_bytes());
        header_data.extend_from_slice(&block.header.nonce.to_le_bytes());

        // Double SHA256
        let first_hash = Sha256::digest(&header_data);
        let second_hash = Sha256::digest(first_hash);

        let mut hash = [0u8; 32];
        hash.copy_from_slice(&second_hash);
        hash
    }
}
