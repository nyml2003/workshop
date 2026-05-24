use crate::error::ApplicationError;
use workc_domain::errors::DomainError;
use workc_domain::knowledge::{KnowledgeEntry, KnowledgeRepository, KnowledgeSourceRef};
use workc_domain::shared::{KnowledgeCandidateId, KnowledgeId, TaskId};

use super::dtos::{
    CandidateMutationResult, CreateKnowledgeCandidateCommand, DeleteKnowledgeCandidateCommand, DeleteKnowledgeCommand,
    KnowledgeMutationResult, KnowledgeObjectSummary, ListKnowledgeCandidatesQuery, ListKnowledgeQuery, PromoteKnowledgeCommand,
    PromoteKnowledgeResult, ShowKnowledgeCandidateQuery, ShowKnowledgeQuery, UpdateKnowledgeCandidateMetaCommand, UpdateKnowledgeMetaCommand,
};

pub trait KnowledgeApplicationService {
    fn create_candidate(&self, command: CreateKnowledgeCandidateCommand) -> Result<CandidateMutationResult, ApplicationError>;
    fn list_candidates(&self, query: ListKnowledgeCandidatesQuery) -> Result<Vec<KnowledgeObjectSummary>, ApplicationError>;
    fn show_candidate(&self, query: ShowKnowledgeCandidateQuery) -> Result<Option<KnowledgeObjectSummary>, ApplicationError>;
    fn update_candidate_meta(&self, command: UpdateKnowledgeCandidateMetaCommand) -> Result<CandidateMutationResult, ApplicationError>;
    fn delete_candidate(&self, command: DeleteKnowledgeCandidateCommand) -> Result<(), ApplicationError>;
    fn list_knowledge(&self, query: ListKnowledgeQuery) -> Result<Vec<KnowledgeObjectSummary>, ApplicationError>;
    fn show_knowledge(&self, query: ShowKnowledgeQuery) -> Result<Option<KnowledgeObjectSummary>, ApplicationError>;
    fn update_knowledge_meta(&self, command: UpdateKnowledgeMetaCommand) -> Result<KnowledgeMutationResult, ApplicationError>;
    fn delete_knowledge(&self, command: DeleteKnowledgeCommand) -> Result<(), ApplicationError>;
    fn promote(&self, command: PromoteKnowledgeCommand) -> Result<PromoteKnowledgeResult, ApplicationError>;
}

pub struct DefaultKnowledgeApplicationService {
    repository: Box<dyn KnowledgeRepository>,
    clock: Box<dyn crate::ports::Clock>,
}

impl DefaultKnowledgeApplicationService {
    pub fn new(repository: Box<dyn KnowledgeRepository>, clock: Box<dyn crate::ports::Clock>) -> Self {
        Self { repository, clock }
    }

    fn to_summary_candidate(candidate: workc_domain::knowledge::KnowledgeCandidate) -> KnowledgeObjectSummary {
        KnowledgeObjectSummary {
            id: candidate.id.to_string(),
            title: candidate.title,
            path: candidate.path,
            category: candidate.category,
            tags: candidate.tags,
            source_count: candidate.sources.len(),
        }
    }

    fn to_summary_entry(entry: KnowledgeEntry) -> KnowledgeObjectSummary {
        KnowledgeObjectSummary {
            id: entry.id.to_string(),
            title: entry.title,
            path: entry.path,
            category: entry.category,
            tags: entry.tags,
            source_count: entry.sources.len(),
        }
    }
}

impl KnowledgeApplicationService for DefaultKnowledgeApplicationService {
    fn create_candidate(&self, command: CreateKnowledgeCandidateCommand) -> Result<CandidateMutationResult, ApplicationError> {
        let now = self.clock.now();
        let candidate = workc_domain::knowledge::KnowledgeCandidate {
            id: KnowledgeCandidateId::from(command.candidate_id.as_str()),
            task_id: TaskId::from(command.task_id.as_str()),
            title: command.title,
            path: camino::Utf8PathBuf::from("tasks")
                .join(command.task_id.as_str())
                .join("knowledge-candidates")
                .join(command.candidate_id.as_str()),
            category: command.category,
            tags: command.tags,
            sources: command
                .source_paths
                .into_iter()
                .map(|source_path| KnowledgeSourceRef {
                    source_path: source_path.into(),
                    section: None,
                    excerpt: None,
                })
                .collect(),
            created_at: Some(now),
            updated_at: Some(now),
        };
        self.repository.create_candidate(&candidate)?;
        Ok(CandidateMutationResult {
            task_id: candidate.task_id.to_string(),
            candidate: Self::to_summary_candidate(candidate),
        })
    }

    fn list_candidates(&self, query: ListKnowledgeCandidatesQuery) -> Result<Vec<KnowledgeObjectSummary>, ApplicationError> {
        Ok(self
            .repository
            .list_candidates(&TaskId::from(query.task_id.as_str()))?
            .into_iter()
            .map(Self::to_summary_candidate)
            .collect())
    }

