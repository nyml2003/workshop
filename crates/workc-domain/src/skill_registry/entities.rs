use camino::Utf8PathBuf;

use crate::shared::{SkillId, SkillSourceId, SkillVersion, Timestamp};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SkillSourceKind {
    Git,
    Local,
    Archive,
    Other(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SkillSource {
    pub id: SkillSourceId,
    pub kind: SkillSourceKind,
    pub location: String,
    pub reference: Option<String>,
    pub imported_at: Option<Timestamp>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SkillDefinition {
    pub id: SkillId,
    pub source: SkillSourceId,
    pub versions: Vec<SkillVersion>,
    pub latest: Option<SkillVersion>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrepareStep {
    pub name: String,
    pub action_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UseStep {
    pub name: String,
    pub action_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SkillExecutionStatus {
    Pending,
    Success,
    Failed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrepareResult {
    pub status: SkillExecutionStatus,
    pub artifact_path: Option<Utf8PathBuf>,
    pub log_path: Option<Utf8PathBuf>,
    pub finished_at: Option<Timestamp>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UseResult {
    pub status: SkillExecutionStatus,
    pub log_path: Option<Utf8PathBuf>,
    pub finished_at: Option<Timestamp>,
}
