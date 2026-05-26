//! File-system backed repositories and stores.

pub mod config_repository;
pub mod knowledge_repository;
pub mod paths;
pub mod repo_catalog_repository;
pub mod skill_registry_repository;
pub mod task_repository;
pub mod task_skill_mount_repository;
pub mod workspace_repository;

pub use config_repository::FsConfigRepository;
pub use repo_catalog_repository::FsRepoCatalogRepository;
pub use skill_registry_repository::FsSkillRegistryRepository;
pub use task_repository::FsTaskRepository;
pub use task_skill_mount_repository::FsTaskSkillMountRepository;
pub use workspace_repository::FsWorkspaceRegistryRepository;
