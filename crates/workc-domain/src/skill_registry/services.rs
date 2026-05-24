use camino::Utf8Path;

use crate::errors::DomainError;
use crate::shared::{MountId, SkillVersion};

use super::entities::SkillDefinition;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlannedSkillMount {
    pub mount_id: MountId,
    pub version: SkillVersion,
}

pub trait SkillMountPlanner {
    fn plan_mount(
        &self,
        task_skill_root: &Utf8Path,
        skill: &SkillDefinition,
        requested_version: Option<&SkillVersion>,
    ) -> Result<PlannedSkillMount, DomainError>;
}
