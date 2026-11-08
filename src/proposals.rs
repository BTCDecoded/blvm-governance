//! Governance proposal storage
//!
//! Stores proposals when receiving GovernanceProposal* events so list-proposals can display them.

use blvm_node::module::EventType;
use blvm_node::module::ipc::protocol::{EventPayload, ModuleMessage};
use serde::{Deserialize, Serialize};

const PROPOSALS_TREE: &str = "proposals";
use std::collections::HashMap;
use std::sync::Arc;

const STORAGE_KEY: &[u8] = b"proposals";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceProposal {
    pub proposal_id: String,
    pub repository: String,
    pub pr_number: u64,
    pub tier: String,
    pub status: ProposalStatus,
    pub votes: Vec<ProposalVote>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProposalStatus {
    Created,
    Merged,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposalVote {
    pub voter: String,
    pub vote: String,
}

/// Proposal store backed by module DB
pub struct ProposalStore {
    db: Arc<dyn blvm_node::storage::database::Database>,
}

impl ProposalStore {
    pub fn new(db: Arc<dyn blvm_node::storage::database::Database>) -> Self {
        Self { db }
    }

    /// Load proposals for RPC/API (read-only).
    pub fn load_proposals(&self) -> Result<Vec<GovernanceProposal>, crate::error::GovernanceError> {
        Self::load_for_display(&self.db)
    }

    pub fn handle_event(&self, event: &ModuleMessage) -> Result<(), crate::error::GovernanceError> {
        let ModuleMessage::Event(event_msg) = event else {
            return Ok(());
        };
        let mut proposals = self.load()?;
        if event_msg.event_type == EventType::ModuleEvent {
            if let EventPayload::ModuleEvent { name, data, .. } = &event_msg.payload {
                let value: serde_json::Value =
                    serde_json::from_slice(data).unwrap_or(serde_json::Value::Null);
                match name.as_str() {
                    "GovernanceProposalCreated" => {
                        apply_created(
                            &mut proposals,
                            json_string(&value, "proposal_id"),
                            json_string(&value, "repository"),
                            value
                                .get("pr_number")
                                .and_then(|item| item.as_u64())
                                .unwrap_or(0),
                            json_string(&value, "tier"),
                        )?;
                        self.save(&proposals)?;
                    }
                    "GovernanceProposalVoted" => {
                        let proposal_id = json_string(&value, "proposal_id");
                        apply_voted(
                            &mut proposals,
                            &proposal_id,
                            json_string(&value, "voter"),
                            json_string(&value, "vote"),
                        )?;
                        self.save(&proposals)?;
                    }
                    "GovernanceProposalMerged" => {
                        let proposal_id = json_string(&value, "proposal_id");
                        apply_merged(
                            &mut proposals,
                            &proposal_id,
                            json_string(&value, "repository"),
                            value
                                .get("pr_number")
                                .and_then(|item| item.as_u64())
                                .unwrap_or(0),
                        )?;
                        self.save(&proposals)?;
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn load(&self) -> Result<HashMap<String, GovernanceProposal>, crate::error::GovernanceError> {
        let tree = self
            .db
            .open_tree(PROPOSALS_TREE)
            .map_err(|e| crate::error::GovernanceError::Storage(format!("open_tree: {e}")))?;
        match tree.get(STORAGE_KEY) {
            Ok(Some(data)) => bincode::deserialize(&data)
                .map_err(|e| crate::error::GovernanceError::Storage(format!("deserialize: {e}"))),
            Ok(None) => Ok(HashMap::new()),
            Err(e) => Err(crate::error::GovernanceError::Storage(format!("get: {e}"))),
        }
    }

    fn save(
        &self,
        proposals: &HashMap<String, GovernanceProposal>,
    ) -> Result<(), crate::error::GovernanceError> {
        let tree = self
            .db
            .open_tree(PROPOSALS_TREE)
            .map_err(|e| crate::error::GovernanceError::Storage(format!("open_tree: {e}")))?;
        let data = bincode::serialize(proposals)
            .map_err(|e| crate::error::GovernanceError::Storage(format!("serialize: {e}")))?;
        tree.insert(STORAGE_KEY, &data)
            .map_err(|e| crate::error::GovernanceError::Storage(format!("insert: {e}")))?;
        Ok(())
    }

    /// Load proposals for CLI (read-only)
    pub fn load_for_display(
        db: &Arc<dyn blvm_node::storage::database::Database>,
    ) -> Result<Vec<GovernanceProposal>, crate::error::GovernanceError> {
        let tree = db
            .open_tree(PROPOSALS_TREE)
            .map_err(|e| crate::error::GovernanceError::Storage(format!("open_tree: {e}")))?;
        match tree.get(STORAGE_KEY) {
            Ok(Some(data)) => {
                let map: HashMap<String, GovernanceProposal> = bincode::deserialize(&data)
                    .map_err(|e| {
                        crate::error::GovernanceError::Storage(format!("deserialize: {e}"))
                    })?;
                Ok(map.into_values().collect())
            }
            Ok(None) => Ok(Vec::new()),
            Err(e) => Err(crate::error::GovernanceError::Storage(format!("get: {e}"))),
        }
    }
}

fn json_string(value: &serde_json::Value, key: &str) -> String {
    value
        .get(key)
        .and_then(|item| item.as_str())
        .unwrap_or("")
        .to_string()
}

fn apply_created(
    proposals: &mut HashMap<String, GovernanceProposal>,
    proposal_id: String,
    repository: String,
    pr_number: u64,
    tier: String,
) -> Result<(), crate::error::GovernanceError> {
    if proposals.contains_key(&proposal_id) {
        return Err(crate::error::GovernanceError::Proposal(format!(
            "proposal {proposal_id} already exists"
        )));
    }
    proposals.insert(
        proposal_id.clone(),
        GovernanceProposal {
            proposal_id,
            repository,
            pr_number,
            tier,
            status: ProposalStatus::Created,
            votes: Vec::new(),
        },
    );
    Ok(())
}

fn apply_voted(
    proposals: &mut HashMap<String, GovernanceProposal>,
    proposal_id: &str,
    voter: String,
    vote: String,
) -> Result<(), crate::error::GovernanceError> {
    let Some(p) = proposals.get_mut(proposal_id) else {
        return Err(crate::error::GovernanceError::Proposal(format!(
            "unknown proposal {proposal_id}"
        )));
    };
    p.votes.push(ProposalVote { voter, vote });
    Ok(())
}

fn apply_merged(
    proposals: &mut HashMap<String, GovernanceProposal>,
    proposal_id: &str,
    repository: String,
    pr_number: u64,
) -> Result<(), crate::error::GovernanceError> {
    let Some(p) = proposals.get_mut(proposal_id) else {
        return Err(crate::error::GovernanceError::Proposal(format!(
            "unknown proposal {proposal_id}"
        )));
    };
    p.status = ProposalStatus::Merged;
    p.repository = repository;
    p.pr_number = pr_number;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn created(id: &str) -> GovernanceProposal {
        GovernanceProposal {
            proposal_id: id.to_string(),
            repository: "org/repo".to_string(),
            pr_number: 1,
            tier: "t".to_string(),
            status: ProposalStatus::Created,
            votes: Vec::new(),
        }
    }

    #[test]
    fn second_create_keeps_votes_and_status() {
        let mut proposals = HashMap::new();
        apply_created(
            &mut proposals,
            "p1".to_string(),
            "org/repo".to_string(),
            1,
            "t".to_string(),
        )
        .unwrap();
        apply_voted(
            &mut proposals,
            "p1",
            "alice".to_string(),
            "approve".to_string(),
        )
        .unwrap();
        proposals.get_mut("p1").unwrap().status = ProposalStatus::Merged;
        let err = apply_created(
            &mut proposals,
            "p1".to_string(),
            "org/repo".to_string(),
            1,
            "t".to_string(),
        )
        .unwrap_err();
        let stored = proposals.get("p1").unwrap();
        assert!(matches!(stored.status, ProposalStatus::Merged));
        assert_eq!(stored.votes.len(), 1);
        assert!(err.to_string().contains("already exists"));
    }

    #[test]
    fn vote_for_unknown_proposal_is_an_error() {
        let mut proposals = HashMap::new();
        proposals.insert("p1".to_string(), created("p1"));
        let err = apply_voted(
            &mut proposals,
            "missing",
            "alice".to_string(),
            "approve".to_string(),
        )
        .unwrap_err();
        assert!(err.to_string().contains("unknown proposal"));
        assert!(!proposals.contains_key("missing"));
        assert!(proposals.get("p1").unwrap().votes.is_empty());
    }
}
