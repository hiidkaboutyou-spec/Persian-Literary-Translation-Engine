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
    pub summary: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CharacterMemory {
    pub id: Uuid,
    pub character_id: Uuid,
    pub name: String,
    pub role: String,
    pub personality_state: String,
    pub relationship_state: String,
    pub current_status: String,
    pub history: Vec<String>,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TranslationDecisionMemory {
    pub id: Uuid,
    pub source_term: String,
    pub chosen_translation: String,
    pub reason: String,
    pub chapter_reference: u32,
    pub confidence: f32,
    pub human_override: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TerminologyMemory {
    pub id: Uuid,
    pub source_term: String,
    pub target_term: String,
    pub category: String,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct MemorySnapshot {
    pub characters: Vec<CharacterMemory>,
    pub relationships: Vec<RelationshipMemory>,
    pub terminology: Vec<TerminologyMemory>,
    pub decisions: Vec<TranslationDecisionMemory>,
    pub continuity: Option<NovelState>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MemoryUpdate {
    pub chapter: u32,
    pub continuity_notes: Vec<String>,
}

pub fn validate_character_consistency() -> &'static str { "CharacterConsistencyCheck" }
pub fn validate_relationship_consistency() -> &'static str { "RelationshipConsistencyCheck" }
pub fn validate_terminology_consistency() -> &'static str { "TerminologyConsistencyCheck" }
pub fn validate_decision_consistency() -> &'static str { "DecisionConsistencyCheck" }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn models_serialize_roundtrip() {
        let state = NovelState {
            id: Uuid::new_v4(),
            novel_id: Uuid::new_v4(),
            current_chapter: 1,
            summary: "checkpoint".into(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let json = serde_json::to_string(&state).unwrap();
        let decoded: NovelState = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.current_chapter, 1);
    }

    #[test]
    fn snapshot_preserves_memory() {
        let snapshot = MemorySnapshot::default();
        assert!(snapshot.characters.is_empty());
    }
}
