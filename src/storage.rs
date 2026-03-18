//! Storage migrations for blvm-governance.
//!
//! v1: Migrate proposals from legacy "items" tree to "proposals".

use blvm_sdk::module::{MigrationContext, MigrationUp};

const PROPOSALS_TREE: &str = "proposals";

pub fn up_v1(ctx: &MigrationContext) -> anyhow::Result<()> {
    let items_tree = ctx.open_tree("items")?;
    if let Some(data) = items_tree.get(b"governance:proposals")? {
        let proposals_tree = ctx.open_tree(PROPOSALS_TREE)?;
        proposals_tree.insert(b"proposals", &data)?;
    }
    Ok(())
}

pub const MIGRATIONS: &[(u32, MigrationUp)] = &[(1, up_v1)];
