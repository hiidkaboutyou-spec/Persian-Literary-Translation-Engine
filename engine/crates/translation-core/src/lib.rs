pub mod pipeline;
pub mod provider;

pub use pipeline::{PipelineInput, PipelineOutput, PipelineStage, TranslationPipeline};
pub use provider::{
    EchoProvider, OpenAIProvider, PassKind, ProviderError, ProviderRequest, ProviderResponse,
    TranslationProvider,
};

#[derive(Debug, Clone)]
pub struct TranslationRequest {
    pub source: String,
    pub target_language: String,
}

#[derive(Debug, Clone)]
pub struct TranslationContext {
    pub glossary_enabled: bool,
    pub character_memory_enabled: bool,
}

pub fn prepare_translation(request: TranslationRequest, context: TranslationContext) -> String {
    format!(
        "Prepared {} translation with glossary={} character_memory={}",
        request.target_language, context.glossary_enabled, context.character_memory_enabled
    )
}

pub fn execute_translation<P: TranslationProvider>(
    provider: &P,
    request: TranslationRequest,
    context_text: impl Into<String>,
) -> Result<ProviderResponse, ProviderError> {
    provider.execute(&ProviderRequest {
        pass: PassKind::Translate,
        source_text: request.source,
        target_language: request.target_language,
        context: context_text.into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translation_execution_is_provider_neutral() {
        let result = execute_translation(
            &EchoProvider,
            TranslationRequest {
                source: "hello".into(),
                target_language: "fa".into(),
            },
            "character context",
        )
        .unwrap();

        assert_eq!(result.text, "hello");
        assert_eq!(result.provider, "echo");
    }
}
