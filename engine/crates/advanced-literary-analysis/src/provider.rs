use crate::models::{
    AnalysisProviderRequest, AnalysisProviderResponse, FindingResponse, UsageMetadata,
};
use std::fmt;
use std::time::Duration;

/// Provider-neutral boundary for advanced literary analysis.
///
/// Implementations may call external models or return deterministic test data.
/// The provider receives a fully constructed, versioned prompt and must return
/// structured JSON-compatible findings — never free-form prose as the primary
/// contract. Providers never decide whether a finding is valid; deterministic
/// validation in `crate::validation` owns that decision.
pub trait LiteraryAnalysisProvider: Send + Sync {
    fn name(&self) -> &str;
    fn model(&self) -> &str;
    fn analyze(
        &self,
        request: &AnalysisProviderRequest,
    ) -> Result<AnalysisProviderResponse, AnalysisProviderError>;
}

#[derive(Debug, Clone)]
pub enum AnalysisProviderError {
    /// Provider could not be reached (retryable).
    Unavailable(String),
    /// Request was malformed.
    InvalidRequest(String),
    /// Provider returned unparseable output (not retried).
    ParseError(String),
    /// Provider returned output that failed schema validation (not retried).
    SchemaViolation(String),
    /// Provider rate-limited the request (retryable).
    RateLimited,
    /// Authentication or configuration failure (not retried).
    Authentication(String),
    /// General failure.
    Failed(String),
}

impl fmt::Display for AnalysisProviderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unavailable(message) => write!(f, "analysis provider unavailable: {message}"),
            Self::InvalidRequest(message) => write!(f, "invalid analysis request: {message}"),
            Self::ParseError(message) => {
                write!(f, "analysis provider output parse error: {message}")
            }
            Self::SchemaViolation(message) => {
                write!(f, "analysis provider schema violation: {message}")
            }
            Self::RateLimited => write!(f, "analysis provider rate limited"),
            Self::Authentication(message) => write!(f, "analysis provider auth error: {message}"),
            Self::Failed(message) => write!(f, "analysis provider failed: {message}"),
        }
    }
}

impl std::error::Error for AnalysisProviderError {}

impl AnalysisProviderError {
    /// Only transport-level failures are retried. Schema, auth, and permanent
    /// validation failures are never retried.
    pub fn is_retryable(&self) -> bool {
        matches!(self, Self::RateLimited | Self::Unavailable(_))
    }
}

// ---------------------------------------------------------------------------
// Deterministic mock provider (never calls external APIs)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct MockFindingTemplate {
    pub category: String,
    pub subject: String,
    pub claim: String,
    pub confidence: f32,
    pub evidence_ordinals: Vec<String>,
    pub uncertainty: Option<String>,
}

