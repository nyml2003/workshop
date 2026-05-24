use crate::errors::DomainError;
use crate::shared::{MountId, TaskId, TaskSlug};

use super::aggregate::TaskWorkspace;
use super::entities::TaskSkillMount;

pub trait TaskRepository {
    fn find_by_id(&self, id: &TaskId) -> Result<Option<TaskWorkspace>, DomainError>;
    fn find_by_slug(&self, slug: &TaskSlug) -> Result<Option<TaskWorkspace>, DomainError>;
    fn list(&self) -> Result<Vec<TaskWorkspace>, DomainError>;
    fn save(&self, task: &TaskWorkspace) -> Result<(), DomainError>;
}

pub trait TaskSkillMountRepository {
    fn list_for_task(&self, task_id: &TaskId) -> Result<Vec<TaskSkillMount>, DomainError>;
    fn save_for_task(&self, task_id: &TaskId, mounts: &[TaskSkillMount])
    -> Result<(), DomainError>;
    fn remove_for_task(&self, task_id: &TaskId, mount_id: &MountId) -> Result<(), DomainError>;
}
