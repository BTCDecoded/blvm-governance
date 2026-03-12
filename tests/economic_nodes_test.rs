//! Unit tests for economic node registry

use blvm_governance::economic_nodes::EconomicNodeRegistry;
use blvm_node::module::ipc::protocol::{EventMessage, ModuleMessage};
use blvm_node::module::traits::{EventPayload, EventType, ModuleContext, NodeAPI};
use blvm_protocol::Hash;
use std::collections::HashMap;
use std::sync::Arc;

// Mock NodeAPI implementation for testing
struct MockNodeAPI {
    block_height: u64,
}

#[async_trait::async_trait]
impl NodeAPI for MockNodeAPI {
    async fn get_block_height(&self) -> Result<u64, blvm_node::module::traits::ModuleError> {
        Ok(self.block_height)
    }
    // Implement all required methods with defaults
    async fn get_block(
        &self,
        _: &Hash,
    ) -> Result<Option<blvm_protocol::Block>, blvm_node::module::traits::ModuleError> {
        Ok(None)
    }
    async fn get_block_header(
        &self,
        _: &Hash,
    ) -> Result<Option<blvm_protocol::BlockHeader>, blvm_node::module::traits::ModuleError> {
        Ok(None)
    }
    async fn get_transaction(
        &self,
        _: &Hash,
    ) -> Result<Option<blvm_protocol::Transaction>, blvm_node::module::traits::ModuleError> {
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
    ) -> Result<tokio::sync::mpsc::Receiver<ModuleMessage>, blvm_node::module::traits::ModuleError>
    {
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
    ) -> Result<Option<blvm_protocol::Transaction>, blvm_node::module::traits::ModuleError> {
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
    async fn delete_file(&self, _: String) -> Result<(), blvm_node::module::traits::ModuleError> {
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
    ) -> Result<blvm_node::module::timers::manager::TimerId, blvm_node::module::traits::ModuleError>
    {
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
    ) -> Result<blvm_node::module::timers::manager::TaskId, blvm_node::module::traits::ModuleError>
    {
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
    ) -> Result<Vec<blvm_node::module::traits::ModuleInfo>, blvm_node::module::traits::ModuleError>
    {
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
    async fn unregister_module_api(&self) -> Result<(), blvm_node::module::traits::ModuleError> {
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
    let event = ModuleMessage::Event(EventMessage {
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
    assert_eq!(node.registered_at, 100);
}
