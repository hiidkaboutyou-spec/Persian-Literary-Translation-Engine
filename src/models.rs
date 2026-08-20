use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Character {
    pub name: String,
    pub aliases: Vec<String>,
    pub voice_style: String,
    pub personality_notes: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GlossaryEntry {
    pub source: String,
    pub preferred_translation: String,
    pub notes: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Chapter {
    pub title: String,
    pub source_text: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TranslationMemoryEntry {
    pub source: String,
    pub translation: String,
    pub context: String,
}
