use camino::Utf8Path;
use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};
use time::format_description::well_known::Rfc3339;
use workc_domain::errors::DomainError;
use workc_domain::errors::EntityKind;
use workc_domain::errors::FieldKind;
use workc_domain::knowledge::{
    KnowledgeBase, KnowledgeCandidate, KnowledgeEntry, KnowledgeRepository, KnowledgeSourceRef,
};
use workc_domain::shared::{KnowledgeCandidateId, KnowledgeId, TaskSlug, Timestamp};

use crate::fs::file_system::FileSystem;

use super::paths;

pub struct FsKnowledgeRepository {
    project_root: Utf8PathBuf,
    fs: Box<dyn FileSystem>,
}

#[derive(Debug, Serialize, Deserialize)]
struct KnowledgeMetaToml {
    id: String,
    #[serde(default)]
    task_slug: Option<String>,
    title: String,
    status: String,
    category: Option<String>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    created_at: Option<String>,
    #[serde(default)]
    updated_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SourceToml {
    task_slug: String,
    source_path: String,
    section: Option<String>,
    excerpt: Option<String>,
}

impl FsKnowledgeRepository {
    pub fn new(project_root: Utf8PathBuf, fs: Box<dyn FileSystem>) -> Self {
        Self { project_root, fs }
    }

    fn knowledge_root() -> Utf8PathBuf {
        paths::workc_knowledge_root()
    }

    fn candidates_root(&self, _task_slug: &TaskSlug) -> Utf8PathBuf {
        self.project_root.join("knowledge-candidates")
    }

    fn candidate_dir(
        &self,
        task_slug: &TaskSlug,
        candidate_id: &KnowledgeCandidateId,
    ) -> Utf8PathBuf {
        self.candidates_root(task_slug).join(candidate_id.as_str())
    }

    fn knowledge_dir(&self, knowledge_id: &KnowledgeId) -> Utf8PathBuf {
        Self::knowledge_root().join(knowledge_id.as_str())
    }

    fn meta_path(dir: &Utf8Path) -> Utf8PathBuf {
        dir.join("meta.toml")
    }

    fn sources_dir(dir: &Utf8Path) -> Utf8PathBuf {
        dir.join("sources")
    }

    fn write_sources(
        &self,
        dir: &Utf8Path,
        task_slug: &TaskSlug,
        sources: &[KnowledgeSourceRef],
    ) -> Result<(), DomainError> {
        let sources_dir = Self::sources_dir(dir);
        self.fs
            .create_dir_all(&sources_dir)
            .map_err(io_error("create knowledge sources dir"))?;

        for (index, source) in sources.iter().enumerate() {
            let file = sources_dir.join(format!("source-{index:03}.toml"));
            let raw = toml::to_string_pretty(&SourceToml {
                task_slug: task_slug.to_string(),
                source_path: source.source_path.to_string(),
                section: source.section.clone(),
                excerpt: source.excerpt.clone(),
            })
            .map_err(invalid_serialize("source.toml"))?;
            self.fs
                .write(&file, &raw)
                .map_err(io_error("write knowledge source"))?;
        }

        Ok(())
    }

