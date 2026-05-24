use camino::Utf8PathBuf;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MountSkillCommand {
    pub task_id: String,
    pub skill_id: String,
    pub version: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SkillMountSummary {
    pub task_id: String,
    pub mount_id: String,
    pub skill_id: String,
    pub version: String,
    pub source: String,
    pub status: String,
    pub path: Utf8PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnmountSkillCommand {
    pub task_id: String,
    pub mount_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OverrideSkillCommand {
    pub task_id: String,
    pub mount_id: String,
    pub relative_path: Utf8PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckSkillUpdatesQuery {
    pub task_id: String,
    pub mount_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UpdateSkillCommand {
    pub task_id: String,
    pub mount_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SandboxSkillCommand {
    pub task_id: String,
    pub mount_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrepareSkillCommand {
    pub task_id: String,
    pub mount_id: String,
    pub step: RuntimeStep,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UseSkillCommand {
    pub task_id: String,
    pub mount_id: String,
    pub step: RuntimeStep,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrepareStatusQuery {
    pub task_id: String,
    pub mount_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SkillUpdateStatus {
    pub mount_id: String,
    pub update_available: bool,
    pub target_version: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SkillSandboxHandle {
    pub mount_id: String,
    pub path: Utf8PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SkillPreparation {
    pub mount_id: String,
    pub status: String,
    pub artifact_path: Option<Utf8PathBuf>,
    pub log_path: Option<Utf8PathBuf>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SkillUseExecution {
    pub mount_id: String,
    pub status: String,
    pub log_path: Option<Utf8PathBuf>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeStep {
    pub name: String,
    pub action_id: String,
}
