use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub source: String,
    pub translation: String,
    pub context: String,
    pub tags: Vec<String>,
}

impl MemoryEntry {
    pub fn new(source: String, translation: String, context: String) -> Self {
        Self {
            source,
            translation,
            context,
            tags: Vec::new(),
        }
    }
}
