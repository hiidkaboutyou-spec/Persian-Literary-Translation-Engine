use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GlossaryEntry {
    pub source: String,
    pub preferred_translation: String,
    pub notes: Option<String>,
}

pub struct GlossaryManager;

impl GlossaryManager {
    pub fn new() -> Self {
        Self
    }
}
