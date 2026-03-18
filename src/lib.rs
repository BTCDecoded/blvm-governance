//! Governance webhook and economic node tracking module for blvm-node

pub mod api;
pub mod config;
pub mod module;
pub mod economic_nodes;
pub mod error;
pub mod proposals;
pub mod storage;
pub mod webhook;

pub use config::GovernanceConfig;
pub use module::GovernanceModule;
pub use economic_nodes::{EconomicNode, EconomicNodeRegistry};
