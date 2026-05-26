use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntityKind {
    Task,
    Skill,
    Knowledge,
    KnowledgeCandidate,
    Repo,
    RepoGroup,
    Mount,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldKind {
    Slug,
    Title,
    Template,
    Tags,
    Name,
    Url,
    Path,
    Status,
    Timestamp,
    Other(&'static str),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainError {
    NotFound {
        entity: EntityKind,
        slug: String,
    },
    AlreadyExists {
        entity: EntityKind,
        slug: String,
    },
    InvalidInput {
        field: FieldKind,
        reason: String,
    },
    Conflict {
        entity: EntityKind,
        reason: String,
    },
    PersistenceFailed {
        operation: &'static str,
        detail: String,
    },
}

impl Display for DomainError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound { entity, slug } => write!(f, "{entity:?} not found: {slug}"),
            Self::AlreadyExists { entity, slug } => write!(f, "{entity:?} already exists: {slug}"),
            Self::InvalidInput { field, reason } => {
                write!(f, "invalid input for {field:?}: {reason}")
            }
            Self::Conflict { entity, reason } => write!(f, "{entity:?} conflict: {reason}"),
            Self::PersistenceFailed { operation, detail } => {
                write!(f, "persistence error during {operation}: {detail}")
            }
        }
    }
}

impl Error for DomainError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_found_display() {
        let err = DomainError::NotFound {
            entity: EntityKind::Task,
            slug: "abc12345".to_owned(),
        };
        assert_eq!(err.to_string(), "Task not found: abc12345");
    }

    #[test]
    fn already_exists_display() {
        let err = DomainError::AlreadyExists {
            entity: EntityKind::Repo,
            slug: "api-gateway".to_owned(),
        };
        assert_eq!(err.to_string(), "Repo already exists: api-gateway");
    }

    #[test]
    fn invalid_input_display() {
        let err = DomainError::InvalidInput {
            field: FieldKind::Slug,
            reason: "slug cannot be empty".to_owned(),
        };
        assert_eq!(
            err.to_string(),
            "invalid input for Slug: slug cannot be empty"
        );
    }

    #[test]
    fn conflict_display() {
        let err = DomainError::Conflict {
            entity: EntityKind::Task,
            reason: "already closed".to_owned(),
        };
        assert_eq!(err.to_string(), "Task conflict: already closed");
    }

    #[test]
    fn persistence_failed_display() {
        let err = DomainError::PersistenceFailed {
            operation: "write",
            detail: "disk full".to_owned(),
        };
        assert_eq!(err.to_string(), "persistence error during write: disk full");
    }
}
