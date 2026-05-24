use std::fs;

use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};
use time::{Date, PrimitiveDateTime, Time, UtcOffset};
use time::macros::format_description;
use time::format_description::well_known::Rfc3339;
use workc_domain::errors::DomainError;
use workc_domain::shared::{RepoGroupId, RepoId, TaskId, TaskSlug, Timestamp};
use workc_domain::task::{TaskIdGenerator, TaskRepository, TaskWorkspace};

pub struct FsTaskRepository {
    workspace_root: Utf8PathBuf,
}

pub struct DefaultTaskIdGenerator;

#[derive(Debug, Serialize, Deserialize)]
struct TaskToml {
    id: String,
    slug: String,
    title: String,
    template: String,
    status: String,
    created_at: TimestampValue,
    updated_at: TimestampValue,
    last_opened_at: Option<TimestampValue>,
    last_activity_at: Option<TimestampValue>,
    last_editor: Option<String>,
    description: Option<String>,
    source_brief: Option<String>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    selected_repo_groups: Vec<String>,
    #[serde(default)]
    repos: Vec<String>,
    paths: TaskTomlPaths,
}

#[derive(Debug, Serialize, Deserialize)]
struct TaskTomlPaths {
    materials: String,
    repos: String,
    knowledge_candidates: String,
    task_skills: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum TimestampValue {
    String(String),
    Parts([i64; 9]),
}

impl FsTaskRepository {
    pub fn new(workspace_root: Utf8PathBuf) -> Self {
        Self { workspace_root }
    }

    fn tasks_root(&self) -> Utf8PathBuf {
        self.workspace_root.join("tasks")
    }

    fn task_root(&self, task_id: &TaskId) -> Utf8PathBuf {
        self.tasks_root().join(task_id.as_str())
    }

    fn task_toml_path(&self, task_id: &TaskId) -> Utf8PathBuf {
        self.task_root(task_id).join("task.toml")
    }

    fn write_default_files(&self, task: &TaskWorkspace) -> Result<(), DomainError> {
        let task_root = self.task_root(&task.meta.id);
        fs::create_dir_all(task_root.join("repos")).map_err(io_error("create repos dir"))?;
        fs::create_dir_all(task_root.join("materials")).map_err(io_error("create materials dir"))?;
        fs::create_dir_all(task_root.join("knowledge-candidates"))
            .map_err(io_error("create knowledge candidates dir"))?;
        fs::create_dir_all(task_root.join(".codex").join("skills"))
            .map_err(io_error("create task skills dir"))?;

        write_if_missing(
            task_root.join("materials").join("README.md"),
            "# Task Materials\n".to_owned(),
        )?;
        write_if_missing(
            task_root.join("knowledge-candidates").join("README.md"),
            "# Knowledge Candidates\n".to_owned(),
        )?;
        write_if_missing(
            task_root.join(".codex").join("skills").join("README.md"),
            "# Task Skills\n".to_owned(),
        )?;

        Ok(())
    }

    fn to_toml(task: &TaskWorkspace) -> Result<TaskToml, DomainError> {
        Ok(TaskToml {
            id: task.meta.id.to_string(),
            slug: task.meta.slug.to_string(),
            title: task.meta.title.clone(),
            template: task.meta.template.clone(),
            status: match task.meta.status {
                workc_domain::task::TaskStatus::Draft => "draft",
                workc_domain::task::TaskStatus::Active => "active",
                workc_domain::task::TaskStatus::Closed => "closed",
                workc_domain::task::TaskStatus::Archived => "archived",
            }
            .to_owned(),
            created_at: TimestampValue::String(format_timestamp(task.activity.created_at)?),
            updated_at: TimestampValue::String(format_timestamp(task.activity.updated_at)?),
            last_opened_at: task
                .activity
                .last_opened_at
                .map(|value| format_timestamp(value).map(TimestampValue::String))
                .transpose()?,
            last_activity_at: task
                .activity
                .last_activity_at
                .map(|value| format_timestamp(value).map(TimestampValue::String))
                .transpose()?,
            last_editor: task.activity.last_editor.clone(),
            description: task.meta.description.clone(),
            source_brief: task.meta.source_brief.clone(),
            tags: task.meta.tags.clone(),
            selected_repo_groups: task
                .repos
                .selected_repo_groups
                .iter()
                .map(ToString::to_string)
                .collect(),
            repos: task.repos.repos.iter().map(ToString::to_string).collect(),
            paths: TaskTomlPaths {
                materials: task.paths.materials_dir.to_string(),
                repos: task.paths.repos_dir.to_string(),
                knowledge_candidates: task.paths.knowledge_candidates_dir.to_string(),
                task_skills: task.paths.task_skills_dir.to_string(),
            },
        })
    }

