use std::fs;

use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};
use workc_domain::errors::{DomainError, FieldKind};
use workc_domain::repo_catalog::{RepoCatalog, RepoCatalogRepository, RepoEntry, RepoGroup};
use workc_domain::shared::{RepoGroupId, RepoId};

use super::paths;

pub struct FsRepoCatalogRepository;

#[derive(Debug, Serialize, Deserialize, Default)]
struct CatalogToml {
    #[serde(default)]
    repos: Vec<CatalogRepoEntry>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct GroupsToml {
    #[serde(default)]
    groups: Vec<CatalogRepoGroup>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CatalogRepoEntry {
    id: String,
    url: String,
    #[serde(default)]
    tags: Vec<String>,
    description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CatalogRepoGroup {
    id: String,
    #[serde(default)]
    repos: Vec<String>,
    #[serde(default)]
    tags: Vec<String>,
    description: Option<String>,
}

impl FsRepoCatalogRepository {
    pub fn new() -> Self {
        Self
    }
}

impl Default for FsRepoCatalogRepository {
    fn default() -> Self {
        Self
    }
}

impl FsRepoCatalogRepository {
    fn repos_root() -> Utf8PathBuf {
        paths::workc_repos_root()
    }

    fn catalog_path() -> Utf8PathBuf {
        Self::repos_root().join("catalog.toml")
    }

    fn groups_path() -> Utf8PathBuf {
        Self::repos_root().join("groups.toml")
    }
}

impl RepoCatalogRepository for FsRepoCatalogRepository {
    fn load(&self) -> Result<RepoCatalog, DomainError> {
        let root = Self::repos_root();
        if !root.exists() {
            return Ok(RepoCatalog {
                repos: Vec::new(),
                groups: Vec::new(),
            });
        }

        let repos = if Self::catalog_path().exists() {
            let raw =
                fs::read_to_string(Self::catalog_path()).map_err(io_error("read repo catalog"))?;
            toml::from_str::<CatalogToml>(&raw)
                .map_err(invalid_toml("catalog.toml"))?
                .repos
                .into_iter()
                .map(|repo| RepoEntry {
                    id: RepoId::from(repo.id),
                    url: repo.url,
                    tags: repo.tags,
                    description: repo.description,
                })
                .collect()
        } else {
            Vec::new()
        };

        let groups = if Self::groups_path().exists() {
            let raw =
                fs::read_to_string(Self::groups_path()).map_err(io_error("read repo groups"))?;
            toml::from_str::<GroupsToml>(&raw)
                .map_err(invalid_toml("groups.toml"))?
                .groups
                .into_iter()
                .map(|group| RepoGroup {
                    id: RepoGroupId::from(group.id),
                    repos: group.repos.into_iter().map(RepoId::from).collect(),
                    tags: group.tags,
                    description: group.description,
                })
                .collect()
        } else {
            Vec::new()
        };

        Ok(RepoCatalog { repos, groups })
    }

    fn save(&self, catalog: &RepoCatalog) -> Result<(), DomainError> {
        fs::create_dir_all(Self::repos_root()).map_err(io_error("create repos root"))?;

        let catalog_toml = CatalogToml {
            repos: catalog
                .repos
                .iter()
                .map(|repo| CatalogRepoEntry {
                    id: repo.id.to_string(),
                    url: repo.url.clone(),
                    tags: repo.tags.clone(),
                    description: repo.description.clone(),
                })
                .collect(),
        };
        let groups_toml = GroupsToml {
            groups: catalog
                .groups
                .iter()
                .map(|group| CatalogRepoGroup {
                    id: group.id.to_string(),
                    repos: group.repos.iter().map(ToString::to_string).collect(),
                    tags: group.tags.clone(),
                    description: group.description.clone(),
                })
                .collect(),
        };

        fs::write(
            Self::catalog_path(),
            toml::to_string_pretty(&catalog_toml).map_err(invalid_serialize("catalog.toml"))?,
        )
        .map_err(io_error("write repo catalog"))?;
        fs::write(
            Self::groups_path(),
            toml::to_string_pretty(&groups_toml).map_err(invalid_serialize("groups.toml"))?,
        )
        .map_err(io_error("write repo groups"))?;
        Ok(())
    }

    fn find_repo(&self, id: &RepoId) -> Result<Option<RepoEntry>, DomainError> {
        Ok(self.load()?.repos.into_iter().find(|repo| repo.id == *id))
    }

    fn find_group(&self, id: &RepoGroupId) -> Result<Option<RepoGroup>, DomainError> {
        Ok(self
            .load()?
            .groups
            .into_iter()
            .find(|group| group.id == *id))
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

#[cfg(test)]
mod tests {
    use workc_domain::repo_catalog::{RepoCatalog, RepoEntry, RepoGroup};

    use super::*;

    #[test]
    fn new_constructs_without_args() {
        let _repository = FsRepoCatalogRepository::new();
    }

    #[test]
    fn save_and_load_roundtrip() {
        let _repository = FsRepoCatalogRepository::new();
        let _catalog = RepoCatalog {
            repos: vec![RepoEntry {
                id: RepoId::from("auth-service"),
                url: "git@github.com:example/auth-service.git".to_owned(),
                tags: vec!["backend".to_owned()],
                description: Some("Auth".to_owned()),
            }],
            groups: vec![RepoGroup {
                id: RepoGroupId::from("auth-core"),
                repos: vec![RepoId::from("auth-service")],
                tags: vec!["auth".to_owned()],
                description: None,
            }],
        };
    }
}

