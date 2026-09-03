//! Translation lifecycle through the application boundary.
//!
//! One application call orchestrates: source ingestion, canon/context
//! loading, provider execution through `translation-core`, the deterministic
//! quality gate, per-chapter artifacts + checkpoints, durable progress, and
//! safe pause/resume. The future desktop app never assembles these steps
//! itself.

use super::analysis::{load_deterministic_artifact, load_manuscript};
use super::error::ApplicationError;
use super::models::{
    content_fingerprint, ProjectEvent, ProjectEventSink, TranslatedChapter, TranslatedParagraph,
    TranslationProgress, TranslationRevision, TranslationState,
};
use super::project::{emit_and_history, ProjectLayout};
use super::review::load_canon;
use chrono::Utc;
use document_engine::Chapter as ManuscriptChapter;
use human_review_workflow::ReviewKind;
use memory_engine::{build_memory_context, MemoryContextConfig, TranslationMemory};
use quality_engine::{evaluate_translation, TerminologyRule};
use std::fs;
use std::path::PathBuf;
use translation_core::{
    EchoProvider, OpenAIProvider, PipelineInput, TranslationPipeline, TranslationProvider,
};

pub const TRANSLATION_PROGRESS_SCHEMA_VERSION: u32 = 1;
pub const CHAPTER_ARTIFACT_SCHEMA_VERSION: u32 = 1;
const PAUSE_FLAG: &str = "pause-requested";
const CANCEL_FLAG: &str = "cancel-requested";

/// Application-facing translation configuration (bounded knobs only).
#[derive(Debug, Clone)]
pub struct TranslationConfig {
    /// `auto` (openai when OPENAI_API_KEY is set, else echo), `echo`, `openai`.
    pub provider: String,
    pub model: Option<String>,
    pub target_language: String,
    /// Optional bound: translate at most this many chapters this run (used by
    /// the CLI/UI to chunk work; resume continues from checkpoints).
    pub max_chapters: Option<usize>,
}

