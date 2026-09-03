use chrono::{DateTime, Utc};
use document_engine::SourceLocation;
use serde::{Deserialize, Serialize};
use std::fmt;

pub const ADVANCED_ANALYSIS_SCHEMA_VERSION: u32 = 1;
pub const PROMPT_VERSION: &str = "literary-analysis-v1";

// ---------------------------------------------------------------------------
// Finding categories
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingCategory {
    PointOfView,
    NarratorVoice,
    Tone,
    CharacterVoice,
    RelationshipDynamic,
    PowerDynamic,
    EmotionalShift,
    Subtext,
    Sarcasm,
    Humor,
    IntimacyRegister,
    DialogueFunction,
    SceneIntent,
    NarrativeTension,
    Motif,
    RecurringImagery,
    RegisterShift,
    ContinuityObservation,
}

impl fmt::Display for FindingCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::PointOfView => "point_of_view",
            Self::NarratorVoice => "narrator_voice",
            Self::Tone => "tone",
            Self::CharacterVoice => "character_voice",
            Self::RelationshipDynamic => "relationship_dynamic",
            Self::PowerDynamic => "power_dynamic",
            Self::EmotionalShift => "emotional_shift",
            Self::Subtext => "subtext",
            Self::Sarcasm => "sarcasm",
            Self::Humor => "humor",
            Self::IntimacyRegister => "intimacy_register",
            Self::DialogueFunction => "dialogue_function",
            Self::SceneIntent => "scene_intent",
            Self::NarrativeTension => "narrative_tension",
            Self::Motif => "motif",
            Self::RecurringImagery => "recurring_imagery",
            Self::RegisterShift => "register_shift",
            Self::ContinuityObservation => "continuity_observation",
        };
        write!(f, "{label}")
    }
}

impl FindingCategory {
    pub fn all() -> &'static [FindingCategory] {
        &[
            Self::PointOfView,
            Self::NarratorVoice,
            Self::Tone,
            Self::CharacterVoice,
            Self::RelationshipDynamic,
            Self::PowerDynamic,
            Self::EmotionalShift,
            Self::Subtext,
            Self::Sarcasm,
            Self::Humor,
            Self::IntimacyRegister,
            Self::DialogueFunction,
            Self::SceneIntent,
            Self::NarrativeTension,
            Self::Motif,
            Self::RecurringImagery,
            Self::RegisterShift,
            Self::ContinuityObservation,
        ]
    }
}

impl FindingCategory {
    pub fn from_label(label: &str) -> Option<Self> {
        Self::all()
            .iter()
            .find(|category| category.to_string() == label.to_ascii_lowercase())
            .copied()
    }
}

// ---------------------------------------------------------------------------
// Narrative scope (where in the manuscript a finding applies)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum NarrativeScope {
    Global,
    Chapter {
        chapter_id: String,
        chapter_index: usize,
    },
    Scene {
        chapter_id: String,
        scene_id: String,
    },
    ChapterRange {
        from_chapter_index: usize,
        to_chapter_index: usize,
    },
    Character {
        character_name: String,
    },
    Relationship {
        character_a: String,
        character_b: String,
    },
}

impl NarrativeScope {
    /// Stable serialized form used for identity generation and filtering.
    pub fn key(&self) -> String {
        match self {
            Self::Global => "global".to_string(),
            Self::Chapter { chapter_id, .. } => format!("chapter:{chapter_id}"),
            Self::Scene {
                chapter_id,
                scene_id,
            } => format!("scene:{chapter_id}:{scene_id}"),
            Self::ChapterRange {
                from_chapter_index,
                to_chapter_index,
            } => format!("range:{from_chapter_index}-{to_chapter_index}"),
            Self::Character { character_name } => {
                format!("character:{}", normalize_for_identity(character_name))
            }
            Self::Relationship {
                character_a,
                character_b,
            } => {
                let (a, b) = normalized_pair(character_a, character_b);
                format!("relationship:{a}:{b}")
            }
        }
    }

    /// Compact human-readable label used in review items and summaries.
    pub fn label(&self) -> String {
        match self {
            Self::Global => "global".to_string(),
            Self::Chapter {
                chapter_id,
                chapter_index,
            } => format!("chapter {} [{chapter_id}]", chapter_index + 1),
            Self::Scene {
                chapter_id,
                scene_id,
            } => format!("scene {scene_id} [{chapter_id}]"),
            Self::ChapterRange {
                from_chapter_index,
                to_chapter_index,
            } => format!(
                "chapters {}–{}",
                from_chapter_index + 1,
                to_chapter_index + 1
            ),
            Self::Character { character_name } => format!("character {character_name}"),
            Self::Relationship {
                character_a,
                character_b,
            } => format!("relationship {character_a} / {character_b}"),
        }
    }

