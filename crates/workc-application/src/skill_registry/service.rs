use workc_domain::errors::EntityKind;
use workc_domain::errors::DomainError;
use workc_domain::shared::{SkillId, SkillSourceId, SkillVersion};
use workc_domain::skill_registry::{
    SkillDefinition, SkillRegistryRepository, SkillSource, SkillSourceKind,
};

use crate::error::ApplicationError;

use super::dtos::{
    ApplicationSkillSourceKind, ImportSkillSourceCommand, ShowSkillQuery, SkillSummary,
};

pub trait SkillRegistryApplicationService {
    fn import_source(&self, command: ImportSkillSourceCommand) -> Result<(), ApplicationError>;
    fn show_skill(&self, query: ShowSkillQuery) -> Result<Option<SkillSummary>, ApplicationError>;
    fn list_skill_versions(&self, query: ShowSkillQuery) -> Result<Vec<String>, ApplicationError>;
}

pub struct DefaultSkillRegistryApplicationService {
    repository: Box<dyn SkillRegistryRepository>,
    clock: Box<dyn crate::ports::Clock>,
}

impl DefaultSkillRegistryApplicationService {
    pub fn new(
        repository: Box<dyn SkillRegistryRepository>,
        clock: Box<dyn crate::ports::Clock>,
    ) -> Self {
        Self { repository, clock }
    }

    fn to_source_kind(value: ApplicationSkillSourceKind) -> SkillSourceKind {
        match value {
            ApplicationSkillSourceKind::Git => SkillSourceKind::Git,
            ApplicationSkillSourceKind::Local => SkillSourceKind::Local,
            ApplicationSkillSourceKind::Archive => SkillSourceKind::Archive,
            ApplicationSkillSourceKind::Other(value) => SkillSourceKind::Other(value),
        }
    }
}

impl SkillRegistryApplicationService for DefaultSkillRegistryApplicationService {
    fn import_source(&self, command: ImportSkillSourceCommand) -> Result<(), ApplicationError> {
        let mut registry = self.repository.load()?;
        let source_id = SkillSourceId::from(command.source_id.as_str());

        if registry.sources.iter().any(|source| source.id == source_id) {
            return Err(ApplicationError::Domain(DomainError::AlreadyExists {
                entity: EntityKind::Skill,
                slug: source_id.to_string(),
            }));
        }

        registry.sources.push(SkillSource {
            id: source_id.clone(),
            kind: Self::to_source_kind(command.kind),
            location: command.location,
            reference: command.reference,
            imported_at: Some(self.clock.now()),
        });

        for skill in command.skills {
            let skill_id = SkillId::from(skill.id.as_str());
            if registry
                .skills
                .iter()
                .any(|existing| existing.id == skill_id)
            {
                return Err(ApplicationError::Domain(DomainError::AlreadyExists {
                    entity: EntityKind::Skill,
                    slug: skill_id.to_string(),
                }));
            }

            registry.skills.push(SkillDefinition {
                id: skill_id,
                source: source_id.clone(),
                versions: skill.versions.into_iter().map(SkillVersion::from).collect(),
                latest: skill.latest.map(SkillVersion::from),
            });
        }

        self.repository.save(&registry)?;
        Ok(())
    }

    fn show_skill(&self, query: ShowSkillQuery) -> Result<Option<SkillSummary>, ApplicationError> {
        Ok(self
            .repository
            .find_skill(&SkillId::from(query.skill_id.as_str()))?
            .map(|skill| SkillSummary {
                id: skill.id.to_string(),
                source: skill.source.to_string(),
                versions: skill
                    .versions
                    .into_iter()
                    .map(|version| version.to_string())
                    .collect(),
                latest: skill.latest.map(|version| version.to_string()),
            }))
    }

