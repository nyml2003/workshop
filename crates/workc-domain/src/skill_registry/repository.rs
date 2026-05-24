use crate::errors::DomainError;
use crate::shared::{SkillId, SkillSourceId};

use super::aggregate::SkillRegistry;
use super::entities::{SkillDefinition, SkillSource};

pub trait SkillRegistryRepository {
    fn load(&self) -> Result<SkillRegistry, DomainError>;
    fn save(&self, registry: &SkillRegistry) -> Result<(), DomainError>;
    fn find_source(&self, id: &SkillSourceId) -> Result<Option<SkillSource>, DomainError>;
    fn find_skill(&self, id: &SkillId) -> Result<Option<SkillDefinition>, DomainError>;
}
