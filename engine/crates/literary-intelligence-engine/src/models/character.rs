use crate::errors::LiteraryIntelligenceError;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CharacterProfile {
    pub id: Uuid,
    pub novel_context_id: Uuid,
    pub name: String,
    pub aliases: Vec<String>,
    pub traits: Vec<String>,
    pub motivations: Vec<String>,
    pub fears: Vec<String>,
    pub desires: Vec<String>,
    pub voice_profile: Option<String>,
    pub speech_style: Option<String>,
    pub emotional_patterns: Vec<String>,
    pub first_seen_chapter: Option<u32>,
    pub importance_score: f32,
}

impl CharacterProfile {
    pub fn validate(&self) -> Result<(), LiteraryIntelligenceError> {
        if self.name.trim().is_empty() {
            return Err(LiteraryIntelligenceError::Validation(
                "character name cannot be empty".into(),
            ));
        }
        if !(0.0..=1.0).contains(&self.importance_score) {
            return Err(LiteraryIntelligenceError::Validation(
                "character importance_score must be between 0.0 and 1.0".into(),
            ));
        }
        Ok(())
    }
}
