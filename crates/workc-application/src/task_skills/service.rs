use std::collections::BTreeSet;

use camino::Utf8PathBuf;

use workc_domain::errors::{DomainError, EntityKind};
use workc_domain::shared::{MountId, SkillId, SkillVersion, TaskSlug};
use workc_domain::skill_registry::SkillRegistryRepository;
use workc_domain::task::{
    TaskRepository, TaskSkillMount, TaskSkillMountRepository, TaskSkillMountStatus,
};

use crate::error::ApplicationError;
use crate::ports::Clock;

use super::dtos::{
    CheckSkillUpdatesQuery, MountSkillCommand, OverrideSkillCommand, SandboxSkillCommand,
    SkillMountSummary, SkillSandboxHandle, SkillUpdateStatus, UnmountSkillCommand,
    UpdateSkillCommand,
};

pub trait TaskSkillsApplicationService {
    fn mount_skill(
        &self,
        command: MountSkillCommand,
    ) -> Result<SkillMountSummary, ApplicationError>;
    fn list_mounts(
        &self,
        slug: &TaskSlug,
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
}

pub struct DefaultTaskSkillsApplicationService {
    tasks: Box<dyn TaskRepository>,
    mounts: Box<dyn TaskSkillMountRepository>,
    registry: Box<dyn SkillRegistryRepository>,
    clock: Box<dyn Clock>,
}

impl DefaultTaskSkillsApplicationService {
    pub fn new(
        tasks: Box<dyn TaskRepository>,
        mounts: Box<dyn TaskSkillMountRepository>,
        registry: Box<dyn SkillRegistryRepository>,
        clock: Box<dyn Clock>,
    ) -> Self {
        Self {
            tasks,
            mounts,
            registry,
            clock,
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

    fn mount_path(_slug: &TaskSlug, mount_id: &MountId) -> Utf8PathBuf {
        Utf8PathBuf::from("skills")
            .join("mounted")
            .join(mount_id.as_str())
    }

    fn mount_skill_path(skill_id: &SkillId) -> Utf8PathBuf {
        Utf8PathBuf::from(".opencode")
            .join("skills")
            .join(skill_id.as_str())
    }

    fn to_summary(slug: &TaskSlug, mount: TaskSkillMount) -> SkillMountSummary {
        SkillMountSummary {
            task_id: slug.to_string(),
            mount_id: mount.id.to_string(),
            skill_id: mount.skill_id.to_string(),
            version: mount.version.to_string(),
            source: mount.source.to_string(),
            status: mount.status,
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
                entity: EntityKind::Skill,
                slug: skill_id.to_string(),
            })
        })?;
        let version = command
            .version
            .map(|value| SkillVersion::from(value.as_str()))
            .or_else(|| skill.latest.clone())
            .ok_or_else(|| {
                ApplicationError::InvalidRequest("skill version is required".to_owned())
            })?;

        let mut mounts = self.mounts.list_for_task(&task.meta.slug)?;
        let mount_id = MountId::generate();
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
        self.mounts.save_for_task(&task.meta.slug, &mounts)?;
        Ok(Self::to_summary(&task.meta.slug, mount))
    }

    fn list_mounts(
        &self,
        slug: &TaskSlug,
    ) -> Result<Vec<SkillMountSummary>, ApplicationError> {
        Ok(self
            .mounts
            .list_for_task(slug)?
            .into_iter()
            .map(|mount| Self::to_summary(slug, mount))
            .collect())
    }

