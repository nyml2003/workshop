use std::error::Error;
use std::fmt::{Display, Formatter};

use camino::{Utf8Path, Utf8PathBuf};
use serde::Serialize;
use workc_domain::shared::Timestamp;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub enum CloneState {
    Missing,
    Ready,
    Dirty,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RepoStatus {
    pub branch: Option<String>,
    pub dirty: bool,
    pub ahead: usize,
    pub behind: usize,
    pub clone_state: CloneState,
}

pub trait EditorLauncher {
    fn open_dir(&self, path: &Utf8Path, editor: &str) -> Result<(), EditorError>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GitError {
    pub detail: String,
}

impl Display for GitError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.detail)
    }
}

impl Error for GitError {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorError {
    pub detail: String,
}

impl Display for EditorError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.detail)
    }
}

impl Error for EditorError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn git_error_display() {
        assert_eq!(
            GitError { detail: "oops".to_owned() }.to_string(),
            "oops"
        );
    }

    #[test]
    fn editor_error_display() {
        assert_eq!(
            EditorError { detail: "fail".to_owned() }.to_string(),
            "fail"
        );
    }
}

pub trait Clock {
    fn now(&self) -> Timestamp;
}

pub trait GitClient {
    fn clone_repo(&self, path: &Utf8Path, url: &str) -> Result<(), GitError>;
    fn get_repo_status(&self, path: &Utf8Path) -> Result<RepoStatus, GitError>;
    fn fetch_repo(&self, path: &Utf8Path) -> Result<(), GitError>;
    fn pull_repo(&self, path: &Utf8Path) -> Result<(), GitError>;
}
