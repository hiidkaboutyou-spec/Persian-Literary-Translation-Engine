pub mod consistency;
pub mod evaluation;

pub use consistency::{
    audit_consistency, check_consistency, ConsistencyConflict, ConsistencyObservation,
    ConsistencyResult,
};
pub use evaluation::{evaluate_translation, QualityEvaluation, TerminologyRule};

#[derive(Debug, Clone)]
pub struct QualityReport {
    pub consistency_score: f32,
    pub notes: Vec<String>,
}

pub fn review_translation() -> QualityReport {
    let consistency = check_consistency();
    QualityReport {
        consistency_score: consistency.score,
        notes: consistency.warnings,
    }
}
