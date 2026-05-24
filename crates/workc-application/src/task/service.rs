use camino::Utf8PathBuf;
use workc_domain::errors::DomainError;
use workc_domain::shared::{RepoGroupId, RepoId, TaskId, TaskSlug};
use workc_domain::task::{TaskIdGenerator, TaskRepository, TaskStatus, TaskWorkspace};

use crate::error::ApplicationError;
use crate::ports::{Clock, EditorLauncher};
use super::dtos::{
    ApplicationTaskStatus, CloseTaskCommand, CreateTaskCommand, CreateTaskResult, ListTasksQuery, OpenTaskCommand, TaskListItem, TaskRef,
};

pub trait TaskApplicationService {
    fn list_tasks(&self, query: ListTasksQuery) -> Result<Vec<TaskListItem>, ApplicationError>;
    fn create_task(&self, command: CreateTaskCommand) -> Result<CreateTaskResult, ApplicationError>;
    fn open_task(&self, command: OpenTaskCommand) -> Result<(), ApplicationError>;
    fn close_task(&self, command: CloseTaskCommand) -> Result<(), ApplicationError>;
}

pub struct DefaultTaskApplicationService {
    workspace_root: Utf8PathBuf,
    tasks: Box<dyn TaskRepository>,
    clock: Box<dyn Clock>,
    id_generator: Box<dyn TaskIdGenerator>,
    editor_launcher: Box<dyn EditorLauncher>,
}

impl DefaultTaskApplicationService {
    pub fn new(
        workspace_root: Utf8PathBuf,
        tasks: Box<dyn TaskRepository>,
        clock: Box<dyn Clock>,
        id_generator: Box<dyn TaskIdGenerator>,
        editor_launcher: Box<dyn EditorLauncher>,
    ) -> Self {
        Self {
            workspace_root,
            tasks,
            clock,
            id_generator,
            editor_launcher,
        }
    }

    fn task_root_path(&self, task: &TaskWorkspace) -> Utf8PathBuf {
        self.workspace_root.join("tasks").join(task.meta.id.as_str())
    }

    fn load_task(&self, task_ref: &TaskRef) -> Result<TaskWorkspace, ApplicationError> {
        let task = match task_ref {
            TaskRef::Id(id) => self.tasks.find_by_id(&TaskId::from(id.as_str()))?,
            TaskRef::Slug(slug) => self.tasks.find_by_slug(&TaskSlug::from(slug.as_str()))?,
        };

        task.ok_or_else(|| ApplicationError::Domain(DomainError::NotFound {
            entity: "task",
            id: match task_ref {
                TaskRef::Id(id) => id.clone(),
                TaskRef::Slug(slug) => slug.clone(),
            },
        }))
    }
}

impl TaskApplicationService for DefaultTaskApplicationService {
    fn list_tasks(&self, query: ListTasksQuery) -> Result<Vec<TaskListItem>, ApplicationError> {
        let mut tasks = self.tasks.list()?;

        tasks.retain(|task| {
            let status_match = query
                .status
                .as_ref()
                .is_none_or(|status| task.meta.status == to_domain_status(status));
            let tag_match = query
                .tag
                .as_ref()
                .is_none_or(|tag| task.meta.tags.iter().any(|value| value == tag));
            status_match && tag_match
        });

        tasks.sort_by(|left, right| {
            right
                .activity
                .last_activity_at
                .cmp(&left.activity.last_activity_at)
                .then_with(|| left.meta.slug.as_str().cmp(right.meta.slug.as_str()))
        });

        let limit = query.limit.unwrap_or(tasks.len());

        Ok(tasks
            .into_iter()
            .take(limit)
            .map(|task| TaskListItem {
                id: task.meta.id.to_string(),
                slug: task.meta.slug.to_string(),
                title: task.meta.title,
                status: from_domain_status(task.meta.status),
                last_activity_at: task.activity.last_activity_at,
            })
            .collect())
    }

