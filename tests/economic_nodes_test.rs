//! Unit tests for economic node registry

mod common;

use blvm_governance::economic_nodes::EconomicNodeRegistry;
use blvm_node::module::ipc::protocol::{EventMessage, EventPayload, ModuleMessage};
use blvm_node::module::traits::{EventType, ModuleContext};
use std::collections::HashMap;
use std::sync::Arc;

#[tokio::test]
async fn test_economic_node_registration() {
    let temp = std::env::temp_dir();
    let ctx = ModuleContext {
        module_id: "test".to_string(),
        config: HashMap::new(),
        data_dir: temp.to_string_lossy().to_string(),
        socket_path: temp.join("blvm_test.sock").to_string_lossy().into_owned(),
    };

    let node_api = Arc::new(common::MockNodeAPI { block_height: 100 });
    let registry = EconomicNodeRegistry::new(&ctx, node_api.clone())
        .await
        .unwrap();

    let node_id = [1u8; 32];
    let node_id_hex = hex::encode(node_id);
    let event = ModuleMessage::Event(EventMessage {
        event_type: EventType::EconomicNodeRegistered,
        payload: EventPayload::EconomicNodeRegistered {
            node_id: node_id_hex,
            node_type: "miner".to_string(),
            hashpower_percent: Some(0.5),
        },
    });

    registry
        .handle_event(&event, node_api.as_ref())
        .await
        .unwrap();

    let nodes = registry.get_nodes_for_test().await;
    assert!(nodes.contains_key(&node_id));
    let node = nodes.get(&node_id).unwrap();
    assert_eq!(node.hashpower_percentage, 0.5);
    assert_eq!(node.economic_activity_percentage, 0.0);
    assert_eq!(node.registered_at, 100);
}
