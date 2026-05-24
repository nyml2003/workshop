use std::collections::BTreeSet;

use camino::Utf8PathBuf;
use workc_domain::errors::DomainError;
use workc_domain::shared::{MountId, SkillId, SkillVersion, TaskId};
use workc_domain::skill_registry::SkillRegistryRepository;
use workc_domain::task::{
    TaskRepository, TaskSkillMount, TaskSkillMountRepository, TaskSkillMountStatus,
};

use crate::error::ApplicationError;
use crate::ports::{Clock, PrepareStatusRecord, SkillRuntime};

use super::dtos::{
    CheckSkillUpdatesQuery, MountSkillCommand, OverrideSkillCommand, PrepareSkillCommand,
    PrepareStatusQuery, SandboxSkillCommand, SkillMountSummary, SkillPreparation,
    SkillSandboxHandle, SkillUpdateStatus, SkillUseExecution, UnmountSkillCommand,
    UpdateSkillCommand, UseSkillCommand,
};

pub trait TaskSkillsApplicationService {
    fn mount_skill(
        &self,
        command: MountSkillCommand,
    ) -> Result<SkillMountSummary, ApplicationError>;
    fn list_mounts(
        &self,
        task_id: &workc_domain::shared::TaskId,
    ) -> Result<Vec<SkillMountSummary>, ApplicationError>;
    fn unmount_skill(&self, command: UnmountSkillCommand) -> Result<(), ApplicationError>;
    fn override_skill(&self, command: OverrideSkillCommand) -> Result<(), ApplicationError>;
    fn check_skill_updates(
        &self,
        query: CheckSkillUpdatesQuery,
    ) -> Result<Vec<SkillUpdateStatus>, ApplicationError>;
    fn update_skill(
        &self,
        command: UpdateSkillCommand,
    ) -> Result<SkillMountSummary, ApplicationError>;
    fn sandbox_skill(
        &self,
        command: SandboxSkillCommand,
    ) -> Result<SkillSandboxHandle, ApplicationError>;
    fn prepare_skill(
        &self,
        command: PrepareSkillCommand,
    ) -> Result<SkillPreparation, ApplicationError>;
    fn use_skill(&self, command: UseSkillCommand) -> Result<SkillUseExecution, ApplicationError>;
    fn get_prepare_status(
        &self,
        query: PrepareStatusQuery,
    ) -> Result<PrepareStatusRecord, ApplicationError>;
}

pub struct DefaultTaskSkillsApplicationService {
    tasks: Box<dyn TaskRepository>,
    mounts: Box<dyn TaskSkillMountRepository>,
    registry: Box<dyn SkillRegistryRepository>,
    clock: Box<dyn Clock>,
    runtime: Option<Box<dyn SkillRuntime>>,
}

impl DefaultTaskSkillsApplicationService {
    pub fn new(
        tasks: Box<dyn TaskRepository>,
        mounts: Box<dyn TaskSkillMountRepository>,
        registry: Box<dyn SkillRegistryRepository>,
        clock: Box<dyn Clock>,
        runtime: Option<Box<dyn SkillRuntime>>,
    ) -> Self {
        Self {
            tasks,
            mounts,
            registry,
            clock,
            runtime,
        }
    }

    fn load_task(
        &self,
        task_ref: &str,
    ) -> Result<workc_domain::task::TaskWorkspace, ApplicationError> {
        let task = if task_ref.starts_with("task-") {
            self.tasks.find_by_id(&TaskId::from(task_ref))?
        } else {
            self.tasks
                .find_by_slug(&workc_domain::shared::TaskSlug::from(task_ref))?
        };
        task.ok_or_else(|| {
            ApplicationError::Domain(DomainError::NotFound {
                entity: "task",
                id: task_ref.to_owned(),
            })
        })
    }

    fn mount_path(_task_id: &TaskId, mount_id: &MountId) -> Utf8PathBuf {
        Utf8PathBuf::from("skills")
            .join("mounted")
            .join(mount_id.as_str())
    }

