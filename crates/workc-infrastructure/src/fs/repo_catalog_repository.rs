use std::fs;

use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};
use workc_domain::errors::DomainError;
use workc_domain::repo_catalog::{RepoCatalog, RepoCatalogRepository, RepoEntry, RepoGroup};
use workc_domain::shared::{RepoGroupId, RepoId};

pub struct FsRepoCatalogRepository {
    workspace_root: Utf8PathBuf,
}

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
    pub fn new(workspace_root: Utf8PathBuf) -> Self {
        Self { workspace_root }
    }

    fn repos_root(&self) -> Utf8PathBuf {
        self.workspace_root.join("repos")
    }

    fn catalog_path(&self) -> Utf8PathBuf {
        self.repos_root().join("catalog.toml")
    }

    fn groups_path(&self) -> Utf8PathBuf {
        self.repos_root().join("groups.toml")
    }
}

impl RepoCatalogRepository for FsRepoCatalogRepository {
    fn load(&self) -> Result<RepoCatalog, DomainError> {
        if !self.repos_root().exists() {
            return Ok(RepoCatalog {
                repos: Vec::new(),
                groups: Vec::new(),
            });
        }

        let repos = if self.catalog_path().exists() {
            let raw =
                fs::read_to_string(self.catalog_path()).map_err(io_error("read repo catalog"))?;
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

        let groups = if self.groups_path().exists() {
            let raw =
                fs::read_to_string(self.groups_path()).map_err(io_error("read repo groups"))?;
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
        fs::create_dir_all(self.repos_root()).map_err(io_error("create repos root"))?;

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
            self.catalog_path(),
            toml::to_string_pretty(&catalog_toml).map_err(invalid_serialize("catalog.toml"))?,
        )
        .map_err(io_error("write repo catalog"))?;
        fs::write(
            self.groups_path(),
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

fn io_error(operation: &'static str) -> impl Fn(std::io::Error) -> DomainError {
    move |error| DomainError::IoError {
        operation,
        detail: error.to_string(),
    }
}

fn invalid_toml(field: &'static str) -> impl Fn(toml::de::Error) -> DomainError {
    move |error| DomainError::InvalidInput {
        field,
        reason: error.to_string(),
    }
}

fn invalid_serialize(field: &'static str) -> impl Fn(toml::ser::Error) -> DomainError {
    move |error| DomainError::InvalidInput {
        field,
        reason: error.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use workc_domain::repo_catalog::{RepoCatalog, RepoCatalogRepository, RepoEntry, RepoGroup};

    use super::*;

    fn temp_workspace() -> Utf8PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        Utf8PathBuf::from_path_buf(
            std::env::temp_dir().join(format!("workc-repo-catalog-{unique}")),
        )
        .unwrap()
    }

    #[test]
    fn save_and_load_roundtrip() {
        let root = temp_workspace();
        let repository = FsRepoCatalogRepository::new(root.clone());
        let catalog = RepoCatalog {
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

        repository.save(&catalog).unwrap();
        let loaded = repository.load().unwrap();

        assert_eq!(loaded.repos.len(), 1);
        assert_eq!(loaded.groups.len(), 1);

        fs::remove_dir_all(root).unwrap();
    }
}