    fn show_candidate(&self, query: ShowKnowledgeCandidateQuery) -> Result<Option<KnowledgeObjectSummary>, ApplicationError> {
        Ok(self
            .repository
            .find_candidate(
                &TaskId::from(query.task_id.as_str()),
                &KnowledgeCandidateId::from(query.candidate_id.as_str()),
            )?
            .map(Self::to_summary_candidate))
    }

    fn update_candidate_meta(&self, command: UpdateKnowledgeCandidateMetaCommand) -> Result<CandidateMutationResult, ApplicationError> {
        let task_id = TaskId::from(command.task_id.as_str());
        let candidate_id = KnowledgeCandidateId::from(command.candidate_id.as_str());
        let mut candidate = self
            .repository
            .find_candidate(&task_id, &candidate_id)?
            .ok_or_else(|| ApplicationError::Domain(DomainError::NotFound {
                entity: "knowledge-candidate",
                id: candidate_id.to_string(),
            }))?;

        if let Some(title) = command.title {
            candidate.title = title;
        }
        if let Some(category) = command.category {
            candidate.category = Some(category);
        }
        if let Some(tags) = command.tags {
            candidate.tags = tags;
        }
        candidate.updated_at = Some(self.clock.now());
        self.repository.update_candidate(&task_id, &candidate)?;
        Ok(CandidateMutationResult {
            task_id: task_id.to_string(),
            candidate: Self::to_summary_candidate(candidate),
        })
    }

    fn delete_candidate(&self, command: DeleteKnowledgeCandidateCommand) -> Result<(), ApplicationError> {
        self.repository.delete_candidate(
            &TaskId::from(command.task_id.as_str()),
            &KnowledgeCandidateId::from(command.candidate_id.as_str()),
        )?;
        Ok(())
    }

    fn list_knowledge(&self, _query: ListKnowledgeQuery) -> Result<Vec<KnowledgeObjectSummary>, ApplicationError> {
        Ok(self.repository.load()?.entries.into_iter().map(Self::to_summary_entry).collect())
    }

    fn show_knowledge(&self, query: ShowKnowledgeQuery) -> Result<Option<KnowledgeObjectSummary>, ApplicationError> {
        Ok(self
            .repository
            .find_entry(&KnowledgeId::from(query.knowledge_id.as_str()))?
            .map(Self::to_summary_entry))
    }

    fn update_knowledge_meta(&self, command: UpdateKnowledgeMetaCommand) -> Result<KnowledgeMutationResult, ApplicationError> {
        let knowledge_id = KnowledgeId::from(command.knowledge_id.as_str());
        let mut entry = self
            .repository
            .find_entry(&knowledge_id)?
            .ok_or_else(|| ApplicationError::Domain(DomainError::NotFound {
                entity: "knowledge",
                id: knowledge_id.to_string(),
            }))?;

        if let Some(title) = command.title {
            entry.title = title;
        }
        if let Some(category) = command.category {
            entry.category = Some(category);
        }
        if let Some(tags) = command.tags {
            entry.tags = tags;
        }
        entry.updated_at = Some(self.clock.now());
        self.repository.update_entry(&entry)?;
        Ok(KnowledgeMutationResult {
            knowledge: Self::to_summary_entry(entry),
        })
    }

    fn delete_knowledge(&self, command: DeleteKnowledgeCommand) -> Result<(), ApplicationError> {
        self.repository.delete_entry(&KnowledgeId::from(command.knowledge_id.as_str()))?;
        Ok(())
    }

    fn promote(&self, command: PromoteKnowledgeCommand) -> Result<PromoteKnowledgeResult, ApplicationError> {
        let task_id = TaskId::from(command.task_id.as_str());
        let candidate_id = KnowledgeCandidateId::from(command.candidate_id.as_str());
        let knowledge_id = KnowledgeId::from(command.knowledge_id.as_str());
        self.repository.promote_candidate(&task_id, &candidate_id, &knowledge_id)?;
        let mut entry = self
            .repository
            .find_entry(&knowledge_id)?
            .ok_or_else(|| ApplicationError::Domain(DomainError::NotFound {
                entity: "knowledge",
                id: knowledge_id.to_string(),
            }))?;
        if let Some(title) = command.title {
            entry.title = title;
        }
        if let Some(category) = command.category {
            entry.category = Some(category);
        }
        if let Some(tags) = command.tags {
            entry.tags = tags;
        }
        entry.updated_at = Some(self.clock.now());
        self.repository.update_entry(&entry)?;
        Ok(PromoteKnowledgeResult {
            task_id: task_id.to_string(),
            candidate_id: candidate_id.to_string(),
            knowledge: Self::to_summary_entry(entry),
        })
    }
}