impl Default for TranslationConfig {
    fn default() -> Self {
        Self {
            provider: "auto".to_string(),
            model: None,
            target_language: "fa".to_string(),
            max_chapters: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Progress persistence
// ---------------------------------------------------------------------------

fn load_progress(layout: &ProjectLayout) -> Result<Option<TranslationProgress>, ApplicationError> {
    if !layout.progress_file.exists() {
        return Ok(None);
    }
    let bytes = fs::read(&layout.progress_file).map_err(|error| {
        ApplicationError::PersistenceFailure(format!(
            "failed to read progress {}: {error}",
            layout.progress_file.display()
        ))
    })?;
    serde_json::from_slice(&bytes).map(Some).map_err(|error| {
        ApplicationError::PersistenceFailure(format!("invalid progress file: {error}"))
    })
}

fn save_progress(
    layout: &ProjectLayout,
    progress: &TranslationProgress,
) -> Result<(), ApplicationError> {
    super::project::atomic_write_json(&layout.progress_file, progress)
}

fn chapter_stem(title: &str, index: usize) -> String {
    let mut stem: String = title
        .chars()
        .map(|character| {
            if character.is_alphanumeric() || character == '-' || character == '_' {
                character
            } else {
                '_'
            }
        })
        .collect();
    stem = stem.trim_matches('_').to_string();
    if stem.is_empty() {
        stem = format!("chapter-{}", index + 1);
    }
    format!("{:03}-{}", index + 1, stem)
}

// ---------------------------------------------------------------------------
// Provider resolution (translation)
// ---------------------------------------------------------------------------

fn configured_translation_provider(
    config: &TranslationConfig,
) -> Result<Box<dyn TranslationProvider>, ApplicationError> {
    let has_openai_key = std::env::var_os("OPENAI_API_KEY").is_some();
    match config.provider.as_str() {
        "echo" => Ok(Box::new(EchoProvider)),
        "openai" => {
            if !has_openai_key {
                return Err(ApplicationError::ProviderNotConfigured(
                    "OPENAI_API_KEY is not configured".to_string(),
                ));
            }
            OpenAIProvider::from_env()
                .map(|provider| Box::new(provider) as Box<dyn TranslationProvider>)
                .map_err(|error| ApplicationError::ProviderAuthenticationFailed(error.to_string()))
        }
        "auto" if has_openai_key => OpenAIProvider::from_env()
            .map(|provider| Box::new(provider) as Box<dyn TranslationProvider>)
            .map_err(|error| ApplicationError::ProviderAuthenticationFailed(error.to_string())),
        "auto" => Ok(Box::new(EchoProvider)),
        other => Err(ApplicationError::ProviderNotConfigured(format!(
            "unsupported translation provider '{other}'; expected auto, echo, or openai"
        ))),
    }
}

// ---------------------------------------------------------------------------
// Context assembly (mirrors the CLI chapter context so CLI and app behave
// identically)
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
fn chapter_context(
    document_title: &str,
    chapter_title: &str,
    source_text: &str,
    characters: &character_engine::CharacterBible,
    glossary: &memory_engine::glossary::Glossary,
    translation_memory: &TranslationMemory,
    seed_context: Option<&str>,
    reviewed_literary_lines: &[String],
) -> String {
    let mut sections = vec![format!(
        "document_title={document_title}\nchapter_title={chapter_title}"
    )];

    let character_context = characters.context_for_text(source_text);
    if !character_context.trim().is_empty() {
        sections.push(format!(
            "CHARACTER BIBLE — preserve voice and relationship continuity:\n{character_context}"
        ));
    }

    let memory_context = build_memory_context(
        source_text,
        translation_memory,
        glossary,
        &MemoryContextConfig::default(),
    );
    if !memory_context.text.trim().is_empty() {
        sections.push(memory_context.text);
    }

    if let Some(seed_context) = seed_context.filter(|value| !value.trim().is_empty()) {
        sections.push(seed_context.to_string());
    }

    if !reviewed_literary_lines.is_empty() {
        sections.push(format!(
            "REVIEWED LITERARY FINDINGS — human-approved context:\n{}",
            reviewed_literary_lines.join("\n")
        ));
    }

    sections.join("\n\n")
}

fn terminology_rules(
    glossary: &memory_engine::glossary::Glossary,
    source_text: &str,
) -> Vec<TerminologyRule> {
    glossary
        .relevant_to_text(source_text)
        .into_iter()
        .map(|entry| {
            TerminologyRule::new(
                entry.source_term.clone(),
                entry.preferred_translation.clone(),
            )
        })
        .collect()
}

/// Load approved/edited literary findings from the review ledger, scoped to
/// their evidence chapters (same behavior the CLI uses).
fn reviewed_literary_lines(layout: &ProjectLayout, chapter_id: &str) -> Vec<String> {
    if !layout.review_file.exists() {
        return Vec::new();
    }
    let Ok(ledger) = super::review::load_ledger(layout) else {
        return Vec::new();
    };
    ledger
        .items
        .iter()
        .filter(|item| item.kind == ReviewKind::Literary)
        .filter(|item| {
            matches!(
                item.status,
                human_review_workflow::ReviewStatus::Approved
                    | human_review_workflow::ReviewStatus::Edited
            )
        })
        .filter_map(|item| {
            let human_review_workflow::ReviewedValue::Literary(value) =
                item.reviewed_value.as_ref()?
            else {
                return None;
            };
            let evidence_chapters = item
                .latest_proposal
                .evidence()
                .iter()
                .map(|evidence| evidence.chapter_id.as_str())
                .collect::<Vec<_>>();
            if !evidence_chapters.contains(&chapter_id) {
                return None;
            }
            Some(format!(
                "[{}] {} ({}) — {}",
                value.finding_category, value.subject, value.scope_label, value.claim
            ))
        })
        .collect()
}

/// Return the stored translated text when the chapter's checkpoint matches the
/// current source and context fingerprints (same contract as the CLI resume).
fn resumable_chapter(
    layout: &ProjectLayout,
    stem: &str,
    expected_source: &str,
    expected_context: &str,
) -> Result<Option<String>, ApplicationError> {
    let txt_path = layout.chapters_dir.join(format!("{stem}.txt"));
    let source_fp_path = layout
        .chapters_dir
        .join(format!("{stem}.source-fingerprint"));
    let context_fp_path = layout
        .chapters_dir
        .join(format!("{stem}.context-fingerprint"));
    if !txt_path.is_file() || !source_fp_path.is_file() || !context_fp_path.is_file() {
        return Ok(None);
    }
    let stored_source = fs::read_to_string(&source_fp_path).map_err(|error| {
        ApplicationError::PersistenceFailure(format!("failed to read checkpoint: {error}"))
    })?;
    let stored_context = fs::read_to_string(&context_fp_path).map_err(|error| {
        ApplicationError::PersistenceFailure(format!("failed to read checkpoint: {error}"))
    })?;
    if stored_source.trim() != expected_source || stored_context.trim() != expected_context {
        return Ok(None);
    }
    let translated = fs::read_to_string(&txt_path).map_err(|error| {
        ApplicationError::PersistenceFailure(format!("failed to read chapter text: {error}"))
    })?;
    Ok(Some(translated))
}

fn count_translated_paragraphs(layout: &ProjectLayout, stem: &str) -> usize {
    load_chapter_artifact(layout, stem)
        .ok()
        .flatten()
        .map(|artifact| artifact.paragraphs.len())
        .unwrap_or(0)
}

// ---------------------------------------------------------------------------
// Chapter artifacts
// ---------------------------------------------------------------------------

fn chapter_artifact_path(layout: &ProjectLayout, stem: &str) -> PathBuf {
    layout.chapters_dir.join(format!("{stem}.chapter.json"))
}

fn write_chapter_artifact(
    layout: &ProjectLayout,
    artifact: &TranslatedChapter,
    stem: &str,
) -> Result<(), ApplicationError> {
    super::project::atomic_write_json(&chapter_artifact_path(layout, stem), artifact)
}

fn load_chapter_artifact(
    layout: &ProjectLayout,
    stem: &str,
) -> Result<Option<TranslatedChapter>, ApplicationError> {
    let path = chapter_artifact_path(layout, stem);
    if !path.exists() {
        return Ok(None);
    }
    let bytes = fs::read(&path).map_err(|error| {
        ApplicationError::PersistenceFailure(format!("failed to read {path:?}: {error}"))
    })?;
    serde_json::from_slice(&bytes).map(Some).map_err(|error| {
        ApplicationError::PersistenceFailure(format!("invalid chapter artifact: {error}"))
    })
}

/// Split translated chapter text into paragraph records aligned with source
/// paragraphs where counts match; otherwise fall back to a single record so
/// the artifact stays bounded and honest.
fn align_paragraphs(chapter: &ManuscriptChapter, translated: &str) -> Vec<TranslatedParagraph> {
    let source_paragraphs = chapter
        .scenes
        .iter()
        .flat_map(|scene| scene.paragraphs.iter())
        .map(|paragraph| (paragraph.id.clone(), paragraph.original_text.clone()))
        .collect::<Vec<_>>();
    let translated_parts = translated
        .split("\n\n")
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(ToString::to_string)
        .collect::<Vec<_>>();

    if source_paragraphs.len() == translated_parts.len() && !source_paragraphs.is_empty() {
        source_paragraphs
            .iter()
            .zip(translated_parts.iter())
            .map(|((id, source), translated)| TranslatedParagraph {
                paragraph_id: id.clone(),
                source: source.clone(),
                translated: translated.clone(),
                origin: "provider".to_string(),
                revisions: Vec::new(),
            })
            .collect()
    } else {
        vec![TranslatedParagraph {
            paragraph_id: format!("chapter-{}", chapter.index + 1),
            source: source_paragraphs
                .iter()
                .map(|(_, text)| text.as_str())
                .collect::<Vec<_>>()
                .join("\n\n"),
            translated: translated.to_string(),
            origin: "provider".to_string(),
            revisions: Vec::new(),
        }]
    }
}

fn join_translated(paragraphs: &[TranslatedParagraph]) -> String {
    paragraphs
        .iter()
        .map(|paragraph| paragraph.translated.clone())
        .collect::<Vec<_>>()
        .join("\n\n")
}

// ---------------------------------------------------------------------------
// Translation run
// ---------------------------------------------------------------------------

/// Orchestrated translation over all (or a bounded subset of) chapters.
/// Pause/cancel flags are checked at chapter boundaries — the safest resume
/// point — and persisted progress is updated after every chapter.
pub fn run_translation(
    layout: &ProjectLayout,
    config: &TranslationConfig,
    sink: &mut dyn ProjectEventSink,
    resume: bool,
) -> Result<TranslationProgress, ApplicationError> {
    let manifest = super::project::load_manifest(layout)?;
    let project_id = manifest.project_id.clone();
    let source_record = manifest.source.as_ref().ok_or_else(|| {
        ApplicationError::InvalidProject("project has no imported source".to_string())
    })?;

    // Translation requires deterministic intelligence for chapter context.
    let intelligence = load_deterministic_artifact(layout)?.ok_or_else(|| {
        ApplicationError::AnalysisFailed("run analysis before starting translation".to_string())
    })?;

    let manuscript = load_manuscript(layout)?;
    let (characters, glossary) = load_canon(layout)?;
    let translation_memory = load_translation_memory()?;
    let pipeline = TranslationPipeline::default_literary_pipeline();
    let provider = configured_translation_provider(config)?;
    let provider_name = provider.name().to_string();

    let mut progress = match (resume, load_progress(layout)?) {
        (true, Some(existing)) => {
            if existing.source_fingerprint == source_record.fingerprint {
                existing
            } else {
                return Err(ApplicationError::ResumeIncompatible(
                    "progress belongs to a different source fingerprint".to_string(),
                ));
            }
        }
        (false, Some(_)) => return Err(ApplicationError::TranslationAlreadyRunning),
        (_, None) => {
            let total_paragraphs: usize = manuscript
                .chapters
                .iter()
                .flat_map(|chapter| chapter.scenes.iter())
                .map(|scene| scene.paragraphs.len())
                .sum();
            TranslationProgress {
                schema_version: TRANSLATION_PROGRESS_SCHEMA_VERSION,
                project_id: project_id.clone(),
                source_fingerprint: source_record.fingerprint.clone(),
                run_id: format!("run-{:x}", chrono::Utc::now().timestamp_millis() as u128),
                state: TranslationState::Running,
                provider: provider_name.clone(),
                model: config.model.clone(),
                target_language: config.target_language.clone(),
                total_chapters: manuscript.chapters.len(),
                completed_chapters: 0,
                current_chapter: None,
                completed_paragraphs: 0,
                total_paragraphs,
                percent: 0.0,
                last_checkpoint: None,
                warnings: Vec::new(),
                started_at: Utc::now(),
                updated_at: Utc::now(),
            }
        }
    };
    progress.state = TranslationState::Running;
    progress.provider = provider_name.clone();
    progress.updated_at = Utc::now();
    save_progress(layout, &progress)?;

    emit_and_history(
        layout,
        &project_id,
        sink,
        None,
        ProjectEvent::TranslationStarted {
            project_id: project_id.clone(),
            run_id: progress.run_id.clone(),
        },
    );

    let canon_fp = super::snapshot::canon_fingerprint(layout)?;
    let max_chapters = config.max_chapters.unwrap_or(usize::MAX);
    let mut completed_this_run = 0usize;

    for chapter in &manuscript.chapters {
        if completed_this_run >= max_chapters {
            break;
        }
        if pause_requested(layout) || cancel_requested(layout) {
            break;
        }

        let stem = chapter_stem(&chapter.title, chapter.index);
        let source_text = chapter.content.clone();
        let literary_lines = reviewed_literary_lines(layout, &chapter.id);
        let context = chapter_context(
            &manuscript.book.title,
            &chapter.title,
            &source_text,
            &characters,
            &glossary,
            &translation_memory,
            intelligence.initialization.context_for_chapter(&chapter.id),
            &literary_lines,
        );
        let source_fingerprint = content_fingerprint(source_text.as_bytes());
        let context_fingerprint = content_fingerprint(context.as_bytes());

        // Resume: reuse only chapters whose checkpoints match both the source
        // and the assembled context (canon changes invalidate reuse).
        if resume {
            if let Some(existing) =
                resumable_chapter(layout, &stem, &source_fingerprint, &context_fingerprint)?
            {
                progress.completed_chapters = progress.completed_chapters.max(chapter.index + 1);
                progress.completed_paragraphs += count_translated_paragraphs(layout, &stem);
                progress.percent = if progress.total_chapters == 0 {
                    1.0
                } else {
                    progress.completed_chapters as f32 / progress.total_chapters as f32
                };
                progress.last_checkpoint = Some(format!("{stem}.txt"));
                progress.updated_at = Utc::now();
                save_progress(layout, &progress)?;
                completed_this_run += 1;
                let _ = existing;
                continue;
            }
        }

        emit_and_history(
            layout,
            &project_id,
            sink,
            None,
            ProjectEvent::ChapterStarted {
                project_id: project_id.clone(),
                chapter_index: chapter.index,
            },
        );
        progress.current_chapter = Some(chapter.index);
        progress.updated_at = Utc::now();
        save_progress(layout, &progress)?;

        let output = pipeline
            .execute(
                provider.as_ref(),
                PipelineInput {
                    source_text: source_text.clone(),
                    target_language: config.target_language.clone(),
                    context,
                },
            )
            .map_err(|error| {
                ApplicationError::Internal(format!(
                    "pipeline failed for {}: {error}",
                    chapter.title
                ))
            })?;

        let rules = terminology_rules(&glossary, &source_text);
        let quality = evaluate_translation(&source_text, &output.quality_review, &rules);
        if !quality.passes() {
            return Err(ApplicationError::Internal(format!(
                "quality gate blocked {}: {}",
                chapter.title,
                quality.blocking_errors.join("; ")
            )));
        }

        // Persist artifacts: translated text + fingerprint checkpoints +
        // paragraph-structured chapter artifact (for the future editor).
        let txt_path = layout.chapters_dir.join(format!("{stem}.txt"));
        super::project::atomic_write_bytes(&txt_path, output.quality_review.as_bytes())?;
        let source_fp_path = layout
            .chapters_dir
            .join(format!("{stem}.source-fingerprint"));
        super::project::atomic_write_bytes(&source_fp_path, source_fingerprint.as_bytes())?;
        let context_fp_path = layout
            .chapters_dir
            .join(format!("{stem}.context-fingerprint"));
        super::project::atomic_write_bytes(&context_fp_path, context_fingerprint.as_bytes())?;
        let artifact = TranslatedChapter {
            schema_version: CHAPTER_ARTIFACT_SCHEMA_VERSION,
            chapter_index: chapter.index,
            chapter_id: chapter.id.clone(),
            title: chapter.title.clone(),
            source_fingerprint,
            context_fingerprint,
            paragraphs: align_paragraphs(chapter, &output.quality_review),
            quality_stale: false,
        };
        write_chapter_artifact(layout, &artifact, &stem)?;

        progress.completed_chapters = progress.completed_chapters.max(chapter.index + 1);
        progress.completed_paragraphs += artifact.paragraphs.len();
        progress.percent = if progress.total_chapters == 0 {
            1.0
        } else {
            progress.completed_chapters as f32 / progress.total_chapters as f32
        };
        progress.last_checkpoint = Some(format!("{stem}.txt"));
        progress.updated_at = Utc::now();
        save_progress(layout, &progress)?;
        completed_this_run += 1;

        emit_and_history(
            layout,
            &project_id,
            sink,
            None,
            ProjectEvent::ChapterCompleted {
                project_id: project_id.clone(),
                chapter_index: chapter.index,
                quality_score: quality.score,
            },
        );
        emit_and_history(
            layout,
            &project_id,
            sink,
            None,
            ProjectEvent::CheckpointSaved {
                project_id: project_id.clone(),
                chapter_index: chapter.index,
            },
        );
    }

    // Update project manifest translation record.
    let mut manifest = super::project::load_manifest(layout)?;
    let state = if cancel_requested(layout) || pause_requested(layout) {
        TranslationState::Paused
    } else if progress.completed_chapters >= progress.total_chapters {
        TranslationState::Completed
    } else {
        TranslationState::Paused
    };
    progress.state = state;
    progress.updated_at = Utc::now();
    save_progress(layout, &progress)?;

    manifest.translation = Some(super::models::TranslationRecord {
        run_id: progress.run_id.clone(),
        state,
        provider: provider_name.clone(),
        model: config.model.clone(),
        target_language: config.target_language.clone(),
        total_chapters: progress.total_chapters,
        completed_chapters: progress.completed_chapters,
        percent: progress.percent,
        started_at: progress.started_at,
        updated_at: progress.updated_at,
        canon_fingerprint_at_start: canon_fp,
    });
    manifest.updated_at = Utc::now();
    super::project::save_manifest(layout, &manifest)?;

    match state {
        TranslationState::Completed => {
            emit_and_history(
                layout,
                &project_id,
                sink,
                None,
                ProjectEvent::TranslationCompleted {
                    project_id: project_id.clone(),
                    run_id: progress.run_id.clone(),
                    chapters: progress.completed_chapters,
                },
            );
        }
        TranslationState::Paused => {
            emit_and_history(
                layout,
                &project_id,
                sink,
                None,
                ProjectEvent::TranslationPaused {
                    project_id: project_id.clone(),
                    run_id: progress.run_id.clone(),
                },
            );
        }
        _ => {}
    }

    Ok(progress)
}

fn pause_requested(layout: &ProjectLayout) -> bool {
    layout.translation_dir.join(PAUSE_FLAG).exists()
}

fn cancel_requested(layout: &ProjectLayout) -> bool {
    layout.translation_dir.join(CANCEL_FLAG).exists()
}

/// Request a pause at the next safe boundary (between chapters).
pub fn request_pause(layout: &ProjectLayout) -> Result<(), ApplicationError> {
    fs::write(layout.translation_dir.join(PAUSE_FLAG), b"requested").map_err(|error| {
        ApplicationError::PersistenceFailure(format!("failed to write pause flag: {error}"))
    })
}

/// Clear pause/cancel flags so a resumed run can proceed.
pub fn clear_pause(layout: &ProjectLayout) -> Result<(), ApplicationError> {
    let _ = fs::remove_file(layout.translation_dir.join(PAUSE_FLAG));
    let _ = fs::remove_file(layout.translation_dir.join(CANCEL_FLAG));
    Ok(())
}

pub fn get_progress(layout: &ProjectLayout) -> Result<TranslationProgress, ApplicationError> {
    load_progress(layout)?.ok_or(ApplicationError::NoCheckpoint)
}

pub fn translation_state(layout: &ProjectLayout) -> Option<TranslationState> {
    load_progress(layout)
        .ok()
        .flatten()
        .map(|progress| progress.state)
}

pub fn get_translated_chapter(
    layout: &ProjectLayout,
    chapter_index: usize,
) -> Result<TranslatedChapter, ApplicationError> {
    let manuscript = load_manuscript(layout)?;
    let chapter = manuscript
        .chapters
        .get(chapter_index)
        .ok_or_else(|| ApplicationError::Internal("chapter index out of range".to_string()))?;
    let stem = chapter_stem(&chapter.title, chapter.index);
    load_chapter_artifact(layout, &stem)?.ok_or(ApplicationError::NoCheckpoint)
}

/// Read the translated text of a chapter from its artifact (editor access).
pub fn get_translated_text(
    layout: &ProjectLayout,
    chapter_index: usize,
) -> Result<String, ApplicationError> {
    Ok(join_translated(
        &get_translated_chapter(layout, chapter_index)?.paragraphs,
    ))
}

/// Apply a manual translation edit to one paragraph: preserves the source,
/// records the previous value + a revision, invalidates the stored quality
/// result, and rewrites the chapter artifact + text.
pub fn apply_manual_translation_edit(
    layout: &ProjectLayout,
    chapter_index: usize,
    paragraph_id: &str,
    new_text: &str,
    reviewer: Option<&str>,
    sink: &mut dyn ProjectEventSink,
) -> Result<TranslationRevision, ApplicationError> {
    let manifest = super::project::load_manifest(layout)?;
    let project_id = manifest.project_id.clone();
    let manuscript = load_manuscript(layout)?;
    let chapter = manuscript
        .chapters
        .get(chapter_index)
        .ok_or_else(|| ApplicationError::Internal("chapter index out of range".to_string()))?;
    let stem = chapter_stem(&chapter.title, chapter.index);
    let mut artifact =
        load_chapter_artifact(layout, &stem)?.ok_or(ApplicationError::NoCheckpoint)?;

    let index = artifact
        .paragraphs
        .iter()
        .position(|paragraph| paragraph.paragraph_id == paragraph_id)
        .ok_or_else(|| {
            ApplicationError::Internal(format!(
                "paragraph '{paragraph_id}' not found in chapter {}",
                chapter_index + 1
            ))
        })?;
    let paragraph = &mut artifact.paragraphs[index];
    let previous = paragraph.translated.clone();
    if previous == new_text {
        return Err(ApplicationError::Internal(
            "new text is identical to the current translation".to_string(),
        ));
    }
    let revision = TranslationRevision {
        revision_id: content_fingerprint(
            format!("{chapter_index}\0{paragraph_id}\0{previous}\0{new_text}").as_bytes(),
        ),
        paragraph_id: paragraph_id.to_string(),
        previous: previous.clone(),
        new: new_text.to_string(),
        origin: "manual".to_string(),
        reviewer: reviewer.map(ToString::to_string),
        timestamp: Utc::now(),
    };
    paragraph.translated = new_text.to_string();
    paragraph.origin = "manual".to_string();
    paragraph.revisions.push(revision.clone());
    artifact.quality_stale = true;

    write_chapter_artifact(layout, &artifact, &stem)?;
    let txt_path = layout.chapters_dir.join(format!("{stem}.txt"));
    super::project::atomic_write_bytes(
        &txt_path,
        join_translated(&artifact.paragraphs).as_bytes(),
    )?;

    emit_and_history(
        layout,
        &project_id,
        sink,
        None,
        ProjectEvent::ManualEdit {
            project_id: project_id.clone(),
            chapter_index,
            revision_id: revision.revision_id.clone(),
        },
    );
    Ok(revision)
}

// ---------------------------------------------------------------------------
// Export
// ---------------------------------------------------------------------------

pub fn export_translation(
    layout: &ProjectLayout,
    sink: &mut dyn ProjectEventSink,
) -> Result<super::models::ExportRecord, ApplicationError> {
    let manifest = super::project::load_manifest(layout)?;
    let project_id = manifest.project_id.clone();
    let manuscript = load_manuscript(layout)?;
    if manuscript.chapters.is_empty() {
        return Err(ApplicationError::ExportUnavailable(
            "no chapters to export".to_string(),
        ));
    }

    emit_and_history(
        layout,
        &project_id,
        sink,
        None,
        ProjectEvent::ExportStarted {
            project_id: project_id.clone(),
        },
    );

    let mut translated = Vec::new();
    for chapter in &manuscript.chapters {
        let stem = chapter_stem(&chapter.title, chapter.index);
        let artifact = load_chapter_artifact(layout, &stem)?.ok_or_else(|| {
            ApplicationError::ExportUnavailable(format!(
                "chapter {} is not translated yet",
                chapter.index + 1
            ))
        })?;
        translated.push(document_engine::Chapter::translated(
            chapter.index,
            chapter.title.clone(),
            join_translated(&artifact.paragraphs),
        ));
    }

    let path = layout.export_dir.join("manuscript.docx");
    document_engine::export_persian_docx(&path, &manifest.name, &translated).map_err(|error| {
        ApplicationError::ExportUnavailable(format!("DOCX export failed: {error}"))
    })?;

    let record = super::models::ExportRecord {
        format: "docx".to_string(),
        relative_path: "manuscript.docx".to_string(),
        chapters: translated.len(),
        created_at: Utc::now(),
    };
    let mut manifest = super::project::load_manifest(layout)?;
    manifest.export = Some(record.clone());
    manifest.updated_at = Utc::now();
    super::project::save_manifest(layout, &manifest)?;

    emit_and_history(
        layout,
        &project_id,
        sink,
        None,
        ProjectEvent::ExportCompleted {
            project_id: project_id.clone(),
            format: "docx".to_string(),
        },
    );
    Ok(record)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn load_translation_memory() -> Result<TranslationMemory, ApplicationError> {
    match std::env::var("LITERARY_ENGINE_MEMORY_FILE") {
        Ok(path) if !path.trim().is_empty() => memory_engine::load_translation_memory(&path)
            .map_err(|error| {
                ApplicationError::PersistenceFailure(format!(
                    "failed to load translation memory {path}: {error}"
                ))
            }),
        _ => Ok(TranslationMemory::new()),
    }
}
