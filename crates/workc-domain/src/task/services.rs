use crate::errors::DomainError;
use crate::repo_catalog::entities::RepoGroup;
use crate::shared::{RepoGroupId, RepoId, TaskId, TaskSlug, Timestamp};

use super::entities::TaskActivity;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RepoSelectionInput {
    pub selected_repo_groups: Vec<RepoGroupId>,
    pub explicit_repos: Vec<RepoId>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedRepoSelection {
    pub selected_repo_groups: Vec<RepoGroupId>,
    pub repos: Vec<RepoId>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TaskActivityEvent {
    Created,
    Opened { editor: Option<String> },
    ReposChanged,
    NotesEdited,
    SkillMounted,
    SkillUnmounted,
    KnowledgePromoted,
    Closed,
}

pub trait TaskIdGenerator {
    fn next_id(&self, now: Timestamp, slug_hint: &TaskSlug) -> Result<TaskId, DomainError>;
}

pub trait RepoSelectionResolver {
    fn resolve(
        &self,
        repo_groups: &[RepoGroup],
        input: &RepoSelectionInput,
    ) -> Result<ResolvedRepoSelection, DomainError>;
}

pub trait TaskActivityPolicy {
    fn apply(&self, activity: &TaskActivity, event: TaskActivityEvent, occurred_at: Timestamp) -> TaskActivity;
}
