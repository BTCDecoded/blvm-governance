//! Governance webhook client

use crate::error::GovernanceError;
use blvm_node::module::EventType;
use blvm_node::module::ipc::protocol::EventPayload;
use blvm_node::module::ipc::protocol::ModuleMessage;
use blvm_node::module::traits::NodeAPI;
use reqwest::Client;
use tracing::{debug, info, warn};

/// Governance webhook client
pub struct GovernanceWebhookClient {
    client: Client,
    webhook_url: Option<String>,
    node_id: Option<String>,
    enabled: bool,
    /// Absent posts once. `Some(n)` posts up to `n` times, stopping after a success.
    webhook_retry_count: Option<u32>,
}

/// How many times `notify_governance_event` posts. An omitted count posts once.
pub(crate) fn webhook_post_attempts(retry_count: Option<u32>) -> u32 {
    match retry_count {
        Some(count) if count > 0 => count,
        _ => 1,
    }
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
        let webhook_retry_count = ctx
            .get_config("governance.webhook_retry_count")
            .and_then(|value| value.parse().ok());
        let enabled = webhook_url.is_some();

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|e| {
                GovernanceError::WebhookError(format!("Failed to create HTTP client: {e}"))
            })?;

        if enabled {
            if let Some(ref url) = webhook_url {
                info!("Governance webhook client initialized: {}", url);
            }
        } else {
            debug!("Governance webhook client disabled (no URL configured)");
        }

        Ok(Self {
            client,
            webhook_url,
            node_id,
            enabled,
            webhook_retry_count,
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
                    EventType::ModuleEvent => {
                        if let EventPayload::ModuleEvent { name, data, .. } = &event_msg.payload {
                            let value: serde_json::Value =
                                serde_json::from_slice(data).unwrap_or(serde_json::Value::Null);
                            let kind = match name.as_str() {
                                "GovernanceProposalCreated" => "proposal_created",
                                "GovernanceProposalVoted" => "proposal_voted",
                                "GovernanceProposalMerged" => "proposal_merged",
                                _ => "",
                            };
                            if !kind.is_empty() {
                                info!("Governance module event: {name}");
                                self.notify_governance_event(kind, value, node_api).await?;
                            }
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

        let Some(url) = self.webhook_url.as_ref() else {
            return Ok(());
        };

        // Prepare payload
        let payload = serde_json::json!({
            "event_type": event_type,
            "data": data,
            "node_id": self.node_id.as_deref(),
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        });

        let attempts = webhook_post_attempts(self.webhook_retry_count);
        let mut last_error = String::new();
        for _ in 0..attempts {
            match self.client.post(url).json(&payload).send().await {
                Ok(response) if response.status().is_success() => {
                    debug!(
                        "Governance webhook sent successfully: event_type={}",
                        event_type
                    );
                    let payload = EventPayload::module_event_json(
                        "WebhookSent",
                        &serde_json::json!({
                            "webhook_url": url,
                            "event_type": event_type,
                            "success": true,
                        }),
                    );
                    let _ = node_api
                        .publish_event(EventType::ModuleEvent, payload)
                        .await;
                    return Ok(());
                }
                Ok(response) => {
                    last_error = format!("HTTP {}", response.status());
                    warn!(
                        "Governance webhook returned error status {} for event_type={}",
                        response.status(),
                        event_type
                    );
                }
                Err(e) => {
                    last_error = e.to_string();
                    warn!(
                        "Failed to send governance webhook for event_type={}: {}",
                        event_type, e
                    );
                }
            }
        }
        let payload = EventPayload::module_event_json(
            "WebhookFailed",
            &serde_json::json!({
                "webhook_url": url,
                "event_type": event_type,
                "error": last_error,
            }),
        );
        let _ = node_api
            .publish_event(EventType::ModuleEvent, payload)
            .await;

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

        let Some(url) = self.webhook_url.as_ref() else {
            return Ok(());
        };

        // Calculate block hash
        let block_hash = self.calculate_block_hash(block);

        // Serialize block to JSON
        let block_json = serde_json::to_value(block).map_err(|e| {
            GovernanceError::WebhookError(format!("Failed to serialize block: {e}"))
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
                    let payload = EventPayload::module_event_json(
                        "WebhookSent",
                        &serde_json::json!({
                            "webhook_url": url,
                            "event_type": event_type,
                            "success": true,
                        }),
                    );
                    let _ = node_api
                        .publish_event(EventType::ModuleEvent, payload)
                        .await;
                } else {
                    warn!(
                        "Governance webhook returned error status {} for block {} at height {}",
                        response.status(),
                        hex::encode(block_hash),
                        height
                    );
                    let payload = EventPayload::module_event_json(
                        "WebhookFailed",
                        &serde_json::json!({
                            "webhook_url": url,
                            "event_type": event_type,
                            "error": format!("HTTP {}", response.status()),
                        }),
                    );
                    let _ = node_api
                        .publish_event(EventType::ModuleEvent, payload)
                        .await;
                }
            }
            Err(e) => {
                warn!(
                    "Failed to send governance webhook for block {} at height {}: {}",
                    hex::encode(block_hash),
                    height,
                    e
                );
                let payload = EventPayload::module_event_json(
                    "WebhookFailed",
                    &serde_json::json!({
                        "webhook_url": url,
                        "event_type": event_type,
                        "error": e.to_string(),
                    }),
                );
                let _ = node_api
                    .publish_event(EventType::ModuleEvent, payload)
                    .await;
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

#[cfg(test)]
mod webhook_attempt_tests {
    use super::webhook_post_attempts;

    #[test]
    fn omitted_retry_count_posts_once() {
        assert_eq!(webhook_post_attempts(None), 1);
        assert_eq!(webhook_post_attempts(Some(3)), 3);
    }
}
