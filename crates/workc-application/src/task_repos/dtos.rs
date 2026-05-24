use camino::Utf8PathBuf;
use serde::Serialize;
use workc_domain::shared::Timestamp;

use crate::ports::RepoStatus;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SetTaskReposCommand {
    pub task_id: String,
    pub selected_repo_groups: Vec<String>,
    pub repos: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AddTaskReposCommand {
    pub task_id: String,
    pub repos: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RemoveTaskReposCommand {
    pub task_id: String,
    pub repos: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CloneTaskReposCommand {
    pub task_id: String,
    pub repos: Option<Vec<String>>,
    pub missing_only: bool,
    pub dry_run: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RepoCloneOutcome {
    pub repo_id: String,
    pub path: Utf8PathBuf,
    pub cloned: bool,
    pub dry_run: bool,
    pub skipped_reason: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RepoStatusQuery {
    pub task_id: String,
    pub repos: Option<Vec<String>>,
    pub clone_state: Option<CloneStateFilter>,
    pub dry_run: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TaskRepoStatusItem {
    pub repo_id: String,
    pub path: Utf8PathBuf,
    pub status: RepoStatus,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CloneStateFilter {
    Missing,
    Ready,
    Dirty,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TaskReposResult {
    pub task_id: String,
    pub selected_repo_groups: Vec<String>,
    pub repos: Vec<String>,
    pub updated_at: Timestamp,
}
