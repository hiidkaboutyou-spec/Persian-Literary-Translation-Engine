use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Explains why a literary decision was made before translation.
///
/// This keeps interpretation decisions auditable without coupling
/// literary intelligence to providers or orchestration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecisionTrace {
    pub id: Uuid,
    pub decision_id: Uuid,
    pub context_summary: String,
    pub influencing_factors: Vec<String>,
    pub confidence: f32,
    pub human_override: Option<String>,
}

impl DecisionTrace {
    pub fn validate(&self) -> Result<(), String> {
        if !(0.0..=1.0).contains(&self.confidence) {
            return Err("decision trace confidence must be between 0.0 and 1.0".into());
        }
        Ok(())
    }
}
