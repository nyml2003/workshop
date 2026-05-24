use std::fs;

use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};
use time::format_description::well_known::Rfc3339;
use workc_domain::errors::DomainError;
use workc_domain::knowledge::{
    KnowledgeBase, KnowledgeCandidate, KnowledgeEntry, KnowledgeRepository, KnowledgeSourceRef,
};
use workc_domain::shared::{KnowledgeCandidateId, KnowledgeId, TaskId, Timestamp};

use super::paths;

pub struct FsKnowledgeRepository {
    project_root: Utf8PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
struct KnowledgeMetaToml {
    id: String,
    #[serde(default)]
    task_id: Option<String>,
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
    task_id: String,
    source_path: String,
    section: Option<String>,
    excerpt: Option<String>,
}

impl FsKnowledgeRepository {
    pub fn new(project_root: Utf8PathBuf) -> Self {
        Self { project_root }
    }

    fn knowledge_root() -> Utf8PathBuf {
        paths::workc_knowledge_root()
    }

    fn candidates_root(&self, _task_id: &TaskId) -> Utf8PathBuf {
        self.project_root.join("knowledge-candidates")
    }

    fn candidate_dir(&self, task_id: &TaskId, candidate_id: &KnowledgeCandidateId) -> Utf8PathBuf {
        self.candidates_root(task_id).join(candidate_id.as_str())
    }

    fn knowledge_dir(&self, knowledge_id: &KnowledgeId) -> Utf8PathBuf {
        Self::knowledge_root().join(knowledge_id.as_str())
    }

    fn meta_path(dir: &Utf8PathBuf) -> Utf8PathBuf {
        dir.join("meta.toml")
    }

    fn sources_dir(dir: &Utf8PathBuf) -> Utf8PathBuf {
        dir.join("sources")
    }

    fn write_sources(
        dir: &Utf8PathBuf,
        task_id: &TaskId,
        sources: &[KnowledgeSourceRef],
    ) -> Result<(), DomainError> {
        let sources_dir = Self::sources_dir(dir);
        fs::create_dir_all(&sources_dir).map_err(io_error("create knowledge sources dir"))?;

        for (index, source) in sources.iter().enumerate() {
            let file = sources_dir.join(format!("source-{index:03}.toml"));
            let raw = toml::to_string_pretty(&SourceToml {
                task_id: task_id.to_string(),
                source_path: source.source_path.to_string(),
                section: source.section.clone(),
                excerpt: source.excerpt.clone(),
            })
            .map_err(invalid_serialize("source.toml"))?;
            fs::write(file, raw).map_err(io_error("write knowledge source"))?;
        }

        Ok(())
    }

    fn read_sources(dir: &Utf8PathBuf) -> Result<Vec<KnowledgeSourceRef>, DomainError> {
        let sources_dir = Self::sources_dir(dir);
        if !sources_dir.exists() {
            return Ok(Vec::new());
        }

        let mut sources = Vec::new();
        for entry in fs::read_dir(sources_dir).map_err(io_error("read knowledge sources"))? {
            let entry = entry.map_err(io_error("iterate knowledge sources"))?;
            let raw =
                fs::read_to_string(entry.path()).map_err(io_error("read knowledge source"))?;
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
                        field: "timestamp",
                        reason: error.to_string(),
                    })
            })
            .transpose()
    }

    fn parse_timestamp(value: Option<String>) -> Result<Option<Timestamp>, DomainError> {
        value
            .map(|raw| {
                Timestamp::parse(&raw, &Rfc3339).map_err(|error| DomainError::InvalidInput {
                    field: "timestamp",
                    reason: error.to_string(),
                })
            })
            .transpose()
    }
}

