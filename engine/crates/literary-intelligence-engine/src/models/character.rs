use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterProfile {
    pub id: Uuid,
    pub novel_context_id: Uuid,
    pub name: String,
    pub aliases: Vec<String>,
    pub traits: Vec<String>,
    pub motivations: Vec<String>,
    pub fears: Vec<String>,
    pub voice_profile: Option<String>,
    pub first_seen_chapter: Option<u32>,
    pub importance_score: f32,
}
