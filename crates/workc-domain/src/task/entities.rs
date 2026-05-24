use camino::Utf8PathBuf;

use crate::shared::{MountId, RepoGroupId, RepoId, SkillId, SkillSourceId, SkillVersion, TaskId, TaskSlug, Timestamp};

use super::value_objects::{TaskSkillMountStatus, TaskStatus};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskMeta {
    pub id: TaskId,
    pub slug: TaskSlug,
    pub title: String,
    pub template: String,
    pub status: TaskStatus,
    pub description: Option<String>,
    pub source_brief: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskRepoSelection {
    pub selected_repo_groups: Vec<RepoGroupId>,
    pub repos: Vec<RepoId>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskActivity {
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub last_opened_at: Option<Timestamp>,
    pub last_activity_at: Option<Timestamp>,
    pub last_editor: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskPaths {
    pub materials_dir: Utf8PathBuf,
    pub repos_dir: Utf8PathBuf,
    pub knowledge_candidates_dir: Utf8PathBuf,
    pub task_skills_dir: Utf8PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskSkillMount {
    pub id: MountId,
    pub skill_id: SkillId,
    pub version: SkillVersion,
    pub source: SkillSourceId,
    pub mounted_at: Timestamp,
    pub status: TaskSkillMountStatus,
    pub path: Utf8PathBuf,
}

impl TaskMeta {
    pub fn new(
        id: TaskId,
        slug: TaskSlug,
        title: String,
        template: String,
        description: Option<String>,
        source_brief: Option<String>,
        tags: Vec<String>,
    ) -> Result<Self, crate::errors::DomainError> {
        if slug.as_str().trim().is_empty() {
            return Err(crate::errors::DomainError::InvalidInput {
                field: "slug",
                reason: "slug cannot be empty".to_owned(),
            });
        }

        if title.trim().is_empty() {
            return Err(crate::errors::DomainError::InvalidInput {
                field: "title",
                reason: "title cannot be empty".to_owned(),
            });
        }

        if template.trim().is_empty() {
            return Err(crate::errors::DomainError::InvalidInput {
                field: "template",
                reason: "template cannot be empty".to_owned(),
            });
        }

        if tags.iter().any(|tag| tag.trim().is_empty()) {
            return Err(crate::errors::DomainError::InvalidInput {
                field: "tags",
                reason: "tags cannot contain empty values".to_owned(),
            });
        }

        Ok(Self {
            id,
            slug,
            title,
            template,
            status: TaskStatus::Active,
            description,
            source_brief,
            tags,
        })
    }
}
