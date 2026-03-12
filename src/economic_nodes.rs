//! Economic node registry

#[cfg(test)]
mod tests {
    use super::*;
    use blvm_node::module::ipc::protocol::EventPayload;
    use blvm_node::module::ipc::protocol::ModuleMessage;
    use blvm_node::module::traits::{ModuleContext, NodeAPI};
    use blvm_node::module::EventType;
    use blvm_protocol::Hash;
    use std::collections::HashMap;
    use std::sync::Arc;

    // Simplified mock for testing
    struct MockNodeAPI {
        block_height: u64,
    }

    #[async_trait::async_trait]
    impl NodeAPI for MockNodeAPI {
        async fn get_block_height(&self) -> Result<u64, blvm_node::module::traits::ModuleError> {
            Ok(self.block_height)
        }
        // ... other methods return defaults
        async fn get_block(
            &self,
            _: &Hash,
        ) -> Result<Option<blvm_protocol::Block>, blvm_node::module::traits::ModuleError> {
            Ok(None)
        }
        async fn get_block_header(
            &self,
            _: &Hash,
        ) -> Result<Option<blvm_protocol::BlockHeader>, blvm_node::module::traits::ModuleError>
        {
            Ok(None)
        }
        async fn get_transaction(
            &self,
            _: &Hash,
        ) -> Result<Option<blvm_protocol::Transaction>, blvm_node::module::traits::ModuleError>
        {
            Ok(None)
        }
        async fn has_transaction(
            &self,
            _: &Hash,
        ) -> Result<bool, blvm_node::module::traits::ModuleError> {
            Ok(false)
        }
        async fn get_chain_tip(&self) -> Result<Hash, blvm_node::module::traits::ModuleError> {
            Ok([0u8; 32])
        }
        async fn get_utxo(
            &self,
            _: &blvm_protocol::OutPoint,
        ) -> Result<Option<blvm_protocol::UTXO>, blvm_node::module::traits::ModuleError> {
            Ok(None)
        }
        async fn subscribe_events(
            &self,
            _: Vec<EventType>,
        ) -> Result<
            tokio::sync::mpsc::Receiver<blvm_node::module::ipc::protocol::ModuleMessage>,
            blvm_node::module::traits::ModuleError,
        > {
            let (_tx, rx) = tokio::sync::mpsc::channel(100);
            Ok(rx)
        }
        async fn get_mempool_transactions(
            &self,
        ) -> Result<Vec<Hash>, blvm_node::module::traits::ModuleError> {
            Ok(Vec::new())
        }
        async fn get_mempool_transaction(
            &self,
            _: &Hash,
        ) -> Result<Option<blvm_protocol::Transaction>, blvm_node::module::traits::ModuleError>
        {
            Ok(None)
        }
        async fn get_mempool_size(
            &self,
        ) -> Result<blvm_node::module::traits::MempoolSize, blvm_node::module::traits::ModuleError>
        {
            Ok(blvm_node::module::traits::MempoolSize {
                count: 0,
                size_bytes: 0,
            })
        }
        async fn get_network_stats(
            &self,
        ) -> Result<blvm_node::module::traits::NetworkStats, blvm_node::module::traits::ModuleError>
        {
            Ok(blvm_node::module::traits::NetworkStats {
                connected_peers: 0,
                bytes_sent: 0,
                bytes_received: 0,
            })
        }
        async fn get_network_peers(
            &self,
        ) -> Result<Vec<blvm_node::module::traits::PeerInfo>, blvm_node::module::traits::ModuleError>
        {
            Ok(Vec::new())
        }
        async fn get_chain_info(
            &self,
        ) -> Result<blvm_node::module::traits::ChainInfo, blvm_node::module::traits::ModuleError>
        {
            Ok(blvm_node::module::traits::ChainInfo {
                tip: [0u8; 32],
                height: self.block_height,
                difficulty: 1.0,
            })
        }
        async fn get_block_by_height(
            &self,
            _: u64,
        ) -> Result<Option<blvm_protocol::Block>, blvm_node::module::traits::ModuleError> {
            Ok(None)
        }
        async fn get_lightning_node_url(
            &self,
        ) -> Result<Option<String>, blvm_node::module::traits::ModuleError> {
            Ok(None)
        }
        async fn get_lightning_info(
            &self,
        ) -> Result<
            Option<blvm_node::module::traits::LightningInfo>,
            blvm_node::module::traits::ModuleError,
        > {
            Ok(None)
        }
        async fn get_payment_state(
            &self,
            _: &str,
        ) -> Result<
            Option<blvm_node::module::traits::PaymentState>,
            blvm_node::module::traits::ModuleError,
        > {
            Ok(None)
        }
        async fn check_transaction_in_mempool(
            &self,
            _: &Hash,
        ) -> Result<bool, blvm_node::module::traits::ModuleError> {
            Ok(false)
        }
        async fn get_fee_estimate(
            &self,
            _: u32,
        ) -> Result<u64, blvm_node::module::traits::ModuleError> {
            Ok(1)
        }
        async fn read_file(
            &self,
            _: String,
        ) -> Result<Vec<u8>, blvm_node::module::traits::ModuleError> {
            Ok(Vec::new())
        }
        async fn write_file(
            &self,
            _: String,
            _: Vec<u8>,
        ) -> Result<(), blvm_node::module::traits::ModuleError> {
            Ok(())
        }
        async fn delete_file(
            &self,
            _: String,
        ) -> Result<(), blvm_node::module::traits::ModuleError> {
            Ok(())
        }
        async fn list_directory(
            &self,
            _: String,
        ) -> Result<Vec<String>, blvm_node::module::traits::ModuleError> {
            Ok(Vec::new())
        }
        async fn create_directory(
            &self,
            _: String,
        ) -> Result<(), blvm_node::module::traits::ModuleError> {
            Ok(())
        }
        async fn get_file_metadata(
            &self,
            _: String,
        ) -> Result<
            blvm_node::module::ipc::protocol::FileMetadata,
            blvm_node::module::traits::ModuleError,
        > {
            Ok(blvm_node::module::ipc::protocol::FileMetadata {
                size: 0,
                modified: 0,
                is_dir: false,
            })
        }
        async fn storage_open_tree(
            &self,
            _: String,
        ) -> Result<String, blvm_node::module::traits::ModuleError> {
            Ok("test".to_string())
        }
        async fn storage_insert(
            &self,
            _: String,
            _: Vec<u8>,
            _: Vec<u8>,
        ) -> Result<(), blvm_node::module::traits::ModuleError> {
            Ok(())
        }
        async fn storage_get(
            &self,
            _: String,
            _: Vec<u8>,
        ) -> Result<Option<Vec<u8>>, blvm_node::module::traits::ModuleError> {
            Ok(None)
        }
        async fn storage_remove(
            &self,
            _: String,
            _: Vec<u8>,
        ) -> Result<(), blvm_node::module::traits::ModuleError> {
            Ok(())
        }
        async fn storage_contains_key(
            &self,
            _: String,
            _: Vec<u8>,
        ) -> Result<bool, blvm_node::module::traits::ModuleError> {
            Ok(false)
        }
        async fn storage_iter(
            &self,
            _: String,
        ) -> Result<Vec<(Vec<u8>, Vec<u8>)>, blvm_node::module::traits::ModuleError> {
            Ok(Vec::new())
        }
        async fn storage_transaction(
            &self,
            _: String,
            _: Vec<blvm_node::module::ipc::protocol::StorageOperation>,
        ) -> Result<(), blvm_node::module::traits::ModuleError> {
            Ok(())
        }
        async fn register_rpc_endpoint(
            &self,
            _: String,
            _: String,
        ) -> Result<(), blvm_node::module::traits::ModuleError> {
            Ok(())
        }
        async fn unregister_rpc_endpoint(
            &self,
            _: &str,
        ) -> Result<(), blvm_node::module::traits::ModuleError> {
            Ok(())
        }
        async fn register_timer(
            &self,
            _: u64,
            _: Arc<dyn blvm_node::module::timers::manager::TimerCallback>,
        ) -> Result<
            blvm_node::module::timers::manager::TimerId,
            blvm_node::module::traits::ModuleError,
        > {
            Ok(0)
        }
        async fn cancel_timer(
            &self,
            _: blvm_node::module::timers::manager::TimerId,
        ) -> Result<(), blvm_node::module::traits::ModuleError> {
            Ok(())
        }
        async fn schedule_task(
            &self,
            _: u64,
            _: Arc<dyn blvm_node::module::timers::manager::TaskCallback>,
        ) -> Result<
            blvm_node::module::timers::manager::TaskId,
            blvm_node::module::traits::ModuleError,
        > {
            Ok(0)
        }
        async fn report_metric(
            &self,
            _: blvm_node::module::metrics::manager::Metric,
        ) -> Result<(), blvm_node::module::traits::ModuleError> {
            Ok(())
        }
        async fn get_module_metrics(
            &self,
            _: &str,
        ) -> Result<
            Vec<blvm_node::module::metrics::manager::Metric>,
            blvm_node::module::traits::ModuleError,
        > {
            Ok(Vec::new())
        }
        async fn initialize_module(
            &self,
            _: &str,
            _: blvm_node::module::traits::ModuleManifest,
        ) -> Result<(), blvm_node::module::traits::ModuleError> {
            Ok(())
        }
        async fn discover_modules(
            &self,
        ) -> Result<
            Vec<blvm_node::module::traits::ModuleInfo>,
            blvm_node::module::traits::ModuleError,
        > {
            Ok(Vec::new())
        }
        async fn get_module_info(
            &self,
            _: &str,
        ) -> Result<
            Option<blvm_node::module::traits::ModuleInfo>,
            blvm_node::module::traits::ModuleError,
        > {
            Ok(None)
        }
        async fn is_module_available(
            &self,
            _: &str,
        ) -> Result<bool, blvm_node::module::traits::ModuleError> {
            Ok(false)
        }
        async fn publish_event(
            &self,
            _: EventType,
            _: EventPayload,
        ) -> Result<(), blvm_node::module::traits::ModuleError> {
            Ok(())
        }
        async fn call_module(
            &self,
            _: Option<&str>,
            _: &str,
            _: Vec<u8>,
        ) -> Result<Vec<u8>, blvm_node::module::traits::ModuleError> {
            Ok(Vec::new())
        }
        async fn register_module_api(
            &self,
            _: Vec<String>,
            _: u32,
        ) -> Result<(), blvm_node::module::traits::ModuleError> {
            Ok(())
        }
        async fn unregister_module_api(
            &self,
        ) -> Result<(), blvm_node::module::traits::ModuleError> {
            Ok(())
        }
        async fn get_module_health(
            &self,
            _: &str,
        ) -> Result<
            Option<blvm_node::module::process::monitor::ModuleHealth>,
            blvm_node::module::traits::ModuleError,
        > {
            Ok(None)
        }
        async fn get_all_module_health(
            &self,
        ) -> Result<
            Vec<(String, blvm_node::module::process::monitor::ModuleHealth)>,
            blvm_node::module::traits::ModuleError,
        > {
            Ok(Vec::new())
        }
        async fn report_module_health(
            &self,
            _: blvm_node::module::process::monitor::ModuleHealth,
        ) -> Result<(), blvm_node::module::traits::ModuleError> {
            Ok(())
        }
        async fn send_mesh_packet_to_module(
            &self,
            _: &str,
            _: Vec<u8>,
            _: String,
        ) -> Result<(), blvm_node::module::traits::ModuleError> {
            Ok(())
        }
        async fn send_mesh_packet_to_peer(
            &self,
            _: String,
            _: Vec<u8>,
        ) -> Result<(), blvm_node::module::traits::ModuleError> {
            Ok(())
        }
        async fn send_stratum_v2_message_to_peer(
            &self,
            _: String,
            _: Vec<u8>,
        ) -> Result<(), blvm_node::module::traits::ModuleError> {
            Ok(())
        }
        async fn get_node_public_key(
            &self,
        ) -> Result<Option<Vec<u8>>, blvm_node::module::traits::ModuleError> {
            Ok(None)
        }
        async fn get_event_publisher(
            &self,
        ) -> Result<
            Option<Arc<blvm_node::node::event_publisher::EventPublisher>>,
            blvm_node::module::traits::ModuleError,
        > {
            Ok(None)
        }
    }

