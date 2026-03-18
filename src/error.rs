//! Error types for Governance module

use thiserror::Error;

#[derive(Debug, Error)]
pub enum GovernanceError {
    #[error("Module error: {0}")]
    ModuleError(String),

    #[error("Webhook error: {0}")]
    WebhookError(String),

    #[error("Economic node error: {0}")]
    EconomicNodeError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Storage error: {0}")]
    Storage(String),
}

impl From<GovernanceError> for blvm_node::module::traits::ModuleError {
    fn from(e: GovernanceError) -> Self {
        Self::OperationError(e.to_string())
    }
}