    fn from_toml(task: TaskToml) -> Result<TaskWorkspace, DomainError> {
        let status = match task.status.as_str() {
            "draft" => workc_domain::task::TaskStatus::Draft,
            "active" => workc_domain::task::TaskStatus::Active,
            "closed" => workc_domain::task::TaskStatus::Closed,
            "archived" => workc_domain::task::TaskStatus::Archived,
            other => {
                return Err(DomainError::InvalidInput {
                    field: "status",
                    reason: format!("unknown task status: {other}"),
                })
            }
        };

        Ok(TaskWorkspace {
            meta: workc_domain::task::TaskMeta {
                id: TaskId::from(task.id),
                slug: TaskSlug::from(task.slug),
                title: task.title,
                template: task.template,
                status,
                description: task.description,
                source_brief: task.source_brief,
                tags: task.tags,
            },
            repos: workc_domain::task::TaskRepoSelection {
                selected_repo_groups: task
                    .selected_repo_groups
                    .into_iter()
                    .map(RepoGroupId::from)
                    .collect(),
                repos: task.repos.into_iter().map(RepoId::from).collect(),
            },
            activity: workc_domain::task::TaskActivity {
                created_at: parse_timestamp_value(&task.created_at)?,
                updated_at: parse_timestamp_value(&task.updated_at)?,
                last_opened_at: task
                    .last_opened_at
                    .as_ref()
                    .map(parse_timestamp_value)
                    .transpose()?,
                last_activity_at: task
                    .last_activity_at
                    .as_ref()
                    .map(parse_timestamp_value)
                    .transpose()?,
                last_editor: task.last_editor,
            },
            paths: workc_domain::task::TaskPaths {
                materials_dir: task.paths.materials.into(),
                repos_dir: task.paths.repos.into(),
                knowledge_candidates_dir: task.paths.knowledge_candidates.into(),
                task_skills_dir: task.paths.task_skills.into(),
            },
        })
    }
}

impl TaskRepository for FsTaskRepository {
    fn find_by_id(&self, id: &TaskId) -> Result<Option<TaskWorkspace>, DomainError> {
        let path = self.task_toml_path(id);
        if !path.exists() {
            return Ok(None);
        }

        let raw = fs::read_to_string(&path).map_err(io_error("read task.toml"))?;
        let parsed = toml::from_str::<TaskToml>(&raw).map_err(|error| DomainError::InvalidInput {
            field: "task.toml",
            reason: error.to_string(),
        })?;
        Ok(Some(Self::from_toml(parsed)?))
    }

    fn find_by_slug(&self, slug: &TaskSlug) -> Result<Option<TaskWorkspace>, DomainError> {
        Ok(self.list()?.into_iter().find(|task| task.meta.slug == *slug))
    }

    fn list(&self) -> Result<Vec<TaskWorkspace>, DomainError> {
        let tasks_root = self.tasks_root();
        if !tasks_root.exists() {
            return Ok(Vec::new());
        }

        let mut tasks = Vec::new();
        for entry in fs::read_dir(&tasks_root).map_err(io_error("read tasks root"))? {
            let entry = entry.map_err(io_error("iterate tasks root"))?;
            let path = entry.path().join("task.toml");
            if !path.exists() {
                continue;
            }
            let raw = fs::read_to_string(&path).map_err(io_error("read task.toml"))?;
            let parsed = toml::from_str::<TaskToml>(&raw).map_err(|error| DomainError::InvalidInput {
                field: "task.toml",
                reason: error.to_string(),
            })?;
            tasks.push(Self::from_toml(parsed)?);
        }

        Ok(tasks)
    }

