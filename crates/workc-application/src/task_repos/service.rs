use std::collections::BTreeSet;

use camino::Utf8PathBuf;
use workc_domain::errors::EntityKind;
use workc_domain::errors::DomainError;
use workc_domain::repo_catalog::RepoCatalogRepository;
use workc_domain::shared::{RepoGroupId, RepoId, TaskSlug};
use workc_domain::task::TaskRepository;

use crate::error::ApplicationError;
use crate::ports::{Clock, GitClient};

use super::dtos::{
    AddTaskReposCommand, CloneStateFilter, CloneTaskReposCommand, RemoveTaskReposCommand,
    RepoCloneOutcome, RepoStatusQuery, SetTaskReposCommand, TaskRepoStatusItem, TaskReposResult,
};

pub trait TaskReposApplicationService {
    fn set_task_repos(
        &self,
        command: SetTaskReposCommand,
    ) -> Result<TaskReposResult, ApplicationError>;
    fn add_task_repos(
        &self,
        command: AddTaskReposCommand,
    ) -> Result<TaskReposResult, ApplicationError>;
    fn remove_task_repos(
        &self,
        command: RemoveTaskReposCommand,
    ) -> Result<TaskReposResult, ApplicationError>;
    fn clone_task_repos(
        &self,
        command: CloneTaskReposCommand,
    ) -> Result<Vec<RepoCloneOutcome>, ApplicationError>;
    fn get_repo_statuses(
        &self,
        query: RepoStatusQuery,
    ) -> Result<Vec<TaskRepoStatusItem>, ApplicationError>;
}

pub struct DefaultTaskReposApplicationService {
    tasks: Box<dyn TaskRepository>,
    repo_catalog: Box<dyn RepoCatalogRepository>,
    clock: Box<dyn Clock>,
    git_client: Box<dyn GitClient>,
}

impl DefaultTaskReposApplicationService {
    pub fn new(
        tasks: Box<dyn TaskRepository>,
        repo_catalog: Box<dyn RepoCatalogRepository>,
        clock: Box<dyn Clock>,
        git_client: Box<dyn GitClient>,
    ) -> Self {
        Self {
            tasks,
            repo_catalog,
            clock,
            git_client,
        }
    }

    fn load_task(
        &self,
        task_ref: &str,
    ) -> Result<workc_domain::task::TaskWorkspace, ApplicationError> {
        let slug = TaskSlug::from(task_ref);
        let task = self.tasks.find(&slug)?;

        task.ok_or_else(|| {
            ApplicationError::Domain(DomainError::NotFound {
                entity: EntityKind::Task,
                slug: task_ref.to_owned(),
            })
        })
    }

    fn validate_repos_exist(&self, repo_ids: &[RepoId]) -> Result<(), ApplicationError> {
        let catalog = self.repo_catalog.load()?;
        for repo_id in repo_ids {
            if !catalog.repos.iter().any(|repo| repo.id == *repo_id) {
                return Err(ApplicationError::Domain(DomainError::NotFound {
                    entity: EntityKind::Repo,
                    slug: repo_id.to_string(),
                }));
            }
        }
        Ok(())
    }

    fn resolve_group_repos(
        &self,
        selected_repo_groups: &[RepoGroupId],
    ) -> Result<Vec<RepoId>, ApplicationError> {
        let catalog = self.repo_catalog.load()?;
        let mut repos = Vec::new();
        for group_id in selected_repo_groups {
            let group = catalog
                .groups
                .iter()
                .find(|group| group.id == *group_id)
                .ok_or_else(|| {
                    ApplicationError::Domain(DomainError::NotFound {
                        entity: EntityKind::RepoGroup,
                        slug: group_id.to_string(),
                    })
                })?;
            repos.extend(group.repos.clone());
        }
        Ok(repos)
    }

