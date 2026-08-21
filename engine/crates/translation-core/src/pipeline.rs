use crate::provider::{
    PassKind, ProviderError, ProviderRequest, ProviderResponse, TranslationProvider,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PipelineStage {
    DocumentAnalysis,
    ContextBuilding,
    Translation,
    Revision,
    QualityReview,
    Export,
}

#[derive(Debug, Clone)]
pub struct TranslationPipeline {
    pub stages: Vec<PipelineStage>,
}

#[derive(Debug, Clone)]
pub struct PipelineInput {
    pub source_text: String,
    pub target_language: String,
    pub context: String,
}

#[derive(Debug, Clone)]
pub struct PipelineOutput {
    pub translated_text: String,
    pub revised_text: String,
    pub quality_review: String,
    pub provider: String,
}

impl TranslationPipeline {
    pub fn default_literary_pipeline() -> Self {
        Self {
            stages: vec![
                PipelineStage::DocumentAnalysis,
                PipelineStage::ContextBuilding,
                PipelineStage::Translation,
                PipelineStage::Revision,
                PipelineStage::QualityReview,
                PipelineStage::Export,
            ],
        }
    }

    /// Execute the provider-facing literary passes in a deterministic order.
    /// Document ingestion, memory retrieval and export remain separate layers and
    /// can feed/consume this runtime without coupling the core to a vendor.
    pub fn execute<P: TranslationProvider>(
        &self,
        provider: &P,
        input: PipelineInput,
    ) -> Result<PipelineOutput, ProviderError> {
        let translated = provider.execute(&ProviderRequest {
            pass: PassKind::Translate,
            source_text: input.source_text,
            target_language: input.target_language.clone(),
            context: input.context.clone(),
        })?;

        let revised = provider.execute(&ProviderRequest {
            pass: PassKind::Revision,
            source_text: translated.text.clone(),
            target_language: input.target_language.clone(),
            context: input.context.clone(),
        })?;

        let reviewed = provider.execute(&ProviderRequest {
            pass: PassKind::QualityReview,
            source_text: revised.text.clone(),
            target_language: input.target_language,
            context: input.context,
        })?;

        Ok(PipelineOutput {
            translated_text: translated.text,
            revised_text: revised.text,
            quality_review: reviewed.text,
            provider: reviewed.provider,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::EchoProvider;

    #[test]
    fn default_pipeline_runs_all_provider_passes() {
        let pipeline = TranslationPipeline::default_literary_pipeline();
        let output = pipeline
            .execute(
                &EchoProvider,
                PipelineInput {
                    source_text: "A chapter".into(),
                    target_language: "fa".into(),
                    context: "voice bible".into(),
                },
            )
            .unwrap();

        assert_eq!(output.translated_text, "A chapter");
        assert_eq!(output.revised_text, "A chapter");
        assert_eq!(output.quality_review, "A chapter");
        assert_eq!(output.provider, "echo");
    }
}
