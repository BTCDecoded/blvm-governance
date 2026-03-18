//! Unit tests for governance webhook client

mod common;

use blvm_governance::webhook::GovernanceWebhookClient;
use blvm_node::module::ipc::protocol::{EventMessage, EventPayload, ModuleMessage};
use blvm_node::module::traits::{EventType, ModuleContext};
use std::collections::HashMap;
use std::sync::Arc;

#[tokio::test]
async fn test_webhook_client_disabled() {
    let mut config = HashMap::new();
    let temp = std::env::temp_dir();
    let ctx = ModuleContext {
        module_id: "test".to_string(),
        config,
        data_dir: temp.to_string_lossy().to_string(),
        socket_path: temp.join("blvm_test.sock").to_string_lossy().into_owned(),
    };

    let client = GovernanceWebhookClient::new(&ctx).await.unwrap();
    assert!(!client.is_enabled());

    let event = ModuleMessage::Event(EventMessage {
        event_type: EventType::GovernanceProposalCreated,
        payload: EventPayload::GovernanceProposalCreated {
            proposal_id: "test".to_string(),
            repository: "test/repo".to_string(),
            pr_number: 1,
            tier: "standard".to_string(),
        },
    });

    let node_api = Arc::new(common::MockNodeAPI { block_height: 100 });
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
        data_dir: temp.to_string_lossy().to_string(),
        socket_path: temp.join("blvm_test.sock").to_string_lossy().into_owned(),
    };

    let client = GovernanceWebhookClient::new(&ctx).await.unwrap();
    assert!(client.is_enabled());
    assert_eq!(client.webhook_url().unwrap(), "http://localhost:8080/webhook");
    assert_eq!(client.node_id().unwrap(), "test_node");
}
