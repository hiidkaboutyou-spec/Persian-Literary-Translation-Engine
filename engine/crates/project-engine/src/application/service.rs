//! `ApplicationService` — one coherent facade over the application layer.
//!
//! CLI and the future desktop app call this service; they never orchestrate
//! engine crates themselves. Operations are synchronous and file-based; the
//! service holds no mutable shared state, so concurrent callers (tests, a
//! future Tauri backend) are safe as long as they respect project locking.

use super::analysis::{analyze_book, run_advanced_analysis, AdvancedAnalysisSettings};
use super::error::{ApplicationError, ApplicationErrorPayload};
use super::models::{
    ApplicationCapabilities, ArtifactState, HistoryEvent, NextAction, ProjectEvent,
    ProjectEventSink, ProjectSnapshot, ReviewItemSummary, TranslatedChapter, TranslationProgress,
    TranslationRevision, VecEventSink,
};
use super::project::{
    create_project, load_history, load_manifest, open_project, save_manifest, HistorySink,
    ProjectLayout, ProjectLock,
};
use super::review as review_ops;
use super::snapshot as snapshot_ops;
use super::translation as translation_ops;
use chrono::Utc;
use human_review_workflow::{
    CanonPromotionPlan, ConflictResolution, DecisionAction, ReviewedValue,
};
use std::path::Path;

/// A single opened project: resolved layout. The manifest is re-read from disk
/// on each operation so concurrent processes never see stale in-memory state.
#[derive(Debug, Clone)]
pub struct Project {
    pub layout: ProjectLayout,
}

/// Sink that writes every event to project history and forwards to the
/// caller's sink (CLI prints, tests collect).
struct HistoryForwardingSink<'a> {
    history: HistorySink<'a>,
    inner: &'a mut dyn ProjectEventSink,
}

impl ProjectEventSink for HistoryForwardingSink<'_> {
    fn emit(&mut self, event: ProjectEvent) {
        self.history.emit(event.clone());
        self.inner.emit(event);
    }
}

pub struct ApplicationService;

impl ApplicationService {
    pub fn capabilities() -> ApplicationCapabilities {
        ApplicationCapabilities::current()
    }

    pub fn list_supported_providers() -> (Vec<String>, Vec<String>) {
        (
            vec!["auto".to_string(), "echo".to_string(), "openai".to_string()],
            vec!["mock".to_string(), "openai".to_string()],
        )
    }

    /// Validate a provider configuration without exposing secrets. `openai`
    /// requires `OPENAI_API_KEY` in the environment; `echo`/`mock`/`auto`
    /// never require credentials.
    pub fn test_provider_configuration(
        translation_provider: &str,
        analysis_provider: &str,
    ) -> Result<(), ApplicationError> {
        let has_openai_key = std::env::var_os("OPENAI_API_KEY").is_some();
        match translation_provider {
            "echo" | "auto" => {}
            "openai" if has_openai_key => {}
            "openai" => {
                return Err(ApplicationError::ProviderNotConfigured(
                    "OPENAI_API_KEY is not configured".to_string(),
                ))
            }
            other => {
                return Err(ApplicationError::ProviderNotConfigured(format!(
                    "unsupported translation provider '{other}'"
                )))
            }
        }
        match analysis_provider {
            "mock" => {}
            "openai" if has_openai_key => {}
            "openai" => {
                return Err(ApplicationError::ProviderNotConfigured(
                    "OPENAI_API_KEY is not configured".to_string(),
                ))
            }
            other => {
                return Err(ApplicationError::ProviderNotConfigured(format!(
                    "unsupported analysis provider '{other}'"
                )))
            }
        }
        Ok(())
    }

    // ------------------------------------------------------------------
    // Project lifecycle
    // ------------------------------------------------------------------

    pub fn create_project(
        root: impl Into<std::path::PathBuf>,
        name: impl Into<String>,
        source: Option<&Path>,
        sink: &mut dyn ProjectEventSink,
    ) -> Result<Project, ApplicationError> {
        let layout = create_project(root, name, source, sink)?;
        Ok(Project { layout })
    }

    pub fn open_project(root: impl Into<std::path::PathBuf>) -> Result<Project, ApplicationError> {
        let (layout, _manifest) = open_project(root)?;
        Ok(Project { layout })
    }

