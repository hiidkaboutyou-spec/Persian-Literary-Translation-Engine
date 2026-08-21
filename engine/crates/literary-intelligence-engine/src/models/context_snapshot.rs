use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSnapshot {
    pub id: Uuid,
    pub novel_context_id: Uuid,
    pub chapter_id: Uuid,
    pub active_character_ids: Vec<Uuid>,
    pub active_relationship_ids: Vec<Uuid>,
    pub relevant_rules: Vec<Uuid>,
}
