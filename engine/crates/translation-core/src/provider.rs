use reqwest::blocking::Client;
use serde_json::{json, Value};
use std::env;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::time::Duration;

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
        validate_request(request)?;

        Ok(ProviderResponse {
            text: request.source_text.clone(),
            provider: self.name().to_owned(),
            model: None,
        })
    }
}

/// OpenAI Responses API provider for production translation passes.
///
/// Credentials are never stored in repository files. Use `OPENAI_API_KEY` and
/// optionally `OPENAI_MODEL`. Response storage is disabled in each request so
/// manuscript content is not intentionally retained as application state.
#[derive(Clone)]
pub struct OpenAIProvider {
    api_key: String,
    model: String,
    base_url: String,
    timeout: Duration,
}

impl Debug for OpenAIProvider {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OpenAIProvider")
            .field("model", &self.model)
            .field("base_url", &self.base_url)
            .field("timeout", &self.timeout)
            .finish_non_exhaustive()
    }
}

impl OpenAIProvider {
    pub fn from_env() -> Result<Self, ProviderError> {
        let api_key = env::var("OPENAI_API_KEY").map_err(|_| {
            ProviderError::Unavailable("OPENAI_API_KEY is not configured".to_string())
        })?;
        let model = env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-5.6".to_string());
        Self::new(api_key, model)
    }

    pub fn new(
        api_key: impl Into<String>,
        model: impl Into<String>,
    ) -> Result<Self, ProviderError> {
        let api_key = api_key.into();
        let model = model.into();
        if api_key.trim().is_empty() {
            return Err(ProviderError::InvalidRequest(
                "OpenAI API key is empty".to_string(),
            ));
        }
        if model.trim().is_empty() {
            return Err(ProviderError::InvalidRequest(
                "OpenAI model name is empty".to_string(),
            ));
        }

        Ok(Self {
            api_key,
            model,
            base_url: "https://api.openai.com/v1/responses".to_string(),
            timeout: Duration::from_secs(180),
        })
    }

    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    fn instructions_for(pass: &PassKind, target_language: &str) -> String {
        let task = match pass {
            PassKind::Translate => {
                "Translate the source passage faithfully as polished literary prose. Preserve every narrative beat, dialogue turn, characterization choice, emotional cue, relationship dynamic, and meaningful detail. Do not summarize, censor, omit, invent, or explain."
            }
            PassKind::Revise => {
                "Revise the supplied translation as a professional literary editor. Improve naturalness, rhythm, dialogue, register, and readability while preserving all meaning, events, characterization, emotional intensity, and continuity. Do not add commentary or remove content."
            }
            PassKind::QualityReview => {
                "Perform the final publication-quality editorial pass. Correct mistranslation, inconsistency, awkward phrasing, voice drift, terminology drift, grammar, and punctuation while preserving the complete text and its intended tone. Return only the corrected passage with no notes."
            }
        };

        format!(
            "You are the production translation engine for a long-form fiction workflow. Target language: {target_language}. {task} Treat the provided project context as binding continuity guidance when relevant. Return only the resulting passage."
        )
    }

    fn user_input(request: &ProviderRequest) -> String {
        if request.context.trim().is_empty() {
            return request.source_text.clone();
        }

        format!(
            "PROJECT CONTEXT\n{}\n\nPASSAGE\n{}",
            request.context, request.source_text
        )
    }

    fn extract_output_text(value: &Value) -> Result<String, ProviderError> {
        let output = value
            .get("output")
            .and_then(Value::as_array)
            .ok_or_else(|| ProviderError::Failed("OpenAI response has no output array".into()))?;

        let mut parts = Vec::new();
        for item in output {
            let Some(content) = item.get("content").and_then(Value::as_array) else {
                continue;
            };
            for part in content {
                if part.get("type").and_then(Value::as_str) != Some("output_text") {
                    continue;
                }
                if let Some(text) = part.get("text").and_then(Value::as_str) {
                    if !text.is_empty() {
                        parts.push(text);
                    }
                }
            }
        }

        if parts.is_empty() {
            return Err(ProviderError::Failed(
                "OpenAI response contained no output text".into(),
            ));
        }

        Ok(parts.join(""))
    }
}

impl TranslationProvider for OpenAIProvider {
    fn name(&self) -> &str {
        "openai"
    }

    fn execute(&self, request: &ProviderRequest) -> Result<ProviderResponse, ProviderError> {
        validate_request(request)?;

        let client = Client::builder()
            .timeout(self.timeout)
            .build()
            .map_err(|error| ProviderError::Unavailable(error.to_string()))?;

        let payload = json!({
            "model": self.model,
            "instructions": Self::instructions_for(&request.pass, &request.target_language),
            "input": Self::user_input(request),
            "store": false
        });

        let response = client
            .post(&self.base_url)
            .bearer_auth(&self.api_key)
            .json(&payload)
            .send()
            .map_err(|error| {
                if error.is_timeout() {
                    ProviderError::Timeout
                } else {
                    ProviderError::Unavailable(error.to_string())
                }
            })?;

        let status = response.status();
        let body = response
            .text()
            .map_err(|error| ProviderError::Failed(error.to_string()))?;

        if !status.is_success() {
            let detail = serde_json::from_str::<Value>(&body)
                .ok()
                .and_then(|value| {
                    value
                        .pointer("/error/message")
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned)
                })
                .unwrap_or_else(|| format!("HTTP {status}"));
            return Err(ProviderError::Failed(detail));
        }

        let value: Value = serde_json::from_str(&body)
            .map_err(|error| ProviderError::Failed(format!("invalid OpenAI JSON: {error}")))?;
        let text = Self::extract_output_text(&value)?;

        Ok(ProviderResponse {
            text,
            provider: self.name().to_owned(),
            model: Some(self.model.clone()),
        })
    }
}

fn validate_request(request: &ProviderRequest) -> Result<(), ProviderError> {
    if request.source_text.trim().is_empty() {
        return Err(ProviderError::InvalidRequest("source text is empty".into()));
    }
    if request.target_language.trim().is_empty() {
        return Err(ProviderError::InvalidRequest(
            "target language is empty".into(),
        ));
    }
    Ok(())
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

    #[test]
    fn openai_provider_extracts_text_from_responses_shape() {
        let value = json!({
            "output": [
                {
                    "type": "message",
                    "content": [
                        {"type": "output_text", "text": "سلام "},
                        {"type": "output_text", "text": "دنیا"}
                    ]
                }
            ]
        });

        assert_eq!(
            OpenAIProvider::extract_output_text(&value).unwrap(),
            "سلام دنیا"
        );
    }

    #[test]
    fn openai_provider_context_is_explicitly_separated_from_passage() {
        let request = ProviderRequest {
            pass: PassKind::Translate,
            source_text: "source passage".into(),
            target_language: "fa".into(),
            context: "character voice".into(),
        };

        assert_eq!(
            OpenAIProvider::user_input(&request),
            "PROJECT CONTEXT\ncharacter voice\n\nPASSAGE\nsource passage"
        );
    }

    #[test]
    fn pass_instructions_are_distinct() {
        let translate = OpenAIProvider::instructions_for(&PassKind::Translate, "fa");
        let revise = OpenAIProvider::instructions_for(&PassKind::Revise, "fa");
        let review = OpenAIProvider::instructions_for(&PassKind::QualityReview, "fa");

        assert_ne!(translate, revise);
        assert_ne!(revise, review);
        assert!(review.contains("final publication-quality"));
    }
}
