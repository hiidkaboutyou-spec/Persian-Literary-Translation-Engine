#[derive(Debug, Clone)]
pub struct MemoryEntry {
    pub source: String,
    pub translation: String,
}

pub struct TranslationMemory;

impl TranslationMemory {
    pub fn search(&self, query: &str) -> Vec<MemoryEntry> {
        let _ = query;
        Vec::new()
    }
}
