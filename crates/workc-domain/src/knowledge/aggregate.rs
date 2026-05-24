use super::entities::KnowledgeEntry;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KnowledgeBase {
    pub entries: Vec<KnowledgeEntry>,
}

impl Default for KnowledgeBase {
    fn default() -> Self {
        Self { entries: Vec::new() }
    }
}
