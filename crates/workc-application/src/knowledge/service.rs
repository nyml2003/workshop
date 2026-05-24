use crate::error::ApplicationError;
use workc_domain::errors::DomainError;
use workc_domain::knowledge::{KnowledgeEntry, KnowledgeRepository, KnowledgeSourceRef};
use workc_domain::shared::{KnowledgeCandidateId, KnowledgeId, TaskId};

use super::dtos::{
    CandidateMutationResult, CreateKnowledgeCandidateCommand, DeleteKnowledgeCandidateCommand,
    DeleteKnowledgeCommand, KnowledgeMutationResult, KnowledgeObjectSummary,
    ListKnowledgeCandidatesQuery, ListKnowledgeQuery, PromoteKnowledgeCommand,
    PromoteKnowledgeResult, ShowKnowledgeCandidateQuery, ShowKnowledgeQuery,
    UpdateKnowledgeCandidateMetaCommand, UpdateKnowledgeMetaCommand,
};

pub trait KnowledgeApplicationService {
    fn create_candidate(
        &self,
        command: CreateKnowledgeCandidateCommand,
    ) -> Result<CandidateMutationResult, ApplicationError>;
    fn list_candidates(
        &self,
        query: ListKnowledgeCandidatesQuery,
    ) -> Result<Vec<KnowledgeObjectSummary>, ApplicationError>;
    fn show_candidate(
        &self,
        query: ShowKnowledgeCandidateQuery,
    ) -> Result<Option<KnowledgeObjectSummary>, ApplicationError>;
    fn update_candidate_meta(
        &self,
        command: UpdateKnowledgeCandidateMetaCommand,
    ) -> Result<CandidateMutationResult, ApplicationError>;
    fn delete_candidate(
        &self,
        command: DeleteKnowledgeCandidateCommand,
    ) -> Result<(), ApplicationError>;
    fn list_knowledge(
        &self,
        query: ListKnowledgeQuery,
    ) -> Result<Vec<KnowledgeObjectSummary>, ApplicationError>;
    fn show_knowledge(
        &self,
        query: ShowKnowledgeQuery,
    ) -> Result<Option<KnowledgeObjectSummary>, ApplicationError>;
    fn update_knowledge_meta(
        &self,
        command: UpdateKnowledgeMetaCommand,
    ) -> Result<KnowledgeMutationResult, ApplicationError>;
    fn delete_knowledge(&self, command: DeleteKnowledgeCommand) -> Result<(), ApplicationError>;
    fn promote(
        &self,
        command: PromoteKnowledgeCommand,
    ) -> Result<PromoteKnowledgeResult, ApplicationError>;
}

pub struct DefaultKnowledgeApplicationService {
    repository: Box<dyn KnowledgeRepository>,
    clock: Box<dyn crate::ports::Clock>,
}

impl DefaultKnowledgeApplicationService {
    pub fn new(
        repository: Box<dyn KnowledgeRepository>,
        clock: Box<dyn crate::ports::Clock>,
    ) -> Self {
        Self { repository, clock }
    }