    fn mount_skill_path(skill_id: &SkillId) -> Utf8PathBuf {
        Utf8PathBuf::from(".opencode")
            .join("skills")
            .join(skill_id.as_str())
    }

    fn to_summary(task_id: &TaskId, mount: TaskSkillMount) -> SkillMountSummary {
        SkillMountSummary {
            task_id: task_id.to_string(),
            mount_id: mount.id.to_string(),
            skill_id: mount.skill_id.to_string(),
            version: mount.version.to_string(),
            source: mount.source.to_string(),
            status: match mount.status {
                TaskSkillMountStatus::Active => "active",
                TaskSkillMountStatus::Inactive => "inactive",
                TaskSkillMountStatus::Removed => "removed",
            }
            .to_owned(),
            path: mount.path,
        }
    }
}

impl TaskSkillsApplicationService for DefaultTaskSkillsApplicationService {
    fn mount_skill(
        &self,
        command: MountSkillCommand,
    ) -> Result<SkillMountSummary, ApplicationError> {
        let task = self.load_task(&command.task_id)?;
        let skill_id = SkillId::from(command.skill_id.as_str());
        let skill = self.registry.find_skill(&skill_id)?.ok_or_else(|| {
            ApplicationError::Domain(DomainError::NotFound {
                entity: "skill",
                id: skill_id.to_string(),
            })
        })?;
        let version = command
            .version
            .map(|value| SkillVersion::from(value.as_str()))
            .or_else(|| skill.latest.clone())
            .ok_or_else(|| {
                ApplicationError::InvalidRequest("skill version is required".to_owned())
            })?;

        let mut mounts = self.mounts.list_for_task(&task.meta.id)?;
        let mount_id = MountId::from(format!("mount-{:03}", mounts.len() + 1));
        let mount = TaskSkillMount {
            id: mount_id.clone(),
            skill_id: skill_id.clone(),
            version,
            source: skill.source,
            mounted_at: self.clock.now(),
            status: TaskSkillMountStatus::Active,
            path: Self::mount_skill_path(&skill_id),
        };
        mounts.push(mount.clone());
        self.mounts.save_for_task(&task.meta.id, &mounts)?;
        Ok(Self::to_summary(&task.meta.id, mount))
    }

    fn list_mounts(
        &self,
        task_id: &workc_domain::shared::TaskId,
    ) -> Result<Vec<SkillMountSummary>, ApplicationError> {
        Ok(self
            .mounts
            .list_for_task(task_id)?
            .into_iter()
            .map(|mount| Self::to_summary(task_id, mount))
            .collect())
    }

    fn unmount_skill(&self, command: UnmountSkillCommand) -> Result<(), ApplicationError> {
        self.mounts.remove_for_task(
            &TaskId::from(command.task_id.as_str()),
            &MountId::from(command.mount_id.as_str()),
        )?;
        Ok(())
    }

    fn override_skill(&self, _command: OverrideSkillCommand) -> Result<(), ApplicationError> {
        Err(ApplicationError::AdapterUnavailable("skill override"))
    }

    fn check_skill_updates(
        &self,
        query: CheckSkillUpdatesQuery,
    ) -> Result<Vec<SkillUpdateStatus>, ApplicationError> {
        let task_id = TaskId::from(query.task_id.as_str());
        let mounts = self.mounts.list_for_task(&task_id)?;
        let selected_mounts: Option<BTreeSet<String>> =
            query.mount_id.map(|value| [value].into_iter().collect());
        let mut result = Vec::new();

        for mount in mounts {
            if let Some(selected) = &selected_mounts {
                if !selected.contains(mount.id.as_str()) {
                    continue;
                }
            }
            let skill = self.registry.find_skill(&mount.skill_id)?.ok_or_else(|| {
                ApplicationError::Domain(DomainError::NotFound {
                    entity: "skill",
                    id: mount.skill_id.to_string(),
                })
            })?;
            let update_available = skill
                .latest
                .as_ref()
                .is_some_and(|latest| latest != &mount.version);
            result.push(SkillUpdateStatus {
                mount_id: mount.id.to_string(),
                update_available,
                target_version: skill.latest.map(|value| value.to_string()),
            });
        }

        Ok(result)
    }

