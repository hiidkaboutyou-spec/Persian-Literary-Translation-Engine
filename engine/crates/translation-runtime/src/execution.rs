use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiteraryContext {
    pub scene_information: String,
    pub character_information: String,
    pub relationship_information: String,
    pub emotional_context: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextSnapshot {
    pub previous_chapter_state: String,
    pub continuity_information: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DecisionTrace {
    pub decisions: Vec<String>,
    pub rationale: String,
    pub confidence: f32,
    pub human_overrides: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TranslationExecutionRequest {
    pub source_text: String,
    pub literary_context: LiteraryContext,
    pub context_snapshot: ContextSnapshot,
    pub decision_trace: DecisionTrace,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TranslationOutput {
    pub translated_text: String,
    pub execution_id: Uuid,
    pub provider_name: String,
    pub created_at: DateTime<Utc>,
}
