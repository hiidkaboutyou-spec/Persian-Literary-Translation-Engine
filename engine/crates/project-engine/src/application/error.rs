//! Application-facing error contract for Phase 16.
//!
//! The future desktop UI must receive typed errors, not internal engine error
//! strings. Every variant carries an optional machine-readable recovery hint;
//! underlying context is preserved through the `source` chain for logs while
//! the serialized payload stays UI-friendly.

use serde::Serialize;
use std::path::PathBuf;
use thiserror::Error;

/// Machine-readable recovery hint attached to an application error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryHint {
    ConfigureProvider,
    RunAnalysisFirst,
    ResolveReviewConflict,
    ResumeExistingRun,
    ReimportSource,
    UnlockProject,
    None,
}

/// Serializable UI-facing error payload. Never includes internal file paths
/// or engine error text beyond the message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ApplicationErrorPayload {
    pub code: String,
    pub message: String,
    pub recovery_hint: RecoveryHint,
}

#[derive(Debug, Error)]
pub enum ApplicationError {
    #[error("project not found at {0}")]
    ProjectNotFound(PathBuf),
    #[error("invalid project: {0}")]
    InvalidProject(String),
    #[error("unsupported project schema version {0}; expected {1}")]
    UnsupportedProjectVersion(u32, u32),
    #[error("unsupported input format for {0}")]
    UnsupportedInputFormat(PathBuf),
    #[error("import failed for {0}: {1}")]
    ImportFailed(PathBuf, String),
    #[error("analysis failed: {0}")]
    AnalysisFailed(String),
    #[error("advanced analysis failed: {0}")]
    AdvancedAnalysisFailed(String),
    #[error("provider is not configured: {0}")]
    ProviderNotConfigured(String),
    #[error("provider authentication failed: {0}")]
    ProviderAuthenticationFailed(String),
    #[error("review conflict: {0}")]
    ReviewConflict(String),
    #[error("promotion blocked: {0}")]
    PromotionBlocked(String),
    #[error("translation is already running")]
    TranslationAlreadyRunning,
    #[error("no checkpoint available to resume from")]
    NoCheckpoint,
    #[error("resume incompatible: {0}")]
    ResumeIncompatible(String),
    #[error("export unavailable: {0}")]
    ExportUnavailable(String),
    #[error("project is locked by another process")]
    ProjectLocked,
    #[error("persistence failure: {0}")]
    PersistenceFailure(String),
    #[error("{0}")]
    Internal(String),
}

impl ApplicationError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::ProjectNotFound(_) => "project_not_found",
            Self::InvalidProject(_) => "invalid_project",
            Self::UnsupportedProjectVersion(_, _) => "unsupported_project_version",
            Self::UnsupportedInputFormat(_) => "unsupported_input_format",
            Self::ImportFailed(_, _) => "import_failed",
            Self::AnalysisFailed(_) => "analysis_failed",
            Self::AdvancedAnalysisFailed(_) => "advanced_analysis_failed",
            Self::ProviderNotConfigured(_) => "provider_not_configured",
            Self::ProviderAuthenticationFailed(_) => "provider_authentication_failed",
            Self::ReviewConflict(_) => "review_conflict",
            Self::PromotionBlocked(_) => "promotion_blocked",
            Self::TranslationAlreadyRunning => "translation_already_running",
            Self::NoCheckpoint => "no_checkpoint",
            Self::ResumeIncompatible(_) => "resume_incompatible",
            Self::ExportUnavailable(_) => "export_unavailable",
            Self::ProjectLocked => "project_locked",
            Self::PersistenceFailure(_) => "persistence_failure",
            Self::Internal(_) => "internal_error",
        }
    }

    pub fn recovery_hint(&self) -> RecoveryHint {
        match self {
            Self::ProviderNotConfigured(_) | Self::ProviderAuthenticationFailed(_) => {
                RecoveryHint::ConfigureProvider
            }
            Self::InvalidProject(_) | Self::ImportFailed(_, _) => RecoveryHint::ReimportSource,
            Self::UnsupportedProjectVersion(_, _) => RecoveryHint::ReimportSource,
            Self::AnalysisFailed(_) | Self::AdvancedAnalysisFailed(_) => {
                RecoveryHint::RunAnalysisFirst
            }
            Self::ReviewConflict(_) | Self::PromotionBlocked(_) => {
                RecoveryHint::ResolveReviewConflict
            }
            Self::NoCheckpoint | Self::ResumeIncompatible(_) => RecoveryHint::ResumeExistingRun,
            Self::ProjectLocked => RecoveryHint::UnlockProject,
            Self::ProjectNotFound(_) => RecoveryHint::None,
            Self::UnsupportedInputFormat(_) => RecoveryHint::ReimportSource,
            Self::TranslationAlreadyRunning => RecoveryHint::ResumeExistingRun,
            Self::ExportUnavailable(_) => RecoveryHint::None,
            Self::PersistenceFailure(_) => RecoveryHint::None,
            Self::Internal(_) => RecoveryHint::None,
        }
    }

    pub fn to_payload(&self) -> ApplicationErrorPayload {
        ApplicationErrorPayload {
            code: self.code().to_string(),
            message: self.to_string(),
            recovery_hint: self.recovery_hint(),
        }
    }
}

impl From<std::io::Error> for ApplicationError {
    fn from(error: std::io::Error) -> Self {
        Self::PersistenceFailure(error.to_string())
    }
}
