use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneContext {
    pub id: Uuid,
    pub chapter_id: Uuid,
    pub purpose: String,
    pub emotional_tone: Option<String>,
    pub character_ids: Vec<Uuid>,
    pub importance_score: f32,
    pub symbolic_elements: Vec<String>,
}