    #[tokio::test]
    async fn test_economic_node_registration() {
        let temp = std::env::temp_dir();
        let ctx = ModuleContext {
            module_id: "test".to_string(),
            config: HashMap::new(),
            data_dir: temp.clone(),
            socket_path: temp.join("blvm_test.sock").to_string_lossy().into_owned(),
        };

        let node_api = Arc::new(MockNodeAPI { block_height: 100 });
        let registry = EconomicNodeRegistry::new(&ctx, node_api.clone())
            .await
            .unwrap();

        let node_id = [1u8; 32];
        let event = ModuleMessage::Event(blvm_node::module::ipc::protocol::EventMessage {
            event_type: EventType::EconomicNodeRegistered,
            payload: EventPayload::EconomicNodeRegistered {
                node_id,
                public_key: vec![2u8; 33],
                hashpower_percentage: 50.0,
                economic_activity_percentage: 30.0,
                block_height: 100,
            },
        });

        registry
            .handle_event(&event, node_api.as_ref())
            .await
            .unwrap();

        let nodes = registry.nodes.read().await;
        assert!(nodes.contains_key(&node_id));
        let node = nodes.get(&node_id).unwrap();
        assert_eq!(node.hashpower_percentage, 0.5);
        assert_eq!(node.economic_activity_percentage, 0.3);
    }
}

