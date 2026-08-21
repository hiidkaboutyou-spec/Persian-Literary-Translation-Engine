use crate::provider::{PassKind, ProviderError, ProviderRequest, ProviderResponse, TranslationProvider};

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
pub struct PipelineResult {
    pub translated: ProviderResponse,
    pub revised: ProviderResponse,
    pub reviewed: ProviderResponse,
}

impl PipelineResult {
    pub fn final_text(&self) -> &str {
        &self.reviewed.text
    }
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

    /// Execute the model-facing portion of the literary pipeline.
    ///
    /// Each pass receives the previous pass output rather than the original
    /// source so providers can progressively translate, revise, and review a
    /// chapter while sharing the same retrieved project context.
    pub fn execute<P: TranslationProvider>(
        &self,
        provider: &P,
        source_text: impl Into<String>,
        target_language: impl Into<String>,
        context: impl Into<String>,
    ) -> Result<PipelineResult, ProviderError> {
        let source_text = source_text.into();
        let target_language = target_language.into();
        let context = context.into();

        if source_text.trim().is_empty() {
            return Err(ProviderError::InvalidRequest("source text is empty".into()));
        }

        let translated = provider.execute(&ProviderRequest {
            pass: PassKind::Translate,
            source_text,
            target_language: target_language.clone(),
            context: context.clone(),
        })?;

        let revised = provider.execute(&ProviderRequest {
            pass: PassKind::Revise,
            source_text: translated.text.clone(),
            target_language: target_language.clone(),
            context: context.clone(),
        })?;

        let reviewed = provider.execute(&ProviderRequest {
            pass: PassKind::QualityReview,
            source_text: revised.text.clone(),
            target_language,
            context,
        })?;

        Ok(PipelineResult {
            translated,
            revised,
            reviewed,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::EchoProvider;

    #[test]
    fn default_pipeline_contains_revision_and_quality_review() {
        let pipeline = TranslationPipeline::default_literary_pipeline();
        assert_eq!(
            pipeline.stages,
            vec![
                PipelineStage::DocumentAnalysis,
                PipelineStage::ContextBuilding,
                PipelineStage::Translation,
                PipelineStage::Revision,
                PipelineStage::QualityReview,
                PipelineStage::Export,
            ]
        );
    }

    #[test]
    fn pipeline_executes_all_model_passes() {
        let result = TranslationPipeline::default_literary_pipeline()
            .execute(&EchoProvider, "chapter text", "fa", "project context")
            .unwrap();

        assert_eq!(result.translated.text, "chapter text");
        assert_eq!(result.revised.text, "chapter text");
        assert_eq!(result.reviewed.text, "chapter text");
        assert_eq!(result.final_text(), "chapter text");
    }

    #[test]
    fn pipeline_rejects_blank_source_before_provider_call() {
        let result = TranslationPipeline::default_literary_pipeline()
            .execute(&EchoProvider, "   ", "fa", "context");
        assert!(matches!(result, Err(ProviderError::InvalidRequest(_))));
    }
}
