//! blvm-governance - Governance webhook and economic node tracking module
//!
//! When spawned by the node: reads MODULE_ID, SOCKET_PATH, DATA_DIR from env.
//! For manual testing: blvm-governance --module-id <id> --socket-path <path> --data-dir <dir>

use anyhow::Result;
use blvm_governance::storage::up_v1;
use blvm_governance::{
    api::GovernanceModuleApi,
    economic_nodes, proposals, webhook,
    GovernanceConfig, GovernanceModule,
};
use blvm_sdk::migrations;
use blvm_sdk::module::{ModuleBootstrap, ModuleDb};
use std::sync::Arc;
use tracing::warn;

const MODULE_NAME: &str = "blvm-governance";

#[tokio::main]
async fn main() -> Result<()> {
    let bootstrap = ModuleBootstrap::init_module(MODULE_NAME);
    let db = ModuleDb::open_with_migrations(&bootstrap.data_dir, migrations!(1 => up_v1))?;

    let setup = |node_api: Arc<dyn blvm_node::module::traits::NodeAPI>,
                 db: Arc<dyn blvm_node::storage::database::Database>,
                 data_dir: &std::path::Path| {
        let bootstrap = bootstrap.clone();
        let data_dir = data_dir.to_path_buf();
        async move {
            let (ctx, config) = bootstrap.context_with_config::<GovernanceConfig>(&data_dir);
            let webhook_url = config.webhook_url.clone();
            let webhook_client = webhook::GovernanceWebhookClient::new(&ctx)
                .await
                .map_err(|e| blvm_node::module::traits::ModuleError::Other(format!("Failed to create webhook client: {}", e)))?;
            let economic_nodes = Arc::new(
                economic_nodes::EconomicNodeRegistry::new(&ctx, Arc::clone(&node_api))
                    .await
                    .map_err(|e| blvm_node::module::traits::ModuleError::Other(format!("Failed to create economic node registry: {}", e)))?,
            );
            let proposal_store = Arc::new(proposals::ProposalStore::new(Arc::clone(&db)));
            let governance_api = Arc::new(GovernanceModuleApi::new(
                Arc::clone(&proposal_store),
                Arc::clone(&economic_nodes),
                webhook_url,
                Arc::clone(&node_api),
            ));
            if let Err(e) = node_api.register_module_api(governance_api).await {
                warn!("Failed to register governance module API: {}", e);
            }
            tracing::info!("Governance module initialized and running");
            let module = GovernanceModule {
                proposal_store,
                webhook_client: Arc::new(webhook_client),
                economic_nodes,
            };
            Ok((module.clone(), module))
        }
    };

    blvm_sdk::run_module! {
        bootstrap: &bootstrap,
        module_name: MODULE_NAME,
        module_type: GovernanceModule,
        cli_type: GovernanceModule,
        db: db.as_db(),
        setup: setup,
        event_types: GovernanceModule::event_types(),
    }?;

    warn!("Event receiver closed, module shutting down");
    Ok(())
}
