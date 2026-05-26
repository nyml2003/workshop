use camino::Utf8PathBuf;

use crate::errors::FieldKind;
use crate::shared::{
    MountId, RepoGroupId, RepoId, SkillId, SkillSourceId, SkillVersion, TaskSlug, Timestamp,
};

use super::value_objects::{TaskSkillMountStatus, TaskStatus};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskMeta {
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
        slug: TaskSlug,
        title: String,
        template: String,
        description: Option<String>,
        source_brief: Option<String>,
        tags: Vec<String>,
    ) -> Result<Self, crate::errors::DomainError> {
        if slug.as_str().trim().is_empty() {
            return Err(crate::errors::DomainError::InvalidInput {
                field: FieldKind::Slug,
                reason: "slug cannot be empty".to_owned(),
            });
        }

        if title.trim().is_empty() {
            return Err(crate::errors::DomainError::InvalidInput {
                field: FieldKind::Title,
                reason: "title cannot be empty".to_owned(),
            });
        }

        if template.trim().is_empty() {
            return Err(crate::errors::DomainError::InvalidInput {
                field: FieldKind::Template,
                reason: "template cannot be empty".to_owned(),
            });
        }

        if tags.iter().any(|tag| tag.trim().is_empty()) {
            return Err(crate::errors::DomainError::InvalidInput {
                field: FieldKind::Tags,
                reason: "tags cannot contain empty values".to_owned(),
            });
        }

        Ok(Self {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::{DomainError, FieldKind};

    fn valid_meta() -> TaskMeta {
        TaskMeta::new(
            TaskSlug::from("test-slug"),
            "Test Title".to_owned(),
            "default".to_owned(),
            Some("Description".to_owned()),
            Some("Source brief".to_owned()),
            vec!["rust".to_owned(), "cli".to_owned()],
        )
        .unwrap()
    }

    #[test]
    fn creates_meta_with_valid_inputs() {
        let meta = valid_meta();
        assert_eq!(meta.slug.as_str(), "test-slug");
        assert_eq!(meta.title, "Test Title");
        assert_eq!(meta.template, "default");
        assert_eq!(meta.status, TaskStatus::Active);
        assert_eq!(meta.description.as_deref(), Some("Description"));
        assert_eq!(meta.source_brief.as_deref(), Some("Source brief"));
        assert_eq!(meta.tags, vec!["rust", "cli"]);
    }

    #[test]
    fn rejects_empty_slug() {
        let result = TaskMeta::new(
            TaskSlug::from(" "),
            "Title".to_owned(),
            "default".to_owned(),
            None,
            None,
            vec![],
        );
        assert!(
            matches!(result, Err(DomainError::InvalidInput { field, .. }) if field == FieldKind::Slug)
        );
    }

    #[test]
    fn rejects_empty_title() {
        let result = TaskMeta::new(
            TaskSlug::from("slug"),
            "".to_owned(),
            "default".to_owned(),
            None,
            None,
            vec![],
        );
        assert!(
            matches!(result, Err(DomainError::InvalidInput { field, .. }) if field == FieldKind::Title)
        );
    }

    #[test]
    fn rejects_whitespace_only_title() {
        let result = TaskMeta::new(
            TaskSlug::from("slug"),
            "   ".to_owned(),
            "default".to_owned(),
            None,
            None,
            vec![],
        );
        assert!(
            matches!(result, Err(DomainError::InvalidInput { field, .. }) if field == FieldKind::Title)
        );
    }

    #[test]
    fn rejects_empty_template() {
        let result = TaskMeta::new(
            TaskSlug::from("slug"),
            "Title".to_owned(),
            "".to_owned(),
            None,
            None,
            vec![],
        );
        assert!(
            matches!(result, Err(DomainError::InvalidInput { field, .. }) if field == FieldKind::Template)
        );
    }

    #[test]
    fn rejects_empty_tag_in_list() {
        let result = TaskMeta::new(
            TaskSlug::from("slug"),
            "Title".to_owned(),
            "default".to_owned(),
            None,
            None,
            vec!["rust".to_owned(), "".to_owned(), "cli".to_owned()],
        );
        assert!(
            matches!(result, Err(DomainError::InvalidInput { field, .. }) if field == FieldKind::Tags)
        );
    }

    #[test]
    fn accepts_empty_tags_vec() {
        let result = TaskMeta::new(
            TaskSlug::from("slug"),
            "Title".to_owned(),
            "default".to_owned(),
            None,
            None,
            vec![],
        );
        assert!(result.is_ok());
    }
}
