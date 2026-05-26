use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Draft,
    Active,
    Closed,
    Archived,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskSkillMountStatus {
    Active,
    Inactive,
    Removed,
}

impl Display for TaskSkillMountStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "active"),
            Self::Inactive => write!(f, "inactive"),
            Self::Removed => write!(f, "removed"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_active() {
        assert_eq!(TaskSkillMountStatus::Active.to_string(), "active");
    }

    #[test]
    fn display_inactive() {
        assert_eq!(TaskSkillMountStatus::Inactive.to_string(), "inactive");
    }

    #[test]
    fn display_removed() {
        assert_eq!(TaskSkillMountStatus::Removed.to_string(), "removed");
    }
}
