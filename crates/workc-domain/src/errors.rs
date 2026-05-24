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
    ExternalCommandFailed {
        command: &'static str,
        detail: String,
    },
    IoError {
        operation: &'static str,
        detail: String,
    },
}

impl Display for DomainError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound { entity, id } => write!(f, "{entity} not found: {id}"),
            Self::AlreadyExists { entity, id } => write!(f, "{entity} already exists: {id}"),
            Self::InvalidInput { field, reason } => write!(f, "invalid input for {field}: {reason}"),
            Self::Conflict { entity, reason } => write!(f, "{entity} conflict: {reason}"),
            Self::ExternalCommandFailed { command, detail } => {
                write!(f, "external command failed ({command}): {detail}")
            }
            Self::IoError { operation, detail } => write!(f, "io error during {operation}: {detail}"),
        }
    }
}

impl Error for DomainError {}
