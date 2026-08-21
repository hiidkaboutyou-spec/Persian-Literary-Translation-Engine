use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationDecision {
    pub id: Uuid,
    pub source_text: String,
    pub translated_text: String,
    pub reason: String,
    pub chapter_id: Option<Uuid>,
    pub decision_type: String,
    pub confidence: f32,
}
