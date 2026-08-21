use crate::errors::LiteraryIntelligenceError;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TranslationDecision {
    pub id: Uuid,
    pub source_text: String,
    pub translated_text: String,
    pub reason: String,
    pub chapter_id: Option<Uuid>,
    pub decision_type: String,
    pub confidence: f32,
}

impl TranslationDecision {
    pub fn validate(&self) -> Result<(), LiteraryIntelligenceError> {
        if !(0.0..=1.0).contains(&self.confidence) {
            return Err(LiteraryIntelligenceError::Validation(
                "translation confidence must be between 0.0 and 1.0".into(),
            ));
        }
        Ok(())
    }
}
