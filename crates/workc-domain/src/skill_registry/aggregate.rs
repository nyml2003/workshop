use super::entities::{SkillDefinition, SkillSource};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SkillRegistry {
    pub sources: Vec<SkillSource>,
    pub skills: Vec<SkillDefinition>,
}
