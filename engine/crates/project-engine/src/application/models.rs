//! Application-layer data models for Phase 16.
//!
//! These types form the stable boundary a future desktop UI consumes. They
//! reference project artifacts by ID/path/count — never by re-reading the
//! manuscript — so snapshots stay cheap and remain readable without domain
//! engine knowledge.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;

pub const PROJECT_SCHEMA_VERSION: u32 = 1;

// ---------------------------------------------------------------------------
// project.json
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectFile {
    pub schema_version: u32,
    pub project_id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub target_language: String,
    pub source: Option<SourceRecord>,
    pub analysis: Option<AnalysisRecord>,
    pub advanced: Option<AdvancedRecord>,
    /// Fingerprint of the canonical Character Bible + Glossary at the last
    /// known-good point. Compared against current canon to surface staleness.
    pub canon_fingerprint: String,
    pub translation: Option<TranslationRecord>,
    pub export: Option<ExportRecord>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceRecord {
    pub original_filename: String,
    /// Relative path under `source/` inside the project root.
    pub stored_relative_path: String,
    pub format: String,
    pub fingerprint: String,
    pub size_bytes: u64,
    pub book_id: String,
    pub title: String,
    pub chapter_count: usize,
    pub scene_count: usize,
    pub paragraph_count: usize,
    pub imported_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnalysisRecord {
    /// Source fingerprint the deterministic analysis was computed from.
    pub source_fingerprint: String,
    pub schema_version: u32,
    pub analyzer: String,
    pub analyzed_at: DateTime<Utc>,
    pub character_seeds: usize,
    pub relationship_seeds: usize,
    pub terminology_seeds: usize,
    pub review_items_added: usize,
    pub review_items_unchanged: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdvancedRecord {
    pub source_fingerprint: String,
    pub provider: String,
    pub model: String,
    pub prompt_version: String,
    pub analyzed_at: DateTime<Utc>,
    pub total_units: usize,
    pub succeeded_units: usize,
    pub cached_units: usize,
    pub failed_units: usize,
    pub findings: usize,
    pub review_items_added: usize,
    pub review_items_unchanged: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TranslationRecord {
    pub run_id: String,
    pub state: TranslationState,
    pub provider: String,
    pub model: Option<String>,
    pub target_language: String,
    pub total_chapters: usize,
    pub completed_chapters: usize,
    /// Monotonic fraction (0.0..=1.0) derived from completed chapters.
    pub percent: f32,
    pub started_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// Canon fingerprint at translation start, for context-staleness checks.
    pub canon_fingerprint_at_start: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExportRecord {
    pub format: String,
    /// Relative path under the project root.
    pub relative_path: String,
    pub chapters: usize,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TranslationState {
    NotStarted,
    Running,
    Paused,
    Completed,
    Failed,
}

// ---------------------------------------------------------------------------
// Artifact freshness
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactState {
    Missing,
    Fresh,
    Stale,
    InProgress,
    Failed,
}

// ---------------------------------------------------------------------------
// Project status & next action (derived deterministically)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectStatus {
    Empty,
    Imported,
    Analyzed,
    NeedsReview,
    ReadyToTranslate,
    Translating,
    Paused,
    TranslationComplete,
    ReadyToExport,
    Exported,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NextAction {
    ImportBook,
    RunAnalysis,
    ReviewIntelligence,
    ConfigureProvider,
    StartTranslation,
    ResumeTranslation,
    ReviewTranslation,
    ReimportSource,
    Export,
    None,
}

// ---------------------------------------------------------------------------
// Snapshot (UI home screen in one structure)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectSnapshot {
    pub schema_version: u32,
    pub project_id: String,
    pub name: String,
    pub root: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub target_language: String,
    pub source: Option<SourceSummary>,
    pub analysis: Option<ArtifactSummary>,
    pub advanced: Option<ArtifactSummary>,
    pub review: ReviewSummary,
    pub canon: CanonSummary,
    pub translation: Option<TranslationSummary>,
    pub export: Option<ExportSummary>,
    pub status: ProjectStatus,
    pub next_action: NextAction,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceSummary {
    pub original_filename: String,
    pub format: String,
    pub fingerprint: String,
    pub title: String,
    pub chapters: usize,
    pub scenes: usize,
    pub paragraphs: usize,
    pub state: ArtifactState,
    pub imported_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArtifactSummary {
    pub state: ArtifactState,
    pub detail: String,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ReviewSummary {
    pub total: usize,
    pub pending: usize,
    pub approved: usize,
    pub edited: usize,
    pub rejected: usize,
    pub deferred: usize,
    pub applied: usize,
    pub conflicted: usize,
    pub obsolete: usize,
    pub literary: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CanonSummary {
    pub characters: usize,
    pub aliases: usize,
    pub relationships: usize,
    pub glossary_entries: usize,
    pub fingerprint: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TranslationSummary {
    pub run_id: String,
    pub state: TranslationState,
    pub provider: String,
    pub model: Option<String>,
    pub target_language: String,
    pub completed_chapters: usize,
    pub total_chapters: usize,
    pub percent: f32,
    pub context_stale: bool,
    pub started_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExportSummary {
    pub format: String,
    pub relative_path: String,
    pub chapters: usize,
    pub created_at: DateTime<Utc>,
    pub exists: bool,
}

// ---------------------------------------------------------------------------
// Review item summary (UI-facing, no domain ledger internals)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReviewItemSummary {
    pub id: String,
    pub kind: String,
    pub status: String,
    pub availability: String,
    pub revision: u32,
    pub subject: String,
    pub confidence: String,
    pub evidence_count: usize,
}

// ---------------------------------------------------------------------------
// Translation progress (persisted and streamed)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TranslationProgress {
    pub schema_version: u32,
    pub project_id: String,
    pub source_fingerprint: String,
    pub run_id: String,
    pub state: TranslationState,
    pub provider: String,
    pub model: Option<String>,
    pub target_language: String,
    pub total_chapters: usize,
    pub completed_chapters: usize,
    pub current_chapter: Option<usize>,
    pub completed_paragraphs: usize,
    pub total_paragraphs: usize,
    pub percent: f32,
    pub last_checkpoint: Option<String>,
    pub warnings: Vec<String>,
    pub started_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// Translated chapter artifact (for editor + manual edits)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TranslatedChapter {
    pub schema_version: u32,
    pub chapter_index: usize,
    pub chapter_id: String,
    pub title: String,
    pub source_fingerprint: String,
    pub context_fingerprint: String,
    pub paragraphs: Vec<TranslatedParagraph>,
    /// Set true after any manual edit so the editor/quality layer knows the
    /// stored quality evaluation is stale.
    pub quality_stale: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TranslatedParagraph {
    pub paragraph_id: String,
    pub source: String,
    pub translated: String,
    /// `provider` or `manual`.
    pub origin: String,
    pub revisions: Vec<TranslationRevision>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TranslationRevision {
    pub revision_id: String,
    pub paragraph_id: String,
    pub previous: String,
    pub new: String,
    pub origin: String,
    pub reviewer: Option<String>,
    pub timestamp: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// Capabilities
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ApplicationCapabilities {
    pub schema_version: u32,
    pub import_formats: Vec<String>,
    pub export_formats: Vec<String>,
    pub advanced_analysis_available: bool,
    pub pause_supported: bool,
    pub manual_edit_supported: bool,
    pub translation_providers: Vec<String>,
    pub analysis_providers: Vec<String>,
}

impl ApplicationCapabilities {
    pub fn current() -> Self {
        Self {
            schema_version: 1,
            import_formats: vec![
                "txt".to_string(),
                "md".to_string(),
                "docx".to_string(),
                "epub".to_string(),
                "pdf".to_string(),
            ],
            export_formats: vec!["docx".to_string()],
            advanced_analysis_available: true,
            pause_supported: true,
            manual_edit_supported: true,
            translation_providers: vec!["echo".to_string(), "openai".to_string()],
            analysis_providers: vec!["mock".to_string(), "openai".to_string()],
        }
    }
}

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum ProjectEvent {
    ImportStarted {
        project_id: String,
    },
    ImportCompleted {
        project_id: String,
        chapters: usize,
    },
    AnalysisStarted {
        project_id: String,
    },
    AnalysisCompleted {
        project_id: String,
        review_items_added: usize,
    },
    AdvancedAnalysisStarted {
        project_id: String,
    },
    AdvancedAnalysisCompleted {
        project_id: String,
        findings: usize,
        cached_units: usize,
        failed_units: usize,
    },
    ReviewStateChanged {
        project_id: String,
        item_id: String,
    },
    CanonPromoted {
        project_id: String,
        item_count: usize,
    },
    TranslationStarted {
        project_id: String,
        run_id: String,
    },
    ChapterStarted {
        project_id: String,
        chapter_index: usize,
    },
    ChapterCompleted {
        project_id: String,
        chapter_index: usize,
        quality_score: f32,
    },
    CheckpointSaved {
        project_id: String,
        chapter_index: usize,
    },
    TranslationPaused {
        project_id: String,
        run_id: String,
    },
    TranslationCompleted {
        project_id: String,
        run_id: String,
        chapters: usize,
    },
    ManualEdit {
        project_id: String,
        chapter_index: usize,
        revision_id: String,
    },
    ExportStarted {
        project_id: String,
    },
    ExportCompleted {
        project_id: String,
        format: String,
    },
    Warning {
        project_id: String,
        message: String,
    },
    Failure {
        project_id: String,
        message: String,
    },
}

/// Sink for application events. Pure Rust — no UI framework dependency.
pub trait ProjectEventSink {
    fn emit(&mut self, event: ProjectEvent);
}

/// Collects events for tests and JSON output.
#[derive(Debug, Default)]
pub struct VecEventSink {
    pub events: Vec<ProjectEvent>,
}

impl ProjectEventSink for VecEventSink {
    fn emit(&mut self, event: ProjectEvent) {
        self.events.push(event);
    }
}

impl VecEventSink {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }
}

/// History entry persisted in `history/history.json` (bounded).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HistoryEvent {
    pub history_schema_version: u32,
    pub project_id: String,
    pub event: String,
    pub detail: String,
    pub timestamp: DateTime<Utc>,
}

pub(crate) fn project_id_for(name: &str, canonical_root: &str) -> String {
    use sha2::{Digest, Sha256};
    let identity = format!("{canonical_root}\0{name}");
    format!("project-{:x}", Sha256::digest(identity.as_bytes()))
}

/// Cheap content fingerprint (FNV-1a 64-bit) matching CLI conventions.
pub fn content_fingerprint(bytes: &[u8]) -> String {
    const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;
    let hash = bytes.iter().fold(FNV_OFFSET_BASIS, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(FNV_PRIME)
    });
    format!("fnv1a64-{hash:016x}")
}

pub(crate) fn canonical_root(root: &Path) -> String {
    root.canonicalize()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|_| root.display().to_string())
}