    fn create_task(&self, command: CreateTaskCommand) -> Result<CreateTaskResult, ApplicationError> {
        if !command.selected_repo_groups.is_empty() {
            return Err(ApplicationError::InvalidRequest(
                "repo-group enrichment is not supported in this phase".to_owned(),
            ));
        }

        if !command.initial_skills.is_empty() {
            return Err(ApplicationError::InvalidRequest(
                "initial skill mounts are not supported in this phase".to_owned(),
            ));
        }

        let slug = TaskSlug::from(command.slug.as_str());

        if self.tasks.find_by_slug(&slug)?.is_some() {
            return Err(ApplicationError::Domain(DomainError::AlreadyExists {
                entity: "task",
                id: slug.to_string(),
            }));
        }

        let now = self.clock.now();
        let task_id = self.id_generator.next_id(now, &slug)?;
        let task = TaskWorkspace::create(
            task_id,
            slug,
            command.title,
            command.template,
            command.description,
            command.source_brief,
            command.tags,
            command
                .selected_repo_groups
                .into_iter()
                .map(RepoGroupId::from)
                .collect(),
            command.repos.into_iter().map(RepoId::from).collect(),
            now,
        )
        .map_err(ApplicationError::from)?;

        self.tasks.save(&task)?;

        Ok(CreateTaskResult {
            task_id: task.meta.id.to_string(),
            slug: task.meta.slug.to_string(),
            title: task.meta.title.clone(),
            template: task.meta.template.clone(),
        })
    }

    fn open_task(&self, command: OpenTaskCommand) -> Result<(), ApplicationError> {
        let editor = command.editor.ok_or(ApplicationError::InvalidRequest(
            "missing --editor; default-editor lookup is deferred in this phase".to_owned(),
        ))?;

        let mut task = self.load_task(&command.task)?;
        let now = self.clock.now();
        let editor_name = match &editor {
            crate::ports::EditorKind::Cursor => "cursor".to_owned(),
            crate::ports::EditorKind::VsCode => "vscode".to_owned(),
            crate::ports::EditorKind::Other(name) => name.clone(),
        };
        self.editor_launcher
            .open_dir(self.task_root_path(&task).as_path(), editor.clone())
            .map_err(|error| ApplicationError::ExternalFailure {
                port: "editor",
                detail: error.to_string(),
            })?;
        task.mark_opened(now, editor_name);
        self.tasks.save(&task)?;

        Ok(())
    }

    fn close_task(&self, command: CloseTaskCommand) -> Result<(), ApplicationError> {
        let mut task = self
            .tasks
            .find_by_id(&TaskId::from(command.task_id.as_str()))?
            .ok_or_else(|| ApplicationError::Domain(DomainError::NotFound {
                entity: "task",
                id: command.task_id.clone(),
            }))?;
        task.close(self.clock.now());
        self.tasks.save(&task)?;
        Ok(())
    }
}

fn to_domain_status(value: &ApplicationTaskStatus) -> TaskStatus {
    match value {
        ApplicationTaskStatus::Draft => TaskStatus::Draft,
        ApplicationTaskStatus::Active => TaskStatus::Active,
        ApplicationTaskStatus::Closed => TaskStatus::Closed,
        ApplicationTaskStatus::Archived => TaskStatus::Archived,
    }
}

