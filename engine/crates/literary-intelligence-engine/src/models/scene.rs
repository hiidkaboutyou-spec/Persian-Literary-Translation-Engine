use crate::errors::LiteraryIntelligenceError;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneContext {
    pub id: Uuid,
    pub chapter_id: Uuid,
    pub purpose: String,
    pub emotional_tone: Option<String>,
    pub conflict_level: Option<String>,
    pub character_ids: Vec<Uuid>,
    pub importance_score: f32,
    pub symbolic_elements: Vec<String>,
    pub narrative_turning_points: Vec<String>,
    pub subtext_notes: Vec<String>,
}

impl SceneContext {
    pub fn validate(&self) -> Result<(), LiteraryIntelligenceError> {
        if !(0.0..=1.0).contains(&self.importance_score) {
            return Err(LiteraryIntelligenceError::Validation(
                "scene importance_score must be between 0.0 and 1.0".into(),
            ));
        }
        if self.purpose.trim().is_empty() {
            return Err(LiteraryIntelligenceError::Validation(
                "scene purpose cannot be empty".into(),
            ));
        }
        Ok(())
    }
}
