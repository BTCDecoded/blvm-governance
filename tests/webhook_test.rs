//! Unit tests for governance webhook client

mod common;

use blvm_governance::webhook::GovernanceWebhookClient;
use blvm_node::module::ipc::protocol::{EventMessage, EventPayload, ModuleMessage};
use blvm_node::module::traits::{EventType, ModuleContext};
use std::collections::HashMap;
use std::sync::Arc;

#[tokio::test]
async fn test_webhook_client_disabled() {
    let config = HashMap::new();
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
        event_type: EventType::ModuleEvent,
        payload: EventPayload::module_event(
            "GovernanceProposalCreated",
            br#"{"proposal_id":"test","repository":"test/repo","pr_number":1,"tier":"standard"}"#
                .to_vec(),
        ),
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
    assert_eq!(
        client.webhook_url().unwrap(),
        "http://localhost:8080/webhook"
    );
    assert_eq!(client.node_id().unwrap(), "test_node");
}

fn governance_event(source: &str) -> ModuleMessage {
    ModuleMessage::Event(EventMessage {
        event_type: EventType::ModuleEvent,
        payload: EventPayload::ModuleEvent {
            source_module: source.to_string(),
            name: "GovernanceProposalVoted".to_string(),
            data: br#"{"proposal_id":"p1","voter":"alice","vote":"approve"}"#.to_vec(),
        },
    })
}

#[tokio::test]
async fn a_foreign_governance_event_is_not_posted_and_ours_is() {
    use std::io::Write;
    use std::sync::atomic::{AtomicUsize, Ordering};

    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let hits = Arc::new(AtomicUsize::new(0));
    let recorded = Arc::clone(&hits);
    std::thread::spawn(move || {
        let Ok((mut stream, _)) = listener.accept() else {
            return;
        };
        recorded.fetch_add(1, Ordering::SeqCst);
        let _ = stream.write_all(
            b"HTTP/1.1 204 No Content\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
        );
    });

    let mut config = HashMap::new();
    config.insert(
        "governance.webhook_url".to_string(),
        format!("http://127.0.0.1:{port}/webhook"),
    );
    let temp = std::env::temp_dir();
    let ctx = ModuleContext {
        module_id: "blvm-governance".to_string(),
        config,
        data_dir: temp.to_string_lossy().to_string(),
        socket_path: temp.join("blvm_test.sock").to_string_lossy().into_owned(),
    };
    let client = GovernanceWebhookClient::new(&ctx).await.unwrap();
    let node_api = Arc::new(common::MockNodeAPI { block_height: 100 });

    client
        .handle_event(&governance_event("blvm-mesh"), node_api.as_ref())
        .await
        .unwrap();
    assert_eq!(hits.load(Ordering::SeqCst), 0);

    client
        .handle_event(&governance_event("blvm-governance"), node_api.as_ref())
        .await
        .unwrap();
    for _ in 0..50 {
        if hits.load(Ordering::SeqCst) == 1 {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    assert_eq!(hits.load(Ordering::SeqCst), 1);
}
