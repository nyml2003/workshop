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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn as_str_active() {
        assert_eq!(WorkspaceStatus::Active.as_str(), "active");
    }

    #[test]
    fn as_str_closed() {
        assert_eq!(WorkspaceStatus::Closed.as_str(), "closed");
    }

    #[test]
    fn as_str_archived() {
        assert_eq!(WorkspaceStatus::Archived.as_str(), "archived");
    }

    #[test]
    fn parse_active() {
        assert_eq!(WorkspaceStatus::parse("active"), Some(WorkspaceStatus::Active));
    }

    #[test]
    fn parse_closed() {
        assert_eq!(WorkspaceStatus::parse("closed"), Some(WorkspaceStatus::Closed));
    }

    #[test]
    fn parse_archived() {
        assert_eq!(WorkspaceStatus::parse("archived"), Some(WorkspaceStatus::Archived));
    }

    #[test]
    fn parse_unknown_returns_none() {
        assert_eq!(WorkspaceStatus::parse("unknown"), None);
    }
}
