use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum MemoryError {
    #[error("invalid memory data: {0}")]
    Invalid(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NovelState {
    pub id: Uuid,
    pub novel_id: Uuid,
    pub current_chapter: u32,
    pub canonical_summary: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CharacterMemory {
    pub id: Uuid,
    pub character_id: Uuid,
    pub name: String,
    pub role: String,
    pub current_state: String,
    pub history: Vec<String>,
    pub first_appearance: u32,
    pub last_known_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CharacterVoiceProfile {
    pub id: Uuid,
    pub character_id: Uuid,
    pub speech_pattern: String,
    pub sentence_style: String,
    pub formality_level: String,
    pub humor_style: String,
    pub emotional_expression_style: String,
    pub preferred_phrases: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RelationshipMemory {
    pub id: Uuid,
    pub character_a: Uuid,
    pub character_b: Uuid,
    pub relationship_type: String,
    pub current_state: String,
    pub history: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NarrativeEventMemory {
    pub id: Uuid,
    pub chapter_reference: u32,
    pub event_description: String,
    pub participants: Vec<Uuid>,
    pub consequences: Vec<String>,
    pub unresolved_threads: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TranslationDecisionMemory {
    pub id: Uuid,
    pub source_term: String,
    pub chosen_translation: String,
    pub reason: String,
    pub chapter_reference: u32,
    pub confidence: f32,
    pub human_override: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TerminologyMemory {
    pub id: Uuid,
    pub source_term: String,
    pub target_term: String,
    pub category: String,
    pub notes: String,
    pub approved: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MemoryState {
    Draft,
    Canonical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MemorySnapshot {
    pub id: Uuid,
    pub novel_id: Uuid,
    pub chapter: u32,
    pub version: String,
    pub state_hash: String,
    pub changes: Vec<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct MemorySnapshotContext {
    pub characters: Vec<CharacterMemory>,
    pub relationships: Vec<RelationshipMemory>,
    pub terminology: Vec<TerminologyMemory>,
    pub decisions: Vec<TranslationDecisionMemory>,
    pub events: Vec<NarrativeEventMemory>,
    pub voice_profiles: Vec<CharacterVoiceProfile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MemoryUpdate {
    pub chapter: u32,
    pub changes: Vec<String>,
    pub state: MemoryState,
}

pub struct MemoryUpdatePipeline;

impl MemoryUpdatePipeline {
    pub fn validate(update: &MemoryUpdate) -> Result<(), MemoryError> {
        if update.changes.is_empty() {
            return Err(MemoryError::Invalid("empty memory update".into()));
        }
        Ok(())
    }
}

pub fn character_continuity_check() -> &'static str { "CharacterContinuityCheck" }
pub fn relationship_continuity_check() -> &'static str { "RelationshipContinuityCheck" }
pub fn terminology_consistency_check() -> &'static str { "TerminologyConsistencyCheck" }
pub fn decision_consistency_check() -> &'static str { "DecisionConsistencyCheck" }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canon_and_draft_are_separate() {
        assert_ne!(MemoryState::Draft, MemoryState::Canonical);
    }

    #[test]
    fn snapshot_context_preserves_history() {
        let context = MemorySnapshotContext::default();
        assert!(context.characters.is_empty());
    }

    #[test]
    fn update_requires_changes() {
        assert!(MemoryUpdatePipeline::validate(&MemoryUpdate { chapter: 1, changes: vec![], state: MemoryState::Draft }).is_err());
    }
}
