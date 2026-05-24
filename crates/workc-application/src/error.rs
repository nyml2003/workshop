use std::error::Error;
use std::fmt::{Display, Formatter};

use workc_domain::errors::DomainError;

#[derive(Debug)]
pub enum ApplicationError {
    Domain(DomainError),
    InvalidRequest(String),
    AdapterUnavailable(&'static str),
    ExternalFailure {
        port: &'static str,
        detail: String,
    },
}

impl Display for ApplicationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Domain(error) => Display::fmt(error, f),
            Self::InvalidRequest(message) => write!(f, "invalid request: {message}"),
            Self::AdapterUnavailable(port) => write!(f, "adapter unavailable: {port}"),
            Self::ExternalFailure { port, detail } => write!(f, "external failure on {port}: {detail}"),
        }
    }
}

impl Error for ApplicationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Domain(error) => Some(error),
            _ => None,
        }
    }
}

impl From<DomainError> for ApplicationError {
    fn from(value: DomainError) -> Self {
        Self::Domain(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use workc_domain::errors::DomainError;

    #[test]
    fn domain_error_display_passthrough() {
        let err = ApplicationError::Domain(DomainError::NotFound {
            entity: "repo",
            id: "x".to_owned(),
        });
        assert_eq!(err.to_string(), "repo not found: x");
    }

    #[test]
    fn invalid_request_display() {
        let err = ApplicationError::InvalidRequest("bad input".to_owned());
        assert_eq!(err.to_string(), "invalid request: bad input");
    }

    #[test]
    fn adapter_unavailable_display() {
        let err = ApplicationError::AdapterUnavailable("git");
        assert_eq!(err.to_string(), "adapter unavailable: git");
    }

    #[test]
    fn external_failure_display() {
        let err = ApplicationError::ExternalFailure {
            port: "editor",
            detail: "launch failed".to_owned(),
        };
        assert_eq!(err.to_string(), "external failure on editor: launch failed");
    }

    #[test]
    fn domain_error_is_source() {
        let domain = DomainError::NotFound {
            entity: "task",
            id: "t1".to_owned(),
        };
        let app_err = ApplicationError::Domain(domain.clone());
        let source = app_err.source();
        assert!(source.is_some());
        assert_eq!(source.unwrap().to_string(), domain.to_string());
    }

    #[test]
    fn non_domain_error_has_no_source() {
        let err = ApplicationError::InvalidRequest("nope".to_owned());
        assert!(err.source().is_none());
    }

    #[test]
    fn from_domain_error_conversion() {
        let domain = DomainError::Conflict {
            entity: "task",
            reason: "locked".to_owned(),
        };
        let app_err: ApplicationError = domain.into();
        assert!(matches!(app_err, ApplicationError::Domain(..)));
    }
}