    fn to_summary_candidate(
        candidate: workc_domain::knowledge::KnowledgeCandidate,
    ) -> KnowledgeObjectSummary {
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
    fn create_candidate(
        &self,
        command: CreateKnowledgeCandidateCommand,
    ) -> Result<CandidateMutationResult, ApplicationError> {
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

    fn list_candidates(
        &self,
        query: ListKnowledgeCandidatesQuery,
    ) -> Result<Vec<KnowledgeObjectSummary>, ApplicationError> {
        Ok(self
            .repository
            .list_candidates(&TaskId::from(query.task_id.as_str()))?
            .into_iter()
            .map(Self::to_summary_candidate)
            .collect())
    }

    fn show_candidate(
        &self,
        query: ShowKnowledgeCandidateQuery,
    ) -> Result<Option<KnowledgeObjectSummary>, ApplicationError> {
        Ok(self
            .repository
            .find_candidate(
                &TaskId::from(query.task_id.as_str()),
                &KnowledgeCandidateId::from(query.candidate_id.as_str()),
            )?
            .map(Self::to_summary_candidate))
    }

    fn update_candidate_meta(
        &self,
        command: UpdateKnowledgeCandidateMetaCommand,
    ) -> Result<CandidateMutationResult, ApplicationError> {
        let task_id = TaskId::from(command.task_id.as_str());
        let candidate_id = KnowledgeCandidateId::from(command.candidate_id.as_str());
        let mut candidate = self
            .repository
            .find_candidate(&task_id, &candidate_id)?
            .ok_or_else(|| {
                ApplicationError::Domain(DomainError::NotFound {
                    entity: "knowledge-candidate",
                    id: candidate_id.to_string(),
                })
            })?;

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

    fn delete_candidate(
        &self,
        command: DeleteKnowledgeCandidateCommand,
    ) -> Result<(), ApplicationError> {
        self.repository.delete_candidate(
            &TaskId::from(command.task_id.as_str()),
            &KnowledgeCandidateId::from(command.candidate_id.as_str()),
        )?;
        Ok(())
    }

    fn list_knowledge(
        &self,
        _query: ListKnowledgeQuery,
    ) -> Result<Vec<KnowledgeObjectSummary>, ApplicationError> {
        Ok(self
            .repository
            .load()?
            .entries
            .into_iter()
            .map(Self::to_summary_entry)
            .collect())
    }

    fn show_knowledge(
        &self,
        query: ShowKnowledgeQuery,
    ) -> Result<Option<KnowledgeObjectSummary>, ApplicationError> {
        Ok(self
            .repository
            .find_entry(&KnowledgeId::from(query.knowledge_id.as_str()))?
            .map(Self::to_summary_entry))
    }

    fn update_knowledge_meta(
        &self,
        command: UpdateKnowledgeMetaCommand,
    ) -> Result<KnowledgeMutationResult, ApplicationError> {
        let knowledge_id = KnowledgeId::from(command.knowledge_id.as_str());
        let mut entry = self.repository.find_entry(&knowledge_id)?.ok_or_else(|| {
            ApplicationError::Domain(DomainError::NotFound {
                entity: "knowledge",
                id: knowledge_id.to_string(),
            })
        })?;

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
        self.repository
            .delete_entry(&KnowledgeId::from(command.knowledge_id.as_str()))?;
        Ok(())
    }

    fn promote(
        &self,
        command: PromoteKnowledgeCommand,
    ) -> Result<PromoteKnowledgeResult, ApplicationError> {
        let task_id = TaskId::from(command.task_id.as_str());
        let candidate_id = KnowledgeCandidateId::from(command.candidate_id.as_str());
        let knowledge_id = KnowledgeId::from(command.knowledge_id.as_str());
        self.repository
            .promote_candidate(&task_id, &candidate_id, &knowledge_id)?;
        let mut entry = self.repository.find_entry(&knowledge_id)?.ok_or_else(|| {
            ApplicationError::Domain(DomainError::NotFound {
                entity: "knowledge",
                id: knowledge_id.to_string(),
            })
        })?;
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

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::collections::BTreeMap;

    use camino::Utf8PathBuf;
    use time::OffsetDateTime;
    use workc_domain::errors::DomainError;
    use workc_domain::knowledge::{
        KnowledgeBase, KnowledgeCandidate, KnowledgeEntry, KnowledgeRepository,
    };
    use workc_domain::shared::{KnowledgeCandidateId, KnowledgeId, TaskId};

    use crate::ports::Clock;

    use super::*;

    struct InMemoryKnowledgeRepository {
        base: RefCell<KnowledgeBase>,
        candidates: RefCell<BTreeMap<(String, String), KnowledgeCandidate>>,
    }

    impl InMemoryKnowledgeRepository {
        fn new() -> Self {
            Self {
                base: RefCell::new(KnowledgeBase::default()),
                candidates: RefCell::new(BTreeMap::new()),
            }
        }
    }

    impl KnowledgeRepository for InMemoryKnowledgeRepository {
        fn load(&self) -> Result<KnowledgeBase, DomainError> {
            Ok(self.base.borrow().clone())
        }

        fn save(&self, knowledge_base: &KnowledgeBase) -> Result<(), DomainError> {
            *self.base.borrow_mut() = knowledge_base.clone();
            Ok(())
        }

        fn list_candidates(
            &self,
            task_id: &TaskId,
        ) -> Result<Vec<KnowledgeCandidate>, DomainError> {
            let prefix = task_id.to_string();
            Ok(self
                .candidates
                .borrow()
                .iter()
                .filter(|((tid, _), _)| tid == &prefix)
                .map(|(_, c)| c.clone())
                .collect())
        }

        fn create_candidate(&self, candidate: &KnowledgeCandidate) -> Result<(), DomainError> {
            self.candidates.borrow_mut().insert(
                (candidate.task_id.to_string(), candidate.id.to_string()),
                candidate.clone(),
            );
            Ok(())
        }

        fn update_candidate(
            &self,
            _task_id: &TaskId,
            candidate: &KnowledgeCandidate,
        ) -> Result<(), DomainError> {
            self.candidates.borrow_mut().insert(
                (candidate.task_id.to_string(), candidate.id.to_string()),
                candidate.clone(),
            );
            Ok(())
        }

        fn delete_candidate(
            &self,
            task_id: &TaskId,
            candidate_id: &KnowledgeCandidateId,
        ) -> Result<(), DomainError> {
            self.candidates
                .borrow_mut()
                .remove(&(task_id.to_string(), candidate_id.to_string()));
            Ok(())
        }

        fn find_candidate(
            &self,
            task_id: &TaskId,
            candidate_id: &KnowledgeCandidateId,
        ) -> Result<Option<KnowledgeCandidate>, DomainError> {
            Ok(self
                .candidates
                .borrow()
                .get(&(task_id.to_string(), candidate_id.to_string()))
                .cloned())
        }

        fn create_entry(&self, entry: &KnowledgeEntry) -> Result<(), DomainError> {
            self.base.borrow_mut().entries.push(entry.clone());
            Ok(())
        }

        fn update_entry(&self, entry: &KnowledgeEntry) -> Result<(), DomainError> {
            let mut base = self.base.borrow_mut();
            if let Some(existing) = base.entries.iter_mut().find(|e| e.id == entry.id) {
                *existing = entry.clone();
            }
            Ok(())
        }

        fn delete_entry(&self, id: &KnowledgeId) -> Result<(), DomainError> {
            self.base.borrow_mut().entries.retain(|e| e.id != *id);
            Ok(())
        }

        fn promote_candidate(
            &self,
            task_id: &TaskId,
            candidate_id: &KnowledgeCandidateId,
            knowledge_id: &KnowledgeId,
        ) -> Result<(), DomainError> {
            let candidate = self.find_candidate(task_id, candidate_id)?.ok_or_else(|| {
                DomainError::NotFound {
                    entity: "knowledge-candidate",
                    id: candidate_id.to_string(),
                }
            })?;
            let entry = KnowledgeEntry {
                id: knowledge_id.clone(),
                title: candidate.title,
                path: Utf8PathBuf::from("knowledge").join(knowledge_id.as_str()),
                category: candidate.category,
                tags: candidate.tags,
                sources: candidate.sources,
                created_at: Some(OffsetDateTime::UNIX_EPOCH),
                updated_at: Some(OffsetDateTime::UNIX_EPOCH),
            };
            self.create_entry(&entry)?;
            Ok(())
        }

        fn find_entry(&self, id: &KnowledgeId) -> Result<Option<KnowledgeEntry>, DomainError> {
            Ok(self
                .base
                .borrow()
                .entries
                .iter()
                .find(|e| e.id == *id)
                .cloned())
        }
    }

    struct FixedClock;

    impl Clock for FixedClock {
        fn now(&self) -> OffsetDateTime {
            OffsetDateTime::UNIX_EPOCH
        }
    }

    fn service() -> DefaultKnowledgeApplicationService {
        DefaultKnowledgeApplicationService::new(
            Box::new(InMemoryKnowledgeRepository::new()),
            Box::new(FixedClock),
        )
    }

    #[test]
    fn create_candidate_returns_summary() {
        let svc = service();
        let result = svc
            .create_candidate(CreateKnowledgeCandidateCommand {
                task_id: "task-20260524-auth".to_owned(),
                candidate_id: "cand-01".to_owned(),
                title: "Auth patterns".to_owned(),
                category: Some("security".to_owned()),
                tags: vec!["auth".to_owned()],
                source_paths: vec!["materials/notes.md".to_owned()],
            })
            .unwrap();

        assert_eq!(result.task_id, "task-20260524-auth");
        assert_eq!(result.candidate.title, "Auth patterns");
        assert_eq!(result.candidate.source_count, 1);
        assert_eq!(result.candidate.category.as_deref(), Some("security"));
    }

    #[test]
    fn list_candidates_returns_created_candidates() {
        let svc = service();
        svc.create_candidate(CreateKnowledgeCandidateCommand {
            task_id: "task-20260524-auth".to_owned(),
            candidate_id: "cand-01".to_owned(),
            title: "Auth patterns".to_owned(),
            category: None,
            tags: vec![],
            source_paths: vec![],
        })
        .unwrap();

        let items = svc
            .list_candidates(ListKnowledgeCandidatesQuery {
                task_id: "task-20260524-auth".to_owned(),
            })
            .unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].id, "cand-01");
    }

    #[test]
    fn list_candidates_empty_for_unknown_task() {
        let svc = service();
        let items = svc
            .list_candidates(ListKnowledgeCandidatesQuery {
                task_id: "task-unknown".to_owned(),
            })
            .unwrap();
        assert!(items.is_empty());
    }

    #[test]
    fn show_candidate_returns_none_for_missing() {
        let svc = service();
        let item = svc
            .show_candidate(ShowKnowledgeCandidateQuery {
                task_id: "task-x".to_owned(),
                candidate_id: "cand-y".to_owned(),
            })
            .unwrap();
        assert!(item.is_none());
    }

    #[test]
    fn show_candidate_returns_summary() {
        let svc = service();
        svc.create_candidate(CreateKnowledgeCandidateCommand {
            task_id: "task-20260524-auth".to_owned(),
            candidate_id: "cand-01".to_owned(),
            title: "Auth patterns".to_owned(),
            category: None,
            tags: vec![],
            source_paths: vec![],
        })
        .unwrap();

        let item = svc
            .show_candidate(ShowKnowledgeCandidateQuery {
                task_id: "task-20260524-auth".to_owned(),
                candidate_id: "cand-01".to_owned(),
            })
            .unwrap()
            .unwrap();
        assert_eq!(item.title, "Auth patterns");
    }

    #[test]
    fn update_candidate_meta_changes_fields() {
        let svc = service();
        svc.create_candidate(CreateKnowledgeCandidateCommand {
            task_id: "task-20260524-auth".to_owned(),
            candidate_id: "cand-01".to_owned(),
            title: "Old title".to_owned(),
            category: Some("old".to_owned()),
            tags: vec!["old".to_owned()],
            source_paths: vec![],
        })
        .unwrap();

        let updated = svc
            .update_candidate_meta(UpdateKnowledgeCandidateMetaCommand {
                task_id: "task-20260524-auth".to_owned(),
                candidate_id: "cand-01".to_owned(),
                title: Some("New title".to_owned()),
                category: Some("new".to_owned()),
                tags: Some(vec!["new".to_owned()]),
            })
            .unwrap();

        assert_eq!(updated.candidate.title, "New title");
        assert_eq!(updated.candidate.category.as_deref(), Some("new"));
        assert_eq!(updated.candidate.tags, vec!["new"]);
    }

    #[test]
    fn update_candidate_meta_partial_updates_do_not_clear_other_fields() {
        let svc = service();
        svc.create_candidate(CreateKnowledgeCandidateCommand {
            task_id: "task-20260524-auth".to_owned(),
            candidate_id: "cand-01".to_owned(),
            title: "Old title".to_owned(),
            category: Some("old".to_owned()),
            tags: vec!["old".to_owned()],
            source_paths: vec![],
        })
        .unwrap();

        let updated = svc
            .update_candidate_meta(UpdateKnowledgeCandidateMetaCommand {
                task_id: "task-20260524-auth".to_owned(),
                candidate_id: "cand-01".to_owned(),
                title: Some("New title".to_owned()),
                category: None,
                tags: None,
            })
            .unwrap();

        assert_eq!(updated.candidate.title, "New title");
        assert_eq!(updated.candidate.category.as_deref(), Some("old"));
        assert_eq!(updated.candidate.tags, vec!["old"]);
    }

    #[test]
    fn update_candidate_meta_fails_for_missing_candidate() {
        let svc = service();
        let result = svc.update_candidate_meta(UpdateKnowledgeCandidateMetaCommand {
            task_id: "task-x".to_owned(),
            candidate_id: "cand-y".to_owned(),
            title: Some("x".to_owned()),
            category: None,
            tags: None,
        });
        assert!(result.is_err());
    }

    #[test]
    fn delete_candidate_removes_it() {
        let svc = service();
        svc.create_candidate(CreateKnowledgeCandidateCommand {
            task_id: "task-20260524-auth".to_owned(),
            candidate_id: "cand-01".to_owned(),
            title: "Auth patterns".to_owned(),
            category: None,
            tags: vec![],
            source_paths: vec![],
        })
        .unwrap();

        svc.delete_candidate(DeleteKnowledgeCandidateCommand {
            task_id: "task-20260524-auth".to_owned(),
            candidate_id: "cand-01".to_owned(),
        })
        .unwrap();

        let items = svc
            .list_candidates(ListKnowledgeCandidatesQuery {
                task_id: "task-20260524-auth".to_owned(),
            })
            .unwrap();
        assert!(items.is_empty());
    }

    #[test]
    fn list_knowledge_is_initially_empty() {
        let svc = service();
        let items = svc.list_knowledge(ListKnowledgeQuery).unwrap();
        assert!(items.is_empty());
    }

    #[test]
    fn show_knowledge_returns_none_for_missing() {
        let svc = service();
        let item = svc
            .show_knowledge(ShowKnowledgeQuery {
                knowledge_id: "k-unknown".to_owned(),
            })
            .unwrap();
        assert!(item.is_none());
    }

    #[test]
    fn update_knowledge_meta_fails_for_missing() {
        let svc = service();
        let result = svc.update_knowledge_meta(UpdateKnowledgeMetaCommand {
            knowledge_id: "k-missing".to_owned(),
            title: Some("x".to_owned()),
            category: None,
            tags: None,
        });
        assert!(result.is_err());
    }

    #[test]
    fn delete_knowledge_does_not_panic_for_missing() {
        let svc = service();
        svc.delete_knowledge(DeleteKnowledgeCommand {
            knowledge_id: "k-missing".to_owned(),
        })
        .unwrap();
    }

    #[test]
    fn promote_creates_entry_and_applies_overrides() {
        let svc = service();
        svc.create_candidate(CreateKnowledgeCandidateCommand {
            task_id: "task-20260524-auth".to_owned(),
            candidate_id: "cand-01".to_owned(),
            title: "Auth patterns".to_owned(),
            category: Some("security".to_owned()),
            tags: vec!["auth".to_owned()],
            source_paths: vec!["notes.md".to_owned()],
        })
        .unwrap();

        let result = svc
            .promote(PromoteKnowledgeCommand {
                task_id: "task-20260524-auth".to_owned(),
                candidate_id: "cand-01".to_owned(),
                knowledge_id: "k-001".to_owned(),
                title: Some("Promoted Auth".to_owned()),
                category: None,
                tags: None,
            })
            .unwrap();

        assert_eq!(result.task_id, "task-20260524-auth");
        assert_eq!(result.candidate_id, "cand-01");
        assert_eq!(result.knowledge.title, "Promoted Auth");
        assert_eq!(result.knowledge.source_count, 1);

        let entries = svc.list_knowledge(ListKnowledgeQuery).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, "k-001");
    }
}
