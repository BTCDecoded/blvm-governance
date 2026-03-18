//! Shared test utilities for governance tests

use blvm_node::module::ipc::protocol::{EventMessage, ModuleMessage};
use blvm_node::module::traits::{EventType, NodeAPI};
use blvm_protocol::Hash;
use std::collections::HashMap;
use std::sync::Arc;

/// Minimal MockNodeAPI for governance tests - implements only required NodeAPI methods.
pub struct MockNodeAPI {
    pub block_height: u64,
}

#[async_trait::async_trait]
impl NodeAPI for MockNodeAPI {
    async fn get_block_height(&self) -> Result<u64, blvm_node::module::traits::ModuleError> {
        Ok(self.block_height)
    }
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
    ) -> Result<
        tokio::sync::mpsc::Receiver<ModuleMessage>,
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
    ) -> Result<Option<blvm_protocol::Transaction>, blvm_node::module::traits::ModuleError> {
        Ok(None)
    }
    async fn get_mempool_size(
        &self,
    ) -> Result<blvm_node::module::traits::MempoolSize, blvm_node::module::traits::ModuleError>
    {
        Ok(blvm_node::module::traits::MempoolSize {
            transaction_count: 0,
            size_bytes: 0,
            total_fee_sats: 0,
        })
    }
    async fn get_network_stats(
        &self,
    ) -> Result<
        blvm_node::module::traits::NetworkStats,
        blvm_node::module::traits::ModuleError,
    > {
        Ok(blvm_node::module::traits::NetworkStats {
            peer_count: 0,
            hash_rate: 0.0,
            bytes_sent: 0,
            bytes_received: 0,
        })
    }
    async fn get_network_peers(
        &self,
    ) -> Result<
        Vec<blvm_node::module::traits::PeerInfo>,
        blvm_node::module::traits::ModuleError,
    > {
        Ok(Vec::new())
    }
    async fn get_chain_info(
        &self,
    ) -> Result<blvm_node::module::traits::ChainInfo, blvm_node::module::traits::ModuleError>
    {
        Ok(blvm_node::module::traits::ChainInfo {
            tip_hash: [0u8; 32],
            height: self.block_height,
            difficulty: 1,
            chain_work: 0,
            is_synced: true,
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
            path: String::new(),
            size: 0,
            is_file: false,
            is_directory: false,
            modified: None,
            created: None,
        })
    }
    async fn get_all_metrics(
        &self,
    ) -> Result<
        HashMap<String, Vec<blvm_node::module::metrics::manager::Metric>>,
        blvm_node::module::traits::ModuleError,
    > {
        Ok(HashMap::new())
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
        _: String,
        _: std::path::PathBuf,
        _: std::path::PathBuf,
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
        _: blvm_node::module::ipc::protocol::EventPayload,
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
        _: Arc<dyn blvm_node::module::inter_module::api::ModuleAPI>,
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
    async fn get_block_template(
        &self,
        _: Vec<String>,
        _: Option<Vec<u8>>,
        _: Option<String>,
    ) -> Result<blvm_protocol::mining::BlockTemplate, blvm_node::module::traits::ModuleError> {
        Err(blvm_node::module::traits::ModuleError::Other(
            "not implemented".into(),
        ))
    }
    async fn submit_block(
        &self,
        _: blvm_protocol::Block,
    ) -> Result<
        blvm_node::module::traits::SubmitBlockResult,
        blvm_node::module::traits::ModuleError,
    > {
        Err(blvm_node::module::traits::ModuleError::Other(
            "not implemented".into(),
        ))
    }
}
