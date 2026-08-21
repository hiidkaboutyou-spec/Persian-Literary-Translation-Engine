use crate::errors::LiteraryIntelligenceError;
use crate::models::{
    character::CharacterProfile, relationship::RelationshipState, scene::SceneContext,
};
use uuid::Uuid;

pub trait CharacterAnalyzer {
    fn analyze(&self, chapter_id: Uuid)
        -> Result<Vec<CharacterProfile>, LiteraryIntelligenceError>;
}

pub trait SceneAnalyzer {
    fn analyze(&self, chapter_id: Uuid) -> Result<Vec<SceneContext>, LiteraryIntelligenceError>;
}

pub trait RelationshipAnalyzer {
    fn analyze(
        &self,
        chapter_id: Uuid,
    ) -> Result<Vec<RelationshipState>, LiteraryIntelligenceError>;
}