fn from_domain_status(value: TaskStatus) -> ApplicationTaskStatus {
    match value {
        TaskStatus::Draft => ApplicationTaskStatus::Draft,
        TaskStatus::Active => ApplicationTaskStatus::Active,
        TaskStatus::Closed => ApplicationTaskStatus::Closed,
        TaskStatus::Archived => ApplicationTaskStatus::Archived,
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::collections::BTreeMap;

    use camino::{Utf8Path, Utf8PathBuf};
    use time::OffsetDateTime;
    use workc_domain::shared::{RepoId, TaskId, TaskSlug};
    use workc_domain::task::{TaskActivity, TaskMeta, TaskPaths, TaskRepoSelection, TaskStatus};

    use crate::ports::{Clock, EditorError, EditorKind, EditorLauncher};

    use super::*;

    #[derive(Default)]
    struct InMemoryTaskRepository {
        tasks: RefCell<BTreeMap<String, TaskWorkspace>>,
    }

    impl TaskRepository for InMemoryTaskRepository {
        fn find_by_id(&self, id: &workc_domain::shared::TaskId) -> Result<Option<TaskWorkspace>, DomainError> {
            Ok(self.tasks.borrow().values().find(|task| task.meta.id == *id).cloned())
        }

        fn find_by_slug(&self, slug: &TaskSlug) -> Result<Option<TaskWorkspace>, DomainError> {
            Ok(self
                .tasks
                .borrow()
                .values()
                .find(|task| task.meta.slug == *slug)
                .cloned())
        }

        fn list(&self) -> Result<Vec<TaskWorkspace>, DomainError> {
            Ok(self.tasks.borrow().values().cloned().collect())
        }

        fn save(&self, task: &TaskWorkspace) -> Result<(), DomainError> {
            self.tasks
                .borrow_mut()
                .insert(task.meta.id.to_string(), task.clone());
            Ok(())
        }
    }

    struct FixedClock {
        now: OffsetDateTime,
    }

    impl Clock for FixedClock {
        fn now(&self) -> OffsetDateTime {
            self.now
        }
    }

    struct FixedTaskIdGenerator {
        next: TaskId,
    }

    impl TaskIdGenerator for FixedTaskIdGenerator {
        fn next_id(&self, _now: OffsetDateTime, _slug_hint: &TaskSlug) -> Result<TaskId, DomainError> {
            Ok(self.next.clone())
        }
    }

    #[derive(Default)]
    struct RecordingEditorLauncher {
        calls: RefCell<Vec<(Utf8PathBuf, String)>>,
    }

    impl EditorLauncher for RecordingEditorLauncher {
        fn open_dir(&self, path: &Utf8Path, editor: EditorKind) -> Result<(), EditorError> {
            let editor_name = match editor {
                EditorKind::Cursor => "cursor".to_owned(),
                EditorKind::VsCode => "vscode".to_owned(),
                EditorKind::Other(name) => name,
            };
            self.calls
                .borrow_mut()
                .push((path.to_path_buf(), editor_name));
            Ok(())
        }
    }

    struct FailingEditorLauncher;

    impl EditorLauncher for FailingEditorLauncher {
        fn open_dir(&self, _path: &Utf8Path, _editor: EditorKind) -> Result<(), EditorError> {
            Err(EditorError {
                detail: "editor unavailable".to_owned(),
            })
        }
    }

    fn service(
        repo: InMemoryTaskRepository,
        launcher: RecordingEditorLauncher,
    ) -> DefaultTaskApplicationService {
        let now = OffsetDateTime::UNIX_EPOCH;
        DefaultTaskApplicationService::new(
            Utf8PathBuf::from("/workspace"),
            Box::new(repo),
            Box::new(FixedClock { now }),
            Box::new(FixedTaskIdGenerator {
                next: TaskId::from("task-20260524-auth-session-fix"),
            }),
            Box::new(launcher),
        )
    }

    fn sample_task(id: &str, slug: &str, title: &str, last_activity_at: Option<OffsetDateTime>) -> TaskWorkspace {
        TaskWorkspace {
            meta: TaskMeta {
                id: TaskId::from(id),
                slug: TaskSlug::from(slug),
                title: title.to_owned(),
                template: "default".to_owned(),
                status: TaskStatus::Active,
                description: None,
                source_brief: None,
                tags: vec!["auth".to_owned()],
            },
            repos: TaskRepoSelection {
                selected_repo_groups: vec![],
                repos: vec![RepoId::from("api-gateway")],
            },
            activity: TaskActivity {
                created_at: OffsetDateTime::UNIX_EPOCH,
                updated_at: OffsetDateTime::UNIX_EPOCH,
                last_opened_at: None,
                last_activity_at,
                last_editor: None,
            },
            paths: TaskPaths {
                materials_dir: "materials".into(),
                repos_dir: "repos".into(),
                knowledge_candidates_dir: "knowledge-candidates".into(),
                task_skills_dir: ".codex/skills".into(),
            },
        }
    }

    #[test]
    fn create_task_rejects_initial_skill_inputs() {
        let service = service(InMemoryTaskRepository::default(), RecordingEditorLauncher::default());

        let result = service.create_task(CreateTaskCommand {
            slug: "auth-session-fix".to_owned(),
            title: "Fix session renewal".to_owned(),
            template: "default".to_owned(),
            description: None,
            source_brief: None,
            tags: vec![],
            selected_repo_groups: vec![],
            repos: vec![],
            initial_skills: vec!["frontend-testing".to_owned()],
        });

        assert!(matches!(result, Err(ApplicationError::InvalidRequest(message)) if message.contains("initial skill mounts")));
    }

    #[test]
    fn list_tasks_sorts_by_last_activity_desc() {
        let repo = InMemoryTaskRepository::default();
        repo.save(&sample_task(
            "task-older",
            "older",
            "Older task",
            Some(OffsetDateTime::UNIX_EPOCH),
        ))
        .unwrap();
        repo.save(&sample_task(
            "task-newer",
            "newer",
            "Newer task",
            Some(OffsetDateTime::UNIX_EPOCH + time::Duration::hours(1)),
        ))
        .unwrap();
        let service = service(repo, RecordingEditorLauncher::default());

        let items = service
            .list_tasks(ListTasksQuery {
                status: Some(ApplicationTaskStatus::Active),
                tag: Some("auth".to_owned()),
                limit: None,
            })
            .unwrap();

        assert_eq!(items.len(), 2);
        assert_eq!(items[0].slug.as_str(), "newer");
        assert_eq!(items[1].slug.as_str(), "older");
    }

    #[test]
    fn open_task_requires_editor_in_this_phase() {
        let repo = InMemoryTaskRepository::default();
        repo.save(&sample_task(
            "task-20260524-auth-session-fix",
            "auth-session-fix",
            "Fix session renewal",
            None,
        ))
        .unwrap();
        let service = service(repo, RecordingEditorLauncher::default());

        let result = service.open_task(OpenTaskCommand {
            task: TaskRef::Slug("auth-session-fix".to_owned()),
            editor: None,
        });

        assert!(matches!(result, Err(ApplicationError::InvalidRequest(message)) if message.contains("missing --editor")));
    }

    #[test]
    fn open_task_does_not_persist_opened_state_when_launcher_fails() {
        let repo = InMemoryTaskRepository::default();
        repo.save(&sample_task(
            "task-20260524-auth-session-fix",
            "auth-session-fix",
            "Fix session renewal",
            None,
        ))
        .unwrap();
        let service = DefaultTaskApplicationService::new(
            Utf8PathBuf::from("/workspace"),
            Box::new(repo),
            Box::new(FixedClock {
                now: OffsetDateTime::UNIX_EPOCH + time::Duration::hours(2),
            }),
            Box::new(FixedTaskIdGenerator {
                next: TaskId::from("task-20260524-auth-session-fix"),
            }),
            Box::new(FailingEditorLauncher),
        );

        let result = service.open_task(OpenTaskCommand {
            task: TaskRef::Slug("auth-session-fix".to_owned()),
            editor: Some(EditorKind::Cursor),
        });

        assert!(matches!(result, Err(ApplicationError::ExternalFailure { port, .. }) if port == "editor"));

        let stored = service
            .tasks
            .find_by_slug(&TaskSlug::from("auth-session-fix"))
            .unwrap()
            .unwrap();
        assert_eq!(stored.activity.last_opened_at, None);
        assert_eq!(stored.activity.last_editor, None);
    }
}