impl Default for MockFindingTemplate {
    fn default() -> Self {
        Self {
            category: "tone".to_string(),
            subject: "scene".to_string(),
            claim: "neutral narrative tone".to_string(),
            confidence: 0.7,
            evidence_ordinals: Vec::new(),
            uncertainty: Some("based on limited textual evidence".to_string()),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct MockAnalysisProviderConfig {
    /// Findings to emit for every analysis unit (one template per finding).
    pub findings_per_unit: Vec<MockFindingTemplate>,
    /// Unit ids (prefix match) that fail with a permanent error.
    pub failing_unit_prefixes: Vec<String>,
    /// Unit ids (prefix match) that fail with a retryable error.
    pub retryable_fail_prefixes: Vec<String>,
    pub usage: UsageMetadata,
}

/// Deterministic mock provider for offline tests and CLI smoke runs.
pub struct MockAnalysisProvider {
    config: MockAnalysisProviderConfig,
}

impl MockAnalysisProvider {
    pub fn new(config: MockAnalysisProviderConfig) -> Self {
        Self { config }
    }

    pub fn with_default_findings() -> Self {
        Self::new(MockAnalysisProviderConfig {
            findings_per_unit: vec![MockFindingTemplate::default()],
            ..Default::default()
        })
    }

    pub fn with_failing_units(prefixes: Vec<String>) -> Self {
        Self::new(MockAnalysisProviderConfig {
            failing_unit_prefixes: prefixes,
            ..Default::default()
        })
    }

    pub fn with_retryable_failures(prefixes: Vec<String>) -> Self {
        Self::new(MockAnalysisProviderConfig {
            retryable_fail_prefixes: prefixes,
            ..Default::default()
        })
    }
}

impl LiteraryAnalysisProvider for MockAnalysisProvider {
    fn name(&self) -> &str {
        "mock"
    }

    fn model(&self) -> &str {
        "mock-v1"
    }

    fn analyze(
        &self,
        request: &AnalysisProviderRequest,
    ) -> Result<AnalysisProviderResponse, AnalysisProviderError> {
        let unit_id = &request.unit.unit_id;

        if self
            .config
            .failing_unit_prefixes
            .iter()
            .any(|prefix| unit_id.starts_with(prefix))
        {
            return Err(AnalysisProviderError::Failed(format!(
                "simulated permanent failure for unit {unit_id}"
            )));
        }
        if self
            .config
            .retryable_fail_prefixes
            .iter()
            .any(|prefix| unit_id.starts_with(prefix))
        {
            return Err(AnalysisProviderError::RateLimited);
        }

        // Cite only evidence identifiers that were actually supplied.
        let supplied = request.unit.supplied_ordinals();
        let default_evidence: Vec<String> = supplied.iter().take(2).cloned().collect();

        let findings: Vec<FindingResponse> = self
            .config
            .findings_per_unit
            .iter()
            .map(|template| FindingResponse {
                category: template.category.clone(),
                subject: template.subject.clone(),
                claim: template.claim.clone(),
                confidence: template.confidence,
                evidence_ordinals: if template.evidence_ordinals.is_empty() {
                    default_evidence.clone()
                } else {
                    template.evidence_ordinals.clone()
                },
                uncertainty: template.uncertainty.clone(),
                alternative_interpretations: Vec::new(),
            })
            .collect();

        Ok(AnalysisProviderResponse {
            findings,
            usage: Some(self.config.usage.clone()),
        })
    }
}

// ---------------------------------------------------------------------------
// OpenAI analysis provider (optional; requires OPENAI_API_KEY at runtime)
// ---------------------------------------------------------------------------

/// OpenAI Responses API provider for literary analysis. Kept provider-neutral
/// at the domain boundary: it only serializes the versioned prompt and parses
/// the model's structured JSON response.
#[derive(Clone)]
pub struct OpenAIAnalysisProvider {
    api_key: String,
    model: String,
    base_url: String,
    timeout: Duration,
}

impl fmt::Debug for OpenAIAnalysisProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OpenAIAnalysisProvider")
            .field("model", &self.model)
            .field("base_url", &self.base_url)
            .field("timeout", &self.timeout)
            .finish_non_exhaustive()
    }
}

impl OpenAIAnalysisProvider {
    pub fn from_env() -> Result<Self, AnalysisProviderError> {
        let api_key = std::env::var("OPENAI_API_KEY").map_err(|_| {
            AnalysisProviderError::Authentication("OPENAI_API_KEY is not configured".to_string())
        })?;
        let model = std::env::var("LITERARY_ENGINE_ANALYSIS_MODEL")
            .or_else(|_| std::env::var("OPENAI_MODEL"))
            .unwrap_or_else(|_| "gpt-5.6".to_string());
        Self::new(api_key, model)
    }

