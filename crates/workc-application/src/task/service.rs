use camino::Utf8PathBuf;
use workc_domain::errors::{DomainError, EntityKind};
use workc_domain::shared::{RepoGroupId, RepoId, TaskSlug};
use workc_domain::task::{TaskRepository, TaskStatus, TaskWorkspace};

use super::dtos::{
    ApplicationTaskStatus, CloseTaskCommand, CreateTaskCommand, CreateTaskResult, ListTasksQuery,
    OpenTaskCommand, TaskListItem,
};
use crate::error::ApplicationError;
use crate::ports::{Clock, EditorLauncher};

pub trait TaskApplicationService {
    fn list_tasks(&self, query: ListTasksQuery) -> Result<Vec<TaskListItem>, ApplicationError>;
    fn create_task(&self, command: CreateTaskCommand)
    -> Result<CreateTaskResult, ApplicationError>;
    fn open_task(&self, command: OpenTaskCommand) -> Result<(), ApplicationError>;
    fn close_task(&self, command: CloseTaskCommand) -> Result<(), ApplicationError>;
}

pub struct DefaultTaskApplicationService {
    workspace_root: Utf8PathBuf,
    tasks: Box<dyn TaskRepository>,
    clock: Box<dyn Clock>,
    editor_launcher: Box<dyn EditorLauncher>,
}

impl DefaultTaskApplicationService {
    pub fn new(
        workspace_root: Utf8PathBuf,
        tasks: Box<dyn TaskRepository>,
        clock: Box<dyn Clock>,
        editor_launcher: Box<dyn EditorLauncher>,
    ) -> Self {
        Self {
            workspace_root,
            tasks,
            clock,
            editor_launcher,
        }
    }

    fn task_root_path(&self, _task: &TaskWorkspace) -> Utf8PathBuf {
        self.workspace_root.clone()
    }

    fn load_task(&self, slug: &str) -> Result<TaskWorkspace, ApplicationError> {
        let slug = TaskSlug::from(slug);
        let task = self.tasks.find(&slug)?;

        task.ok_or_else(|| {
            ApplicationError::Domain(DomainError::NotFound {
                entity: EntityKind::Task,
                slug: slug.to_string(),
            })
        })
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

        let limit = query.limit.filter(|&l| l > 0).unwrap_or(tasks.len());

        Ok(tasks
            .into_iter()
            .take(limit)
            .map(|task| TaskListItem {
                id: task.meta.slug.to_string(),
                slug: task.meta.slug.to_string(),
                title: task.meta.title,
                status: from_domain_status(task.meta.status),
                last_activity_at: task.activity.last_activity_at,
            })
            .collect())
    }