use crate::error::GovernanceError;
use blvm_node::module::ipc::protocol::EventPayload;
use blvm_node::module::ipc::protocol::ModuleMessage;
use blvm_node::module::traits::{ModuleContext, NodeAPI};
use blvm_node::module::EventType;
use blvm_protocol::Hash;
use hex;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Economic node information
#[derive(Debug, Clone)]
pub struct EconomicNode {
    pub node_id: [u8; 32],
    pub public_key: Vec<u8>,
    pub hashpower_percentage: f64,
    pub economic_activity_percentage: f64,
    pub registered_at: u64,
    pub last_seen: u64,
    pub veto_count: u32,
}

/// Economic node registry
pub struct EconomicNodeRegistry {
    nodes: Arc<RwLock<HashMap<[u8; 32], EconomicNode>>>,
    node_api: Arc<dyn NodeAPI>,
}

impl EconomicNodeRegistry {
    /// Create a new economic node registry
    pub async fn new(
        _ctx: &ModuleContext,
        node_api: Arc<dyn NodeAPI>,
    ) -> Result<Self, GovernanceError> {
        Ok(Self {
            nodes: Arc::new(RwLock::new(HashMap::new())),
            node_api,
        })
    }

    /// Handle governance events
    pub async fn handle_event(
        &self,
        event: &ModuleMessage,
        _node_api: &dyn NodeAPI,
    ) -> Result<(), GovernanceError> {
        match event {
            ModuleMessage::Event(event_msg) => {
                match event_msg.event_type {
                    EventType::EconomicNodeRegistered => {
                        if let EventPayload::EconomicNodeRegistered {
                            node_id,
                            node_type,
                            hashpower_percent,
                        } = &event_msg.payload
                        {
                            let mut nodes = self.nodes.write().await;
                            let current_height =
                                self.node_api.get_block_height().await.unwrap_or(0);

                            // Parse node_id from String to [u8; 32]
                            let node_id_bytes = if node_id.len() == 64 {
                                // Hex string
                                hex::decode(node_id).ok().and_then(|v| {
                                    if v.len() == 32 {
                                        let mut arr = [0u8; 32];
                                        arr.copy_from_slice(&v);
                                        Some(arr)
                                    } else {
                                        None
                                    }
                                })
                            } else {
                                None
                            };

                            if let Some(node_id_bytes) = node_id_bytes {
                                nodes.insert(
                                    node_id_bytes,
                                    EconomicNode {
                                        node_id: node_id_bytes,
                                        public_key: Vec::new(), // Not provided in event
                                        hashpower_percentage: hashpower_percent.unwrap_or(0.0),
                                        economic_activity_percentage: 0.0, // Not provided in event
                                        registered_at: current_height,
                                        last_seen: current_height,
                                        veto_count: 0,
                                    },
                                );

                                info!(
                                    "Registered economic node: {}, type: {}, hashpower: {:?}%",
                                    node_id, node_type, hashpower_percent
                                );
                            }
                        }
                    }
                    EventType::EconomicNodeVeto => {
                        if let EventPayload::EconomicNodeVeto {
                            proposal_id,
                            node_id,
                            reason,
                        } = &event_msg.payload
                        {
                            let mut nodes = self.nodes.write().await;
                            // Parse node_id from String to [u8; 32]
                            if let Ok(node_id_bytes) = hex::decode(node_id) {
                                if node_id_bytes.len() == 32 {
                                    let mut arr = [0u8; 32];
                                    arr.copy_from_slice(&node_id_bytes);
                                    if let Some(node) = nodes.get_mut(&arr) {
                                        node.veto_count += 1;
                                        warn!("Economic node vetoed: {}, proposal: {}, reason: {}, veto count: {}",
                                            node_id, proposal_id, reason, node.veto_count);
                                    }
                                }
                            }
                        }
                    }
                    EventType::NewBlock => {
                        if let EventPayload::NewBlock { height, .. } = &event_msg.payload {
                            let mut nodes = self.nodes.write().await;
                            for node in nodes.values_mut() {
                                node.last_seen = *height;
                            }
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        Ok(())
    }
}
