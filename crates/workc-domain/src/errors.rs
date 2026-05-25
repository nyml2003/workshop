use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainError {
    NotFound {
        entity: &'static str,
        id: String,
    },
    AlreadyExists {
        entity: &'static str,
        id: String,
    },
    InvalidInput {
        field: &'static str,
        reason: String,
    },
    Conflict {
        entity: &'static str,
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
            Self::NotFound { entity, id } => write!(f, "{entity} not found: {id}"),
            Self::AlreadyExists { entity, id } => write!(f, "{entity} already exists: {id}"),
            Self::InvalidInput { field, reason } => {
                write!(f, "invalid input for {field}: {reason}")
            }
            Self::Conflict { entity, reason } => write!(f, "{entity} conflict: {reason}"),
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
            entity: "task",
            id: "task-123".to_owned(),
        };
        assert_eq!(err.to_string(), "task not found: task-123");
    }

    #[test]
    fn already_exists_display() {
        let err = DomainError::AlreadyExists {
            entity: "repo",
            id: "api-gateway".to_owned(),
        };
        assert_eq!(err.to_string(), "repo already exists: api-gateway");
    }

    #[test]
    fn invalid_input_display() {
        let err = DomainError::InvalidInput {
            field: "slug",
            reason: "slug cannot be empty".to_owned(),
        };
        assert_eq!(
            err.to_string(),
            "invalid input for slug: slug cannot be empty"
        );
    }

    #[test]
    fn conflict_display() {
        let err = DomainError::Conflict {
            entity: "task",
            reason: "already closed".to_owned(),
        };
        assert_eq!(err.to_string(), "task conflict: already closed");
    }

    #[test]
    fn persistence_failed_display() {
        let err = DomainError::PersistenceFailed {
            operation: "save",
            detail: "disk full".to_owned(),
        };
        assert_eq!(
            err.to_string(),
            "persistence error during save: disk full"
        );
    }
}
