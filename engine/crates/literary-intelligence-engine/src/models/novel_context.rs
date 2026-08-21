use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NovelContext {
    pub id: Uuid,
    pub project_id: Uuid,
    pub title: String,
    pub genre: Option<String>,
    pub tone: Option<String>,
    pub global_rules: Vec<String>,
    pub created_at: DateTime<Utc>,
}