    fn read_sources(&self, dir: &Utf8Path) -> Result<Vec<KnowledgeSourceRef>, DomainError> {
        let sources_dir = Self::sources_dir(dir);
        if !self.fs.exists(&sources_dir) {
            return Ok(Vec::new());
        }

        let mut sources = Vec::new();
        for path in self
            .fs
            .read_dir(&sources_dir)
            .map_err(io_error("read knowledge sources"))?
        {
            let raw = self
                .fs
                .read_to_string(&path)
                .map_err(io_error("read knowledge source"))?;
            let parsed = toml::from_str::<SourceToml>(&raw).map_err(invalid_toml("source.toml"))?;
            sources.push(KnowledgeSourceRef {
                source_path: parsed.source_path.into(),
                section: parsed.section,
                excerpt: parsed.excerpt,
            });
        }

        Ok(sources)
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
}

impl KnowledgeRepository for FsKnowledgeRepository {
    fn load(&self) -> Result<KnowledgeBase, DomainError> {
        let root = Self::knowledge_root();
        if !self.fs.exists(&root) {
            return Ok(KnowledgeBase::default());
        }

        let mut entries = Vec::new();
        for entry_path in self
            .fs
            .read_dir(&root)
            .map_err(io_error("read knowledge root"))?
        {
            if !self.fs.is_dir(&entry_path) {
                continue;
            }
            let raw = self
                .fs
                .read_to_string(&Self::meta_path(&entry_path))
                .map_err(io_error("read knowledge meta"))?;
            let meta =
                toml::from_str::<KnowledgeMetaToml>(&raw).map_err(invalid_toml("meta.toml"))?;
            let sources = self.read_sources(&entry_path)?;
            entries.push(KnowledgeEntry {
                id: KnowledgeId::from(meta.id),
                title: meta.title,
                path: entry_path,
                category: meta.category,
                tags: meta.tags,
                sources,
                created_at: Self::parse_timestamp(meta.created_at)?,
                updated_at: Self::parse_timestamp(meta.updated_at)?,
            });
        }

        Ok(KnowledgeBase { entries })
    }

    fn save(&self, knowledge_base: &KnowledgeBase) -> Result<(), DomainError> {
        self.fs
            .create_dir_all(&Self::knowledge_root())
            .map_err(io_error("create knowledge root"))?;
        for entry in &knowledge_base.entries {
            self.create_entry(entry)?;
        }
        Ok(())
    }

    fn list_candidates(
        &self,
        task_slug: &TaskSlug,
    ) -> Result<Vec<KnowledgeCandidate>, DomainError> {
        let root = self.candidates_root(task_slug);
        if !self.fs.exists(&root) {
            return Ok(Vec::new());
        }

        let mut candidates = Vec::new();
        for dir in self
            .fs
            .read_dir(&root)
            .map_err(io_error("read candidate root"))?
        {
            if !self.fs.is_dir(&dir) {
                continue;
            }
            let raw = self
                .fs
                .read_to_string(&Self::meta_path(&dir))
                .map_err(io_error("read candidate meta"))?;
            let meta =
                toml::from_str::<KnowledgeMetaToml>(&raw).map_err(invalid_toml("meta.toml"))?;
            let sources = self.read_sources(&dir)?;
            candidates.push(KnowledgeCandidate {
                id: KnowledgeCandidateId::from(meta.id),
                task_slug: TaskSlug::from(meta.task_slug.unwrap_or_else(|| task_slug.to_string())),
                title: meta.title,
                path: dir,
                category: meta.category,
                tags: meta.tags,
                sources,
                created_at: Self::parse_timestamp(meta.created_at)?,
                updated_at: Self::parse_timestamp(meta.updated_at)?,
            });
        }

        Ok(candidates)
    }

    fn create_candidate(&self, candidate: &KnowledgeCandidate) -> Result<(), DomainError> {
        let dir = self.candidate_dir(&candidate.task_slug, &candidate.id);
        self.fs
            .create_dir_all(&dir)
            .map_err(io_error("create candidate dir"))?;
        let raw = toml::to_string_pretty(&KnowledgeMetaToml {
            id: candidate.id.to_string(),
            task_slug: Some(candidate.task_slug.to_string()),
            title: candidate.title.clone(),
            status: "candidate".to_owned(),
            category: candidate.category.clone(),
            tags: candidate.tags.clone(),
            created_at: Self::format_timestamp(candidate.created_at)?,
            updated_at: Self::format_timestamp(candidate.updated_at)?,
        })
        .map_err(invalid_serialize("meta.toml"))?;
        self.fs
            .write(&Self::meta_path(&dir), &raw)
            .map_err(io_error("write candidate meta"))?;
        self.write_sources(&dir, &candidate.task_slug, &candidate.sources)?;
        Ok(())
    }