    fn update_task_repos(
        &self,
        mut task: workc_domain::task::TaskWorkspace,
        selected_repo_groups: Vec<RepoGroupId>,
        repos: Vec<RepoId>,
    ) -> Result<TaskReposResult, ApplicationError> {
        let now = self.clock.now();
        task.repos.selected_repo_groups = selected_repo_groups;
        task.repos.repos = repos.clone();
        task.activity.updated_at = now;
        task.activity.last_activity_at = Some(now);
        self.tasks.save(&task)?;

        Ok(TaskReposResult {
            task_id: task.meta.slug.to_string(),
            selected_repo_groups: task
                .repos
                .selected_repo_groups
                .iter()
                .map(ToString::to_string)
                .collect(),
            repos: repos.into_iter().map(|repo| repo.to_string()).collect(),
            updated_at: now,
        })
    }

    fn task_repo_path(task: &workc_domain::task::TaskWorkspace, repo_id: &RepoId) -> Utf8PathBuf {
        Utf8PathBuf::from(task.paths.repos_dir.as_str())
            .join(repo_id.as_str())
    }

    fn resolve_clone_targets(
        &self,
        task: &workc_domain::task::TaskWorkspace,
        selected_repos: Option<Vec<String>>,
        missing_only: bool,
    ) -> Result<Vec<(RepoId, Utf8PathBuf, String, bool)>, ApplicationError> {
        let catalog = self.repo_catalog.load()?;
        let selected: Option<BTreeSet<String>> =
            selected_repos.map(|repos| repos.into_iter().collect());
        let mut targets = Vec::new();

        for repo_id in &task.repos.repos {
            if let Some(selected) = &selected {
                if !selected.contains(repo_id.as_str()) {
                    continue;
                }
            }

            let repo = catalog
                .repos
                .iter()
                .find(|entry| entry.id == *repo_id)
                .ok_or_else(|| {
                    ApplicationError::Domain(DomainError::NotFound {
                        entity: EntityKind::Repo,
                        slug: repo_id.to_string(),
                    })
                })?;
            let path = Self::task_repo_path(task, repo_id);
            let exists = path.exists();
            if missing_only && exists {
                continue;
            }
            targets.push((repo_id.clone(), path, repo.url.clone(), exists));
        }

        Ok(targets)
    }
}

impl TaskReposApplicationService for DefaultTaskReposApplicationService {
    fn set_task_repos(
        &self,
        command: SetTaskReposCommand,
    ) -> Result<TaskReposResult, ApplicationError> {
        let task = self.load_task(&command.task_id)?;
        let group_ids: Vec<RepoGroupId> = command
            .selected_repo_groups
            .iter()
            .map(|group| RepoGroupId::from(group.as_str()))
            .collect();
        let mut merged: BTreeSet<String> = self
            .resolve_group_repos(&group_ids)?
            .into_iter()
            .map(|repo| repo.to_string())
            .collect();
        for repo in &command.repos {
            merged.insert(repo.clone());
        }
        let repos: Vec<RepoId> = merged.into_iter().map(RepoId::from).collect();
        self.validate_repos_exist(&repos)?;
        self.update_task_repos(task, group_ids, repos)
    }

    fn add_task_repos(
        &self,
        command: AddTaskReposCommand,
    ) -> Result<TaskReposResult, ApplicationError> {
        let task = self.load_task(&command.task_id)?;
        let selected_repo_groups = task.repos.selected_repo_groups.clone();
        let mut merged: BTreeSet<String> =
            task.repos.repos.iter().map(ToString::to_string).collect();
        for repo in &command.repos {
            merged.insert(repo.clone());
        }
        let repos: Vec<RepoId> = merged.into_iter().map(RepoId::from).collect();
        self.validate_repos_exist(&repos)?;
        self.update_task_repos(task, selected_repo_groups, repos)
    }

    fn remove_task_repos(
        &self,
        command: RemoveTaskReposCommand,
    ) -> Result<TaskReposResult, ApplicationError> {
        let task = self.load_task(&command.task_id)?;
        let selected_repo_groups = task.repos.selected_repo_groups.clone();
        let remove: BTreeSet<String> = command.repos.into_iter().collect();
        let repos: Vec<RepoId> = task
            .repos
            .repos
            .iter()
            .filter(|repo| !remove.contains(repo.as_str()))
            .cloned()
            .collect();
        self.update_task_repos(task, selected_repo_groups, repos)
    }

