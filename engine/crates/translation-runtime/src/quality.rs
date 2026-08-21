//! Quality gates validate translation execution outputs.
//!
//! This layer does not rewrite translations.
//! It checks whether runtime output preserved required guarantees.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QualityDimension {
    CharacterVoicePreservation,
    ContinuityPreservation,
    LiteraryConsistency,
    DecisionTraceCompliance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QualityReport {
    pub dimensions: Vec<QualityCheck>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QualityCheck {
    pub dimension: QualityDimension,
    pub passed: bool,
    pub details: String,
}

impl QualityReport {
    pub fn acceptable(&self) -> bool {
        self.dimensions.iter().all(|check| check.passed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quality_gate_reports_explicit_literary_dimensions() {
        let report = QualityReport {
            dimensions: vec![QualityCheck {
                dimension: QualityDimension::CharacterVoicePreservation,
                passed: true,
                details: "voice preserved".into(),
            }],
        };

        assert!(report.acceptable());
    }

    #[test]
    fn failed_dimension_rejects_quality_report() {
        let report = QualityReport {
            dimensions: vec![QualityCheck {
                dimension: QualityDimension::ContinuityPreservation,
                passed: false,
                details: "continuity break".into(),
            }],
        };

        assert!(!report.acceptable());
    }
}
