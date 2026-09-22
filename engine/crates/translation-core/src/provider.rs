use reqwest::blocking::Client;
use reqwest::header::RETRY_AFTER;
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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ProviderUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
}

#[derive(Debug, Clone)]
pub struct ProviderResponse {
    pub text: String,
    pub provider: String,
    pub model: Option<String>,
    pub usage: Option<ProviderUsage>,
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
            usage: None,
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
            "You are the production translation engine for a long-form fiction workflow. Target language: {target_language}. {task} Treat the provided project context as binding continuity guidance when relevant. Structural marker tokens used by the document layer (for example <m1>...</m1> and <r1/>) are immutable placeholders: preserve every marker identifier, count, pairing, and relative order exactly through translation, revision, and quality review. Return only the resulting passage."
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

    fn extract_usage(value: &Value) -> Option<ProviderUsage> {
        let usage = value.get("usage")?;
        let input_tokens = usage.get("input_tokens")?.as_u64()?;
        let output_tokens = usage.get("output_tokens")?.as_u64()?;
        Some(ProviderUsage {
            input_tokens,
            output_tokens,
        })
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
        let usage = Self::extract_usage(&value);

        Ok(ProviderResponse {
            text,
            provider: self.name().to_owned(),
            model: Some(self.model.clone()),
            usage,
        })
    }
}

/// Experimental Atria Responses API provider.
///
/// Phase 31 exposes this provider only to the rights-safe provider qualification
/// path. The production ApplicationService provider selector deliberately does
/// not admit Atria yet. Credentials come only from `ATRIA_API_KEY`.
#[derive(Clone)]
pub struct AtriaProvider {
    api_key: String,
    model: String,
    base_url: String,
    timeout: Duration,
    max_output_tokens: u32,
}

impl Debug for AtriaProvider {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AtriaProvider")
            .field("model", &self.model)
            .field("base_url", &self.base_url)
            .field("timeout", &self.timeout)
            .field("max_output_tokens", &self.max_output_tokens)
            .finish_non_exhaustive()
    }
}

impl AtriaProvider {
    pub const DEFAULT_MODEL: &'static str = "Atria-Dawn-Preview";
    pub const DEFAULT_MAX_OUTPUT_TOKENS: u32 = 8_192;
    pub const MAX_OUTPUT_TOKENS: u32 = 65_536;

    pub fn from_env() -> Result<Self, ProviderError> {
        let api_key = env::var("ATRIA_API_KEY").map_err(|_| {
            ProviderError::Unavailable("ATRIA_API_KEY is not configured".to_string())
        })?;
        let model = env::var("ATRIA_MODEL").unwrap_or_else(|_| Self::DEFAULT_MODEL.to_string());
        let max_output_tokens = match env::var("ATRIA_MAX_OUTPUT_TOKENS") {
            Ok(value) if !value.trim().is_empty() => value.trim().parse::<u32>().map_err(|_| {
                ProviderError::InvalidRequest(
                    "ATRIA_MAX_OUTPUT_TOKENS must be an integer in 1..=65536".to_string(),
                )
            })?,
            _ => Self::DEFAULT_MAX_OUTPUT_TOKENS,
        };
        Self::new(api_key, model)?.with_max_output_tokens(max_output_tokens)
    }

    pub fn new(
        api_key: impl Into<String>,
        model: impl Into<String>,
    ) -> Result<Self, ProviderError> {
        let api_key = api_key.into();
        let model = model.into();
        if api_key.trim().is_empty() {
            return Err(ProviderError::InvalidRequest(
                "Atria API key is empty".to_string(),
            ));
        }
        if model.trim().is_empty() {
            return Err(ProviderError::InvalidRequest(
                "Atria model name is empty".to_string(),
            ));
        }

        Ok(Self {
            api_key,
            model,
            base_url: "https://api.atria-asi.ai/v1/responses".to_string(),
            timeout: Duration::from_secs(180),
            max_output_tokens: Self::DEFAULT_MAX_OUTPUT_TOKENS,
        })
    }

    pub fn model(&self) -> &str {
        &self.model
    }

    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_max_output_tokens(mut self, max_output_tokens: u32) -> Result<Self, ProviderError> {
        if !(1..=Self::MAX_OUTPUT_TOKENS).contains(&max_output_tokens) {
            return Err(ProviderError::InvalidRequest(format!(
                "Atria max_output_tokens must be in 1..={}",
                Self::MAX_OUTPUT_TOKENS
            )));
        }
        self.max_output_tokens = max_output_tokens;
        Ok(self)
    }

    fn input_for(request: &ProviderRequest) -> String {
        format!(
            "INSTRUCTIONS\n{}\n\n{}",
            OpenAIProvider::instructions_for(&request.pass, &request.target_language),
            OpenAIProvider::user_input(request)
        )
    }

