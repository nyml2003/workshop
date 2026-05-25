use serde::Serialize;
use workc_domain::shared::Timestamp;

use crate::ports::EditorKind;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub enum ApplicationTaskStatus {
    Draft,
    Active,
    Closed,
    Archived,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ListTasksQuery {
    pub status: Option<ApplicationTaskStatus>,
    pub tag: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TaskListItem {
    pub id: String,
    pub slug: String,
    pub title: String,
    pub status: ApplicationTaskStatus,
    pub last_activity_at: Option<Timestamp>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CreateTaskCommand {
    pub slug: String,
    pub title: String,
    pub template: String,
    pub description: Option<String>,
    pub source_brief: Option<String>,
    pub tags: Vec<String>,
    pub selected_repo_groups: Vec<String>,
    pub repos: Vec<String>,
    // skills are mounted by CLI layer after task creation, not consumed here
    pub initial_skills: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CreateTaskResult {
    pub task_id: String,
    pub slug: String,
    pub title: String,
    pub template: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TaskRef {
    Id(String),
    Slug(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpenTaskCommand {
    pub task: TaskRef,
    pub editor: Option<EditorKind>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CloseTaskCommand {
    pub task_id: String,
}
