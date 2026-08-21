use crate::errors::LiteraryIntelligenceError;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneContext {
    pub id: Uuid,
    pub chapter_id: Uuid,
    pub purpose: String,
    pub emotional_tone: Option<String>,
    pub character_ids: Vec<Uuid>,
    pub importance_score: f32,
    pub symbolic_elements: Vec<String>,
    pub conflict_level: f32,
    pub character_goals: Vec<String>,
    pub narrative_events: Vec<String>,
}

impl SceneContext {
    pub fn validate(&self) -> Result<(), LiteraryIntelligenceError> {
        for (name, value) in [
            ("importance_score", self.importance_score),
            ("conflict_level", self.conflict_level),
        ] {
            if !(0.0..=1.0).contains(&value) {
                return Err(LiteraryIntelligenceError::Validation(format!(
                    "{name} must be between 0.0 and 1.0"
                )));
            }
        }
        Ok(())
    }
}