    fn payload(&self, request: &ProviderRequest) -> Value {
        json!({
            "model": self.model,
            "input": Self::input_for(request),
            "max_output_tokens": self.max_output_tokens
        })
    }

    fn extract_output_text(value: &Value) -> Result<String, ProviderError> {
        let output = value
            .get("output")
            .and_then(Value::as_array)
            .ok_or_else(|| ProviderError::Failed("Atria response has no output array".into()))?;

        let mut parts = Vec::new();
        for item in output {
            let Some(content) = item.get("content").and_then(Value::as_array) else {
                continue;
            };
            for part in content {
                if let Some(text) = part.get("text").and_then(Value::as_str) {
                    if !text.is_empty() {
                        parts.push(text);
                    }
                }
            }
        }

        if parts.is_empty() {
            return Err(ProviderError::Failed(
                "Atria response contained no output text".into(),
            ));
        }
        Ok(parts.join(""))
    }
}

impl TranslationProvider for AtriaProvider {
    fn name(&self) -> &str {
        "atria"
    }

    fn execute(&self, request: &ProviderRequest) -> Result<ProviderResponse, ProviderError> {
        validate_request(request)?;

        let client = Client::builder()
            .timeout(self.timeout)
            .build()
            .map_err(|error| ProviderError::Unavailable(error.to_string()))?;

        let response = client
            .post(&self.base_url)
            .bearer_auth(&self.api_key)
            .json(&self.payload(request))
            .send()
            .map_err(|error| {
                if error.is_timeout() {
                    ProviderError::Timeout
                } else {
                    ProviderError::Unavailable(error.to_string())
                }
            })?;

        let status = response.status();
        let retry_after = response
            .headers()
            .get(RETRY_AFTER)
            .and_then(|value| value.to_str().ok())
            .map(ToOwned::to_owned);
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

            if status.as_u16() == 429 {
                let suffix = retry_after
                    .map(|seconds| format!("; retry after {seconds}s"))
                    .unwrap_or_default();
                return Err(ProviderError::Unavailable(format!(
                    "Atria rate limit exceeded{suffix}"
                )));
            }
            if status.as_u16() == 401 {
                return Err(ProviderError::Failed(
                    "Atria authentication failed; check ATRIA_API_KEY".to_string(),
                ));
            }
            return Err(ProviderError::Failed(detail));
        }

        let value: Value = serde_json::from_str(&body)
            .map_err(|error| ProviderError::Failed(format!("invalid Atria JSON: {error}")))?;
        let text = Self::extract_output_text(&value)?;
        let usage = OpenAIProvider::extract_usage(&value);

        Ok(ProviderResponse {
            text,
            provider: self.name().to_owned(),
            model: Some(self.model.clone()),
            usage,
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
    fn atria_provider_uses_documented_responses_contract_without_openai_storage_fields() {
        let provider = AtriaProvider::new("secret", AtriaProvider::DEFAULT_MODEL)
            .unwrap()
            .with_max_output_tokens(4096)
            .unwrap();
        let request = ProviderRequest {
            pass: PassKind::Translate,
            source_text: "He did not answer.".into(),
            target_language: "fa".into(),
            context: "Keep the reply restrained.".into(),
        };

        let payload = provider.payload(&request);
        assert_eq!(payload["model"], "Atria-Dawn-Preview");
        assert_eq!(payload["max_output_tokens"], 4096);
        assert!(payload.get("store").is_none());
        let input = payload["input"].as_str().unwrap();
        assert!(input.contains("PROJECT CONTEXT"));
        assert!(input.contains("He did not answer."));
        assert!(input.contains("immutable placeholders"));
    }

    #[test]
    fn atria_provider_extracts_documented_responses_text_shape() {
        let value = json!({
            "output": [{
                "type": "message",
                "content": [
                    {"type": "output_text", "text": "جواب "},
                    {"type": "output_text", "text": "نداد."}
                ]
            }]
        });
        assert_eq!(
            AtriaProvider::extract_output_text(&value).unwrap(),
            "جواب نداد."
        );
    }

    #[test]
    fn atria_output_limit_is_fail_closed() {
        assert!(AtriaProvider::new("secret", AtriaProvider::DEFAULT_MODEL)
            .unwrap()
            .with_max_output_tokens(0)
            .is_err());
        assert!(AtriaProvider::new("secret", AtriaProvider::DEFAULT_MODEL)
            .unwrap()
            .with_max_output_tokens(65_537)
            .is_err());
    }

    #[test]
    fn responses_usage_metadata_is_parsed_when_present() {
        let value = json!({
            "usage": {
                "input_tokens": 123,
                "output_tokens": 45
            }
        });
        assert_eq!(
            OpenAIProvider::extract_usage(&value),
            Some(ProviderUsage {
                input_tokens: 123,
                output_tokens: 45,
            })
        );
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
