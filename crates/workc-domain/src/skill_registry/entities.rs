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

use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkillExecutionStatus {
    Pending,
    Success,
    Failed,
}

impl Display for SkillExecutionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::Success => write!(f, "success"),
            Self::Failed => write!(f, "failed"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_pending() {
        assert_eq!(SkillExecutionStatus::Pending.to_string(), "pending");
    }

    #[test]
    fn display_success() {
        assert_eq!(SkillExecutionStatus::Success.to_string(), "success");
    }

    #[test]
    fn display_failed() {
        assert_eq!(SkillExecutionStatus::Failed.to_string(), "failed");
    }
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