impl KnowledgeRepository for FsKnowledgeRepository {
    fn load(&self) -> Result<KnowledgeBase, DomainError> {
        let root = Self::knowledge_root();
        if !root.exists() {
            return Ok(KnowledgeBase::default());
        }

        let mut entries = Vec::new();
        for entry in fs::read_dir(&root).map_err(io_error("read knowledge root"))? {
            let entry = entry.map_err(io_error("iterate knowledge root"))?;
            let dir = Utf8PathBuf::from_path_buf(entry.path()).map_err(|path| {
                DomainError::InvalidInput {
                    field: "knowledge path",
                    reason: path.display().to_string(),
                }
            })?;
            if !dir.is_dir() {
                continue;
            }
            let raw = fs::read_to_string(Self::meta_path(&dir))
                .map_err(io_error("read knowledge meta"))?;
            let meta =
                toml::from_str::<KnowledgeMetaToml>(&raw).map_err(invalid_toml("meta.toml"))?;
            let sources = Self::read_sources(&dir)?;
            entries.push(KnowledgeEntry {
                id: KnowledgeId::from(meta.id),
                title: meta.title,
                path: dir,
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
        fs::create_dir_all(Self::knowledge_root()).map_err(io_error("create knowledge root"))?;
        for entry in &knowledge_base.entries {
            self.create_entry(entry)?;
        }
        Ok(())
    }

    fn list_candidates(&self, task_id: &TaskId) -> Result<Vec<KnowledgeCandidate>, DomainError> {
        let root = self.candidates_root(task_id);
        if !root.exists() {
            return Ok(Vec::new());
        }

        let mut candidates = Vec::new();
        for entry in fs::read_dir(&root).map_err(io_error("read candidate root"))? {
            let entry = entry.map_err(io_error("iterate candidate root"))?;
            let dir = Utf8PathBuf::from_path_buf(entry.path()).map_err(|path| {
                DomainError::InvalidInput {
                    field: "candidate path",
                    reason: path.display().to_string(),
                }
            })?;
            if !dir.is_dir() {
                continue;
            }
            let raw = fs::read_to_string(Self::meta_path(&dir))
                .map_err(io_error("read candidate meta"))?;
            let meta =
                toml::from_str::<KnowledgeMetaToml>(&raw).map_err(invalid_toml("meta.toml"))?;
            let sources = Self::read_sources(&dir)?;
            candidates.push(KnowledgeCandidate {
                id: KnowledgeCandidateId::from(meta.id),
                task_id: TaskId::from(meta.task_id.unwrap_or_else(|| task_id.to_string())),
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
        let dir = self.candidate_dir(&candidate.task_id, &candidate.id);
        fs::create_dir_all(&dir).map_err(io_error("create candidate dir"))?;
        let raw = toml::to_string_pretty(&KnowledgeMetaToml {
            id: candidate.id.to_string(),
            task_id: Some(candidate.task_id.to_string()),
            title: candidate.title.clone(),
            status: "candidate".to_owned(),
            category: candidate.category.clone(),
            tags: candidate.tags.clone(),
            created_at: Self::format_timestamp(candidate.created_at)?,
            updated_at: Self::format_timestamp(candidate.updated_at)?,
        })
        .map_err(invalid_serialize("meta.toml"))?;
        fs::write(Self::meta_path(&dir), raw).map_err(io_error("write candidate meta"))?;
        Self::write_sources(&dir, &candidate.task_id, &candidate.sources)?;
        Ok(())
    }

    fn update_candidate(
        &self,
        task_id: &TaskId,
        candidate: &KnowledgeCandidate,
    ) -> Result<(), DomainError> {
        if candidate.task_id != *task_id {
            return Err(DomainError::Conflict {
                entity: "knowledge-candidate",
                reason: "task_id mismatch".to_owned(),
            });
        }
        self.create_candidate(candidate)
    }

    fn delete_candidate(
        &self,
        task_id: &TaskId,
        candidate_id: &KnowledgeCandidateId,
    ) -> Result<(), DomainError> {
        let dir = self.candidate_dir(task_id, candidate_id);
        if dir.exists() {
            fs::remove_dir_all(dir).map_err(io_error("delete candidate dir"))?;
        }
        Ok(())
    }

    fn find_candidate(
        &self,
        task_id: &TaskId,
        candidate_id: &KnowledgeCandidateId,
    ) -> Result<Option<KnowledgeCandidate>, DomainError> {
        Ok(self
            .list_candidates(task_id)?
            .into_iter()
            .find(|candidate| candidate.id == *candidate_id))
    }

    fn create_entry(&self, entry: &KnowledgeEntry) -> Result<(), DomainError> {
        fs::create_dir_all(&entry.path).map_err(io_error("create knowledge dir"))?;
        let raw = toml::to_string_pretty(&KnowledgeMetaToml {
            id: entry.id.to_string(),
            task_id: None,
            title: entry.title.clone(),
            status: "published".to_owned(),
            category: entry.category.clone(),
            tags: entry.tags.clone(),
            created_at: Self::format_timestamp(entry.created_at)?,
            updated_at: Self::format_timestamp(entry.updated_at)?,
        })
        .map_err(invalid_serialize("meta.toml"))?;
        fs::write(Self::meta_path(&entry.path), raw).map_err(io_error("write knowledge meta"))?;
        Self::write_sources(&entry.path, &TaskId::from("global"), &entry.sources)?;
        Ok(())
    }

    fn update_entry(&self, entry: &KnowledgeEntry) -> Result<(), DomainError> {
        self.create_entry(entry)
    }

    fn delete_entry(&self, id: &KnowledgeId) -> Result<(), DomainError> {
        let dir = self.knowledge_dir(id);
        if dir.exists() {
            fs::remove_dir_all(dir).map_err(io_error("delete knowledge dir"))?;
        }
        Ok(())
    }

    fn promote_candidate(
        &self,
        task_id: &TaskId,
        candidate_id: &KnowledgeCandidateId,
        knowledge_id: &KnowledgeId,
    ) -> Result<(), DomainError> {
        let candidate =
            self.find_candidate(task_id, candidate_id)?
                .ok_or_else(|| DomainError::NotFound {
                    entity: "knowledge-candidate",
                    id: candidate_id.to_string(),
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
        self.delete_candidate(task_id, candidate_id)?;
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
