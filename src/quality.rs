use crate::domain::TranslationMemoryEntry;

pub struct QualityReport {
    pub issues: Vec<String>,
}

impl QualityReport {
    pub fn new() -> Self {
        Self { issues: Vec::new() }
    }

    pub fn check_translation_memory(&mut self, entries: &[TranslationMemoryEntry]) {
        for entry in entries {
            if entry.translated_text.trim().is_empty() {
                self.issues.push(format!("Missing translation for: {}", entry.source_text));
            }
        }
    }
}
