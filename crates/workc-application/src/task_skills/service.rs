use std::collections::BTreeSet;

use camino::Utf8PathBuf;
use workc_domain::errors::DomainError;
use workc_domain::shared::{MountId, SkillId, SkillVersion, TaskId};
use workc_domain::skill_registry::{SkillRegistryRepository};
use workc_domain::task::{TaskRepository, TaskSkillMount, TaskSkillMountRepository, TaskSkillMountStatus};

use crate::error::ApplicationError;
use crate::ports::{Clock, PrepareStatusRecord, SkillRuntime};

use super::dtos::{
    CheckSkillUpdatesQuery, MountSkillCommand, OverrideSkillCommand, PrepareSkillCommand, PrepareStatusQuery, SandboxSkillCommand,
    SkillMountSummary, SkillPreparation, SkillSandboxHandle, SkillUpdateStatus, SkillUseExecution, UnmountSkillCommand,
    UpdateSkillCommand, UseSkillCommand,
};

pub trait TaskSkillsApplicationService {
    fn mount_skill(&self, command: MountSkillCommand) -> Result<SkillMountSummary, ApplicationError>;
    fn list_mounts(&self, task_id: &workc_domain::shared::TaskId) -> Result<Vec<SkillMountSummary>, ApplicationError>;
    fn unmount_skill(&self, command: UnmountSkillCommand) -> Result<(), ApplicationError>;
    fn override_skill(&self, command: OverrideSkillCommand) -> Result<(), ApplicationError>;
    fn check_skill_updates(&self, query: CheckSkillUpdatesQuery) -> Result<Vec<SkillUpdateStatus>, ApplicationError>;
    fn update_skill(&self, command: UpdateSkillCommand) -> Result<SkillMountSummary, ApplicationError>;
    fn sandbox_skill(&self, command: SandboxSkillCommand) -> Result<SkillSandboxHandle, ApplicationError>;
    fn prepare_skill(&self, command: PrepareSkillCommand) -> Result<SkillPreparation, ApplicationError>;
    fn use_skill(&self, command: UseSkillCommand) -> Result<SkillUseExecution, ApplicationError>;
    fn get_prepare_status(&self, query: PrepareStatusQuery) -> Result<PrepareStatusRecord, ApplicationError>;
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

    fn load_task(&self, task_ref: &str) -> Result<workc_domain::task::TaskWorkspace, ApplicationError> {
        let task = if task_ref.starts_with("task-") {
            self.tasks.find_by_id(&TaskId::from(task_ref))?
        } else {
            self.tasks.find_by_slug(&workc_domain::shared::TaskSlug::from(task_ref))?
        };
        task.ok_or_else(|| ApplicationError::Domain(DomainError::NotFound {
            entity: "task",
            id: task_ref.to_owned(),
        }))
    }

