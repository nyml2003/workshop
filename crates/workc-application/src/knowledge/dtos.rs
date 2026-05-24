use camino::Utf8PathBuf;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CreateKnowledgeCandidateCommand {
    pub task_id: String,
    pub candidate_id: String,
    pub title: String,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub source_paths: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UpdateKnowledgeCandidateMetaCommand {
    pub task_id: String,
    pub candidate_id: String,
    pub title: Option<String>,
    pub category: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShowKnowledgeCandidateQuery {
    pub task_id: String,
    pub candidate_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeleteKnowledgeCandidateCommand {
    pub task_id: String,
    pub candidate_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PromoteKnowledgeCommand {
    pub task_id: String,
    pub candidate_id: String,
    pub knowledge_id: String,
    pub title: Option<String>,
    pub category: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ListKnowledgeCandidatesQuery {
    pub task_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ListKnowledgeQuery;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShowKnowledgeQuery {
    pub knowledge_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UpdateKnowledgeMetaCommand {
    pub knowledge_id: String,
    pub title: Option<String>,
    pub category: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeleteKnowledgeCommand {
    pub knowledge_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct KnowledgeObjectSummary {
    pub id: String,
    pub title: String,
    pub path: Utf8PathBuf,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub source_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CandidateMutationResult {
    pub task_id: String,
    pub candidate: KnowledgeObjectSummary,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct KnowledgeMutationResult {
    pub knowledge: KnowledgeObjectSummary,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PromoteKnowledgeResult {
    pub task_id: String,
    pub candidate_id: String,
    pub knowledge: KnowledgeObjectSummary,
}
