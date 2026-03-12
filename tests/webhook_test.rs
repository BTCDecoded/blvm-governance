//! Unit tests for governance webhook client

use blvm_governance::webhook::GovernanceWebhookClient;
use blvm_node::module::ipc::protocol::{EventMessage, ModuleMessage};
use blvm_node::module::traits::{EventPayload, EventType, ModuleContext, NodeAPI};
use std::collections::HashMap;
use std::sync::Arc;

// Mock NodeAPI for testing
struct MockNodeAPI;

#[async_trait::async_trait]
impl NodeAPI for MockNodeAPI {
    async fn get_block(
        &self,
        _: &blvm_protocol::Hash,
    ) -> Result<Option<blvm_protocol::Block>, blvm_node::module::traits::ModuleError> {
        Ok(None)
    }
    async fn get_block_header(
        &self,
        _: &blvm_protocol::Hash,
    ) -> Result<Option<blvm_protocol::BlockHeader>, blvm_node::module::traits::ModuleError> {
        Ok(None)
    }
    async fn get_transaction(
        &self,
        _: &blvm_protocol::Hash,
    ) -> Result<Option<blvm_protocol::Transaction>, blvm_node::module::traits::ModuleError> {
        Ok(None)
    }
    async fn has_transaction(
        &self,
        _: &blvm_protocol::Hash,
    ) -> Result<bool, blvm_node::module::traits::ModuleError> {
        Ok(false)
    }
    async fn get_chain_tip(
        &self,
    ) -> Result<blvm_protocol::Hash, blvm_node::module::traits::ModuleError> {
        Ok([0u8; 32])
    }
    async fn get_block_height(&self) -> Result<u64, blvm_node::module::traits::ModuleError> {
        Ok(100)
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
    ) -> Result<Vec<blvm_protocol::Hash>, blvm_node::module::traits::ModuleError> {
        Ok(Vec::new())
    }
    async fn get_mempool_transaction(
        &self,
        _: &blvm_protocol::Hash,
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
            height: 100,
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
        _: &blvm_protocol::Hash,
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
async fn test_webhook_client_disabled() {
    let mut config = HashMap::new();
    // No webhook_url configured
    let temp = std::env::temp_dir();
    let ctx = ModuleContext {
        module_id: "test".to_string(),
        config,
        data_dir: temp.clone(),
        socket_path: temp.join("blvm_test.sock").to_string_lossy().into_owned(),
    };

    let client = GovernanceWebhookClient::new(&ctx).await.unwrap();
    assert!(!client.enabled);

    // Should return Ok(()) when disabled
    let event = ModuleMessage::Event(EventMessage {
        event_type: EventType::GovernanceProposalCreated,
        payload: EventPayload::GovernanceProposalCreated {
            proposal_id: "test".to_string(),
            tier: blvm_node::module::traits::ProposalTier::Standard,
            author: "test_author".to_string(),
            block_height: 100,
        },
    });

    let node_api = Arc::new(MockNodeAPI);
    let result = client.handle_event(&event, node_api.as_ref()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_webhook_client_enabled() {
    let mut config = HashMap::new();
    config.insert(
        "governance.webhook_url".to_string(),
        "http://localhost:8080/webhook".to_string(),
    );
    config.insert("governance.node_id".to_string(), "test_node".to_string());

    let temp = std::env::temp_dir();
    let ctx = ModuleContext {
        module_id: "test".to_string(),
        config,
        data_dir: temp.clone(),
        socket_path: temp.join("blvm_test.sock").to_string_lossy().into_owned(),
    };

    let client = GovernanceWebhookClient::new(&ctx).await.unwrap();
    assert!(client.enabled);
    assert_eq!(
        client.webhook_url.as_ref().unwrap(),
        "http://localhost:8080/webhook"
    );
    assert_eq!(client.node_id.as_ref().unwrap(), "test_node");
}
