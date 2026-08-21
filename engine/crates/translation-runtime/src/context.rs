//! Context assembly prepares translation input context.
//!
//! This layer connects runtime execution with previously produced
//! understanding without moving literary intelligence responsibilities here.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TranslationContext {
    pub source_text: String,
    pub character_context: Vec<String>,
    pub relationship_context: Vec<String>,
    pub literary_rules: Vec<String>,
    pub previous_decisions: Vec<String>,
}

impl TranslationContext {
    pub fn new(source_text: String) -> Self {
        Self {
            source_text,
            character_context: Vec::new(),
            relationship_context: Vec::new(),
            literary_rules: Vec::new(),
            previous_decisions: Vec::new(),
        }
    }
}
