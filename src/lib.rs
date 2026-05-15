//! Governance webhook and proposal tracking module for blvm-node

pub mod api;
pub mod config;
pub mod error;
pub mod module;
pub mod proposals;
pub mod storage;
pub mod webhook;

pub use config::GovernanceConfig;
pub use module::GovernanceModule;