    pub fn chapter_ids(&self) -> Vec<String> {
        match self {
            Self::Global | Self::Character { .. } | Self::Relationship { .. } => Vec::new(),
            Self::Chapter { chapter_id, .. } => vec![chapter_id.clone()],
            Self::Scene { chapter_id, .. } => vec![chapter_id.clone()],
            Self::ChapterRange { .. } => Vec::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// Confidence
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfidenceLevel {
    Low,
    Moderate,
    High,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DerivedConfidence {
    pub level: ConfidenceLevel,
    /// Provider-reported confidence is preserved but never trusted verbatim.
    pub model_reported_confidence: Option<f32>,
    /// Fraction of the analysis unit's paragraphs cited by this finding.
    pub evidence_coverage: f32,
    pub supporting_unit_count: usize,
    pub contradicting_unit_count: usize,
    pub has_deterministic_signal: bool,
}

impl DerivedConfidence {
    pub fn validate(&self) -> Result<(), String> {
        if self.evidence_coverage < 0.0 || self.evidence_coverage > 1.0 {
            return Err(format!(
                "evidence_coverage must be 0.0..1.0, got {}",
                self.evidence_coverage
            ));
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Evidence (validated against actual source identifiers)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidatedEvidenceRef {
    pub chapter_id: String,
    pub scene_id: String,
    pub paragraph_id: String,
    pub source: SourceLocation,
}

// ---------------------------------------------------------------------------
// Provider metadata
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderMetadata {
    pub provider_name: String,
    pub model: String,
    pub prompt_version: String,
    pub analysis_schema_version: u32,
    pub configuration_fingerprint: String,
}

// ---------------------------------------------------------------------------
// Bounded analysis units
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisUnitType {
    Scene,
    Chapter,
}

/// One bounded provider request. Never an entire novel.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnalysisUnit {
    pub unit_id: String,
    pub unit_type: AnalysisUnitType,
    pub chapter_id: String,
    pub chapter_index: usize,
    pub scene_id: Option<String>,
    /// Manuscript text for this unit with each paragraph preceded by its
    /// ordinal token (for example `[para-3]`), so the model can cite exactly
    /// the evidence identifiers that were supplied.
    pub text_content: String,
    pub text_fingerprint: String,
    /// Real document paragraph identifiers in presentation order. Provider
    /// output may only reference ordinals that map onto this list.
    pub paragraph_ids: Vec<String>,
    pub previous_scene_context: Option<String>,
    pub next_scene_context: Option<String>,
    pub known_character_names: Vec<String>,
    pub known_glossary_terms: Vec<String>,
    pub source: SourceLocation,
}

impl AnalysisUnit {
    pub fn ordinal_token(position: usize) -> String {
        format!("para-{}", position + 1)
    }

    /// Deterministic identity for this unit derived only from stable inputs.
    pub fn unit_id(
        chapter_id: &str,
        scene_id: Option<&str>,
        text_fingerprint: &str,
        chunk: usize,
    ) -> String {
        use sha2::{Digest, Sha256};
        let input = match scene_id {
            Some(scene) => format!("{chapter_id}\0{scene}\0{chunk}\0{text_fingerprint}"),
            None => format!("{chapter_id}\0chapter\0{chunk}\0{text_fingerprint}"),
        };
        format!("unit-{:x}", Sha256::digest(input.as_bytes()))
    }

    pub fn scene_scope(&self) -> NarrativeScope {
        match &self.scene_id {
            Some(scene_id) => NarrativeScope::Scene {
                chapter_id: self.chapter_id.clone(),
                scene_id: scene_id.clone(),
            },
            None => NarrativeScope::Chapter {
                chapter_id: self.chapter_id.clone(),
                chapter_index: self.chapter_index,
            },
        }
    }

    /// Ordinal tokens that were actually supplied to the model.
    pub fn supplied_ordinals(&self) -> Vec<String> {
        (0..self.paragraph_ids.len())
            .map(Self::ordinal_token)
            .collect()
    }

    /// Real paragraph identifier behind an ordinal token, if supplied.
    pub fn resolve_ordinal(&self, ordinal: &str) -> Option<&str> {
        ordinal
            .strip_prefix("para-")
            .and_then(|index| index.parse::<usize>().ok())
            .and_then(|one_based| one_based.checked_sub(1))
            .and_then(|zero_based| self.paragraph_ids.get(zero_based).map(String::as_str))
    }
}

// ---------------------------------------------------------------------------
// Provider prompt (built once, versioned centrally)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderPrompt {
    pub system: String,
    pub user: String,
    pub prompt_version: String,
}

// ---------------------------------------------------------------------------
// Canon context: what is approved canon vs manuscript evidence
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CanonContext {
    pub character_names: Vec<String>,
    pub character_aliases: Vec<String>,
    pub glossary_terms: Vec<String>,
    pub relationship_pairs: Vec<(String, String)>,
}

// ---------------------------------------------------------------------------
// Provider contract
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisProviderRequest {
    pub unit: AnalysisUnit,
    pub prompt: ProviderPrompt,
    pub categories: Vec<FindingCategory>,
    pub canon_context: CanonContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisProviderResponse {
    pub findings: Vec<FindingResponse>,
    pub usage: Option<UsageMetadata>,
}

/// Raw structured finding as returned by a provider. Never enters domain
/// state before `crate::validation::validate_finding`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindingResponse {
    pub category: String,
    pub subject: String,
    pub claim: String,
    pub confidence: f32,
    pub evidence_ordinals: Vec<String>,
    pub uncertainty: Option<String>,
    pub alternative_interpretations: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UsageMetadata {
    pub input_tokens: usize,
    pub output_tokens: usize,
    pub total_tokens: usize,
    pub request_count: usize,
}

impl UsageMetadata {
    pub fn merge(&mut self, other: &UsageMetadata) {
        self.input_tokens += other.input_tokens;
        self.output_tokens += other.output_tokens;
        self.total_tokens += other.total_tokens;
        self.request_count += other.request_count;
    }
}

// ---------------------------------------------------------------------------
// Validated finding
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdvancedLiteraryFinding {
    pub finding_id: String,
    /// The bounded analysis unit this finding came from.
    pub analysis_unit_id: String,
    pub category: FindingCategory,
    pub subject: String,
    pub claim: String,
    pub confidence: DerivedConfidence,
    pub evidence: Vec<ValidatedEvidenceRef>,
    pub scope: NarrativeScope,
    pub provider_metadata: ProviderMetadata,
    pub uncertainty: Option<String>,
    pub alternative_interpretations: Vec<String>,
    pub is_review_eligible: bool,
}

impl AdvancedLiteraryFinding {
    /// Stable identity for a finding: analysis unit + category + normalized
    /// subject/claim + scope. Model wording is normalized before hashing so
    /// harmless paraphrase does not duplicate review items.
    pub fn stable_id(
        unit_id: &str,
        category: FindingCategory,
        subject: &str,
        claim: &str,
        scope: &NarrativeScope,
    ) -> String {
        use sha2::{Digest, Sha256};
        let identity = format!(
            "{}\0{}\0{}\0{}\0{}",
            unit_id,
            category,
            normalize_for_identity(subject),
            normalize_for_identity(claim),
            scope.key()
        );
        format!("finding-{:x}", Sha256::digest(identity.as_bytes()))
    }
}

// ---------------------------------------------------------------------------
// Result
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FailedUnit {
    pub unit_id: String,
    pub error: String,
    pub retryable: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdvancedAnalysisResult {
    pub schema_version: u32,
    pub analysis_id: String,
    pub manuscript_id: String,
    pub manuscript_title: String,
    pub provider_metadata: ProviderMetadata,
    pub findings: Vec<AdvancedLiteraryFinding>,
    pub failed_units: Vec<FailedUnit>,
    pub total_units: usize,
    pub succeeded_units: usize,
    pub cached_units: usize,
    pub warnings: Vec<String>,
    pub usage: UsageMetadata,
    pub analyzed_at: DateTime<Utc>,
}

impl AdvancedAnalysisResult {
    pub fn analysis_id(manuscript_id: &str, provider: &str, model: &str) -> String {
        use sha2::{Digest, Sha256};
        let identity = format!("{manuscript_id}\0{provider}\0{model}");
        format!("analysis-{:x}", Sha256::digest(identity.as_bytes()))
    }
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum AdvancedAnalysisError {
    #[error("unsupported analysis schema version {0}")]
    UnsupportedSchema(u32),
    #[error("finding validation failed: {0}")]
    FindingValidation(String),
    #[error("provider output parse error: {0}")]
    ProviderOutput(String),
    #[error("evidence reference not found in supplied unit: {0}")]
    InvalidEvidenceRef(String),
    #[error("invalid confidence value: {0}")]
    InvalidConfidence(String),
    #[error("manuscript has no analyzable chapters")]
    EmptyManuscript,
    #[error("analysis unit has no text content")]
    EmptyUnit,
    #[error("prompt construction failed: {0}")]
    PromptConstruction(String),
    #[error("analysis configuration is invalid: {0}")]
    Configuration(String),
    #[error("cache I/O failed: {0}")]
    CacheIo(String),
    #[error("review reconciliation failed: {0}")]
    Review(String),
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Normalize a string for stable identity generation: lowercase, keep only
/// alphanumeric and whitespace, collapse whitespace, trim.
pub fn normalize_for_identity(value: &str) -> String {
    value
        .to_lowercase()
        .chars()
        .filter(|character| character.is_alphanumeric() || character.is_whitespace())
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Canonicalize an evidence ordinal token. Ordinals are the only evidence
/// identifiers the model is allowed to cite.
pub fn normalize_ordinal(value: &str) -> Option<String> {
    let token = value.trim().trim_start_matches('[').trim_end_matches(']');
    let number = token
        .strip_prefix("para-")
        .or_else(|| token.strip_prefix("paragraph-"))?;
    let parsed: usize = number.parse().ok()?;
    if parsed == 0 {
        return None;
    }
    Some(format!("para-{parsed}"))
}

fn normalized_pair(a: &str, b: &str) -> (String, String) {
    let a = normalize_for_identity(a);
    let b = normalize_for_identity(b);
    if a <= b {
        (a, b)
    } else {
        (b, a)
    }
}
