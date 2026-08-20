pub struct TranslationMemory;

impl TranslationMemory {
    pub fn new() -> Self {
        Self
    }

    pub fn store_decision(&self, _source: &str, _translation: &str) {
        // Future SQLite-backed implementation
    }
}
