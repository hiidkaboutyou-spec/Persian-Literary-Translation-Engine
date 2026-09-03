//! Application orchestration layer (Phase 16).
//!
//! One stable boundary between domain engines and callers (CLI today, desktop
//! app tomorrow). Callers use `ApplicationService` and the UI-facing model
//! types; they never orchestrate engine crates or touch engine JSON files
//! directly.

pub mod analysis;
pub mod error;
pub mod models;
pub mod project;
pub mod review;
pub mod service;
pub mod snapshot;
pub mod translation;

pub use analysis::AdvancedAnalysisSettings;
pub use error::{ApplicationError, ApplicationErrorPayload, RecoveryHint};
pub use human_review_workflow::{
    CanonPromotionPlan, ConflictResolution, DecisionAction, ReviewedValue,
};
pub use models::{
    ApplicationCapabilities, ArtifactState, HistoryEvent, NextAction, ProjectEvent,
    ProjectEventSink, ProjectSnapshot, ProjectStatus, ReviewItemSummary, ReviewSummary,
    TranslatedChapter, TranslatedParagraph, TranslationProgress, TranslationRevision,
    TranslationState, VecEventSink,
};
pub use service::{silent_sink, ApplicationService, Project};
pub use translation::TranslationConfig;