    fn clone_task_repos(
        &self,
        command: CloneTaskReposCommand,
    ) -> Result<Vec<RepoCloneOutcome>, ApplicationError> {
        let task = self.load_task(&command.task_id)?;
        let targets = self.resolve_clone_targets(&task, command.repos, command.missing_only)?;

        if command.dry_run {
            return Ok(targets
                .into_iter()
                .map(|(repo_id, path, _url, exists)| RepoCloneOutcome {
                    repo_id: repo_id.to_string(),
                    path,
                    cloned: false,
                    dry_run: true,
                    skipped_reason: if exists {
                        Some("already present".to_owned())
                    } else {
                        Some("dry-run".to_owned())
                    },
                })
                .collect());
        }

        let mut outcomes = Vec::new();
        let now = self.clock.now();
        let mut touched = false;

        for (repo_id, path, url, exists) in targets {
            if exists {
                outcomes.push(RepoCloneOutcome {
                    repo_id: repo_id.to_string(),
                    path,
                    cloned: false,
                    dry_run: false,
                    skipped_reason: Some("already present".to_owned()),
                });
                continue;
            }

            self.git_client
                .clone_repo(path.as_path(), &url)
                .map_err(|error| ApplicationError::ExternalFailure {
                    port: "git",
                    detail: error.to_string(),
                })?;
            touched = true;
            outcomes.push(RepoCloneOutcome {
                repo_id: repo_id.to_string(),
                path,
                cloned: true,
                dry_run: false,
                skipped_reason: None,
            });
        }

        if touched {
            let mut task = task;
            task.activity.updated_at = now;
            task.activity.last_activity_at = Some(now);
            self.tasks.save(&task)?;
        }

        Ok(outcomes)
    }

