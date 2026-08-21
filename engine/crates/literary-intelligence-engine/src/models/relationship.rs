use crate::errors::LiteraryIntelligenceError;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RelationshipState {
    pub id: Uuid,
    pub character_a: Uuid,
    pub character_b: Uuid,
    pub relationship_type: String,
    pub trust_level: f32,
    pub conflict_level: f32,
    pub emotional_distance: f32,
    pub dynamic_notes: Vec<String>,
    pub unresolved_tensions: Vec<String>,
    pub significant_turning_points: Vec<String>,
}

impl RelationshipState {
    pub fn validate(&self) -> Result<(), LiteraryIntelligenceError> {
        for (name, value) in [
            ("trust_level", self.trust_level),
            ("conflict_level", self.conflict_level),
            ("emotional_distance", self.emotional_distance),
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
