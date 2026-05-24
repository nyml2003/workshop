use super::paths;
use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};
use std::fs;
use time::format_description::well_known::Rfc3339;
use workc_domain::errors::DomainError;
use workc_domain::shared::{TaskSlug, Timestamp};
use workc_domain::workspace::{WorkspaceEntry, WorkspaceRegistryRepository, WorkspaceStatus};

pub struct FsWorkspaceRegistryRepository;

#[derive(Debug, Serialize, Deserialize, Default)]
struct WorkspacesToml {
    #[serde(default)]
    workspaces: Vec<WorkspaceToml>,
}

#[derive(Debug, Serialize, Deserialize)]
struct WorkspaceToml {
    slug: String,
    path: String,
    title: String,
    status: String,
    last_activity_at: Option<String>,
}

impl FsWorkspaceRegistryRepository {
    pub fn new() -> Self {
        Self
    }
}

impl Default for FsWorkspaceRegistryRepository {
    fn default() -> Self {
        Self
    }
}

impl FsWorkspaceRegistryRepository {
    fn to_toml(entries: &[WorkspaceEntry]) -> Result<WorkspacesToml, DomainError> {
        let workspaces = entries
            .iter()
            .map(|entry| {
                Ok(WorkspaceToml {
                    slug: entry.slug.to_string(),
                    path: entry.path.as_str().to_owned(),
                    title: entry.title.clone(),
                    status: entry.status.as_str().to_owned(),
                    last_activity_at: entry
                        .last_activity_at
                        .map(|ts| {
                            ts.format(&Rfc3339)
                                .map_err(|error| DomainError::InvalidInput {
                                    field: "timestamp",
                                    reason: error.to_string(),
                                })
                        })
                        .transpose()?,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(WorkspacesToml { workspaces })
    }

    fn from_toml(toml: WorkspacesToml) -> Result<Vec<WorkspaceEntry>, DomainError> {
        toml.workspaces
            .into_iter()
            .map(|w| {
                Ok(WorkspaceEntry {
                    slug: TaskSlug::from(w.slug),
                    path: Utf8PathBuf::from(w.path),
                    title: w.title,
                    status: WorkspaceStatus::parse(&w.status).unwrap_or(WorkspaceStatus::Active),
                    last_activity_at: w
                        .last_activity_at
                        .map(|raw| {
                            Timestamp::parse(&raw, &Rfc3339).map_err(|error| {
                                DomainError::InvalidInput {
                                    field: "timestamp",
                                    reason: error.to_string(),
                                }
                            })
                        })
                        .transpose()?,
                })
            })
            .collect()
    }
}

impl WorkspaceRegistryRepository for FsWorkspaceRegistryRepository {
    fn load(&self) -> Result<Vec<WorkspaceEntry>, DomainError> {
        let path = paths::workc_workspaces_path();
        if !path.exists() {
            return Ok(Vec::new());
        }
        let raw = fs::read_to_string(&path).map_err(|error| DomainError::IoError {
            operation: "read workspaces",
            detail: error.to_string(),
        })?;
        let toml: WorkspacesToml =
            toml::from_str(&raw).map_err(|error| DomainError::InvalidInput {
                field: "workspaces.toml",
                reason: error.to_string(),
            })?;
        Self::from_toml(toml)
    }

    fn save(&self, entries: &[WorkspaceEntry]) -> Result<(), DomainError> {
        let path = paths::workc_workspaces_path();
        let parent = path.parent().ok_or(DomainError::InvalidInput {
            field: "workspaces path",
            reason: "no parent directory".to_owned(),
        })?;
        fs::create_dir_all(parent).map_err(|error| DomainError::IoError {
            operation: "create workc home",
            detail: error.to_string(),
        })?;
        let toml = Self::to_toml(entries)?;
        fs::write(
            &path,
            toml::to_string_pretty(&toml).map_err(|error| DomainError::InvalidInput {
                field: "workspaces.toml",
                reason: error.to_string(),
            })?,
        )
        .map_err(|error| DomainError::IoError {
            operation: "write workspaces",
            detail: error.to_string(),
        })?;
        Ok(())
    }
}