    fn update_candidate(
        &self,
        task_slug: &TaskSlug,
        candidate: &KnowledgeCandidate,
    ) -> Result<(), DomainError> {
        if candidate.task_slug != *task_slug {
            return Err(DomainError::Conflict {
                entity: EntityKind::KnowledgeCandidate,
                reason: "task_slug mismatch".to_owned(),
            });
        }
        self.create_candidate(candidate)
    }

    fn delete_candidate(
        &self,
        task_slug: &TaskSlug,
        candidate_id: &KnowledgeCandidateId,
    ) -> Result<(), DomainError> {
        let dir = self.candidate_dir(task_slug, candidate_id);
        if self.fs.exists(&dir) {
            self.fs
                .remove_dir_all(&dir)
                .map_err(io_error("delete candidate dir"))?;
        }
        Ok(())
    }

    fn find_candidate(
        &self,
        task_slug: &TaskSlug,
        candidate_id: &KnowledgeCandidateId,
    ) -> Result<Option<KnowledgeCandidate>, DomainError> {
        Ok(self
            .list_candidates(task_slug)?
            .into_iter()
            .find(|candidate| candidate.id == *candidate_id))
    }

    fn create_entry(&self, entry: &KnowledgeEntry) -> Result<(), DomainError> {
        self.fs
            .create_dir_all(&entry.path)
            .map_err(io_error("create knowledge dir"))?;
        let raw = toml::to_string_pretty(&KnowledgeMetaToml {
            id: entry.id.to_string(),
            task_slug: None,
            title: entry.title.clone(),
            status: "published".to_owned(),
            category: entry.category.clone(),
            tags: entry.tags.clone(),
            created_at: Self::format_timestamp(entry.created_at)?,
            updated_at: Self::format_timestamp(entry.updated_at)?,
        })
        .map_err(invalid_serialize("meta.toml"))?;
        self.fs
            .write(&Self::meta_path(&entry.path), &raw)
            .map_err(io_error("write knowledge meta"))?;
        self.write_sources(&entry.path, &TaskSlug::from("global"), &entry.sources)?;
        Ok(())
    }

    fn update_entry(&self, entry: &KnowledgeEntry) -> Result<(), DomainError> {
        self.create_entry(entry)
    }

    fn delete_entry(&self, id: &KnowledgeId) -> Result<(), DomainError> {
        let dir = self.knowledge_dir(id);
        if self.fs.exists(&dir) {
            self.fs
                .remove_dir_all(&dir)
                .map_err(io_error("delete knowledge dir"))?;
        }
        Ok(())
    }

    fn promote_candidate(
        &self,
        task_slug: &TaskSlug,
        candidate_id: &KnowledgeCandidateId,
        knowledge_id: &KnowledgeId,
    ) -> Result<(), DomainError> {
        let candidate = self
            .find_candidate(task_slug, candidate_id)?
            .ok_or_else(|| DomainError::NotFound {
                entity: EntityKind::KnowledgeCandidate,
                slug: candidate_id.to_string(),
            })?;
        let target = self.knowledge_dir(knowledge_id);
        let entry = KnowledgeEntry {
            id: knowledge_id.clone(),
            title: candidate.title.clone(),
            path: target,
            category: candidate.category.clone(),
            tags: candidate.tags.clone(),
            sources: candidate.sources.clone(),
            created_at: candidate.created_at,
            updated_at: candidate.updated_at,
        };
        self.create_entry(&entry)?;
        self.delete_candidate(task_slug, candidate_id)?;
        Ok(())
    }

    fn find_entry(&self, id: &KnowledgeId) -> Result<Option<KnowledgeEntry>, DomainError> {
        Ok(self
            .load()?
            .entries
            .into_iter()
            .find(|entry| entry.id == *id))
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
