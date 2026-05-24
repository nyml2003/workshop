use super::entities::{SkillDefinition, SkillSource};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SkillRegistry {
    pub sources: Vec<SkillSource>,
    pub skills: Vec<SkillDefinition>,
}

impl Default for SkillRegistry {
    fn default() -> Self {
        Self {
            sources: Vec::new(),
            skills: Vec::new(),
        }
    }
}