    fn update_skill(
        &self,
        command: UpdateSkillCommand,
    ) -> Result<SkillMountSummary, ApplicationError> {
        let task_id = TaskId::from(command.task_id.as_str());
        let mount_id = MountId::from(command.mount_id.as_str());
        let mut mounts = self.mounts.list_for_task(&task_id)?;
        let mount_index = mounts
            .iter()
            .position(|mount| mount.id == mount_id)
            .ok_or_else(|| {
                ApplicationError::Domain(DomainError::NotFound {
                    entity: "mount",
                    id: mount_id.to_string(),
                })
            })?;
        let skill_id = mounts[mount_index].skill_id.clone();
        let skill = self.registry.find_skill(&skill_id)?.ok_or_else(|| {
            ApplicationError::Domain(DomainError::NotFound {
                entity: "skill",
                id: skill_id.to_string(),
            })
        })?;
        mounts[mount_index].version = skill.latest.clone().ok_or_else(|| {
            ApplicationError::InvalidRequest("no latest version available".to_owned())
        })?;
        let updated_mount = mounts[mount_index].clone();
        self.mounts.save_for_task(&task_id, &mounts)?;
        Ok(Self::to_summary(&task_id, updated_mount))
    }

    fn sandbox_skill(
        &self,
        command: SandboxSkillCommand,
    ) -> Result<SkillSandboxHandle, ApplicationError> {
        let task_id = TaskId::from(command.task_id.as_str());
        let mount_id = MountId::from(command.mount_id.as_str());
        Ok(SkillSandboxHandle {
            mount_id: mount_id.to_string(),
            path: Self::mount_path(&task_id, &mount_id),
        })
    }

    fn prepare_skill(
        &self,
        command: PrepareSkillCommand,
    ) -> Result<SkillPreparation, ApplicationError> {
        let task_id = TaskId::from(command.task_id.as_str());
        let mount_id = MountId::from(command.mount_id.as_str());
        let path = Self::mount_path(&task_id, &mount_id);
        let runtime = self
            .runtime
            .as_ref()
            .ok_or(ApplicationError::AdapterUnavailable("skill prepare"))?;
        let result = runtime
            .prepare(
                path.as_path(),
                workc_domain::skill_registry::PrepareStep {
                    name: command.step.name,
                    action_id: command.step.action_id,
                },
            )
            .map_err(|error| ApplicationError::ExternalFailure {
                port: "skill-runtime",
                detail: error.to_string(),
            })?;
        Ok(SkillPreparation {
            mount_id: mount_id.to_string(),
            status: format!("{:?}", result.status),
            artifact_path: result.artifact_path,
            log_path: result.log_path,
        })
    }

    fn use_skill(&self, command: UseSkillCommand) -> Result<SkillUseExecution, ApplicationError> {
        let task_id = TaskId::from(command.task_id.as_str());
        let mount_id = MountId::from(command.mount_id.as_str());
        let path = Self::mount_path(&task_id, &mount_id);
        let runtime = self
            .runtime
            .as_ref()
            .ok_or(ApplicationError::AdapterUnavailable("skill use"))?;
        let result = runtime
            .use_skill(
                path.as_path(),
                workc_domain::skill_registry::UseStep {
                    name: command.step.name,
                    action_id: command.step.action_id,
                },
            )
            .map_err(|error| ApplicationError::ExternalFailure {
                port: "skill-runtime",
                detail: error.to_string(),
            })?;
        Ok(SkillUseExecution {
            mount_id: mount_id.to_string(),
            status: format!("{:?}", result.status),
            log_path: result.log_path,
        })
    }

