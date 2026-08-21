use crate::errors::LiteraryIntelligenceError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NovelContext {
    pub id: Uuid,
    pub project_id: Uuid,
    pub title: String,
    pub genre: Option<String>,
    pub tone: Option<String>,
    pub global_rules: Vec<String>,
    pub created_at: DateTime<Utc>,
}

impl NovelContext {
    pub fn validate(&self) -> Result<(), LiteraryIntelligenceError> {
        if self.title.trim().is_empty() {
            return Err(LiteraryIntelligenceError::Validation("novel title cannot be empty".into()));
        }
        Ok(())
    }
}
