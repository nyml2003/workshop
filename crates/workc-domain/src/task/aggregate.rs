use super::entities::{TaskActivity, TaskMeta, TaskPaths, TaskRepoSelection};
use crate::errors::DomainError;
use crate::shared::{RepoGroupId, RepoId, TaskId, TaskSlug, Timestamp};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskWorkspace {
    pub meta: TaskMeta,
    pub repos: TaskRepoSelection,
    pub activity: TaskActivity,
    pub paths: TaskPaths,
}

impl TaskWorkspace {
    pub fn create(
        id: TaskId,
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
            meta: TaskMeta::new(id, slug, title, template, description, source_brief, tags)?,
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

    pub fn close(&mut self, occurred_at: Timestamp) {
        self.meta.status = super::value_objects::TaskStatus::Closed;
        self.activity.updated_at = occurred_at;
        self.activity.last_activity_at = Some(occurred_at);
    }
}
