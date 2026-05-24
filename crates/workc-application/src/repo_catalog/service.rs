use workc_domain::errors::DomainError;
use workc_domain::repo_catalog::{RepoCatalogRepository, RepoEntry, RepoGroup};
use workc_domain::shared::{RepoGroupId, RepoId};

use crate::error::ApplicationError;

use super::dtos::{AddRepoCommand, AddRepoGroupCommand, RepoGroupSummary, RepoSummary};

pub trait RepoCatalogApplicationService {
    fn add_repo(&self, command: AddRepoCommand) -> Result<RepoSummary, ApplicationError>;
    fn list_repos(&self) -> Result<Vec<RepoSummary>, ApplicationError>;
    fn add_repo_group(
        &self,
        command: AddRepoGroupCommand,
    ) -> Result<RepoGroupSummary, ApplicationError>;
    fn list_repo_groups(&self) -> Result<Vec<RepoGroupSummary>, ApplicationError>;
}

pub struct DefaultRepoCatalogApplicationService {
    repository: Box<dyn RepoCatalogRepository>,
}

impl DefaultRepoCatalogApplicationService {
    pub fn new(repository: Box<dyn RepoCatalogRepository>) -> Self {
        Self { repository }
    }
}

impl RepoCatalogApplicationService for DefaultRepoCatalogApplicationService {
    fn add_repo(&self, command: AddRepoCommand) -> Result<RepoSummary, ApplicationError> {
        let repo_id = RepoId::from(command.id.as_str());

        if command.url.trim().is_empty() {
            return Err(ApplicationError::InvalidRequest(
                "repo url cannot be empty".to_owned(),
            ));
        }

        if self.repository.find_repo(&repo_id)?.is_some() {
            return Err(ApplicationError::Domain(DomainError::AlreadyExists {
                entity: "repo",
                id: repo_id.to_string(),
            }));
        }

        let mut catalog = self.repository.load()?;
        let entry = RepoEntry {
            id: repo_id,
            url: command.url,
            tags: command.tags,
            description: command.description,
        };
        catalog.repos.push(entry.clone());
        self.repository.save(&catalog)?;

        Ok(RepoSummary {
            id: entry.id.to_string(),
            url: entry.url,
            tags: entry.tags,
            description: entry.description,
        })
    }

    fn list_repos(&self) -> Result<Vec<RepoSummary>, ApplicationError> {
        let mut repos = self.repository.load()?.repos;
        repos.sort_by(|left, right| left.id.as_str().cmp(right.id.as_str()));

        Ok(repos
            .into_iter()
            .map(|repo| RepoSummary {
                id: repo.id.to_string(),
                url: repo.url,
                tags: repo.tags,
                description: repo.description,
            })
            .collect())
    }

    fn add_repo_group(
        &self,
        command: AddRepoGroupCommand,
    ) -> Result<RepoGroupSummary, ApplicationError> {
        let group_id = RepoGroupId::from(command.id.as_str());

        if command.repos.is_empty() {
            return Err(ApplicationError::InvalidRequest(
                "repo group must contain at least one repo".to_owned(),
            ));
        }

        let mut catalog = self.repository.load()?;
        if catalog.groups.iter().any(|group| group.id == group_id) {
            return Err(ApplicationError::Domain(DomainError::AlreadyExists {
                entity: "repo-group",
                id: group_id.to_string(),
            }));
        }

        let repo_ids: Vec<RepoId> = command
            .repos
            .iter()
            .map(|repo| RepoId::from(repo.as_str()))
            .collect();
        for repo_id in &repo_ids {
            if !catalog.repos.iter().any(|repo| repo.id == *repo_id) {
                return Err(ApplicationError::Domain(DomainError::NotFound {
                    entity: "repo",
                    id: repo_id.to_string(),
                }));
            }
        }

        let group = RepoGroup {
            id: group_id,
            repos: repo_ids,
            tags: command.tags,
            description: command.description,
        };
        catalog.groups.push(group.clone());
        self.repository.save(&catalog)?;

        Ok(RepoGroupSummary {
            id: group.id.to_string(),
            repos: group
                .repos
                .into_iter()
                .map(|repo| repo.to_string())
                .collect(),
            tags: group.tags,
            description: group.description,
        })
    }

