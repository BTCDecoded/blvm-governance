//! Governance proposal storage
//!
//! Stores proposals when receiving GovernanceProposal* events so list-proposals can display them.

use blvm_node::module::ipc::protocol::{EventPayload, ModuleMessage};
use blvm_node::module::EventType;
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
        match event_msg.event_type {
            EventType::GovernanceProposalCreated => {
                if let EventPayload::GovernanceProposalCreated {
                    proposal_id,
                    repository,
                    pr_number,
                    tier,
                } = &event_msg.payload
                {
                    proposals.insert(
                        proposal_id.clone(),
                        GovernanceProposal {
                            proposal_id: proposal_id.clone(),
                            repository: repository.clone(),
                            pr_number: *pr_number,
                            tier: tier.clone(),
                            status: ProposalStatus::Created,
                            votes: Vec::new(),
                        },
                    );
                    self.save(&proposals)?;
                }
            }
            EventType::GovernanceProposalVoted => {
                if let EventPayload::GovernanceProposalVoted {
                    proposal_id,
                    voter,
                    vote,
                } = &event_msg.payload
                {
                    if let Some(p) = proposals.get_mut(proposal_id) {
                        p.votes.push(ProposalVote {
                            voter: voter.clone(),
                            vote: vote.clone(),
                        });
                        self.save(&proposals)?;
                    }
                }
            }
            EventType::GovernanceProposalMerged => {
                if let EventPayload::GovernanceProposalMerged {
                    proposal_id,
                    repository,
                    pr_number,
                } = &event_msg.payload
                {
                    if let Some(p) = proposals.get_mut(proposal_id) {
                        p.status = ProposalStatus::Merged;
                        p.repository = repository.clone();
                        p.pr_number = *pr_number;
                        self.save(&proposals)?;
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn load(&self) -> Result<HashMap<String, GovernanceProposal>, crate::error::GovernanceError> {
        let tree = self.db.open_tree(PROPOSALS_TREE).map_err(|e| {
            crate::error::GovernanceError::Storage(format!("open_tree: {}", e))
        })?;
        match tree.get(STORAGE_KEY) {
            Ok(Some(data)) => bincode::deserialize(&data).map_err(|e| {
                crate::error::GovernanceError::Storage(format!("deserialize: {}", e))
            }),
            Ok(None) => Ok(HashMap::new()),
            Err(e) => Err(crate::error::GovernanceError::Storage(format!("get: {}", e))),
        }
    }

    fn save(
        &self,
        proposals: &HashMap<String, GovernanceProposal>,
    ) -> Result<(), crate::error::GovernanceError> {
        let tree = self.db.open_tree(PROPOSALS_TREE).map_err(|e| {
            crate::error::GovernanceError::Storage(format!("open_tree: {}", e))
        })?;
        let data = bincode::serialize(proposals).map_err(|e| {
            crate::error::GovernanceError::Storage(format!("serialize: {}", e))
        })?;
        tree.insert(STORAGE_KEY, &data).map_err(|e| {
            crate::error::GovernanceError::Storage(format!("insert: {}", e))
        })?;
        Ok(())
    }

    /// Load proposals for CLI (read-only)
    pub fn load_for_display(db: &Arc<dyn blvm_node::storage::database::Database>) -> Result<Vec<GovernanceProposal>, crate::error::GovernanceError> {
        let tree = db.open_tree(PROPOSALS_TREE).map_err(|e| {
            crate::error::GovernanceError::Storage(format!("open_tree: {}", e))
        })?;
        match tree.get(STORAGE_KEY) {
            Ok(Some(data)) => {
                let map: HashMap<String, GovernanceProposal> = bincode::deserialize(&data)
                    .map_err(|e| crate::error::GovernanceError::Storage(format!("deserialize: {}", e)))?;
                Ok(map.into_values().collect())
            }
            Ok(None) => Ok(Vec::new()),
            Err(e) => Err(crate::error::GovernanceError::Storage(format!("get: {}", e))),
        }
    }
}