    pub fn import_book(
        &self,
        project: &Project,
        source_path: &Path,
        sink: &mut dyn ProjectEventSink,
    ) -> Result<super::models::SourceRecord, ApplicationError> {
        let _lock = ProjectLock::acquire(&project.layout, "import")?;
        let manifest = load_manifest(&project.layout)?;
        let project_id = manifest.project_id.clone();
        let mut sink = HistoryForwardingSink {
            history: HistorySink::new(&project.layout, &project_id),
            inner: sink,
        };
        let manuscript = document_engine::ingest_file(source_path).map_err(|error| {
            ApplicationError::ImportFailed(
                source_path.to_path_buf(),
                format!("ingestion failed: {error}"),
            )
        })?;
        let record = super::project::import_source_record(
            &project.layout,
            source_path,
            &manuscript,
            Utc::now(),
        )?;
        let mut manifest = load_manifest(&project.layout)?;
        manifest.source = Some(record.clone());
        manifest.updated_at = Utc::now();
        save_manifest(&project.layout, &manifest)?;
        // The history-forwarding sink records this as `source_imported`; no
        // separate append here (would double-record).
        sink.emit(ProjectEvent::ImportCompleted {
            project_id: project_id.clone(),
            chapters: manuscript.chapters.len(),
        });
        Ok(record)
    }

    pub fn snapshot(&self, project: &Project) -> Result<ProjectSnapshot, ApplicationError> {
        let manifest = load_manifest(&project.layout)?;
        snapshot_ops::snapshot(&project.layout, &manifest.project_id)
    }

    // ------------------------------------------------------------------
    // Analysis
    // ------------------------------------------------------------------

    pub fn analyze_book(
        &self,
        project: &Project,
        sink: &mut dyn ProjectEventSink,
    ) -> Result<super::models::AnalysisRecord, ApplicationError> {
        let _lock = ProjectLock::acquire(&project.layout, "analyze")?;
        let manifest = load_manifest(&project.layout)?;
        let project_id = manifest.project_id.clone();
        let mut sink = HistoryForwardingSink {
            history: HistorySink::new(&project.layout, &project_id),
            inner: sink,
        };
        let (_, record) = analyze_book(&project.layout, &mut sink)?;
        Ok(record)
    }

    pub fn run_advanced_analysis(
        &self,
        project: &Project,
        settings: &AdvancedAnalysisSettings,
        sink: &mut dyn ProjectEventSink,
    ) -> Result<super::models::AdvancedRecord, ApplicationError> {
        let _lock = ProjectLock::acquire(&project.layout, "analyze-advanced")?;
        let manifest = load_manifest(&project.layout)?;
        let project_id = manifest.project_id.clone();
        let mut sink = HistoryForwardingSink {
            history: HistorySink::new(&project.layout, &project_id),
            inner: sink,
        };
        run_advanced_analysis(&project.layout, settings, &mut sink)?;
        // Reflect the persisted advanced record.
        let manifest = load_manifest(&project.layout)?;
        manifest.advanced.ok_or_else(|| {
            ApplicationError::AdvancedAnalysisFailed(
                "advanced analysis completed but no record was persisted".to_string(),
            )
        })
    }

    // ------------------------------------------------------------------
    // Review
    // ------------------------------------------------------------------

    pub fn list_review_items(
        &self,
        project: &Project,
        kind_filter: Option<&str>,
        status_filter: Option<&str>,
    ) -> Result<Vec<ReviewItemSummary>, ApplicationError> {
        review_ops::list_review_items(&project.layout, kind_filter, status_filter)
    }