    fn save(&self, task: &TaskWorkspace) -> Result<(), DomainError> {
        let task_root = self.task_root(&task.meta.id);
        fs::create_dir_all(&task_root).map_err(io_error("create task root"))?;
        self.write_default_files(task)?;
        let raw = toml::to_string_pretty(&Self::to_toml(task)?).map_err(|error| DomainError::InvalidInput {
            field: "task",
            reason: error.to_string(),
        })?;
        fs::write(self.task_toml_path(&task.meta.id), raw).map_err(io_error("write task.toml"))?;
        Ok(())
    }
}

impl TaskIdGenerator for DefaultTaskIdGenerator {
    fn next_id(&self, now: Timestamp, slug_hint: &TaskSlug) -> Result<TaskId, DomainError> {
        let date = now
            .format(&format_description!("[year][month][day]"))
            .map_err(|error| DomainError::InvalidInput {
                field: "timestamp",
                reason: error.to_string(),
            })?;
        Ok(TaskId::from(format!("task-{date}-{}", slug_hint.as_str())))
    }
}

fn format_timestamp(value: Timestamp) -> Result<String, DomainError> {
    value
        .format(&Rfc3339)
        .map_err(|error| DomainError::InvalidInput {
            field: "timestamp",
            reason: error.to_string(),
        })
}

fn parse_timestamp_value(value: &TimestampValue) -> Result<Timestamp, DomainError> {
    match value {
        TimestampValue::String(raw) => Timestamp::parse(raw, &Rfc3339).map_err(|error| DomainError::InvalidInput {
            field: "timestamp",
            reason: error.to_string(),
        }),
        TimestampValue::Parts(parts) => {
            let date = Date::from_ordinal_date(parts[0] as i32, parts[1] as u16).map_err(|error| {
                DomainError::InvalidInput {
                    field: "timestamp",
                    reason: error.to_string(),
                }
            })?;
            let time = Time::from_hms_nano(parts[2] as u8, parts[3] as u8, parts[4] as u8, parts[5] as u32).map_err(
                |error| DomainError::InvalidInput {
                    field: "timestamp",
                    reason: error.to_string(),
                },
            )?;
            let offset = UtcOffset::from_hms(parts[6] as i8, parts[7] as i8, parts[8] as i8).map_err(|error| {
                DomainError::InvalidInput {
                    field: "timestamp",
                    reason: error.to_string(),
                }
            })?;
            Ok(PrimitiveDateTime::new(date, time).assume_offset(offset))
        }
    }
}

fn io_error(operation: &'static str) -> impl Fn(std::io::Error) -> DomainError {
    move |error| DomainError::IoError {
        operation,
        detail: error.to_string(),
    }
}

fn write_if_missing(path: Utf8PathBuf, content: String) -> Result<(), DomainError> {
    if path.exists() {
        return Ok(());
    }

    fs::write(path, content).map_err(io_error("write default task file"))
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use camino::Utf8PathBuf;
    use time::OffsetDateTime;

    use super::*;

    fn temp_workspace() -> Utf8PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("workc-test-{unique}"));
        Utf8PathBuf::from_path_buf(path).unwrap()
    }

    fn sample_task() -> TaskWorkspace {
        TaskWorkspace::create(
            TaskId::from("task-20260524-auth-session-fix"),
            TaskSlug::from("auth-session-fix"),
            "Fix session renewal".to_owned(),
            "default".to_owned(),
            Some("Desc".to_owned()),
            Some("Brief".to_owned()),
            vec!["auth".to_owned()],
            vec![],
            vec![RepoId::from("api-gateway")],
            OffsetDateTime::UNIX_EPOCH,
        )
        .unwrap()
    }

    #[test]
    fn save_creates_expected_layout_and_roundtrips() {
        let workspace_root = temp_workspace();
        let repo = FsTaskRepository::new(workspace_root.clone());
        let task = sample_task();

        repo.save(&task).unwrap();

        let task_root = workspace_root.join("tasks").join(task.meta.id.as_str());
        assert!(task_root.join("task.toml").exists());
        assert!(task_root.join("repos").exists());
        assert!(task_root.join("materials").join("README.md").exists());
        assert!(task_root.join("knowledge-candidates").join("README.md").exists());
        assert!(task_root.join(".codex").join("skills").join("README.md").exists());

        let loaded = repo.find_by_slug(&TaskSlug::from("auth-session-fix")).unwrap().unwrap();
        assert_eq!(loaded.meta.template, "default");
        assert_eq!(loaded.repos.repos.len(), 1);

        fs::remove_dir_all(workspace_root).unwrap();
    }

    #[test]
    fn load_accepts_legacy_array_timestamp_format() {
        let workspace_root = temp_workspace();
        let repo = FsTaskRepository::new(workspace_root.clone());
        let task_root = workspace_root.join("tasks").join("task-20260524-auth-session-fix");
        fs::create_dir_all(&task_root).unwrap();
        fs::write(
            task_root.join("task.toml"),
            r#"
id = "task-20260524-auth-session-fix"
slug = "auth-session-fix"
title = "Fix session renewal"
template = "default"
status = "active"
created_at = [2026, 144, 9, 5, 31, 812261600, 0, 0, 0]
updated_at = [2026, 144, 9, 5, 31, 812261600, 0, 0, 0]
last_activity_at = [2026, 144, 9, 5, 31, 812261600, 0, 0, 0]
tags = []
selected_repo_groups = []
repos = []

[paths]
materials = "materials"
repos = "repos"
knowledge_candidates = "knowledge-candidates"
task_skills = ".codex/skills"
"#,
        )
        .unwrap();

        let loaded = repo.find_by_slug(&TaskSlug::from("auth-session-fix")).unwrap().unwrap();
        assert_eq!(loaded.meta.template, "default");
        assert!(loaded.activity.last_activity_at.is_some());

        fs::remove_dir_all(workspace_root).unwrap();
    }
}