    pub fn new(
        api_key: impl Into<String>,
        model: impl Into<String>,
    ) -> Result<Self, AnalysisProviderError> {
        let api_key = api_key.into();
        let model = model.into();
        if api_key.trim().is_empty() {
            return Err(AnalysisProviderError::InvalidRequest(
                "OpenAI API key is empty".to_string(),
            ));
        }
        if model.trim().is_empty() {
            return Err(AnalysisProviderError::InvalidRequest(
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

    #[cfg(test)]
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    fn parse_response_body(body: &str) -> Result<AnalysisProviderResponse, AnalysisProviderError> {
        let value: serde_json::Value = serde_json::from_str(body).map_err(|error| {
            AnalysisProviderError::ParseError(format!("invalid provider JSON: {error}"))
        })?;
        let output = value
            .get("output")
            .and_then(serde_json::Value::as_array)
            .ok_or_else(|| {
                AnalysisProviderError::SchemaViolation(
                    "provider response has no output array".to_string(),
                )
            })?;
        let mut text_parts = Vec::new();
        for item in output {
            let Some(content) = item.get("content").and_then(serde_json::Value::as_array) else {
                continue;
            };
            for part in content {
                if part.get("type").and_then(serde_json::Value::as_str) != Some("output_text") {
                    continue;
                }
                if let Some(text) = part.get("text").and_then(serde_json::Value::as_str) {
                    if !text.is_empty() {
                        text_parts.push(text);
                    }
                }
            }
        }
        if text_parts.is_empty() {
            return Err(AnalysisProviderError::SchemaViolation(
                "provider response contained no output text".to_string(),
            ));
        }
        let joined = text_parts.join("\n");
        let trimmed = joined
            .trim()
            .trim_start_matches("```json")
            .trim_end_matches("```")
            .trim();
        let parsed: crate::models::AnalysisProviderResponse = serde_json::from_str(trimmed)
            .map_err(|error| {
                AnalysisProviderError::ParseError(format!(
                    "provider did not return the analysis JSON schema: {error}"
                ))
            })?;
        Ok(parsed)
    }
}

impl LiteraryAnalysisProvider for OpenAIAnalysisProvider {
    fn name(&self) -> &str {
        "openai"
    }

    fn model(&self) -> &str {
        &self.model
    }

    fn analyze(
        &self,
        request: &AnalysisProviderRequest,
    ) -> Result<AnalysisProviderResponse, AnalysisProviderError> {
        let client = reqwest::blocking::Client::builder()
            .timeout(self.timeout)
            .build()
            .map_err(|error| AnalysisProviderError::Unavailable(error.to_string()))?;

        let payload = serde_json::json!({
            "model": self.model,
            "instructions": request.prompt.system,
            "input": request.prompt.user,
            "store": false
        });

        let response = client
            .post(&self.base_url)
            .bearer_auth(&self.api_key)
            .json(&payload)
            .send()
            .map_err(|error| {
                if error.is_timeout() {
                    AnalysisProviderError::Unavailable("provider request timed out".to_string())
                } else if error.is_connect() {
                    AnalysisProviderError::Unavailable(error.to_string())
                } else {
                    AnalysisProviderError::Failed(error.to_string())
                }
            })?;

        let status = response.status();
        let body = response
            .text()
            .map_err(|error| AnalysisProviderError::Failed(error.to_string()))?;

        if !status.is_success() {
            let detail = serde_json::from_str::<serde_json::Value>(&body)
                .ok()
                .and_then(|value| {
                    value
                        .pointer("/error/message")
                        .and_then(serde_json::Value::as_str)
                        .map(ToOwned::to_owned)
                })
                .unwrap_or_else(|| format!("HTTP {status}"));
            if status.as_u16() == 401 || status.as_u16() == 403 {
                return Err(AnalysisProviderError::Authentication(detail));
            }
            if status.as_u16() == 429 {
                return Err(AnalysisProviderError::RateLimited);
            }
            return Err(AnalysisProviderError::Failed(detail));
        }

        Self::parse_response_body(&body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        AnalysisUnit, AnalysisUnitType, CanonContext, ProviderPrompt, PROMPT_VERSION,
    };
    use document_engine::{DocumentFormat, SourceLocation};

    fn unit() -> AnalysisUnit {
        AnalysisUnit {
            unit_id: "unit-test".to_string(),
            unit_type: AnalysisUnitType::Scene,
            chapter_id: "chapter-1".to_string(),
            chapter_index: 0,
            scene_id: Some("scene-1".to_string()),
            text_content: "[para-1] first paragraph\n\n[para-2] second paragraph".to_string(),
            text_fingerprint: "fp".to_string(),
            paragraph_ids: vec!["para-id-1".to_string(), "para-id-2".to_string()],
            previous_scene_context: None,
            next_scene_context: None,
            known_character_names: Vec::new(),
            known_glossary_terms: Vec::new(),
            source: SourceLocation::new("synthetic.txt", DocumentFormat::Txt),
        }
    }

    fn request(unit: &AnalysisUnit) -> AnalysisProviderRequest {
        AnalysisProviderRequest {
            unit: unit.clone(),
            prompt: ProviderPrompt {
                system: "system".to_string(),
                user: "user".to_string(),
                prompt_version: PROMPT_VERSION.to_string(),
            },
            categories: Vec::new(),
            canon_context: CanonContext::default(),
        }
    }

    #[test]
    fn mock_provider_cites_only_supplied_evidence() {
        let provider = MockAnalysisProvider::with_default_findings();
        let response = provider.analyze(&request(&unit())).unwrap();
        let finding = &response.findings[0];
        assert!(finding
            .evidence_ordinals
            .iter()
            .all(|ordinal| unit().resolve_ordinal(ordinal).is_some()));
    }

    #[test]
    fn mock_provider_reports_permanent_and_retryable_failures() {
        let provider = MockAnalysisProvider::with_failing_units(vec!["unit-test".to_string()]);
        assert!(matches!(
            provider.analyze(&request(&unit())),
            Err(AnalysisProviderError::Failed(_))
        ));
        let provider = MockAnalysisProvider::with_retryable_failures(vec!["unit-test".to_string()]);
        assert!(matches!(
            provider.analyze(&request(&unit())),
            Err(AnalysisProviderError::RateLimited)
        ));
        assert!(AnalysisProviderError::RateLimited.is_retryable());
        assert!(!matches!(
            AnalysisProviderError::ParseError("bad".to_string()),
            AnalysisProviderError::RateLimited
        ));
        assert!(!AnalysisProviderError::ParseError("bad".to_string()).is_retryable());
    }

    #[test]
    fn mock_provider_usage_is_reported() {
        let provider = MockAnalysisProvider::new(MockAnalysisProviderConfig {
            usage: UsageMetadata {
                input_tokens: 10,
                output_tokens: 5,
                total_tokens: 15,
                request_count: 1,
            },
            ..Default::default()
        });
        let response = provider.analyze(&request(&unit())).unwrap();
        let usage = response.usage.unwrap();
        assert_eq!(usage.input_tokens, 10);
        assert_eq!(usage.total_tokens, 15);
    }
}
