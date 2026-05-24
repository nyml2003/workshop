use crate::errors::DomainError;
use crate::shared::{RepoGroupId, RepoId};

use super::aggregate::RepoCatalog;
use super::entities::{RepoEntry, RepoGroup};

pub trait RepoCatalogRepository {
    fn load(&self) -> Result<RepoCatalog, DomainError>;
    fn save(&self, catalog: &RepoCatalog) -> Result<(), DomainError>;
    fn find_repo(&self, id: &RepoId) -> Result<Option<RepoEntry>, DomainError>;
    fn find_group(&self, id: &RepoGroupId) -> Result<Option<RepoGroup>, DomainError>;
}
