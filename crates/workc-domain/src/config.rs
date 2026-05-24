use crate::errors::DomainError;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WorkcConfig {
    pub knowledge_remote: Option<String>,
}

pub trait ConfigRepository {
    fn load(&self) -> Result<WorkcConfig, DomainError>;
    fn save(&self, config: &WorkcConfig) -> Result<(), DomainError>;
}
