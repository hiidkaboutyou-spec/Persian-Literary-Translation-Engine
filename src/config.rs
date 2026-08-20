use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub source_language: String,
    pub target_language: String,
    pub enable_memory: bool,
    pub enable_glossary: bool,
}
