//! File-system backed repositories and stores.

pub mod knowledge_repository;
pub mod repo_catalog_repository;
pub mod skill_registry_repository;
pub mod task_repository;
pub mod task_skill_mount_repository;

pub use repo_catalog_repository::FsRepoCatalogRepository;
pub use skill_registry_repository::FsSkillRegistryRepository;
pub use task_repository::{DefaultTaskIdGenerator, FsTaskRepository};
pub use task_skill_mount_repository::FsTaskSkillMountRepository;
