//! Context snapshots preserve translation continuity across chapters.
//!
//! Translation runtime stores the execution snapshot while literary
//! intelligence remains the owner of interpretation.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextSnapshot {
    pub chapter_id: String,
    pub character_state: Vec<String>,
    pub relationship_state: Vec<String>,
    pub previous_decisions: Vec<String>,
}

impl ContextSnapshot {
    pub fn new(chapter_id: String) -> Self {
        Self {
            chapter_id,
            character_state: Vec::new(),
            relationship_state: Vec::new(),
            previous_decisions: Vec::new(),
        }
    }

    pub fn add_decision(&mut self, decision: String) {
        self.previous_decisions.push(decision);
    }
}
