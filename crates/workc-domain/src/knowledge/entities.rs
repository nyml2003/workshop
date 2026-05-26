use camino::Utf8PathBuf;

use crate::shared::{KnowledgeCandidateId, KnowledgeId, TaskSlug, Timestamp};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KnowledgeSourceRef {
    pub source_path: Utf8PathBuf,
    pub section: Option<String>,
    pub excerpt: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KnowledgeEntry {
    pub id: KnowledgeId,
    pub title: String,
    pub path: Utf8PathBuf,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub sources: Vec<KnowledgeSourceRef>,
    pub created_at: Option<Timestamp>,
    pub updated_at: Option<Timestamp>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KnowledgeCandidate {
    pub id: KnowledgeCandidateId,
    pub title: String,
    pub task_slug: TaskSlug,
    pub path: Utf8PathBuf,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub sources: Vec<KnowledgeSourceRef>,
    pub created_at: Option<Timestamp>,
    pub updated_at: Option<Timestamp>,
}