    fn mount_path(task_id: &TaskId, mount_id: &MountId) -> Utf8PathBuf {
        Utf8PathBuf::from("tasks")
            .join(task_id.as_str())
            .join(".codex")
            .join("skills")
            .join("mounted")
            .join(mount_id.as_str())
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
    fn mount_skill(&self, command: MountSkillCommand) -> Result<SkillMountSummary, ApplicationError> {
        let task = self.load_task(&command.task_id)?;
        let skill_id = SkillId::from(command.skill_id.as_str());
        let skill = self
            .registry
            .find_skill(&skill_id)?
            .ok_or_else(|| ApplicationError::Domain(DomainError::NotFound {
                entity: "skill",
                id: skill_id.to_string(),
            }))?;
        let version = command
            .version
            .map(|value| SkillVersion::from(value.as_str()))
            .or_else(|| skill.latest.clone())
            .ok_or_else(|| ApplicationError::InvalidRequest("skill version is required".to_owned()))?;

        let mut mounts = self.mounts.list_for_task(&task.meta.id)?;
        let mount_id = MountId::from(format!("mount-{:03}", mounts.len() + 1));
        let mount = TaskSkillMount {
            id: mount_id.clone(),
            skill_id,
            version,
            source: skill.source,
            mounted_at: self.clock.now(),
            status: TaskSkillMountStatus::Active,
            path: Self::mount_path(&task.meta.id, &mount_id),
        };
        mounts.push(mount.clone());
        self.mounts.save_for_task(&task.meta.id, &mounts)?;
        Ok(Self::to_summary(&task.meta.id, mount))
    }

    fn list_mounts(&self, task_id: &workc_domain::shared::TaskId) -> Result<Vec<SkillMountSummary>, ApplicationError> {
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

    fn check_skill_updates(&self, query: CheckSkillUpdatesQuery) -> Result<Vec<SkillUpdateStatus>, ApplicationError> {
        let task_id = TaskId::from(query.task_id.as_str());
        let mounts = self.mounts.list_for_task(&task_id)?;
        let selected_mounts: Option<BTreeSet<String>> = query.mount_id.map(|value| [value].into_iter().collect());
        let mut result = Vec::new();

        for mount in mounts {
            if let Some(selected) = &selected_mounts {
                if !selected.contains(mount.id.as_str()) {
                    continue;
                }
            }
            let skill = self
                .registry
                .find_skill(&mount.skill_id)?
                .ok_or_else(|| ApplicationError::Domain(DomainError::NotFound {
                    entity: "skill",
                    id: mount.skill_id.to_string(),
                }))?;
            let update_available = skill.latest.as_ref().is_some_and(|latest| latest != &mount.version);
            result.push(SkillUpdateStatus {
                mount_id: mount.id.to_string(),
                update_available,
                target_version: skill.latest.map(|value| value.to_string()),
            });
        }

        Ok(result)
    }

    fn update_skill(&self, command: UpdateSkillCommand) -> Result<SkillMountSummary, ApplicationError> {
        let task_id = TaskId::from(command.task_id.as_str());
        let mount_id = MountId::from(command.mount_id.as_str());
        let mut mounts = self.mounts.list_for_task(&task_id)?;
        let mount_index = mounts
            .iter()
            .position(|mount| mount.id == mount_id)
            .ok_or_else(|| ApplicationError::Domain(DomainError::NotFound {
                entity: "mount",
                id: mount_id.to_string(),
            }))?;
        let skill_id = mounts[mount_index].skill_id.clone();
        let skill = self
            .registry
            .find_skill(&skill_id)?
            .ok_or_else(|| ApplicationError::Domain(DomainError::NotFound {
                entity: "skill",
                id: skill_id.to_string(),
            }))?;
        mounts[mount_index].version = skill
            .latest
            .clone()
            .ok_or_else(|| ApplicationError::InvalidRequest("no latest version available".to_owned()))?;
        let updated_mount = mounts[mount_index].clone();
        self.mounts.save_for_task(&task_id, &mounts)?;
        Ok(Self::to_summary(&task_id, updated_mount))
    }

    fn sandbox_skill(&self, command: SandboxSkillCommand) -> Result<SkillSandboxHandle, ApplicationError> {
        let task_id = TaskId::from(command.task_id.as_str());
        let mount_id = MountId::from(command.mount_id.as_str());
        Ok(SkillSandboxHandle {
            mount_id: mount_id.to_string(),
            path: Self::mount_path(&task_id, &mount_id),
        })
    }

    fn prepare_skill(&self, command: PrepareSkillCommand) -> Result<SkillPreparation, ApplicationError> {
        let task_id = TaskId::from(command.task_id.as_str());
        let mount_id = MountId::from(command.mount_id.as_str());
        let path = Self::mount_path(&task_id, &mount_id);
        let runtime = self.runtime.as_ref().ok_or(ApplicationError::AdapterUnavailable("skill prepare"))?;
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
        let runtime = self.runtime.as_ref().ok_or(ApplicationError::AdapterUnavailable("skill use"))?;
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

    fn get_prepare_status(&self, query: PrepareStatusQuery) -> Result<PrepareStatusRecord, ApplicationError> {
        let task_id = TaskId::from(query.task_id.as_str());
        let mount_id = MountId::from(query.mount_id.as_str());
        let path = Self::mount_path(&task_id, &mount_id);
        let runtime = self.runtime.as_ref().ok_or(ApplicationError::AdapterUnavailable("skill prepare-status"))?;
        runtime
            .check_prepare_status(path.as_path())
            .map_err(|error| ApplicationError::ExternalFailure {
                port: "skill-runtime",
                detail: error.to_string(),
            })
    }
}
