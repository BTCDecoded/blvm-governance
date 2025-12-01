//! bllvm-governance - Governance webhook and economic node tracking module
//!
//! This module provides governance integration for bllvm-node, including
//! webhook notifications, economic node tracking, and veto system integration.

use anyhow::Result;
use bllvm_node::module::ipc::protocol::{EventMessage, EventPayload, EventType, LogLevel, ModuleMessage};
use clap::Parser;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

mod webhook;
mod economic_nodes;
mod error;
mod client;
mod nodeapi_ipc;

use error::GovernanceError;
use client::ModuleClient;
use nodeapi_ipc::NodeApiIpc;

/// Command-line arguments for the module
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Module ID (provided by node)
    #[arg(long)]
    module_id: Option<String>,

    /// IPC socket path (provided by node)
    #[arg(long)]
    socket_path: Option<PathBuf>,

    /// Data directory (provided by node)
    #[arg(long)]
    data_dir: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let args = Args::parse();

    // Get module ID (from args or environment)
    let module_id = args.module_id
        .or_else(|| std::env::var("MODULE_NAME").ok())
        .unwrap_or_else(|| "bllvm-governance".to_string());

    // Get socket path (from args, env, or default)
    let socket_path = args.socket_path
        .or_else(|| std::env::var("BLLVM_MODULE_SOCKET").ok().map(PathBuf::from))
        .or_else(|| std::env::var("MODULE_SOCKET_DIR").ok().map(|d| PathBuf::from(d).join("modules.sock")))
        .unwrap_or_else(|| PathBuf::from("data/modules/modules.sock"));

    info!("bllvm-governance module starting... (module_id: {}, socket: {:?})", module_id, socket_path);

    // Connect to node
    let mut client = match ModuleClient::connect(
        socket_path,
        module_id.clone(),
        "bllvm-governance".to_string(),
        env!("CARGO_PKG_VERSION").to_string(),
    ).await {
        Ok(client) => client,
        Err(e) => {
            error!("Failed to connect to node: {}", e);
            return Err(anyhow::anyhow!("Connection failed: {}", e));
        }
    };

    // Subscribe to governance events
    let event_types = vec![
        EventType::GovernanceProposalCreated,
        EventType::GovernanceProposalVoted,
        EventType::GovernanceProposalMerged,
        EventType::EconomicNodeRegistered,
        EventType::EconomicNodeVeto,
        EventType::NewBlock, // For tracking block height
    ];

    if let Err(e) = client.subscribe_events(event_types).await {
        error!("Failed to subscribe to events: {}", e);
        return Err(anyhow::anyhow!("Subscription failed: {}", e));
    }

    // Create NodeAPI wrapper
    let ipc_client = client.get_ipc_client();
    let node_api = Arc::new(NodeApiIpc::new(ipc_client));

    // Create webhook client and economic node registry
    let ctx = bllvm_node::module::traits::ModuleContext {
        module_id: module_id.clone(),
        config: std::collections::HashMap::new(),
        data_dir: args.data_dir.unwrap_or_else(|| PathBuf::from("data/modules/bllvm-governance")),
        socket_path: socket_path.to_string_lossy().to_string(),
    };

    let webhook_client = webhook::GovernanceWebhookClient::new(&ctx).await
        .map_err(|e| anyhow::anyhow!("Failed to create webhook client: {}", e))?;
    let economic_nodes = economic_nodes::EconomicNodeRegistry::new(&ctx, Arc::clone(&node_api)).await
        .map_err(|e| anyhow::anyhow!("Failed to create economic node registry: {}", e))?;

    info!("Governance module initialized and running");

    // Event processing loop
    let mut event_receiver = client.event_receiver();
    while let Some(event) = event_receiver.recv().await {
        // Handle events with webhook client
        if let Err(e) = webhook_client.handle_event(&event, node_api.as_ref()).await {
            warn!("Error handling event in webhook client: {}", e);
        }

        // Handle events with economic node registry
        if let Err(e) = economic_nodes.handle_event(&event, node_api.as_ref()).await {
            warn!("Error handling event in economic node registry: {}", e);
        }

        match event {
            ModuleMessage::Event(event_msg) => {
                match event_msg.event_type {
                    EventType::GovernanceProposalCreated => {
                        info!("Governance proposal created event received");
                    }
                    EventType::GovernanceProposalVoted => {
                        info!("Governance proposal voted event received");
                    }
                    EventType::GovernanceProposalMerged => {
                        info!("Governance proposal merged event received");
                    }
                    EventType::EconomicNodeRegistered => {
                        info!("Economic node registered event received");
                    }
                    EventType::EconomicNodeVeto => {
                        warn!("Economic node veto event received");
                    }
                    EventType::NewBlock => {
                        // Track block height for governance
                        debug!("New block event received (tracking for governance)");
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
    }

    warn!("Event receiver closed, module shutting down");
    Ok(())
}
