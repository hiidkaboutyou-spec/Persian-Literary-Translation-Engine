use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::errors::LiteraryIntelligenceError;
use crate::models::{
    character::CharacterProfile,
    literary_rule::LiteraryRule,
    novel_context::NovelContext,
    relationship::RelationshipState,
    scene::SceneContext,
    translation_decision::TranslationDecision,
};

/// Complete structured understanding state for a literary work before translation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LiteraryUnderstanding {
    pub novel_context: NovelContext,
    pub characters: Vec<CharacterProfile>,
    pub scenes: Vec<SceneContext>,
    pub relationships: Vec<RelationshipState>,
    pub translation_decisions: Vec<TranslationDecision>,
    pub rules: Vec<LiteraryRule>,
    pub created_at: DateTime<Utc>,
}

impl LiteraryUnderstanding {
    pub fn validate(&self) -> Result<(), LiteraryIntelligenceError> {
        self.novel_context.validate()?;

        for character in &self.characters {
            character.validate()?;
        }

        for scene in &self.scenes {
            scene.validate()?;
        }

        for relationship in &self.relationships {
            relationship.validate()?;
        }

        for decision in &self.translation_decisions {
            decision.validate()?;
        }

        Ok(())
    }
}
