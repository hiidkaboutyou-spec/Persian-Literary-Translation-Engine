use crate::domain::GlossaryEntry;

#[derive(Default)]
pub struct GlossaryEngine {
    entries: Vec<GlossaryEntry>,
}

impl GlossaryEngine {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn add(&mut self, entry: GlossaryEntry) {
        self.entries.push(entry);
    }

    pub fn find(&self, source: &str) -> Option<&GlossaryEntry> {
        self.entries.iter().find(|entry| entry.source == source)
    }

    pub fn all(&self) -> &[GlossaryEntry] {
        &self.entries
    }
}
