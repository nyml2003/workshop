pub mod aggregate;
pub mod entities;
pub mod repository;

pub use aggregate::KnowledgeBase;
pub use entities::{KnowledgeCandidate, KnowledgeEntry, KnowledgeSourceRef};
pub use repository::KnowledgeRepository;
