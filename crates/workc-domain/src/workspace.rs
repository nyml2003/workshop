use crate::errors::DomainError;
use crate::shared::{TaskSlug, Timestamp};
use camino::Utf8PathBuf;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkspaceEntry {
    pub slug: TaskSlug,
    pub path: Utf8PathBuf,
    pub title: String,
    pub status: WorkspaceStatus,
    pub last_activity_at: Option<Timestamp>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WorkspaceStatus {
    Active,
    Closed,
    Archived,
}

impl WorkspaceStatus {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Active => "active",
            Self::Closed => "closed",
            Self::Archived => "archived",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "active" => Some(Self::Active),
            "closed" => Some(Self::Closed),
            "archived" => Some(Self::Archived),
            _ => None,
        }
    }
}

pub trait WorkspaceRegistryRepository {
    fn load(&self) -> Result<Vec<WorkspaceEntry>, DomainError>;
    fn save(&self, entries: &[WorkspaceEntry]) -> Result<(), DomainError>;
}
