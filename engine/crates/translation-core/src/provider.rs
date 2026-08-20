use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PassKind {
    Translate,
    Revise,
    QualityReview,
}

#[derive(Debug, Clone)]
pub struct ProviderRequest {
    pub pass: PassKind,
    pub source_text: String,
    pub target_language: String,
    pub context: String,
}

#[derive(Debug, Clone)]
pub struct ProviderResponse {
    pub text: String,
    pub provider: String,
    pub model: Option<String>,
}

#[derive(Debug, Clone)]
pub enum ProviderError {
    InvalidRequest(String),
    Timeout,
    Unavailable(String),
    Failed(String),
}

impl Display for ProviderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidRequest(message) => write!(f, "invalid provider request: {message}"),
            Self::Timeout => write!(f, "translation provider timed out"),
            Self::Unavailable(message) => write!(f, "translation provider unavailable: {message}"),
            Self::Failed(message) => write!(f, "translation provider failed: {message}"),
        }
    }
}

impl Error for ProviderError {}

pub trait TranslationProvider: Send + Sync {
    fn name(&self) -> &str;
    fn execute(&self, request: &ProviderRequest) -> Result<ProviderResponse, ProviderError>;
}

/// Deterministic provider for tests and local pipeline development.
/// It deliberately performs no model call, allowing the runtime to be wired
/// and tested without credentials or network access.
#[derive(Debug, Default, Clone)]
pub struct EchoProvider;

impl TranslationProvider for EchoProvider {
    fn name(&self) -> &str {
        "echo"
    }

    fn execute(&self, request: &ProviderRequest) -> Result<ProviderResponse, ProviderError> {
        if request.source_text.trim().is_empty() {
            return Err(ProviderError::InvalidRequest("source text is empty".into()));
        }

        Ok(ProviderResponse {
            text: request.source_text.clone(),
            provider: self.name().to_owned(),
            model: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn echo_provider_supports_offline_pipeline_tests() {
        let provider = EchoProvider;
        let response = provider
            .execute(&ProviderRequest {
                pass: PassKind::Translate,
                source_text: "chapter text".into(),
                target_language: "fa".into(),
                context: String::new(),
            })
            .unwrap();

        assert_eq!(response.text, "chapter text");
        assert_eq!(response.provider, "echo");
    }

    #[test]
    fn empty_source_is_rejected() {
        let provider = EchoProvider;
        let result = provider.execute(&ProviderRequest {
            pass: PassKind::Translate,
            source_text: "   ".into(),
            target_language: "fa".into(),
            context: String::new(),
        });
        assert!(result.is_err());
    }
}
