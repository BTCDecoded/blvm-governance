//! Governance proposal storage.
//!
//! The module API writes proposals. A `ModuleEvent` on the bus is not a vote,
//! a create, or a merge: any loaded module can publish those names.

use blvm_node::module::ipc::protocol::ModuleMessage;
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

    pub fn create_proposal(
        &self,
        proposal_id: String,
        repository: String,
        pr_number: u64,
        tier: String,
    ) -> Result<(), crate::error::GovernanceError> {
        let mut proposals = self.load()?;
        apply_created(&mut proposals, proposal_id, repository, pr_number, tier)?;
        self.save(&proposals)
    }

    pub fn record_vote(
        &self,
        proposal_id: &str,
        voter: &str,
        vote: &str,
    ) -> Result<(), crate::error::GovernanceError> {
        let mut proposals = self.load()?;
        apply_voted(
            &mut proposals,
            proposal_id,
            voter.to_string(),
            vote.to_string(),
        )?;
        self.save(&proposals)
    }

    pub fn record_merged(
        &self,
        proposal_id: &str,
        repository: String,
        pr_number: u64,
    ) -> Result<(), crate::error::GovernanceError> {
        let mut proposals = self.load()?;
        apply_merged(&mut proposals, proposal_id, repository, pr_number)?;
        self.save(&proposals)
    }

    /// The bus is not a write path. Kept so event delivery still has a handler.
    pub fn handle_event(
        &self,
        _event: &ModuleMessage,
    ) -> Result<(), crate::error::GovernanceError> {
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

/// The voter is the calling module. A body may repeat that id. It may not name another one.
pub(crate) fn voter_from_caller<'a>(
    caller_module_id: &'a str,
    claimed: Option<&str>,
) -> Result<&'a str, crate::error::GovernanceError> {
    if caller_module_id.is_empty() {
        return Err(crate::error::GovernanceError::Proposal(
            "a vote requires the calling module".to_string(),
        ));
    }
    if let Some(name) = claimed
        && name != caller_module_id
    {
        return Err(crate::error::GovernanceError::Proposal(
            "vote is recorded for the calling module".to_string(),
        ));
    }
    Ok(caller_module_id)
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
    if p.votes.iter().any(|existing| existing.voter == voter) {
        return Err(crate::error::GovernanceError::Proposal(format!(
            "proposal {proposal_id} already has a vote from {voter}"
        )));
    }
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

    #[test]
    fn a_second_vote_from_the_same_voter_keeps_the_first() {
        let mut proposals = HashMap::new();
        proposals.insert("p1".to_string(), created("p1"));
        apply_voted(
            &mut proposals,
            "p1",
            "blvm-mesh".to_string(),
            "approve".to_string(),
        )
        .unwrap();
        let err = apply_voted(
            &mut proposals,
            "p1",
            "blvm-mesh".to_string(),
            "reject".to_string(),
        )
        .unwrap_err();
        let votes = &proposals.get("p1").unwrap().votes;
        assert_eq!(votes.len(), 1);
        assert_eq!(votes[0].vote, "approve");
        assert!(err.to_string().contains("already has a vote"));
    }

    #[test]
    fn the_voter_is_the_calling_module() {
        assert_eq!(voter_from_caller("blvm-mesh", None).unwrap(), "blvm-mesh");
        assert_eq!(
            voter_from_caller("blvm-mesh", Some("blvm-mesh")).unwrap(),
            "blvm-mesh"
        );
        assert!(voter_from_caller("blvm-mesh", Some("alice")).is_err());
        assert!(voter_from_caller("", Some("alice")).is_err());
    }

    #[test]
    fn a_module_event_does_not_change_the_store() {
        let dir = std::env::temp_dir().join(format!(
            "blvm-governance-proposals-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let db = blvm_sdk::module::ModuleDb::open(&dir).unwrap();
        let store = ProposalStore::new(db.as_db());
        let created = module_event(
            "blvm-governance",
            "GovernanceProposalCreated",
            br#"{"proposal_id":"p1","repository":"org/repo","pr_number":1,"tier":"t"}"#,
        );
        store.handle_event(&created).unwrap();
        assert!(store.load_proposals().unwrap().is_empty());

        store
            .create_proposal("p1".to_string(), "org/repo".to_string(), 1, "t".to_string())
            .unwrap();
        let voted = module_event(
            "blvm-other",
            "GovernanceProposalVoted",
            br#"{"proposal_id":"p1","voter":"alice","vote":"approve"}"#,
        );
        store.handle_event(&voted).unwrap();
        let merged = module_event(
            "blvm-governance",
            "GovernanceProposalMerged",
            br#"{"proposal_id":"p1","repository":"evil/repo","pr_number":9}"#,
        );
        store.handle_event(&merged).unwrap();

        let stored = store.load_proposals().unwrap();
        assert_eq!(stored.len(), 1);
        assert!(stored[0].votes.is_empty());
        assert!(matches!(stored[0].status, ProposalStatus::Created));
        assert_eq!(stored[0].repository, "org/repo");
        let _ = std::fs::remove_dir_all(&dir);
    }

    fn module_event(source: &str, name: &str, data: &[u8]) -> ModuleMessage {
        use blvm_node::module::ipc::protocol::{EventMessage, EventPayload};
        use blvm_node::module::traits::EventType;
        ModuleMessage::Event(EventMessage {
            event_type: EventType::ModuleEvent,
            payload: EventPayload::ModuleEvent {
                source_module: source.to_string(),
                name: name.to_string(),
                data: data.to_vec(),
            },
        })
    }
}
