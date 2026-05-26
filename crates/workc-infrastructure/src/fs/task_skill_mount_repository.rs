use std::fs;

use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};
use time::format_description::well_known::Rfc3339;
use workc_domain::errors::FieldKind;
use workc_domain::errors::DomainError;
use workc_domain::shared::{MountId, SkillId, SkillSourceId, SkillVersion, TaskSlug, Timestamp};
use workc_domain::task::{TaskSkillMount, TaskSkillMountRepository, TaskSkillMountStatus};

pub struct FsTaskSkillMountRepository {
    project_root: Utf8PathBuf,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct MountsToml {
    #[serde(default)]
    mounts: Vec<MountToml>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MountToml {
    id: String,
    skill_id: String,
    version: String,
    source: String,
    mounted_at: String,
    status: String,
    path: String,
}

impl FsTaskSkillMountRepository {
    pub fn new(project_root: Utf8PathBuf) -> Self {
        Self { project_root }
    }

    fn mounts_path(&self, _slug: &TaskSlug) -> Utf8PathBuf {
        self.project_root.join("skills").join("mounts.toml")
    }

    fn format_timestamp(value: Timestamp) -> Result<String, DomainError> {
        value
            .format(&Rfc3339)
            .map_err(|error| DomainError::InvalidInput {
                field: FieldKind::Timestamp,
                reason: error.to_string(),
            })
    }

    fn parse_timestamp(value: &str) -> Result<Timestamp, DomainError> {
        Timestamp::parse(value, &Rfc3339).map_err(|error| DomainError::InvalidInput {
            field: FieldKind::Timestamp,
            reason: error.to_string(),
        })
    }
}

impl TaskSkillMountRepository for FsTaskSkillMountRepository {
    fn list_for_task(&self, slug: &TaskSlug) -> Result<Vec<TaskSkillMount>, DomainError> {
        let path = self.mounts_path(slug);
        if !path.exists() {
            return Ok(Vec::new());
        }

        let raw = fs::read_to_string(path).map_err(io_error("read skill mounts"))?;
        let parsed = toml::from_str::<MountsToml>(&raw).map_err(invalid_toml("mounts.toml"))?;

        parsed
            .mounts
            .into_iter()
            .map(|mount| {
                Ok(TaskSkillMount {
                    id: MountId::from(mount.id),
                    skill_id: SkillId::from(mount.skill_id),
                    version: SkillVersion::from(mount.version),
                    source: SkillSourceId::from(mount.source),
                    mounted_at: Self::parse_timestamp(&mount.mounted_at)?,
                    status: match mount.status.as_str() {
                        "active" => TaskSkillMountStatus::Active,
                        "inactive" => TaskSkillMountStatus::Inactive,
                        "removed" => TaskSkillMountStatus::Removed,
                        other => {
                            return Err(DomainError::InvalidInput {
                                field: FieldKind::Other("mount status"),
                                reason: format!("unknown mount status: {other}"),
                            });
                        }
                    },
                    path: mount.path.into(),
                })
            })
            .collect()
    }

    fn save_for_task(
        &self,
        slug: &TaskSlug,
        mounts: &[TaskSkillMount],
    ) -> Result<(), DomainError> {
        let path = self.mounts_path(slug);
        let parent = path.parent().ok_or(DomainError::InvalidInput {
            field: FieldKind::Other("mount path"),
            reason: "missing parent directory".to_owned(),
        })?;
        fs::create_dir_all(parent).map_err(io_error("create skill mounts dir"))?;

        let raw = toml::to_string_pretty(&MountsToml {
            mounts: mounts
                .iter()
                .map(|mount| {
                    Ok(MountToml {
                        id: mount.id.to_string(),
                        skill_id: mount.skill_id.to_string(),
                        version: mount.version.to_string(),
                        source: mount.source.to_string(),
                        mounted_at: Self::format_timestamp(mount.mounted_at)?,
                        status: match mount.status {
                            TaskSkillMountStatus::Active => "active",
                            TaskSkillMountStatus::Inactive => "inactive",
                            TaskSkillMountStatus::Removed => "removed",
                        }
                        .to_owned(),
                        path: mount.path.to_string(),
                    })
                })
                .collect::<Result<Vec<_>, DomainError>>()?,
        })
        .map_err(invalid_serialize("mounts.toml"))?;
        fs::write(path, raw).map_err(io_error("write skill mounts"))?;
        Ok(())
    }

    fn remove_for_task(&self, slug: &TaskSlug, mount_id: &MountId) -> Result<(), DomainError> {
        let mut mounts = self.list_for_task(slug)?;
        mounts.retain(|mount| mount.id != *mount_id);
        self.save_for_task(slug, &mounts)
    }
}

fn io_error(operation: &'static str) -> impl Fn(std::io::Error) -> DomainError { move |error| DomainError::PersistenceFailed { operation: operation,
        detail: error.to_string(),
    }
}

fn invalid_toml(field: &'static str) -> impl Fn(toml::de::Error) -> DomainError { move |error| DomainError::InvalidInput { field: FieldKind::Other(field),
        reason: error.to_string(),
    }
}

fn invalid_serialize(field: &'static str) -> impl Fn(toml::ser::Error) -> DomainError { move |error| DomainError::InvalidInput { field: FieldKind::Other(field),
        reason: error.to_string(),
    }
}

