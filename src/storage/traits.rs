use anyhow::Result;

#[derive(Debug, Clone)]
pub struct TranslationMemoryEntry {
    pub source: String,
    pub translation: String,
    pub project_id: String,
}

#[derive(Debug, Clone)]
pub struct GlossaryEntry {
    pub term: String,
    pub preferred_translation: String,
    pub project_id: String,
}

pub trait TranslationMemoryStore {
    fn save(&self, entry: TranslationMemoryEntry) -> Result<()>;

    fn search(&self, query: &str) -> Result<Vec<TranslationMemoryEntry>>;
}

pub trait GlossaryStore {
    fn add(&self, entry: GlossaryEntry) -> Result<()>;

    fn lookup(&self, term: &str) -> Result<Option<GlossaryEntry>>;
}
