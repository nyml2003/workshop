use super::entities::{RepoEntry, RepoGroup};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RepoCatalog {
    pub repos: Vec<RepoEntry>,
    pub groups: Vec<RepoGroup>,
}
