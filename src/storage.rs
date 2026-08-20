use std::collections::HashMap;

use crate::models::TranslationMemoryEntry;

#[derive(Default)]
pub struct TranslationMemoryStore {
    entries: HashMap<String, TranslationMemoryEntry>,
}

impl TranslationMemoryStore {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: String, entry: TranslationMemoryEntry) {
        self.entries.insert(key, entry);
    }

    pub fn get(&self, key: &str) -> Option<&TranslationMemoryEntry> {
        self.entries.get(key)
    }
}
