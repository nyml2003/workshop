use super::entities::{TaskActivity, TaskMeta, TaskPaths, TaskRepoSelection};
use crate::errors::{DomainError, EntityKind};
use crate::shared::{RepoGroupId, RepoId, TaskSlug, Timestamp};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskWorkspace {
    pub meta: TaskMeta,
    pub repos: TaskRepoSelection,
    pub activity: TaskActivity,
    pub paths: TaskPaths,
}

impl TaskWorkspace {
    #[allow(clippy::too_many_arguments)]
    pub fn create(
        slug: TaskSlug,
        title: String,
        template: String,
        description: Option<String>,
        source_brief: Option<String>,
        tags: Vec<String>,
        selected_repo_groups: Vec<RepoGroupId>,
        repos: Vec<RepoId>,
        created_at: Timestamp,
    ) -> Result<Self, DomainError> {
        Ok(Self {
            meta: TaskMeta::new(slug, title, template, description, source_brief, tags)?,
            repos: TaskRepoSelection {
                selected_repo_groups,
                repos,
            },
            activity: TaskActivity {
                created_at,
                updated_at: created_at,
                last_opened_at: None,
                last_activity_at: Some(created_at),
                last_editor: None,
            },
            paths: TaskPaths {
                materials_dir: "materials".into(),
                repos_dir: "repos".into(),
                knowledge_candidates_dir: "knowledge-candidates".into(),
                task_skills_dir: ".codex/skills".into(),
            },
        })
    }

    pub fn mark_opened(&mut self, occurred_at: Timestamp, editor: String) {
        self.activity.updated_at = occurred_at;
        self.activity.last_opened_at = Some(occurred_at);
        self.activity.last_activity_at = Some(occurred_at);
        self.activity.last_editor = Some(editor);
    }

    pub fn close(&mut self, occurred_at: Timestamp) -> Result<(), DomainError> {
        match self.meta.status {
            super::value_objects::TaskStatus::Closed => {
                return Err(DomainError::Conflict {
                    entity: EntityKind::Task,
                    reason: "already closed".to_owned(),
                });
            }
            super::value_objects::TaskStatus::Archived => {
                return Err(DomainError::Conflict {
                    entity: EntityKind::Task,
                    reason: "archived tasks cannot be closed".to_owned(),
                });
            }
            _ => {}
        }
        self.meta.status = super::value_objects::TaskStatus::Closed;
        self.activity.updated_at = occurred_at;
        self.activity.last_activity_at = Some(occurred_at);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use time::OffsetDateTime;

    use super::*;
    use crate::shared::{RepoGroupId, RepoId, TaskSlug};
    use crate::task::value_objects::TaskStatus;

    fn sample_task() -> TaskWorkspace {
        TaskWorkspace::create(
            TaskSlug::from("test-slug"),
            "Test Title".to_owned(),
            "default".to_owned(),
            Some("Description".to_owned()),
            Some("Source brief".to_owned()),
            vec!["rust".to_owned()],
            vec![RepoGroupId::from("auth-core")],
            vec![RepoId::from("api-gateway")],
            OffsetDateTime::UNIX_EPOCH,
        )
        .unwrap()
    }

    #[test]
    fn create_sets_initial_activity() {
        let task = sample_task();
        assert_eq!(task.activity.created_at, OffsetDateTime::UNIX_EPOCH);
        assert_eq!(task.activity.updated_at, OffsetDateTime::UNIX_EPOCH);
        assert_eq!(
            task.activity.last_activity_at,
            Some(OffsetDateTime::UNIX_EPOCH)
        );
        assert_eq!(task.activity.last_opened_at, None);
        assert_eq!(task.activity.last_editor, None);
        assert_eq!(task.meta.status, TaskStatus::Active);
    }

    #[test]
    fn create_stores_repo_selection() {
        let task = sample_task();
        assert_eq!(task.repos.selected_repo_groups.len(), 1);
        assert_eq!(task.repos.selected_repo_groups[0].as_str(), "auth-core");
        assert_eq!(task.repos.repos.len(), 1);
        assert_eq!(task.repos.repos[0].as_str(), "api-gateway");
    }

    #[test]
    fn create_propagates_meta_validation_error() {
        let result = TaskWorkspace::create(
            TaskSlug::from(""),
            "Title".to_owned(),
            "default".to_owned(),
            None,
            None,
            vec![],
            vec![],
            vec![],
            OffsetDateTime::UNIX_EPOCH,
        );
        assert!(result.is_err());
    }

    #[test]
    fn mark_opened_updates_activity_fields() {
        let mut task = sample_task();
        let later = OffsetDateTime::UNIX_EPOCH + time::Duration::hours(2);

        task.mark_opened(later, "cursor".to_owned());

        assert_eq!(task.activity.updated_at, later);
        assert_eq!(task.activity.last_opened_at, Some(later));
        assert_eq!(task.activity.last_activity_at, Some(later));
        assert_eq!(task.activity.last_editor.as_deref(), Some("cursor"));
    }

    #[test]
    fn close_sets_status_to_closed() {
        let mut task = sample_task();
        let later = OffsetDateTime::UNIX_EPOCH + time::Duration::days(5);

        task.close(later).unwrap();

        assert_eq!(task.meta.status, TaskStatus::Closed);
        assert_eq!(task.activity.updated_at, later);
        assert_eq!(task.activity.last_activity_at, Some(later));
    }

    #[test]
    fn paths_are_set_to_default_values() {
        let task = sample_task();
        assert_eq!(task.paths.materials_dir.as_str(), "materials");
        assert_eq!(task.paths.repos_dir.as_str(), "repos");
        assert_eq!(
            task.paths.knowledge_candidates_dir.as_str(),
            "knowledge-candidates"
        );
        assert_eq!(task.paths.task_skills_dir.as_str(), ".codex/skills");
    }

    #[test]
    fn close_on_closed_task_returns_conflict() {
        let mut task = sample_task();
        task.meta.status = TaskStatus::Closed;
        let result = task.close(OffsetDateTime::UNIX_EPOCH);
        assert!(matches!(
            result,
            Err(DomainError::Conflict { entity: EntityKind::Task, reason }) if reason == "already closed"
        ));
    }

    #[test]
    fn close_on_archived_task_returns_conflict() {
        let mut task = sample_task();
        task.meta.status = TaskStatus::Archived;
        let result = task.close(OffsetDateTime::UNIX_EPOCH);
        assert!(matches!(
            result,
            Err(DomainError::Conflict { entity: EntityKind::Task, reason }) if reason == "archived tasks cannot be closed"
        ));
    }
}