    fn create_task(
        &self,
        command: CreateTaskCommand,
    ) -> Result<CreateTaskResult, ApplicationError> {
        let slug = if command.slug.trim().is_empty() {
            TaskSlug::generate()
        } else {
            TaskSlug::from(command.slug.as_str())
        };

        let now = self.clock.now();
        let task = TaskWorkspace::create(
            slug.clone(),
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
            task_id: slug.to_string(),
            slug: task.meta.slug.to_string(),
            title: task.meta.title.clone(),
            template: task.meta.template.clone(),
        })
    }

    fn open_task(&self, command: OpenTaskCommand) -> Result<(), ApplicationError> {
        let editor_name = command.editor.ok_or(ApplicationError::InvalidRequest(
            "missing --editor; default-editor lookup is deferred in this phase".to_owned(),
        ))?;

        let mut task = self.load_task(&command.task)?;
        let now = self.clock.now();
        self.editor_launcher
            .open_dir(self.task_root_path(&task).as_path(), &editor_name)
            .map_err(|error| ApplicationError::ExternalFailure {
                port: "editor",
                detail: error.to_string(),
            })?;
        task.mark_opened(now, editor_name);
        self.tasks.save(&task)?;

        Ok(())
    }

    fn close_task(&self, command: CloseTaskCommand) -> Result<(), ApplicationError> {
        let slug = TaskSlug::from(command.task_id.as_str());
        let mut task = self
            .tasks
            .find(&slug)?
            .ok_or_else(|| {
                ApplicationError::Domain(DomainError::NotFound {
                    entity: EntityKind::Task,
                    slug: command.task_id.clone(),
                })
            })?;
        task.close(self.clock.now())?;
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
    use std::rc::Rc;

    use camino::{Utf8Path, Utf8PathBuf};
    use time::OffsetDateTime;
    use workc_domain::shared::{RepoId, TaskSlug};
    use workc_domain::task::{TaskActivity, TaskMeta, TaskPaths, TaskRepoSelection, TaskStatus};

    use crate::ports::{Clock, EditorError, EditorLauncher};

    use super::*;

    #[derive(Default)]
    struct InMemoryTaskRepository {
        tasks: RefCell<BTreeMap<String, TaskWorkspace>>,
    }

    impl TaskRepository for InMemoryTaskRepository {
        fn find(&self, slug: &TaskSlug) -> Result<Option<TaskWorkspace>, DomainError> {
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
                .insert(task.meta.slug.to_string(), task.clone());
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

    #[derive(Default)]
    struct RecordingEditorLauncher {
        calls: RefCell<Vec<(Utf8PathBuf, String)>>,
    }

    impl EditorLauncher for RecordingEditorLauncher {
        fn open_dir(&self, path: &Utf8Path, editor: &str) -> Result<(), EditorError> {
            self.calls
                .borrow_mut()
                .push((path.to_path_buf(), editor.to_owned()));
            Ok(())
        }
    }

    struct FailingEditorLauncher;

    impl EditorLauncher for FailingEditorLauncher {
        fn open_dir(&self, _path: &Utf8Path, _editor: &str) -> Result<(), EditorError> {
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
            Box::new(launcher),
        )
    }

    fn sample_task(
        slug: &str,
        title: &str,
        last_activity_at: Option<OffsetDateTime>,
    ) -> TaskWorkspace {
        TaskWorkspace {
            meta: TaskMeta {
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
    fn create_task_accepts_initial_skills_and_repo_groups() {
        let service = service(
            InMemoryTaskRepository::default(),
            RecordingEditorLauncher::default(),
        );

        let result = service.create_task(CreateTaskCommand {
            slug: "auth-session-fix".to_owned(),
            title: "Fix session renewal".to_owned(),
            template: "default".to_owned(),
            description: None,
            source_brief: None,
            tags: vec![],
            selected_repo_groups: vec!["auth-core".to_owned()],
            repos: vec![],
            initial_skills: vec!["frontend-testing".to_owned()],
        });

        assert!(result.is_ok());
        let created = result.unwrap();
        assert_eq!(created.slug.as_str(), "auth-session-fix");
    }

    #[test]
    fn list_tasks_sorts_by_last_activity_desc() {
        let repo = InMemoryTaskRepository::default();
        repo.save(&sample_task(
            "older",
            "Older task",
            Some(OffsetDateTime::UNIX_EPOCH),
        ))
        .unwrap();
        repo.save(&sample_task(
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
            "auth-session-fix",
            "Fix session renewal",
            None,
        ))
        .unwrap();
        let service = service(repo, RecordingEditorLauncher::default());

        let result = service.open_task(OpenTaskCommand {
            task: "auth-session-fix".to_owned(),
            editor: None,
        });

        assert!(
            matches!(result, Err(ApplicationError::InvalidRequest(message)) if message.contains("missing --editor"))
        );
    }

    #[test]
    fn open_task_does_not_persist_opened_state_when_launcher_fails() {
        let repo = InMemoryTaskRepository::default();
        repo.save(&sample_task(
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
            Box::new(FailingEditorLauncher),
        );

        let result = service.open_task(OpenTaskCommand {
            task: "auth-session-fix".to_owned(),
            editor: Some("cursor".to_owned()),
        });

        assert!(
            matches!(result, Err(ApplicationError::ExternalFailure { port, .. }) if port == "editor")
        );

        let stored = service
            .tasks
            .find(&TaskSlug::from("auth-session-fix"))
            .unwrap()
            .unwrap();
        assert_eq!(stored.activity.last_opened_at, None);
        assert_eq!(stored.activity.last_editor, None);
    }

    #[test]
    fn close_task_sets_status_to_closed() {
        let repo = InMemoryTaskRepository::default();
        repo.save(&sample_task(
            "auth-session-fix",
            "Fix session renewal",
            None,
        ))
        .unwrap();
        let service = service(repo, RecordingEditorLauncher::default());

        service
            .close_task(CloseTaskCommand {
                task_id: "auth-session-fix".to_owned(),
            })
            .unwrap();

        let stored = service
            .tasks
            .find(&TaskSlug::from("auth-session-fix"))
            .unwrap()
            .unwrap();
        assert_eq!(stored.meta.status, TaskStatus::Closed);
    }

    #[test]
    fn close_task_fails_for_missing_task() {
        let repo = InMemoryTaskRepository::default();
        let service = service(repo, RecordingEditorLauncher::default());

        let result = service.close_task(CloseTaskCommand {
            task_id: "missing".to_owned(),
        });
        assert!(result.is_err());
    }

    #[test]
    fn create_task_generates_slug_when_empty() {
        let service = service(
            InMemoryTaskRepository::default(),
            RecordingEditorLauncher::default(),
        );
        let result = service
            .create_task(CreateTaskCommand {
                slug: String::new(),
                title: "Title".to_owned(),
                template: "default".to_owned(),
                description: None,
                source_brief: None,
                tags: vec![],
                selected_repo_groups: vec![],
                repos: vec![],
                initial_skills: vec![],
            })
            .unwrap();
        assert_eq!(result.slug.len(), 8);
    }

    #[test]
    fn list_tasks_respects_limit() {
        let repo = InMemoryTaskRepository::default();
        repo.save(&sample_task("a", "A", None)).unwrap();
        repo.save(&sample_task("b", "B", None)).unwrap();
        repo.save(&sample_task("c", "C", None)).unwrap();
        let service = service(repo, RecordingEditorLauncher::default());

        let items = service
            .list_tasks(ListTasksQuery {
                status: None,
                tag: None,
                limit: Some(2),
            })
            .unwrap();

        assert_eq!(items.len(), 2);
    }

    #[test]
    fn list_tasks_empty_with_no_tasks() {
        let repo = InMemoryTaskRepository::default();
        let service = service(repo, RecordingEditorLauncher::default());

        let items = service
            .list_tasks(ListTasksQuery {
                status: None,
                tag: None,
                limit: None,
            })
            .unwrap();

        assert!(items.is_empty());
    }

    #[test]
    fn load_task_returns_error_for_missing() {
        let repo = InMemoryTaskRepository::default();
        let service = service(repo, RecordingEditorLauncher::default());

        let result = service.load_task("missing");
        assert!(result.is_err());
    }

    #[test]
    fn open_task_launches_editor_at_workspace_root() {
        let repo = InMemoryTaskRepository::default();
        repo.save(&sample_task(
            "auth-session-fix",
            "Fix session renewal",
            None,
        ))
        .unwrap();
        let launcher = Rc::new(RefCell::new(Vec::new()));
        let calls = launcher.clone();

        struct Capture(Rc<RefCell<Vec<(Utf8PathBuf, String)>>>);
        impl EditorLauncher for Capture {
            fn open_dir(&self, path: &Utf8Path, editor: &str) -> Result<(), EditorError> {
                let name = match editor {
                    "cursor" => "cursor",
                    "vscode" => "vscode",
                    _ => "other",
                };
                self.0.borrow_mut().push((path.to_path_buf(), name.to_owned()));
                Ok(())
            }
        }

        let svc = DefaultTaskApplicationService::new(
            Utf8PathBuf::from("/workspace"),
            Box::new(repo),
            Box::new(FixedClock { now: OffsetDateTime::UNIX_EPOCH }),
            Box::new(Capture(calls)),
        );

        svc.open_task(OpenTaskCommand {
            task: "auth-session-fix".to_owned(),
            editor: Some("cursor".to_owned()),
        })
        .unwrap();

        assert_eq!(launcher.borrow()[0].0.as_str(), "/workspace");
    }
}
