use chrono::Utc;
use uuid::Uuid;

use crate::{execution::{LiteraryContext, ContextSnapshot, DecisionTrace, TranslationExecutionRequest, TranslationOutput}, QualityGate, QualityReport, TranslationProvider, TranslationRuntimeError};

pub struct TranslationExecutionPipeline<P, Q> {
    provider: P,
    quality_gate: Q,
}

impl<P, Q> TranslationExecutionPipeline<P, Q>
where
    P: TranslationProvider,
    Q: QualityGate,
{
    pub fn new(provider: P, quality_gate: Q) -> Self {
        Self { provider, quality_gate }
    }

    pub fn execute(&self, request: TranslationExecutionRequest) -> Result<TranslationOutput, TranslationRuntimeError> {
        if request.source_text.trim().is_empty() {
            return Err(TranslationRuntimeError::InvalidContext("source text cannot be empty".into()));
        }

        let translated_text = self.provider.translate(&request)
            .map_err(|error| TranslationRuntimeError::ProviderFailure(error.to_string()))?;

        let output = TranslationOutput {
            translated_text,
            execution_id: Uuid::new_v4(),
            provider_name: self.provider.provider_name().to_string(),
            created_at: Utc::now(),
        };

        let report = self.quality_gate.validate(&output)?;
        if !report.accepted {
            return Err(TranslationRuntimeError::QualityRejected("quality gate rejected output".into()));
        }

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockProvider;
    impl TranslationProvider for MockProvider {
        fn provider_name(&self) -> &str { "mock" }
        fn translate(&self, _: &TranslationExecutionRequest) -> Result<String, TranslationRuntimeError> { Ok("translated".into()) }
    }

    struct RejectingGate;
    impl QualityGate for RejectingGate {
        fn validate(&self, _: &TranslationOutput) -> Result<QualityReport, TranslationRuntimeError> { Ok(QualityReport { accepted: false }) }
    }

    struct PassingGate;
    impl QualityGate for PassingGate {
        fn validate(&self, _: &TranslationOutput) -> Result<QualityReport, TranslationRuntimeError> { Ok(QualityReport { accepted: true }) }
    }

    fn request() -> TranslationExecutionRequest {
        TranslationExecutionRequest {
            source_text: "text".into(),
            literary_context: LiteraryContext { scene_information: "scene".into(), character_information: "characters".into(), relationship_information: "relationship".into(), emotional_context: "emotion".into() },
            context_snapshot: ContextSnapshot { previous_chapter_state: "state".into(), continuity_information: "continuity".into() },
            decision_trace: DecisionTrace { decisions: vec!["choice".into()], rationale: "reason".into(), confidence: 1.0, human_overrides: vec![] },
        }
    }

    #[test]
    fn successful_execution_preserves_output() {
        let result = TranslationExecutionPipeline::new(MockProvider, PassingGate).execute(request()).unwrap();
        assert_eq!(result.translated_text, "translated");
    }

    #[test]
    fn quality_rejection_is_returned() {
        let result = TranslationExecutionPipeline::new(MockProvider, RejectingGate).execute(request());
        assert!(matches!(result, Err(TranslationRuntimeError::QualityRejected(_))));
    }
}
