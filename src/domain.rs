use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterProfile {
    pub name: String,
    pub aliases: Vec<String>,
    pub voice_style: String,
    pub personality_notes: Vec<String>,
    pub relationship_notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlossaryEntry {
    pub source: String,
    pub target: String,
    pub notes: String,
    pub locked: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneContext {
    pub title: String,
    pub characters: Vec<String>,
    pub emotional_state: String,
    pub narrative_notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationMemoryEntry {
    pub source_text: String,
    pub translated_text: String,
    pub context: String,
}