    fn get_prepare_status(
        &self,
        query: PrepareStatusQuery,
    ) -> Result<PrepareStatusRecord, ApplicationError> {
        let task_id = TaskId::from(query.task_id.as_str());
        let mount_id = MountId::from(query.mount_id.as_str());
        let path = Self::mount_path(&task_id, &mount_id);
        let runtime = self
            .runtime
            .as_ref()
            .ok_or(ApplicationError::AdapterUnavailable("skill prepare-status"))?;
        runtime
            .check_prepare_status(path.as_path())
            .map_err(|error| ApplicationError::ExternalFailure {
                port: "skill-runtime",
                detail: error.to_string(),
            })
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::collections::BTreeMap;

    use camino::Utf8Path;
    use time::OffsetDateTime;
    use workc_domain::errors::DomainError;
    use workc_domain::shared::{MountId, SkillId, SkillSourceId, SkillVersion, TaskId, TaskSlug};
    use workc_domain::skill_registry::{
        PrepareResult, PrepareStep, SkillDefinition, SkillExecutionStatus, SkillRegistry,
        SkillRegistryRepository, SkillSource, UseResult, UseStep,
    };
    use workc_domain::task::{
        TaskActivity, TaskMeta, TaskPaths, TaskRepoSelection, TaskRepository, TaskSkillMount,
        TaskSkillMountRepository, TaskStatus, TaskWorkspace,
    };

    use crate::ports::{Clock, PrepareStatusRecord, RuntimeError, SkillRuntime};

    use super::*;

    #[derive(Default)]
    struct InMemoryTaskRepository {
        tasks: RefCell<BTreeMap<String, TaskWorkspace>>,
    }

    impl TaskRepository for InMemoryTaskRepository {
        fn find_by_id(&self, id: &TaskId) -> Result<Option<TaskWorkspace>, DomainError> {
            Ok(self
                .tasks
                .borrow()
                .values()
                .find(|t| t.meta.id == *id)
                .cloned())
        }

        fn find_by_slug(&self, slug: &TaskSlug) -> Result<Option<TaskWorkspace>, DomainError> {
            Ok(self
                .tasks
                .borrow()
                .values()
                .find(|t| t.meta.slug == *slug)
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

    #[derive(Default)]
    struct InMemorySkillRegistryRepository {
        registry: RefCell<SkillRegistry>,
    }

    impl SkillRegistryRepository for InMemorySkillRegistryRepository {
        fn load(&self) -> Result<SkillRegistry, DomainError> {
            Ok(self.registry.borrow().clone())
        }

        fn save(&self, registry: &SkillRegistry) -> Result<(), DomainError> {
            *self.registry.borrow_mut() = registry.clone();
            Ok(())
        }

        fn find_source(&self, id: &SkillSourceId) -> Result<Option<SkillSource>, DomainError> {
            Ok(self
                .registry
                .borrow()
                .sources
                .iter()
                .find(|s| s.id == *id)
                .cloned())
        }

        fn find_skill(&self, id: &SkillId) -> Result<Option<SkillDefinition>, DomainError> {
            Ok(self
                .registry
                .borrow()
                .skills
                .iter()
                .find(|s| s.id == *id)
                .cloned())
        }
    }

    #[derive(Default)]
    struct InMemoryTaskSkillMountRepository {
        mounts: RefCell<BTreeMap<String, Vec<TaskSkillMount>>>,
    }

    impl TaskSkillMountRepository for InMemoryTaskSkillMountRepository {
        fn list_for_task(&self, task_id: &TaskId) -> Result<Vec<TaskSkillMount>, DomainError> {
            Ok(self
                .mounts
                .borrow()
                .get(task_id.as_str())
                .cloned()
                .unwrap_or_default())
        }

        fn save_for_task(
            &self,
            task_id: &TaskId,
            mounts: &[TaskSkillMount],
        ) -> Result<(), DomainError> {
            self.mounts
                .borrow_mut()
                .insert(task_id.to_string(), mounts.to_vec());
            Ok(())
        }

        fn remove_for_task(&self, task_id: &TaskId, mount_id: &MountId) -> Result<(), DomainError> {
            if let Some(mounts) = self.mounts.borrow_mut().get_mut(task_id.as_str()) {
                mounts.retain(|m| m.id != *mount_id);
            }
            Ok(())
        }
    }

    struct FixedClock;

    impl Clock for FixedClock {
        fn now(&self) -> OffsetDateTime {
            OffsetDateTime::UNIX_EPOCH
        }
    }

    struct StubSkillRuntime;

    impl SkillRuntime for StubSkillRuntime {
        fn prepare(
            &self,
            _mount_path: &Utf8Path,
            _step: PrepareStep,
        ) -> Result<PrepareResult, RuntimeError> {
            Ok(PrepareResult {
                status: SkillExecutionStatus::Success,
                artifact_path: Some("artifact".into()),
                log_path: Some("log".into()),
                finished_at: Some(OffsetDateTime::UNIX_EPOCH),
            })
        }

        fn use_skill(
            &self,
            _mount_path: &Utf8Path,
            _step: UseStep,
        ) -> Result<UseResult, RuntimeError> {
            Ok(UseResult {
                status: SkillExecutionStatus::Success,
                log_path: Some("log".into()),
                finished_at: Some(OffsetDateTime::UNIX_EPOCH),
            })
        }

        fn check_prepare_status(
            &self,
            _mount_path: &Utf8Path,
        ) -> Result<PrepareStatusRecord, RuntimeError> {
            Ok(PrepareStatusRecord {
                status: SkillExecutionStatus::Success,
                last_run_at: Some(OffsetDateTime::UNIX_EPOCH),
                artifact_path: Some("artifact".into()),
                log_path: Some("log".into()),
            })
        }
    }

    fn sample_task() -> TaskWorkspace {
        TaskWorkspace {
            meta: TaskMeta {
                id: TaskId::from("task-20260524-auth"),
                slug: TaskSlug::from("auth-fix"),
                title: "Auth Fix".to_owned(),
                template: "default".to_owned(),
                status: TaskStatus::Active,
                description: None,
                source_brief: None,
                tags: vec![],
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

    fn service_with_runtime() -> DefaultTaskSkillsApplicationService {
        let tasks = InMemoryTaskRepository::default();
        tasks.save(&sample_task()).unwrap();
        let registry = InMemorySkillRegistryRepository::default();
        registry.registry.borrow_mut().skills.push(SkillDefinition {
            id: SkillId::from("frontend-testing"),
            source: SkillSourceId::from("frontend-toolkit"),
            versions: vec![SkillVersion::from("2026-05-22")],
            latest: Some(SkillVersion::from("2026-05-22")),
        });
        DefaultTaskSkillsApplicationService::new(
            Box::new(tasks),
            Box::new(InMemoryTaskSkillMountRepository::default()),
            Box::new(registry),
            Box::new(FixedClock),
            Some(Box::new(StubSkillRuntime)),
        )
    }

    fn service_without_runtime() -> DefaultTaskSkillsApplicationService {
        let tasks = InMemoryTaskRepository::default();
        tasks.save(&sample_task()).unwrap();
        let registry = InMemorySkillRegistryRepository::default();
        registry.registry.borrow_mut().skills.push(SkillDefinition {
            id: SkillId::from("frontend-testing"),
            source: SkillSourceId::from("frontend-toolkit"),
            versions: vec![SkillVersion::from("2026-05-22")],
            latest: Some(SkillVersion::from("2026-05-22")),
        });
        DefaultTaskSkillsApplicationService::new(
            Box::new(tasks),
            Box::new(InMemoryTaskSkillMountRepository::default()),
            Box::new(registry),
            Box::new(FixedClock),
            None,
        )
    }

    #[test]
    fn mount_skill_creates_mount_with_latest_version() {
        let svc = service_without_runtime();
        let summary = svc
            .mount_skill(MountSkillCommand {
                task_id: "task-20260524-auth".to_owned(),
                skill_id: "frontend-testing".to_owned(),
                version: None,
            })
            .unwrap();

        assert_eq!(summary.skill_id, "frontend-testing");
        assert_eq!(summary.version, "2026-05-22");
        assert_eq!(summary.status, "active");
        assert!(summary.path.as_str().contains("frontend-testing"));
    }

    #[test]
    fn mount_skill_fails_for_missing_skill() {
        let svc = service_without_runtime();
        let result = svc.mount_skill(MountSkillCommand {
            task_id: "task-20260524-auth".to_owned(),
            skill_id: "nonexistent".to_owned(),
            version: None,
        });
        assert!(result.is_err());
    }

    #[test]
    fn mount_skill_fails_when_no_version_and_no_latest() {
        let tasks = InMemoryTaskRepository::default();
        tasks.save(&sample_task()).unwrap();
        let registry = InMemorySkillRegistryRepository::default();
        registry.registry.borrow_mut().skills.push(SkillDefinition {
            id: SkillId::from("frontend-testing"),
            source: SkillSourceId::from("frontend-toolkit"),
            versions: vec![],
            latest: None,
        });
        let svc = DefaultTaskSkillsApplicationService::new(
            Box::new(tasks),
            Box::new(InMemoryTaskSkillMountRepository::default()),
            Box::new(registry),
            Box::new(FixedClock),
            None,
        );

        let result = svc.mount_skill(MountSkillCommand {
            task_id: "task-20260524-auth".to_owned(),
            skill_id: "frontend-testing".to_owned(),
            version: None,
        });
        assert!(result.is_err());
    }

    #[test]
    fn list_mounts_returns_empty_initially() {
        let svc = service_without_runtime();
        let mounts = svc
            .list_mounts(&TaskId::from("task-20260524-auth"))
            .unwrap();
        assert!(mounts.is_empty());
    }

    #[test]
    fn unmount_skill_removes_mount() {
        let svc = service_without_runtime();
        let summary = svc
            .mount_skill(MountSkillCommand {
                task_id: "task-20260524-auth".to_owned(),
                skill_id: "frontend-testing".to_owned(),
                version: None,
            })
            .unwrap();

        svc.unmount_skill(UnmountSkillCommand {
            task_id: "task-20260524-auth".to_owned(),
            mount_id: summary.mount_id.clone(),
        })
        .unwrap();

        let mounts = svc
            .list_mounts(&TaskId::from("task-20260524-auth"))
            .unwrap();
        assert!(mounts.is_empty());
    }

    #[test]
    fn override_skill_returns_adapter_unavailable() {
        let svc = service_without_runtime();
        let result = svc.override_skill(OverrideSkillCommand {
            task_id: "task-x".to_owned(),
            mount_id: "m1".to_owned(),
            relative_path: "override".into(),
        });
        assert!(matches!(
            result,
            Err(ApplicationError::AdapterUnavailable("skill override"))
        ));
    }

    #[test]
    fn check_skill_updates_detects_update_available() {
        let svc = service_without_runtime();
        svc.mount_skill(MountSkillCommand {
            task_id: "task-20260524-auth".to_owned(),
            skill_id: "frontend-testing".to_owned(),
            version: Some("2026-05-20".to_owned()),
        })
        .unwrap();

        let updates = svc
            .check_skill_updates(CheckSkillUpdatesQuery {
                task_id: "task-20260524-auth".to_owned(),
                mount_id: None,
            })
            .unwrap();
        assert_eq!(updates.len(), 1);
        assert!(updates[0].update_available);
        assert_eq!(updates[0].target_version.as_deref(), Some("2026-05-22"));
    }

    #[test]
    fn check_skill_updates_no_update_when_already_latest() {
        let svc = service_without_runtime();
        svc.mount_skill(MountSkillCommand {
            task_id: "task-20260524-auth".to_owned(),
            skill_id: "frontend-testing".to_owned(),
            version: None,
        })
        .unwrap();

        let updates = svc
            .check_skill_updates(CheckSkillUpdatesQuery {
                task_id: "task-20260524-auth".to_owned(),
                mount_id: None,
            })
            .unwrap();
        assert_eq!(updates.len(), 1);
        assert!(!updates[0].update_available);
    }

    #[test]
    fn update_skill_changes_version_to_latest() {
        let svc = service_without_runtime();
        svc.mount_skill(MountSkillCommand {
            task_id: "task-20260524-auth".to_owned(),
            skill_id: "frontend-testing".to_owned(),
            version: Some("2026-05-20".to_owned()),
        })
        .unwrap();

        let updated = svc
            .update_skill(UpdateSkillCommand {
                task_id: "task-20260524-auth".to_owned(),
                mount_id: "mount-001".to_owned(),
            })
            .unwrap();

        assert_eq!(updated.version, "2026-05-22");
    }

    #[test]
    fn update_skill_fails_for_missing_mount() {
        let svc = service_without_runtime();
        let result = svc.update_skill(UpdateSkillCommand {
            task_id: "task-20260524-auth".to_owned(),
            mount_id: "mount-999".to_owned(),
        });
        assert!(result.is_err());
    }

    #[test]
    fn sandbox_skill_returns_handle() {
        let svc = service_without_runtime();
        let handle = svc
            .sandbox_skill(SandboxSkillCommand {
                task_id: "task-20260524-auth".to_owned(),
                mount_id: "mount-001".to_owned(),
            })
            .unwrap();

        assert_eq!(handle.mount_id, "mount-001");
        assert!(handle.path.as_str().contains("mount-001"));
    }

    #[test]
    fn prepare_skill_with_runtime() {
        let svc = service_with_runtime();
        let result = svc
            .prepare_skill(PrepareSkillCommand {
                task_id: "task-20260524-auth".to_owned(),
                mount_id: "mount-001".to_owned(),
                step: super::super::dtos::RuntimeStep {
                    name: "install".to_owned(),
                    action_id: "npm-install".to_owned(),
                },
            })
            .unwrap();

        assert!(result.status.contains("Success"));
        assert!(result.artifact_path.is_some());
        assert!(result.log_path.is_some());
    }

    #[test]
    fn prepare_skill_fails_without_runtime() {
        let svc = service_without_runtime();
        let result = svc.prepare_skill(PrepareSkillCommand {
            task_id: "task-20260524-auth".to_owned(),
            mount_id: "mount-001".to_owned(),
            step: super::super::dtos::RuntimeStep {
                name: "install".to_owned(),
                action_id: "npm-install".to_owned(),
            },
        });
        assert!(matches!(
            result,
            Err(ApplicationError::AdapterUnavailable(..))
        ));
    }

    #[test]
    fn use_skill_with_runtime() {
        let svc = service_with_runtime();
        let result = svc
            .use_skill(UseSkillCommand {
                task_id: "task-20260524-auth".to_owned(),
                mount_id: "mount-001".to_owned(),
                step: super::super::dtos::RuntimeStep {
                    name: "lint".to_owned(),
                    action_id: "eslint".to_owned(),
                },
            })
            .unwrap();

        assert!(result.status.contains("Success"));
        assert!(result.log_path.is_some());
    }

    #[test]
    fn use_skill_fails_without_runtime() {
        let svc = service_without_runtime();
        let result = svc.use_skill(UseSkillCommand {
            task_id: "task-20260524-auth".to_owned(),
            mount_id: "mount-001".to_owned(),
            step: super::super::dtos::RuntimeStep {
                name: "lint".to_owned(),
                action_id: "eslint".to_owned(),
            },
        });
        assert!(matches!(
            result,
            Err(ApplicationError::AdapterUnavailable(..))
        ));
    }

    #[test]
    fn get_prepare_status_with_runtime() {
        let svc = service_with_runtime();
        let result = svc
            .get_prepare_status(PrepareStatusQuery {
                task_id: "task-20260524-auth".to_owned(),
                mount_id: "mount-001".to_owned(),
            })
            .unwrap();

        assert_eq!(result.status, SkillExecutionStatus::Success);
        assert!(result.artifact_path.is_some());
    }

    #[test]
    fn get_prepare_status_fails_without_runtime() {
        let svc = service_without_runtime();
        let result = svc.get_prepare_status(PrepareStatusQuery {
            task_id: "task-20260524-auth".to_owned(),
            mount_id: "mount-001".to_owned(),
        });
        assert!(matches!(
            result,
            Err(ApplicationError::AdapterUnavailable(..))
        ));
    }
}
