use super::entities::KnowledgeEntry;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct KnowledgeBase {
    pub entries: Vec<KnowledgeEntry>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_empty_entries() {
        let kb = KnowledgeBase::default();
        assert!(kb.entries.is_empty());
    }
}