    fn unmount_skill(&self, command: UnmountSkillCommand) -> Result<(), ApplicationError> {
        self.mounts.remove_for_task(
            &TaskSlug::from(command.task_id.as_str()),
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
        let task_id = TaskSlug::from(query.task_id.as_str());
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
                    entity: EntityKind::Skill,
                    slug: mount.skill_id.to_string(),
                })
            })?;
            let update_available = skill
                .latest
                .as_ref()
                .is_some_and(|latest| latest != &mount.version);
            result.push(SkillUpdateStatus {
                mount_id: mount.id.to_string(),
                update_available,
                target_version: skill.latest,
            });
        }

        Ok(result)
    }

    fn update_skill(
        &self,
        command: UpdateSkillCommand,
    ) -> Result<SkillMountSummary, ApplicationError> {
        let task_id = TaskSlug::from(command.task_id.as_str());
        let mount_id = MountId::from(command.mount_id.as_str());
        let mut mounts = self.mounts.list_for_task(&task_id)?;
        let mount_index = mounts
            .iter()
            .position(|mount| mount.id == mount_id)
            .ok_or_else(|| {
                ApplicationError::Domain(DomainError::NotFound {
                    entity: EntityKind::Mount,
                    slug: mount_id.to_string(),
                })
            })?;
        let skill_id = mounts[mount_index].skill_id.clone();
        let skill = self.registry.find_skill(&skill_id)?.ok_or_else(|| {
            ApplicationError::Domain(DomainError::NotFound {
                entity: EntityKind::Skill,
                slug: skill_id.to_string(),
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
        let task_id = TaskSlug::from(command.task_id.as_str());
        let mount_id = MountId::from(command.mount_id.as_str());
        Ok(SkillSandboxHandle {
            mount_id: mount_id.to_string(),
            path: Self::mount_path(&task_id, &mount_id),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::collections::BTreeMap;

    use time::OffsetDateTime;
    use workc_domain::errors::DomainError;
    use workc_domain::shared::{MountId, SkillId, SkillSourceId, SkillVersion, TaskSlug};
    use workc_domain::skill_registry::{
        SkillDefinition, SkillRegistry, SkillRegistryRepository, SkillSource,
    };
    use workc_domain::task::{
        TaskActivity, TaskMeta, TaskPaths, TaskRepoSelection, TaskRepository, TaskSkillMount,
        TaskSkillMountRepository, TaskStatus, TaskWorkspace,
    };

    use crate::ports::Clock;

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
                .find(|t| t.meta.slug == *slug)
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
        fn list_for_task(&self, slug: &TaskSlug) -> Result<Vec<TaskSkillMount>, DomainError> {
            Ok(self
                .mounts
                .borrow()
                .get(slug.as_str())
                .cloned()
                .unwrap_or_default())
        }

        fn save_for_task(
            &self,
            slug: &TaskSlug,
            mounts: &[TaskSkillMount],
        ) -> Result<(), DomainError> {
            self.mounts
                .borrow_mut()
                .insert(slug.to_string(), mounts.to_vec());
            Ok(())
        }

        fn remove_for_task(&self, slug: &TaskSlug, mount_id: &MountId) -> Result<(), DomainError> {
            if let Some(mounts) = self.mounts.borrow_mut().get_mut(slug.as_str()) {
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

    fn sample_task() -> TaskWorkspace {
        TaskWorkspace {
            meta: TaskMeta {
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

    fn service() -> DefaultTaskSkillsApplicationService {
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
        )
    }

    #[test]
    fn mount_skill_creates_mount_with_latest_version() {
        let svc = service();
        let summary = svc
            .mount_skill(MountSkillCommand {
                task_id: "auth-fix".to_owned(),
                skill_id: "frontend-testing".to_owned(),
                version: None,
            })
            .unwrap();

        assert_eq!(summary.skill_id, "frontend-testing");
        assert_eq!(summary.version, "2026-05-22");
        assert_eq!(summary.status, TaskSkillMountStatus::Active);
        assert!(summary.path.as_str().contains("frontend-testing"));
    }

    #[test]
    fn mount_skill_fails_for_missing_skill() {
        let svc = service();
        let result = svc.mount_skill(MountSkillCommand {
            task_id: "auth-fix".to_owned(),
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
        );

        let result = svc.mount_skill(MountSkillCommand {
            task_id: "auth-fix".to_owned(),
            skill_id: "frontend-testing".to_owned(),
            version: None,
        });
        assert!(result.is_err());
    }

    #[test]
    fn list_mounts_returns_empty_initially() {
        let svc = service();
        let mounts = svc
            .list_mounts(&TaskSlug::from("auth-fix"))
            .unwrap();
        assert!(mounts.is_empty());
    }

    #[test]
    fn unmount_skill_removes_mount() {
        let svc = service();
        let summary = svc
            .mount_skill(MountSkillCommand {
                task_id: "auth-fix".to_owned(),
                skill_id: "frontend-testing".to_owned(),
                version: None,
            })
            .unwrap();

        svc.unmount_skill(UnmountSkillCommand {
            task_id: "auth-fix".to_owned(),
            mount_id: summary.mount_id.clone(),
        })
        .unwrap();

        let mounts = svc
            .list_mounts(&TaskSlug::from("auth-fix"))
            .unwrap();
        assert!(mounts.is_empty());
    }

    #[test]
    fn override_skill_returns_adapter_unavailable() {
        let svc = service();
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
        let svc = service();
        svc.mount_skill(MountSkillCommand {
            task_id: "auth-fix".to_owned(),
            skill_id: "frontend-testing".to_owned(),
            version: Some("2026-05-20".to_owned()),
        })
        .unwrap();

        let updates = svc
            .check_skill_updates(CheckSkillUpdatesQuery {
                task_id: "auth-fix".to_owned(),
                mount_id: None,
            })
            .unwrap();
        assert_eq!(updates.len(), 1);
        assert!(updates[0].update_available);
        assert_eq!(updates[0].target_version.as_ref().map(|v| v.as_str()), Some("2026-05-22"));
    }

    #[test]
    fn check_skill_updates_no_update_when_already_latest() {
        let svc = service();
        svc.mount_skill(MountSkillCommand {
            task_id: "auth-fix".to_owned(),
            skill_id: "frontend-testing".to_owned(),
            version: None,
        })
        .unwrap();

        let updates = svc
            .check_skill_updates(CheckSkillUpdatesQuery {
                task_id: "auth-fix".to_owned(),
                mount_id: None,
            })
            .unwrap();
        assert_eq!(updates.len(), 1);
        assert!(!updates[0].update_available);
    }

    #[test]
    fn update_skill_changes_version_to_latest() {
        let svc = service();
        let summary = svc.mount_skill(MountSkillCommand {
            task_id: "auth-fix".to_owned(),
            skill_id: "frontend-testing".to_owned(),
            version: Some("2026-05-20".to_owned()),
        })
        .unwrap();

        let updated = svc
            .update_skill(UpdateSkillCommand {
                task_id: "auth-fix".to_owned(),
                mount_id: summary.mount_id,
            })
            .unwrap();

        assert_eq!(updated.version, "2026-05-22");
    }

    #[test]
    fn update_skill_fails_for_missing_mount() {
        let svc = service();
        let result = svc.update_skill(UpdateSkillCommand {
            task_id: "auth-fix".to_owned(),
            mount_id: "mount-999".to_owned(),
        });
        assert!(result.is_err());
    }

    #[test]
    fn sandbox_skill_returns_handle() {
        let svc = service();
        let handle = svc
            .sandbox_skill(SandboxSkillCommand {
                task_id: "auth-fix".to_owned(),
                mount_id: "mount-001".to_owned(),
            })
            .unwrap();

        assert_eq!(handle.mount_id, "mount-001");
        assert!(handle.path.as_str().contains("mount-001"));
    }
}