    pub fn get_review_item(
        &self,
        project: &Project,
        id: &str,
    ) -> Result<ReviewItemSummary, ApplicationError> {
        review_ops::get_review_item(&project.layout, id)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn decide_review_item(
        &self,
        project: &Project,
        id: &str,
        action: DecisionAction,
        replacement: Option<ReviewedValue>,
        reviewer: &str,
        reason: &str,
        sink: &mut dyn ProjectEventSink,
    ) -> Result<ReviewItemSummary, ApplicationError> {
        let _lock = ProjectLock::acquire(&project.layout, "review")?;
        let manifest = load_manifest(&project.layout)?;
        let project_id = manifest.project_id.clone();
        let mut sink = HistoryForwardingSink {
            history: HistorySink::new(&project.layout, &project_id),
            inner: sink,
        };
        let summary = review_ops::decide_review_item(
            &project.layout,
            id,
            action,
            replacement,
            reviewer,
            reason,
            Utc::now(),
        )?;
        sink.emit(ProjectEvent::ReviewStateChanged {
            project_id: project_id.clone(),
            item_id: id.to_string(),
        });
        Ok(summary)
    }

    pub fn preview_promotion(
        &self,
        project: &Project,
        selected_ids: &[String],
        resolutions: &[ConflictResolution],
    ) -> Result<CanonPromotionPlan, ApplicationError> {
        review_ops::preview_promotion(&project.layout, selected_ids, resolutions)
    }

    pub fn apply_promotion(
        &self,
        project: &Project,
        plan: &CanonPromotionPlan,
        reviewer: &str,
        reason: &str,
        sink: &mut dyn ProjectEventSink,
    ) -> Result<super::review::store::PromotionApplyResult, ApplicationError> {
        let _lock = ProjectLock::acquire(&project.layout, "promote")?;
        let manifest = load_manifest(&project.layout)?;
        let project_id = manifest.project_id.clone();
        let mut sink = HistoryForwardingSink {
            history: HistorySink::new(&project.layout, &project_id),
            inner: sink,
        };
        let result =
            review_ops::apply_promotion(&project.layout, plan, reviewer, reason, Utc::now())?;
        sink.emit(ProjectEvent::CanonPromoted {
            project_id: project_id.clone(),
            item_count: result.applied_item_ids.len(),
        });
        // Refresh the canon fingerprint so translation-staleness tracking is
        // accurate after promotion.
        let mut manifest = load_manifest(&project.layout)?;
        manifest.canon_fingerprint = snapshot_ops::canon_fingerprint(&project.layout)?;
        save_manifest(&project.layout, &manifest)?;
        Ok(result)
    }

    // ------------------------------------------------------------------
    // Character Bible / Glossary
    // ------------------------------------------------------------------

    pub fn list_characters(
        &self,
        project: &Project,
    ) -> Result<Vec<character_engine::CharacterProfile>, ApplicationError> {
        review_ops::list_characters(&project.layout)
    }

    pub fn get_character(
        &self,
        project: &Project,
        name: &str,
    ) -> Result<Option<character_engine::CharacterProfile>, ApplicationError> {
        review_ops::get_character(&project.layout, name)
    }

    pub fn upsert_character(
        &self,
        project: &Project,
        profile: character_engine::CharacterProfile,
        replace_existing: bool,
    ) -> Result<(), ApplicationError> {
        let _lock = ProjectLock::acquire(&project.layout, "canon-edit")?;
        review_ops::upsert_character(&project.layout, profile, replace_existing)?;
        let mut manifest = load_manifest(&project.layout)?;
        manifest.canon_fingerprint = snapshot_ops::canon_fingerprint(&project.layout)?;
        save_manifest(&project.layout, &manifest)
    }

    pub fn list_glossary_entries(
        &self,
        project: &Project,
    ) -> Result<Vec<memory_engine::glossary::GlossaryEntry>, ApplicationError> {
        review_ops::list_glossary_entries(&project.layout)
    }

    pub fn get_glossary_entry(
        &self,
        project: &Project,
        source_term: &str,
    ) -> Result<Option<memory_engine::glossary::GlossaryEntry>, ApplicationError> {
        review_ops::get_glossary_entry(&project.layout, source_term)
    }

    pub fn upsert_glossary_entry(
        &self,
        project: &Project,
        entry: memory_engine::glossary::GlossaryEntry,
        replace_existing: bool,
    ) -> Result<(), ApplicationError> {
        let _lock = ProjectLock::acquire(&project.layout, "canon-edit")?;
        review_ops::upsert_glossary_entry(&project.layout, entry, replace_existing)?;
        let mut manifest = load_manifest(&project.layout)?;
        manifest.canon_fingerprint = snapshot_ops::canon_fingerprint(&project.layout)?;
        save_manifest(&project.layout, &manifest)
    }

    // ------------------------------------------------------------------
    // Translation lifecycle
    // ------------------------------------------------------------------

    pub fn start_translation(
        &self,
        project: &Project,
        config: &translation_ops::TranslationConfig,
        sink: &mut dyn ProjectEventSink,
    ) -> Result<TranslationProgress, ApplicationError> {
        let _lock = ProjectLock::acquire(&project.layout, "translate")?;
        let manifest = load_manifest(&project.layout)?;
        let project_id = manifest.project_id.clone();
        let mut sink = HistoryForwardingSink {
            history: HistorySink::new(&project.layout, &project_id),
            inner: sink,
        };
        translation_ops::clear_pause(&project.layout)?;
        translation_ops::run_translation(&project.layout, config, &mut sink, false)
    }

    pub fn resume_translation(
        &self,
        project: &Project,
        config: &translation_ops::TranslationConfig,
        sink: &mut dyn ProjectEventSink,
    ) -> Result<TranslationProgress, ApplicationError> {
        let _lock = ProjectLock::acquire(&project.layout, "translate")?;
        let manifest = load_manifest(&project.layout)?;
        let project_id = manifest.project_id.clone();
        let mut sink = HistoryForwardingSink {
            history: HistorySink::new(&project.layout, &project_id),
            inner: sink,
        };
        translation_ops::clear_pause(&project.layout)?;
        translation_ops::run_translation(&project.layout, config, &mut sink, true)
    }

    pub fn request_pause(&self, project: &Project) -> Result<(), ApplicationError> {
        translation_ops::request_pause(&project.layout)
    }

    pub fn get_progress(&self, project: &Project) -> Result<TranslationProgress, ApplicationError> {
        translation_ops::get_progress(&project.layout)
    }

    pub fn get_translated_chapter(
        &self,
        project: &Project,
        chapter_index: usize,
    ) -> Result<TranslatedChapter, ApplicationError> {
        translation_ops::get_translated_chapter(&project.layout, chapter_index)
    }

    pub fn get_translated_text(
        &self,
        project: &Project,
        chapter_index: usize,
    ) -> Result<String, ApplicationError> {
        translation_ops::get_translated_text(&project.layout, chapter_index)
    }

    pub fn apply_manual_translation_edit(
        &self,
        project: &Project,
        chapter_index: usize,
        paragraph_id: &str,
        new_text: &str,
        reviewer: Option<&str>,
        sink: &mut dyn ProjectEventSink,
    ) -> Result<TranslationRevision, ApplicationError> {
        let _lock = ProjectLock::acquire(&project.layout, "manual-edit")?;
        let manifest = load_manifest(&project.layout)?;
        let project_id = manifest.project_id.clone();
        let mut sink = HistoryForwardingSink {
            history: HistorySink::new(&project.layout, &project_id),
            inner: sink,
        };
        translation_ops::apply_manual_translation_edit(
            &project.layout,
            chapter_index,
            paragraph_id,
            new_text,
            reviewer,
            &mut sink,
        )
    }

    pub fn export_project(
        &self,
        project: &Project,
        sink: &mut dyn ProjectEventSink,
    ) -> Result<super::models::ExportRecord, ApplicationError> {
        let _lock = ProjectLock::acquire(&project.layout, "export")?;
        let manifest = load_manifest(&project.layout)?;
        let project_id = manifest.project_id.clone();
        let mut sink = HistoryForwardingSink {
            history: HistorySink::new(&project.layout, &project_id),
            inner: sink,
        };
        translation_ops::export_translation(&project.layout, &mut sink)
    }

    // ------------------------------------------------------------------
    // History
    // ------------------------------------------------------------------

    pub fn history(&self, project: &Project) -> Result<Vec<HistoryEvent>, ApplicationError> {
        load_history(&project.layout)
    }

    pub fn error_payload(error: &ApplicationError) -> ApplicationErrorPayload {
        error.to_payload()
    }

    /// Verify the imported source is unchanged (full re-hash). Snapshot uses a
    /// cheap size check; this is the authoritative check.
    pub fn verify_source(&self, project: &Project) -> Result<ArtifactState, ApplicationError> {
        let manifest = load_manifest(&project.layout)?;
        let Some(record) = manifest.source.as_ref() else {
            return Ok(ArtifactState::Missing);
        };
        let source_path = project
            .layout
            .source_dir
            .join(record.stored_relative_path.trim_start_matches("source/"));
        let bytes = std::fs::read(&source_path).map_err(|error| {
            ApplicationError::PersistenceFailure(format!(
                "failed to read source {}: {error}",
                source_path.display()
            ))
        })?;
        let fingerprint = super::models::content_fingerprint(&bytes);
        if fingerprint == record.fingerprint {
            Ok(ArtifactState::Fresh)
        } else {
            Ok(ArtifactState::Stale)
        }
    }
}

/// Helpers for tests: default no-op sinks.
pub fn silent_sink() -> VecEventSink {
    VecEventSink::new()
}

/// Recommended next action helper (pure function, no mutation).
pub fn recommended_next_action(snapshot: &ProjectSnapshot) -> NextAction {
    snapshot.next_action
}
