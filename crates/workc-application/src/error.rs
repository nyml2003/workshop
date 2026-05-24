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
