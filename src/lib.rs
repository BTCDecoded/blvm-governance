//! Governance webhook and economic node tracking module for blvm-node

pub mod client;
pub mod economic_nodes;
pub mod error;
pub mod nodeapi_ipc;
pub mod webhook;

pub use economic_nodes::{EconomicNode, EconomicNodeRegistry};
