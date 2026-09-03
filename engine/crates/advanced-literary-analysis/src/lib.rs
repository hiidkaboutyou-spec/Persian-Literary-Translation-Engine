//! Advanced model-assisted literary analysis (Phase 15).
//!
//! An optional, explicitly requested layer on top of the deterministic Phase 13
//! analysis. The planner splits a manuscript into bounded analysis units, a
//! provider-neutral `LiteraryAnalysisProvider` returns structured findings,
//! deterministic validation rejects hallucinated evidence and malformed output,
//! and validated findings flow into the Phase 14 human review ledger as
//! review-only `Literary` proposals — never directly into canon.
//!
//! Invariant preserved end-to-end:
//!
//! ```text
//! MODEL INFERENCE != HUMAN APPROVAL != CANON
//! ```
//!
//! Deterministic analysis remains fully functional with no provider configured;
//! advanced analysis is never triggered implicitly.

pub mod cache;
pub mod models;
pub mod orchestrator;
pub mod planner;
pub mod prompts;
pub mod provider;
pub mod validation;

pub use cache::{cache_key, canon_fingerprint, AnalysisCache, CachedUnitResult};
pub use models::{
    normalize_for_identity, normalize_ordinal, AdvancedAnalysisError, AdvancedAnalysisResult,
    AdvancedLiteraryFinding, AnalysisProviderRequest, AnalysisProviderResponse, AnalysisUnit,
    AnalysisUnitType, CanonContext, ConfidenceLevel, DerivedConfidence, FailedUnit,
    FindingCategory, FindingResponse, NarrativeScope, ProviderMetadata, ProviderPrompt,
    UsageMetadata, ValidatedEvidenceRef, ADVANCED_ANALYSIS_SCHEMA_VERSION, PROMPT_VERSION,
};
pub use orchestrator::{run_advanced_analysis, AdvancedAnalysisConfig};
pub use planner::{AnalysisPlanner, PlannerConfig};
pub use provider::{
    AnalysisProviderError, LiteraryAnalysisProvider, MockAnalysisProvider,
    MockAnalysisProviderConfig, MockFindingTemplate, OpenAIAnalysisProvider,
};
pub use validation::{MAX_ALTERNATIVES, MAX_CLAIM_CHARS, MAX_SUBJECT_CHARS, MAX_UNCERTAINTY_CHARS};
