use crate::shared::{RepoGroupId, RepoId};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RepoEntry {
    pub id: RepoId,
    pub url: String,
    pub tags: Vec<String>,
    pub description: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RepoGroup {
    pub id: RepoGroupId,
    pub repos: Vec<RepoId>,
    pub tags: Vec<String>,
    pub description: Option<String>,
}
