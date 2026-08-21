use chrono::Utc;
use uuid::Uuid;

use crate::{execution::{TranslationExecutionRequest, TranslationOutput}, QualityGate, TranslationProvider, TranslationRuntimeError};

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

    pub fn execute(
        &self,
        request: TranslationExecutionRequest,
    ) -> Result<TranslationOutput, TranslationRuntimeError> {
        if request.source_text.trim().is_empty() {
            return Err(TranslationRuntimeError::InvalidContext(
                "source text cannot be empty".into(),
            ));
        }

        let translated_text = self
            .provider
            .translate(&request)
            .map_err(|error| TranslationRuntimeError::ProviderFailure(error.to_string()))?;

        let output = TranslationOutput {
            translated_text,
            execution_id: Uuid::new_v4(),
            provider_name: self.provider.provider_name().to_string(),
            created_at: Utc::now(),
        };

        let report = self.quality_gate.validate(&output)?;

        if !report.accepted {
            return Err(TranslationRuntimeError::QualityRejected(
                "quality gate rejected output".into(),
            ));
        }

        Ok(output)
    }
}
