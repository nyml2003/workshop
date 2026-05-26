use camino::{Utf8Path, Utf8PathBuf};
use serde::{Deserialize, Serialize};
use time::format_description::well_known::Rfc3339;
use workc_domain::errors::DomainError;
use workc_domain::errors::FieldKind;
use workc_domain::shared::{SkillId, SkillSourceId, SkillVersion, Timestamp};
use workc_domain::skill_registry::{
    SkillDefinition, SkillRegistry, SkillRegistryRepository, SkillSource, SkillSourceKind,
};

use crate::fs::file_system::FileSystem;

use super::paths;

pub struct FsSkillRegistryRepository {
    fs: Box<dyn FileSystem>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct SourcesToml {
    #[serde(default)]
    sources: Vec<SourceToml>,
}
#[derive(Debug, Serialize, Deserialize, Default)]
struct SkillsToml {
    #[serde(default)]
    skills: Vec<SkillToml>,
}
#[derive(Debug, Serialize, Deserialize)]
struct SourceToml {
    id: String,
    kind: String,
    location: String,
    reference: Option<String>,
    imported_at: Option<String>,
}
#[derive(Debug, Serialize, Deserialize)]
struct SkillToml {
    id: String,
    source: String,
    #[serde(default)]
    versions: Vec<String>,
    latest: Option<String>,
}
impl FsSkillRegistryRepository {
    pub fn new(fs: Box<dyn FileSystem>) -> Self {
        Self { fs }
    }
}
impl Default for FsSkillRegistryRepository {
    fn default() -> Self {
        Self {
            fs: Box::new(crate::fs::real_fs::RealFileSystem),
        }
    }
}
impl FsSkillRegistryRepository {
    fn registry_root() -> Utf8PathBuf {
        paths::workc_skills_registry_root()
    }
    pub fn cache_dir(&self, source_id: &str, version: &str) -> Utf8PathBuf {
        paths::workc_skills_cache_root()
            .join(source_id)
            .join(version)
    }
    pub fn cache_local_source(
        &self,
        source_id: &str,
        version: &str,
        src_dir: &Utf8Path,
    ) -> Result<(), DomainError> {
        let cache_dir = paths::workc_skills_cache_root()
            .join(source_id)
            .join(version);
        self.fs
            .create_dir_all(&cache_dir)
            .map_err(|e| DomainError::PersistenceFailed {
                operation: "CreateDir",
                detail: e.to_string(),
            })?;
        self.fs
            .copy_dir(src_dir, &cache_dir)
            .map_err(|e| DomainError::PersistenceFailed {
                operation: "CopyFile",
                detail: e.to_string(),
            })?;
        Ok(())
    }
    fn sources_path() -> Utf8PathBuf {
        Self::registry_root().join("sources.toml")
    }
    fn skills_path() -> Utf8PathBuf {
        Self::registry_root().join("skills.toml")
    }
    fn format_timestamp(value: Option<Timestamp>) -> Result<Option<String>, DomainError> {
        value
            .map(|timestamp| {
                timestamp
                    .format(&Rfc3339)
                    .map_err(|error| DomainError::InvalidInput {
                        field: FieldKind::Timestamp,
                        reason: error.to_string(),
                    })
            })
            .transpose()
    }
    fn parse_timestamp(value: Option<String>) -> Result<Option<Timestamp>, DomainError> {
        value
            .map(|raw| {
                Timestamp::parse(&raw, &Rfc3339).map_err(|error| DomainError::InvalidInput {
                    field: FieldKind::Timestamp,
                    reason: error.to_string(),
                })
            })
            .transpose()
    }
    fn source_kind_to_string(kind: &SkillSourceKind) -> String {
        match kind {
            SkillSourceKind::Git => "git".to_owned(),
            SkillSourceKind::Local => "local".to_owned(),
            SkillSourceKind::Archive => "archive".to_owned(),
            SkillSourceKind::Other(value) => value.clone(),
        }
    }
    fn source_kind_from_string(value: &str) -> SkillSourceKind {
        match value {
            "git" => SkillSourceKind::Git,
            "local" => SkillSourceKind::Local,
            "archive" => SkillSourceKind::Archive,
            other => SkillSourceKind::Other(other.to_owned()),
        }
    }
}
impl SkillRegistryRepository for FsSkillRegistryRepository {
    fn load(&self) -> Result<SkillRegistry, DomainError> {
        let root = Self::registry_root();
        if !self.fs.exists(&root) {
            return Ok(SkillRegistry::default());
        }
        let sources = if self.fs.exists(&Self::sources_path()) {
            let raw = self
                .fs
                .read_to_string(&Self::sources_path())
                .map_err(io_error("read skill sources"))?;
            toml::from_str::<SourcesToml>(&raw)
                .map_err(invalid_toml("sources.toml"))?
                .sources
                .into_iter()
                .map(|source| {
                    Ok(SkillSource {
                        id: SkillSourceId::from(source.id),
                        kind: Self::source_kind_from_string(&source.kind),
                        location: source.location,
                        reference: source.reference,
                        imported_at: Self::parse_timestamp(source.imported_at)?,
                    })
                })
                .collect::<Result<Vec<_>, DomainError>>()?
        } else {
            Vec::new()
        };
        let skills = if self.fs.exists(&Self::skills_path()) {
            let raw = self
                .fs
                .read_to_string(&Self::skills_path())
                .map_err(io_error("read skills index"))?;
            toml::from_str::<SkillsToml>(&raw)
                .map_err(invalid_toml("skills.toml"))?
                .skills
                .into_iter()
                .map(|skill| SkillDefinition {
                    id: SkillId::from(skill.id),
                    source: SkillSourceId::from(skill.source),
                    versions: skill.versions.into_iter().map(SkillVersion::from).collect(),
                    latest: skill.latest.map(SkillVersion::from),
                })
                .collect()
        } else {
            Vec::new()
        };
        Ok(SkillRegistry { sources, skills })
    }
    fn save(&self, registry: &SkillRegistry) -> Result<(), DomainError> {
        self.fs
            .create_dir_all(&Self::registry_root())
            .map_err(io_error("create skill registry root"))?;
        let sources = SourcesToml {
            sources: registry
                .sources
                .iter()
                .map(|source| {
                    Ok(SourceToml {
                        id: source.id.to_string(),
                        kind: Self::source_kind_to_string(&source.kind),
                        location: source.location.clone(),
                        reference: source.reference.clone(),
                        imported_at: Self::format_timestamp(source.imported_at)?,
                    })
                })
                .collect::<Result<Vec<_>, DomainError>>()?,
        };
        let skills = SkillsToml {
            skills: registry
                .skills
                .iter()
                .map(|skill| SkillToml {
                    id: skill.id.to_string(),
                    source: skill.source.to_string(),
                    versions: skill.versions.iter().map(ToString::to_string).collect(),
                    latest: skill.latest.as_ref().map(ToString::to_string),
                })
                .collect(),
        };
        self.fs
            .write(
                &Self::sources_path(),
                &toml::to_string_pretty(&sources).map_err(invalid_serialize("sources.toml"))?,
            )
            .map_err(io_error("write skill sources"))?;
        self.fs
            .write(
                &Self::skills_path(),
                &toml::to_string_pretty(&skills).map_err(invalid_serialize("skills.toml"))?,
            )
            .map_err(io_error("write skills index"))?;
        Ok(())
    }
    fn find_source(&self, id: &SkillSourceId) -> Result<Option<SkillSource>, DomainError> {
        Ok(self
            .load()?
            .sources
            .into_iter()
            .find(|source| source.id == *id))
    }
    fn find_skill(&self, id: &SkillId) -> Result<Option<SkillDefinition>, DomainError> {
        Ok(self
            .load()?
            .skills
            .into_iter()
            .find(|skill| skill.id == *id))
    }
}

fn io_error(operation: &'static str) -> impl Fn(std::io::Error) -> DomainError {
    move |error| DomainError::PersistenceFailed {
        operation,
        detail: error.to_string(),
    }
}
fn invalid_toml(field: &'static str) -> impl Fn(toml::de::Error) -> DomainError {
    move |error| DomainError::InvalidInput {
        field: FieldKind::Other(field),
        reason: error.to_string(),
    }
}
fn invalid_serialize(field: &'static str) -> impl Fn(toml::ser::Error) -> DomainError {
    move |error| DomainError::InvalidInput {
        field: FieldKind::Other(field),
        reason: error.to_string(),
    }
}
