use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipState {
    pub id: Uuid,
    pub character_a: Uuid,
    pub character_b: Uuid,
    pub relationship_type: String,
    pub trust_level: f32,
    pub conflict_level: f32,
    pub emotional_distance: f32,
}
