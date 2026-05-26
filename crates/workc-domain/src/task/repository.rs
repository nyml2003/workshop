use crate::errors::DomainError;
use crate::shared::{MountId, TaskSlug};

use super::aggregate::TaskWorkspace;
use super::entities::TaskSkillMount;

pub trait TaskRepository {
    fn find(&self, slug: &TaskSlug) -> Result<Option<TaskWorkspace>, DomainError>;
    fn list(&self) -> Result<Vec<TaskWorkspace>, DomainError>;
    fn save(&self, task: &TaskWorkspace) -> Result<(), DomainError>;
}

pub trait TaskSkillMountRepository {
    fn list_for_task(&self, slug: &TaskSlug) -> Result<Vec<TaskSkillMount>, DomainError>;
    fn save_for_task(&self, slug: &TaskSlug, mounts: &[TaskSkillMount]) -> Result<(), DomainError>;
    fn remove_for_task(&self, slug: &TaskSlug, mount_id: &MountId) -> Result<(), DomainError>;
}