    fn get_repo_statuses(
        &self,
        query: RepoStatusQuery,
    ) -> Result<Vec<TaskRepoStatusItem>, ApplicationError> {
        let task = self.load_task(&query.task_id)?;
        let selected: Option<BTreeSet<String>> =
            query.repos.map(|repos| repos.into_iter().collect());
        let mut items = Vec::new();

        for repo_id in &task.repos.repos {
            if let Some(selected) = &selected {
                if !selected.contains(repo_id.as_str()) {
                    continue;
                }
            }

            let path = Self::task_repo_path(&task, repo_id);
            let status = if query.dry_run {
                if path.exists() {
                    crate::ports::RepoStatus {
                        branch: None,
                        dirty: false,
                        ahead: 0,
                        behind: 0,
                        clone_state: crate::ports::CloneState::Ready,
                    }
                } else {
                    crate::ports::RepoStatus {
                        branch: None,
                        dirty: false,
                        ahead: 0,
                        behind: 0,
                        clone_state: crate::ports::CloneState::Missing,
                    }
                }
            } else {
                self.git_client
                    .get_repo_status(path.as_path())
                    .map_err(|error| ApplicationError::ExternalFailure {
                        port: "git",
                        detail: error.to_string(),
                    })?
            };

            if let Some(filter) = &query.clone_state {
                let matches = matches!(
                    (filter, &status.clone_state),
                    (CloneStateFilter::Missing, crate::ports::CloneState::Missing)
                        | (CloneStateFilter::Ready, crate::ports::CloneState::Ready)
                        | (CloneStateFilter::Dirty, crate::ports::CloneState::Dirty)
                        | (CloneStateFilter::Unknown, crate::ports::CloneState::Unknown)
                );
                if !matches {
                    continue;
                }
            }

            items.push(TaskRepoStatusItem {
                repo_id: repo_id.to_string(),
                path,
                status,
            });
        }

        Ok(items)
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::collections::BTreeMap;

    use time::OffsetDateTime;
    use workc_domain::repo_catalog::{RepoCatalog, RepoCatalogRepository, RepoEntry, RepoGroup};
    use workc_domain::shared::{RepoId, TaskSlug};
    use workc_domain::task::{
        TaskActivity, TaskMeta, TaskPaths, TaskRepoSelection, TaskRepository, TaskStatus,
        TaskWorkspace,
    };

    use crate::ports::{Clock, CloneState, GitClient, RepoStatus};

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

    struct InMemoryRepoCatalogRepository {
        catalog: RefCell<RepoCatalog>,
    }

    impl RepoCatalogRepository for InMemoryRepoCatalogRepository {
        fn load(&self) -> Result<RepoCatalog, DomainError> {
            Ok(self.catalog.borrow().clone())
        }

        fn save(&self, catalog: &RepoCatalog) -> Result<(), DomainError> {
            *self.catalog.borrow_mut() = catalog.clone();
            Ok(())
        }

        fn find_repo(&self, id: &RepoId) -> Result<Option<RepoEntry>, DomainError> {
            Ok(self
                .catalog
                .borrow()
                .repos
                .iter()
                .find(|repo| repo.id == *id)
                .cloned())
        }

        fn find_group(&self, id: &RepoGroupId) -> Result<Option<RepoGroup>, DomainError> {
            Ok(self
                .catalog
                .borrow()
                .groups
                .iter()
                .find(|group| group.id == *id)
                .cloned())
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

    fn sample_task() -> TaskWorkspace {
        TaskWorkspace {
            meta: TaskMeta {
                slug: TaskSlug::from("auth-session-fix"),
                title: "Fix session renewal".to_owned(),
                template: "default".to_owned(),
                status: TaskStatus::Active,
                description: None,
                source_brief: None,
                tags: vec!["auth".to_owned()],
            },
            repos: TaskRepoSelection {
                selected_repo_groups: vec![],
                repos: vec![],
            },
            activity: TaskActivity {
                created_at: OffsetDateTime::UNIX_EPOCH,
                updated_at: OffsetDateTime::UNIX_EPOCH,
                last_opened_at: None,
                last_activity_at: Some(OffsetDateTime::UNIX_EPOCH),
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

    fn sample_catalog() -> RepoCatalog {
        RepoCatalog {
            repos: vec![
                RepoEntry {
                    id: RepoId::from("api-gateway"),
                    url: "git@github.com:example/api-gateway.git".to_owned(),
                    tags: vec!["backend".to_owned()],
                    description: None,
                },
                RepoEntry {
                    id: RepoId::from("auth-service"),
                    url: "git@github.com:example/auth-service.git".to_owned(),
                    tags: vec!["backend".to_owned()],
                    description: None,
                },
            ],
            groups: vec![RepoGroup {
                id: RepoGroupId::from("auth-core"),
                repos: vec![RepoId::from("api-gateway"), RepoId::from("auth-service")],
                tags: vec!["auth".to_owned()],
                description: None,
            }],
        }
    }

    struct RecordingGitClient;

    impl GitClient for RecordingGitClient {
        fn clone_repo(
            &self,
            _path: &camino::Utf8Path,
            _url: &str,
        ) -> Result<(), crate::ports::GitError> {
            Ok(())
        }

        fn get_repo_status(
            &self,
            _path: &camino::Utf8Path,
        ) -> Result<RepoStatus, crate::ports::GitError> {
            Ok(RepoStatus {
                branch: Some("main".to_owned()),
                dirty: false,
                ahead: 0,
                behind: 0,
                clone_state: CloneState::Ready,
            })
        }

        fn fetch_repo(&self, _path: &camino::Utf8Path) -> Result<(), crate::ports::GitError> {
            Ok(())
        }

        fn pull_repo(&self, _path: &camino::Utf8Path) -> Result<(), crate::ports::GitError> {
            Ok(())
        }
    }

    #[test]
    fn set_task_repos_merges_group_and_explicit_repo_ids() {
        let tasks = InMemoryTaskRepository::default();
        tasks.save(&sample_task()).unwrap();
        let service = DefaultTaskReposApplicationService::new(
            Box::new(tasks),
            Box::new(InMemoryRepoCatalogRepository {
                catalog: RefCell::new(sample_catalog()),
            }),
            Box::new(FixedClock {
                now: OffsetDateTime::UNIX_EPOCH + time::Duration::hours(1),
            }),
            Box::new(RecordingGitClient),
        );

        let result = service
            .set_task_repos(SetTaskReposCommand {
                task_id: "auth-session-fix".to_owned(),
                selected_repo_groups: vec!["auth-core".to_owned()],
                repos: vec!["api-gateway".to_owned()],
            })
            .unwrap();

        assert_eq!(result.selected_repo_groups, vec!["auth-core".to_owned()]);
        assert_eq!(result.repos.len(), 2);
    }

    #[test]
    fn add_task_repos_rejects_unknown_repo() {
        let tasks = InMemoryTaskRepository::default();
        tasks.save(&sample_task()).unwrap();
        let service = DefaultTaskReposApplicationService::new(
            Box::new(tasks),
            Box::new(InMemoryRepoCatalogRepository {
                catalog: RefCell::new(sample_catalog()),
            }),
            Box::new(FixedClock {
                now: OffsetDateTime::UNIX_EPOCH,
            }),
            Box::new(RecordingGitClient),
        );

        let result = service.add_task_repos(AddTaskReposCommand {
            task_id: "auth-session-fix".to_owned(),
            repos: vec!["unknown-repo".to_owned()],
        });

        assert!(
            matches!(result, Err(ApplicationError::Domain(DomainError::NotFound { entity, .. })) if entity == EntityKind::Repo)
        );
    }

    #[test]
    fn clone_task_repos_dry_run_reports_targets_without_cloning() {
        let tasks = InMemoryTaskRepository::default();
        let mut task = sample_task();
        task.repos.repos = vec![RepoId::from("api-gateway")];
        tasks.save(&task).unwrap();
        let service = DefaultTaskReposApplicationService::new(
            Box::new(tasks),
            Box::new(InMemoryRepoCatalogRepository {
                catalog: RefCell::new(sample_catalog()),
            }),
            Box::new(FixedClock {
                now: OffsetDateTime::UNIX_EPOCH,
            }),
            Box::new(RecordingGitClient),
        );

        let result = service
            .clone_task_repos(CloneTaskReposCommand {
                task_id: "auth-session-fix".to_owned(),
                repos: None,
                missing_only: false,
                dry_run: true,
            })
            .unwrap();

        assert_eq!(result.len(), 1);
        assert!(result[0].dry_run);
    }

    #[test]
    fn repo_status_dry_run_reports_missing_clone_state() {
        let tasks = InMemoryTaskRepository::default();
        let mut task = sample_task();
        task.repos.repos = vec![RepoId::from("api-gateway")];
        tasks.save(&task).unwrap();
        let service = DefaultTaskReposApplicationService::new(
            Box::new(tasks),
            Box::new(InMemoryRepoCatalogRepository {
                catalog: RefCell::new(sample_catalog()),
            }),
            Box::new(FixedClock {
                now: OffsetDateTime::UNIX_EPOCH,
            }),
            Box::new(RecordingGitClient),
        );

        let result = service
            .get_repo_statuses(RepoStatusQuery {
                task_id: "auth-session-fix".to_owned(),
                repos: None,
                clone_state: Some(CloneStateFilter::Missing),
                dry_run: true,
            })
            .unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].status.clone_state, CloneState::Missing);
    }

    #[test]
    fn remove_task_repos_removes_repository() {
        let tasks = InMemoryTaskRepository::default();
        let mut task = sample_task();
        task.repos.repos = vec![RepoId::from("api-gateway"), RepoId::from("auth-service")];
        tasks.save(&task).unwrap();
        let service = DefaultTaskReposApplicationService::new(
            Box::new(tasks),
            Box::new(InMemoryRepoCatalogRepository {
                catalog: RefCell::new(sample_catalog()),
            }),
            Box::new(FixedClock {
                now: OffsetDateTime::UNIX_EPOCH,
            }),
            Box::new(RecordingGitClient),
        );

        let result = service
            .remove_task_repos(RemoveTaskReposCommand {
                task_id: "auth-session-fix".to_owned(),
                repos: vec!["api-gateway".to_owned()],
            })
            .unwrap();

        assert_eq!(result.repos.len(), 1);
        assert_eq!(result.repos[0], "auth-service");
    }

    #[test]
    fn clone_task_repos_real_execution_uses_git_client() {
        let tasks = InMemoryTaskRepository::default();
        let mut task = sample_task();
        task.repos.repos = vec![RepoId::from("api-gateway")];
        tasks.save(&task).unwrap();
        let service = DefaultTaskReposApplicationService::new(
            Box::new(tasks),
            Box::new(InMemoryRepoCatalogRepository {
                catalog: RefCell::new(sample_catalog()),
            }),
            Box::new(FixedClock {
                now: OffsetDateTime::UNIX_EPOCH,
            }),
            Box::new(RecordingGitClient),
        );

        let result = service
            .clone_task_repos(CloneTaskReposCommand {
                task_id: "auth-session-fix".to_owned(),
                repos: None,
                missing_only: false,
                dry_run: false,
            })
            .unwrap();

        assert_eq!(result.len(), 1);
        assert!(result[0].cloned);
        assert!(!result[0].dry_run);
    }

    #[test]
    fn get_repo_statuses_with_real_git() {
        let tasks = InMemoryTaskRepository::default();
        let mut task = sample_task();
        task.repos.repos = vec![RepoId::from("api-gateway")];
        tasks.save(&task).unwrap();
        let service = DefaultTaskReposApplicationService::new(
            Box::new(tasks),
            Box::new(InMemoryRepoCatalogRepository {
                catalog: RefCell::new(sample_catalog()),
            }),
            Box::new(FixedClock {
                now: OffsetDateTime::UNIX_EPOCH,
            }),
            Box::new(RecordingGitClient),
        );

        let result = service
            .get_repo_statuses(RepoStatusQuery {
                task_id: "auth-session-fix".to_owned(),
                repos: None,
                clone_state: None,
                dry_run: false,
            })
            .unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].status.clone_state, CloneState::Ready);
        assert_eq!(result[0].status.branch.as_deref(), Some("main"));
    }

    #[test]
    fn get_repo_statuses_filters_by_dirty_clone_state() {
        let tasks = InMemoryTaskRepository::default();
        let mut task = sample_task();
        task.repos.repos = vec![RepoId::from("api-gateway")];
        tasks.save(&task).unwrap();
        let service = DefaultTaskReposApplicationService::new(
            Box::new(tasks),
            Box::new(InMemoryRepoCatalogRepository {
                catalog: RefCell::new(sample_catalog()),
            }),
            Box::new(FixedClock {
                now: OffsetDateTime::UNIX_EPOCH,
            }),
            Box::new(RecordingGitClient),
        );

        let result = service
            .get_repo_statuses(RepoStatusQuery {
                task_id: "auth-session-fix".to_owned(),
                repos: None,
                clone_state: Some(CloneStateFilter::Dirty),
                dry_run: false,
            })
            .unwrap();

        assert!(result.is_empty());
    }

    #[test]
    fn set_task_repos_with_groups_only() {
        let tasks = InMemoryTaskRepository::default();
        let mut task = sample_task();
        task.repos.repos = vec![];
        task.repos.selected_repo_groups = vec![];
        tasks.save(&task).unwrap();
        let service = DefaultTaskReposApplicationService::new(
            Box::new(tasks),
            Box::new(InMemoryRepoCatalogRepository {
                catalog: RefCell::new(sample_catalog()),
            }),
            Box::new(FixedClock {
                now: OffsetDateTime::UNIX_EPOCH,
            }),
            Box::new(RecordingGitClient),
        );

        let result = service
            .set_task_repos(SetTaskReposCommand {
                task_id: "auth-session-fix".to_owned(),
                selected_repo_groups: vec!["auth-core".to_owned()],
                repos: vec![],
            })
            .unwrap();

        assert_eq!(result.repos.len(), 2);
    }

    #[test]
    fn clone_task_repos_missing_only_skips_existing() {
        let tasks = InMemoryTaskRepository::default();
        let mut task = sample_task();
        task.repos.repos = vec![RepoId::from("api-gateway")];
        tasks.save(&task).unwrap();
        let service = DefaultTaskReposApplicationService::new(
            Box::new(tasks),
            Box::new(InMemoryRepoCatalogRepository {
                catalog: RefCell::new(sample_catalog()),
            }),
            Box::new(FixedClock {
                now: OffsetDateTime::UNIX_EPOCH,
            }),
            Box::new(RecordingGitClient),
        );

        let result = service
            .clone_task_repos(CloneTaskReposCommand {
                task_id: "auth-session-fix".to_owned(),
                repos: None,
                missing_only: true,
                dry_run: false,
            })
            .unwrap();

        assert_eq!(result.len(), 1);
        assert!(result[0].cloned);
    }

    #[test]
    fn load_task_by_slug_fallback() {
        let tasks = InMemoryTaskRepository::default();
        tasks.save(&sample_task()).unwrap();
        let service = DefaultTaskReposApplicationService::new(
            Box::new(tasks),
            Box::new(InMemoryRepoCatalogRepository {
                catalog: RefCell::new(sample_catalog()),
            }),
            Box::new(FixedClock {
                now: OffsetDateTime::UNIX_EPOCH,
            }),
            Box::new(RecordingGitClient),
        );

        let result = service.set_task_repos(SetTaskReposCommand {
            task_id: "auth-session-fix".to_owned(),
            selected_repo_groups: vec![],
            repos: vec!["api-gateway".to_owned()],
        });
        assert!(result.is_ok());
    }

    #[test]
    fn get_repo_statuses_filters_by_ready_clone_state() {
        let tasks = InMemoryTaskRepository::default();
        let mut task = sample_task();
        task.repos.repos = vec![RepoId::from("api-gateway")];
        tasks.save(&task).unwrap();
        let service = DefaultTaskReposApplicationService::new(
            Box::new(tasks),
            Box::new(InMemoryRepoCatalogRepository {
                catalog: RefCell::new(sample_catalog()),
            }),
            Box::new(FixedClock {
                now: OffsetDateTime::UNIX_EPOCH,
            }),
            Box::new(RecordingGitClient),
        );

        let result = service
            .get_repo_statuses(RepoStatusQuery {
                task_id: "auth-session-fix".to_owned(),
                repos: None,
                clone_state: Some(CloneStateFilter::Ready),
                dry_run: false,
            })
            .unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].status.clone_state, CloneState::Ready);
    }

    #[test]
    fn resolve_group_repos_fails_for_unknown_group() {
        let tasks = InMemoryTaskRepository::default();
        tasks.save(&sample_task()).unwrap();
        let service = DefaultTaskReposApplicationService::new(
            Box::new(tasks),
            Box::new(InMemoryRepoCatalogRepository {
                catalog: RefCell::new(sample_catalog()),
            }),
            Box::new(FixedClock {
                now: OffsetDateTime::UNIX_EPOCH,
            }),
            Box::new(RecordingGitClient),
        );

        let result = service.set_task_repos(SetTaskReposCommand {
            task_id: "auth-session-fix".to_owned(),
            selected_repo_groups: vec!["nonexistent".to_owned()],
            repos: vec![],
        });
        assert!(
            matches!(result, Err(ApplicationError::Domain(DomainError::NotFound { entity, .. })) if entity == EntityKind::RepoGroup)
        );
    }
}
