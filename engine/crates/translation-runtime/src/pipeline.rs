use crate::{TranslationProvider, TranslationRuntimeError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TranslationRequest {
    pub source_text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TranslationOutput {
    pub translated_text: String,
}

pub struct TranslationPipeline<P> {
    provider: P,
}

impl<P> TranslationPipeline<P>
where
    P: TranslationProvider,
{
    pub fn new(provider: P) -> Self {
        Self { provider }
    }

    pub fn execute(
        &self,
        request: TranslationRequest,
    ) -> Result<TranslationOutput, TranslationRuntimeError> {
        let translated_text = self.provider.translate(&request.source_text)?;

        Ok(TranslationOutput { translated_text })
    }
}
