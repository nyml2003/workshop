use crate::errors::DomainError;
use crate::shared::{KnowledgeCandidateId, KnowledgeId, TaskId};

use super::aggregate::KnowledgeBase;
use super::entities::{KnowledgeCandidate, KnowledgeEntry};

pub trait KnowledgeRepository {
    fn load(&self) -> Result<KnowledgeBase, DomainError>;
    fn save(&self, knowledge_base: &KnowledgeBase) -> Result<(), DomainError>;
    fn list_candidates(&self, task_id: &TaskId) -> Result<Vec<KnowledgeCandidate>, DomainError>;
    fn create_candidate(&self, candidate: &KnowledgeCandidate) -> Result<(), DomainError>;
    fn update_candidate(&self, task_id: &TaskId, candidate: &KnowledgeCandidate) -> Result<(), DomainError>;
    fn delete_candidate(&self, task_id: &TaskId, candidate_id: &KnowledgeCandidateId) -> Result<(), DomainError>;
    fn find_candidate(&self, task_id: &TaskId, candidate_id: &KnowledgeCandidateId) -> Result<Option<KnowledgeCandidate>, DomainError>;
    fn create_entry(&self, entry: &KnowledgeEntry) -> Result<(), DomainError>;
    fn update_entry(&self, entry: &KnowledgeEntry) -> Result<(), DomainError>;
    fn delete_entry(&self, id: &KnowledgeId) -> Result<(), DomainError>;
    fn promote_candidate(&self, task_id: &TaskId, candidate_id: &KnowledgeCandidateId, knowledge_id: &KnowledgeId) -> Result<(), DomainError>;
    fn find_entry(&self, id: &KnowledgeId) -> Result<Option<KnowledgeEntry>, DomainError>;
}
