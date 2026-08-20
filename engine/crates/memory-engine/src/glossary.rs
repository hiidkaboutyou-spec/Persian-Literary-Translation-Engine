#[derive(Debug, Clone)]
pub struct GlossaryEntry {
    pub source_term: String,
    pub preferred_translation: String,
    pub context: String,
}

#[derive(Default)]
pub struct Glossary {
    entries: Vec<GlossaryEntry>,
}

impl Glossary {
    pub fn add(&mut self, entry: GlossaryEntry) {
        self.entries.push(entry);
    }

    pub fn entries(&self) -> &[GlossaryEntry] {
        &self.entries
    }
}
