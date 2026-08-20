//! Relationship context memory for character consistency.

#[derive(Debug, Clone)]
pub struct RelationshipMemory {
    pub character_a: String,
    pub character_b: String,
    pub dynamic_notes: String,
}

impl RelationshipMemory {
    pub fn new(a: impl Into<String>, b: impl Into<String>, notes: impl Into<String>) -> Self {
        Self {
            character_a: a.into(),
            character_b: b.into(),
            dynamic_notes: notes.into(),
        }
    }
}
