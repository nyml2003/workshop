use camino::Utf8PathBuf;
use serde::Serialize;
use workc_domain::shared::SkillVersion;
use workc_domain::task::TaskSkillMountStatus;

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
    pub status: TaskSkillMountStatus,
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SkillUpdateStatus {
    pub mount_id: String,
    pub update_available: bool,
    pub target_version: Option<SkillVersion>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SkillSandboxHandle {
    pub mount_id: String,
    pub path: Utf8PathBuf,
}
