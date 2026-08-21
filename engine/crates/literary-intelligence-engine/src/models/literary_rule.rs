use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiteraryRule {
    pub id: Uuid,
    pub novel_context_id: Uuid,
    pub rule_text: String,
    pub scope: String,
    pub active: bool,
}
