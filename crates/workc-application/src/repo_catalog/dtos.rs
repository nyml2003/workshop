use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AddRepoCommand {
    pub id: String,
    pub url: String,
    pub tags: Vec<String>,
    pub description: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RepoSummary {
    pub id: String,
    pub url: String,
    pub tags: Vec<String>,
    pub description: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AddRepoGroupCommand {
    pub id: String,
    pub repos: Vec<String>,
    pub tags: Vec<String>,
    pub description: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RepoGroupSummary {
    pub id: String,
    pub repos: Vec<String>,
    pub tags: Vec<String>,
    pub description: Option<String>,
}