    fn list_repo_groups(&self) -> Result<Vec<RepoGroupSummary>, ApplicationError> {
        let mut groups = self.repository.load()?.groups;
        groups.sort_by(|left, right| left.id.as_str().cmp(right.id.as_str()));

        Ok(groups
            .into_iter()
            .map(|group| RepoGroupSummary {
                id: group.id.to_string(),
                repos: group
                    .repos
                    .into_iter()
                    .map(|repo| repo.to_string())
                    .collect(),
                tags: group.tags,
                description: group.description,
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use workc_domain::repo_catalog::{RepoCatalog, RepoCatalogRepository, RepoEntry, RepoGroup};

    use super::*;

    struct InMemoryRepoCatalogRepository {
        catalog: RefCell<RepoCatalog>,
    }

    impl Default for InMemoryRepoCatalogRepository {
        fn default() -> Self {
            Self {
                catalog: RefCell::new(RepoCatalog {
                    repos: Vec::new(),
                    groups: Vec::new(),
                }),
            }
        }
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

    #[test]
    fn add_repo_persists_and_lists() {
        let service = DefaultRepoCatalogApplicationService::new(Box::new(
            InMemoryRepoCatalogRepository::default(),
        ));

        let repo = service
            .add_repo(AddRepoCommand {
                id: "auth-service".to_owned(),
                url: "git@github.com:example/auth-service.git".to_owned(),
                tags: vec!["backend".to_owned()],
                description: Some("Auth service".to_owned()),
            })
            .unwrap();

        assert_eq!(repo.id, "auth-service");
        assert_eq!(service.list_repos().unwrap().len(), 1);
    }

    #[test]
    fn add_repo_group_requires_existing_repos() {
        let service = DefaultRepoCatalogApplicationService::new(Box::new(
            InMemoryRepoCatalogRepository::default(),
        ));
        let result = service.add_repo_group(AddRepoGroupCommand {
            id: "auth-core".to_owned(),
            repos: vec!["auth-service".to_owned()],
            tags: vec![],
            description: None,
        });

        assert!(
            matches!(result, Err(ApplicationError::Domain(DomainError::NotFound { entity, .. })) if entity == "repo")
        );
    }

    #[test]
    fn add_repo_rejects_duplicate() {
        let service = DefaultRepoCatalogApplicationService::new(Box::new(
            InMemoryRepoCatalogRepository::default(),
        ));
        service
            .add_repo(AddRepoCommand {
                id: "auth-service".to_owned(),
                url: "git@github.com:example/auth-service.git".to_owned(),
                tags: vec![],
                description: None,
            })
            .unwrap();

        let result = service.add_repo(AddRepoCommand {
            id: "auth-service".to_owned(),
            url: "git@github.com:example/auth-service-2.git".to_owned(),
            tags: vec![],
            description: None,
        });
        assert!(
            matches!(result, Err(ApplicationError::Domain(DomainError::AlreadyExists { entity, .. })) if entity == "repo")
        );
    }

    #[test]
    fn add_repo_rejects_empty_url() {
        let service = DefaultRepoCatalogApplicationService::new(Box::new(
            InMemoryRepoCatalogRepository::default(),
        ));
        let result = service.add_repo(AddRepoCommand {
            id: "auth-service".to_owned(),
            url: "   ".to_owned(),
            tags: vec![],
            description: None,
        });
        assert!(result.is_err());
    }

    #[test]
    fn add_repo_group_creates_successfully_when_repos_exist() {
        let service = DefaultRepoCatalogApplicationService::new(Box::new(
            InMemoryRepoCatalogRepository::default(),
        ));
        service
            .add_repo(AddRepoCommand {
                id: "auth-service".to_owned(),
                url: "git@github.com:example/auth-service.git".to_owned(),
                tags: vec![],
                description: None,
            })
            .unwrap();

        let group = service
            .add_repo_group(AddRepoGroupCommand {
                id: "auth-core".to_owned(),
                repos: vec!["auth-service".to_owned()],
                tags: vec!["core".to_owned()],
                description: None,
            })
            .unwrap();

        assert_eq!(group.id, "auth-core");
        assert_eq!(group.repos, vec!["auth-service"]);
    }

    #[test]
    fn add_repo_group_rejects_empty_repos_list() {
        let service = DefaultRepoCatalogApplicationService::new(Box::new(
            InMemoryRepoCatalogRepository::default(),
        ));
        let result = service.add_repo_group(AddRepoGroupCommand {
            id: "auth-core".to_owned(),
            repos: vec![],
            tags: vec![],
            description: None,
        });
        assert!(result.is_err());
    }

    #[test]
    fn list_repos_sorts_alphabetically() {
        let service = DefaultRepoCatalogApplicationService::new(Box::new(
            InMemoryRepoCatalogRepository::default(),
        ));
        service
            .add_repo(AddRepoCommand {
                id: "zulu".to_owned(),
                url: "url-z".to_owned(),
                tags: vec![],
                description: None,
            })
            .unwrap();
        service
            .add_repo(AddRepoCommand {
                id: "alpha".to_owned(),
                url: "url-a".to_owned(),
                tags: vec![],
                description: None,
            })
            .unwrap();

        let repos = service.list_repos().unwrap();
        assert_eq!(repos.len(), 2);
        assert_eq!(repos[0].id, "alpha");
        assert_eq!(repos[1].id, "zulu");
    }

    #[test]
    fn list_repo_groups_sorts_alphabetically() {
        let service = DefaultRepoCatalogApplicationService::new(Box::new(
            InMemoryRepoCatalogRepository::default(),
        ));
        service
            .add_repo(AddRepoCommand {
                id: "auth-service".to_owned(),
                url: "url".to_owned(),
                tags: vec![],
                description: None,
            })
            .unwrap();

        service
            .add_repo_group(AddRepoGroupCommand {
                id: "z-group".to_owned(),
                repos: vec!["auth-service".to_owned()],
                tags: vec![],
                description: None,
            })
            .unwrap();
        service
            .add_repo_group(AddRepoGroupCommand {
                id: "a-group".to_owned(),
                repos: vec!["auth-service".to_owned()],
                tags: vec![],
                description: None,
            })
            .unwrap();

        let groups = service.list_repo_groups().unwrap();
        assert_eq!(groups[0].id, "a-group");
        assert_eq!(groups[1].id, "z-group");
    }
}