    fn list_skill_versions(&self, query: ShowSkillQuery) -> Result<Vec<String>, ApplicationError> {
        Ok(self
            .repository
            .find_skill(&SkillId::from(query.skill_id.as_str()))?
            .map(|skill| {
                skill
                    .versions
                    .into_iter()
                    .map(|version| version.to_string())
                    .collect()
            })
            .unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use time::OffsetDateTime;
    use workc_domain::shared::{SkillId, SkillSourceId};
    use workc_domain::skill_registry::{SkillDefinition, SkillRegistryRepository, SkillSource};

    use crate::ports::Clock;

    use super::*;
    use crate::skill_registry::ImportedSkillDefinition;

    struct InMemorySkillRegistryRepository {
        registry: RefCell<workc_domain::skill_registry::SkillRegistry>,
    }

    impl Default for InMemorySkillRegistryRepository {
        fn default() -> Self {
            Self {
                registry: RefCell::new(workc_domain::skill_registry::SkillRegistry::default()),
            }
        }
    }

    impl SkillRegistryRepository for InMemorySkillRegistryRepository {
        fn load(&self) -> Result<workc_domain::skill_registry::SkillRegistry, DomainError> {
            Ok(self.registry.borrow().clone())
        }

        fn save(
            &self,
            registry: &workc_domain::skill_registry::SkillRegistry,
        ) -> Result<(), DomainError> {
            *self.registry.borrow_mut() = registry.clone();
            Ok(())
        }

        fn find_source(&self, id: &SkillSourceId) -> Result<Option<SkillSource>, DomainError> {
            Ok(self
                .registry
                .borrow()
                .sources
                .iter()
                .find(|source| source.id == *id)
                .cloned())
        }

        fn find_skill(&self, id: &SkillId) -> Result<Option<SkillDefinition>, DomainError> {
            Ok(self
                .registry
                .borrow()
                .skills
                .iter()
                .find(|skill| skill.id == *id)
                .cloned())
        }
    }

    struct FixedClock;

    impl Clock for FixedClock {
        fn now(&self) -> OffsetDateTime {
            OffsetDateTime::UNIX_EPOCH
        }
    }

    #[test]
    fn import_source_persists_source_and_skills() {
        let service = DefaultSkillRegistryApplicationService::new(
            Box::new(InMemorySkillRegistryRepository::default()),
            Box::new(FixedClock),
        );

        service
            .import_source(ImportSkillSourceCommand {
                source_id: "frontend-toolkit".to_owned(),
                kind: ApplicationSkillSourceKind::Local,
                location: "C:/skills/frontend-toolkit".to_owned(),
                reference: Some("2026-05-24".to_owned()),
                skills: vec![crate::skill_registry::ImportedSkillDefinition {
                    id: "frontend-testing".to_owned(),
                    versions: vec!["2026-05-22".to_owned()],
                    latest: Some("2026-05-22".to_owned()),
                }],
            })
            .unwrap();

        let shown = service
            .show_skill(ShowSkillQuery {
                skill_id: "frontend-testing".to_owned(),
            })
            .unwrap()
            .unwrap();

        assert_eq!(shown.id, "frontend-testing");
        assert_eq!(shown.source, "frontend-toolkit");
    }

    #[test]
    fn import_source_rejects_duplicate_source() {
        let service = DefaultSkillRegistryApplicationService::new(
            Box::new(InMemorySkillRegistryRepository::default()),
            Box::new(FixedClock),
        );
        service
            .import_source(ImportSkillSourceCommand {
                source_id: "frontend-toolkit".to_owned(),
                kind: ApplicationSkillSourceKind::Local,
                location: "C:/skills".to_owned(),
                reference: None,
                skills: vec![],
            })
            .unwrap();

        let result = service.import_source(ImportSkillSourceCommand {
            source_id: "frontend-toolkit".to_owned(),
            kind: ApplicationSkillSourceKind::Local,
            location: "C:/skills2".to_owned(),
            reference: None,
            skills: vec![],
        });
        assert!(
            matches!(result, Err(ApplicationError::Domain(DomainError::AlreadyExists { entity, .. })) if entity == EntityKind::Skill)
        );
    }

    #[test]
    fn import_source_rejects_duplicate_skill() {
        let service = DefaultSkillRegistryApplicationService::new(
            Box::new(InMemorySkillRegistryRepository::default()),
            Box::new(FixedClock),
        );
        service
            .import_source(ImportSkillSourceCommand {
                source_id: "frontend-toolkit".to_owned(),
                kind: ApplicationSkillSourceKind::Local,
                location: "C:/skills".to_owned(),
                reference: None,
                skills: vec![ImportedSkillDefinition {
                    id: "frontend-testing".to_owned(),
                    versions: vec![],
                    latest: None,
                }],
            })
            .unwrap();

        let result = service.import_source(ImportSkillSourceCommand {
            source_id: "frontend-toolkit-2".to_owned(),
            kind: ApplicationSkillSourceKind::Local,
            location: "C:/skills2".to_owned(),
            reference: None,
            skills: vec![ImportedSkillDefinition {
                id: "frontend-testing".to_owned(),
                versions: vec![],
                latest: None,
            }],
        });
        assert!(
            matches!(result, Err(ApplicationError::Domain(DomainError::AlreadyExists { entity, .. })) if entity == EntityKind::Skill)
        );
    }

    #[test]
    fn show_skill_returns_none_for_missing() {
        let service = DefaultSkillRegistryApplicationService::new(
            Box::new(InMemorySkillRegistryRepository::default()),
            Box::new(FixedClock),
        );
        let result = service
            .show_skill(ShowSkillQuery {
                skill_id: "nonexistent".to_owned(),
            })
            .unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn list_skill_versions_returns_empty_for_missing() {
        let service = DefaultSkillRegistryApplicationService::new(
            Box::new(InMemorySkillRegistryRepository::default()),
            Box::new(FixedClock),
        );
        let versions = service
            .list_skill_versions(ShowSkillQuery {
                skill_id: "nonexistent".to_owned(),
            })
            .unwrap();
        assert!(versions.is_empty());
    }

    #[test]
    fn list_skill_versions_returns_versions_for_existing_skill() {
        let repo = InMemorySkillRegistryRepository::default();
        repo.registry.borrow_mut().skills.push(SkillDefinition {
            id: SkillId::from("frontend-testing"),
            source: SkillSourceId::from("src"),
            versions: vec![
                SkillVersion::from("v1"),
                SkillVersion::from("v2"),
            ],
            latest: Some(SkillVersion::from("v2")),
        });
        let service = DefaultSkillRegistryApplicationService::new(Box::new(repo), Box::new(FixedClock));
        let versions = service
            .list_skill_versions(ShowSkillQuery {
                skill_id: "frontend-testing".to_owned(),
            })
            .unwrap();
        assert_eq!(versions, vec!["v1".to_owned(), "v2".to_owned()]);
    }

    #[test]
    fn import_source_with_archive_kind() {
        let service = DefaultSkillRegistryApplicationService::new(
            Box::new(InMemorySkillRegistryRepository::default()),
            Box::new(FixedClock),
        );
        let result = service.import_source(ImportSkillSourceCommand {
            source_id: "my-source".to_owned(),
            kind: ApplicationSkillSourceKind::Archive,
            location: "https://example.com/skill.zip".to_owned(),
            reference: None,
            skills: vec![ImportedSkillDefinition {
                id: "archive-skill".to_owned(),
                versions: vec![],
                latest: None,
            }],
        });
        assert!(result.is_ok());
    }

    #[test]
    fn import_source_with_other_kind() {
        let service = DefaultSkillRegistryApplicationService::new(
            Box::new(InMemorySkillRegistryRepository::default()),
            Box::new(FixedClock),
        );
        let result = service.import_source(ImportSkillSourceCommand {
            source_id: "custom-src".to_owned(),
            kind: ApplicationSkillSourceKind::Other("custom-type".to_owned()),
            location: "/tmp/skill".to_owned(),
            reference: None,
            skills: vec![ImportedSkillDefinition {
                id: "custom-skill".to_owned(),
                versions: vec![],
                latest: None,
            }],
        });
        assert!(result.is_ok());
    }

    #[test]
    fn import_source_with_local_kind() {
        let service = DefaultSkillRegistryApplicationService::new(
            Box::new(InMemorySkillRegistryRepository::default()),
            Box::new(FixedClock),
        );
        let result = service.import_source(ImportSkillSourceCommand {
            source_id: "local-src".to_owned(),
            kind: ApplicationSkillSourceKind::Local,
            location: "/home/user/skills/my-skill".to_owned(),
            reference: None,
            skills: vec![ImportedSkillDefinition {
                id: "local-skill".to_owned(),
                versions: vec![],
                latest: None,
            }],
        });
        assert!(result.is_ok());
    }
}
