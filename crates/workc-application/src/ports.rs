use std::error::Error;
use std::fmt::{Display, Formatter};

use camino::{Utf8Path, Utf8PathBuf};
use serde::Serialize;
use workc_domain::shared::Timestamp;
use workc_domain::skill_registry::entities::{
    PrepareResult, PrepareStep, SkillExecutionStatus, UseResult, UseStep,
};

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
pub struct PrepareStatusRecord {
    pub status: SkillExecutionStatus,
    pub last_run_at: Option<Timestamp>,
    pub artifact_path: Option<Utf8PathBuf>,
    pub log_path: Option<Utf8PathBuf>,
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeError {
    pub detail: String,
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.detail)
    }
}

impl Error for RuntimeError {}

pub trait Clock {
    fn now(&self) -> Timestamp;
}

pub trait GitClient {
    fn clone_repo(&self, path: &Utf8Path, url: &str) -> Result<(), GitError>;
    fn get_repo_status(&self, path: &Utf8Path) -> Result<RepoStatus, GitError>;
    fn fetch_repo(&self, path: &Utf8Path) -> Result<(), GitError>;
    fn pull_repo(&self, path: &Utf8Path) -> Result<(), GitError>;
}

pub trait SkillRuntime {
    fn prepare(
        &self,
        mount_path: &Utf8Path,
        step: PrepareStep,
    ) -> Result<PrepareResult, RuntimeError>;
    fn use_skill(&self, mount_path: &Utf8Path, step: UseStep) -> Result<UseResult, RuntimeError>;
    fn check_prepare_status(
        &self,
        mount_path: &Utf8Path,
    ) -> Result<PrepareStatusRecord, RuntimeError>;
}
