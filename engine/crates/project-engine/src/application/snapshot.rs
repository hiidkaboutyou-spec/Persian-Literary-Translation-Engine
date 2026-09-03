//! Snapshot assembly: turns persisted artifacts into one UI-ready structure.
//!
//! Snapshot never re-ingests the manuscript and never re-runs analysis. It
//! reads only small persisted metadata: `project.json`, the review ledger,
//! the Character Bible / Glossary counts, and translation progress.

use super::error::ApplicationError;
use super::models::{
    ArtifactState, ArtifactSummary, CanonSummary, ExportSummary, NextAction, ProjectFile,
    ProjectSnapshot, ProjectStatus, ReviewSummary, SourceSummary, TranslationState,
    TranslationSummary,
};
use super::project::{load_manifest, ProjectLayout};
use std::fs;

/// Cheap source-change check: compares stored fingerprint against current file
/// metadata hash only when size differs, so snapshots stay fast.
pub(crate) fn source_state(
    layout: &ProjectLayout,
    manifest: &ProjectFile,
) -> (Option<SourceSummary>, Vec<String>) {
    let mut warnings = Vec::new();
    let record = match &manifest.source {
        Some(record) => record,
        None => return (None, warnings),
    };
    let stored = layout
        .source_dir
        .join(record.stored_relative_path.trim_start_matches("source/"));
    let state = match fs::metadata(&stored) {
        Ok(metadata) => {
            if metadata.len() == record.size_bytes {
                ArtifactState::Fresh
            } else {
                warnings.push(
                    "source file size changed since import; analysis and checkpoints may be stale"
                        .to_string(),
                );
                ArtifactState::Stale
            }
        }
        Err(_) => {
            warnings.push("imported source file is missing".to_string());
            ArtifactState::Missing
        }
    };
    let summary = SourceSummary {
        original_filename: record.original_filename.clone(),
        format: record.format.clone(),
        fingerprint: record.fingerprint.clone(),
        title: record.title.clone(),
        chapters: record.chapter_count,
        scenes: record.scene_count,
        paragraphs: record.paragraph_count,
        state,
        imported_at: record.imported_at,
    };
    (Some(summary), warnings)
}

fn analysis_state(manifest: &ProjectFile) -> Option<ArtifactSummary> {
    let record = manifest.analysis.as_ref()?;
    let state = if record.source_fingerprint == manifest.source.as_ref()?.fingerprint {
        ArtifactState::Fresh
    } else {
        ArtifactState::Stale
    };
    Some(ArtifactSummary {
        state,
        detail: format!(
            "{} characters, {} relationships, {} terminology seeds (analyzer {})",
            record.character_seeds,
            record.relationship_seeds,
            record.terminology_seeds,
            record.analyzer
        ),
    })
}

fn advanced_state(manifest: &ProjectFile) -> Option<ArtifactSummary> {
    let record = manifest.advanced.as_ref()?;
    let state = if record.source_fingerprint == manifest.source.as_ref()?.fingerprint {
        ArtifactState::Fresh
    } else {
        ArtifactState::Stale
    };
    Some(ArtifactSummary {
        state,
        detail: format!(
            "{} findings, {} units ({} cached, {} failed) via {}/{}",
            record.findings,
            record.total_units,
            record.cached_units,
            record.failed_units,
            record.provider,
            record.model
        ),
    })
}

fn translation_summary(manifest: &ProjectFile) -> Option<TranslationSummary> {
    let record = manifest.translation.as_ref()?;
    Some(TranslationSummary {
        run_id: record.run_id.clone(),
        state: record.state,
        provider: record.provider.clone(),
        model: record.model.clone(),
        target_language: record.target_language.clone(),
        completed_chapters: record.completed_chapters,
        total_chapters: record.total_chapters,
        percent: record.percent,
        context_stale: !manifest.canon_fingerprint.is_empty()
            && manifest.canon_fingerprint != record.canon_fingerprint_at_start,
        started_at: record.started_at,
        updated_at: record.updated_at,
    })
}

fn export_summary(manifest: &ProjectFile, layout: &ProjectLayout) -> Option<ExportSummary> {
    let record = manifest.export.as_ref()?;
    Some(ExportSummary {
        format: record.format.clone(),
        relative_path: record.relative_path.clone(),
        chapters: record.chapters,
        created_at: record.created_at,
        exists: layout.export_dir.join(&record.relative_path).exists(),
    })
}

fn review_summary(
    layout: &ProjectLayout,
    project_id: &str,
) -> Result<ReviewSummary, ApplicationError> {
    use human_review_workflow::{ProposalAvailability, ReviewKind, ReviewStatus};
    let mut summary = ReviewSummary::default();
    if !layout.review_file.exists() {
        return Ok(summary);
    }
    let ledger = super::review::load_ledger(layout)?;
    for item in &ledger.items {
        summary.total += 1;
        match item.status {
            ReviewStatus::Pending => summary.pending += 1,
            ReviewStatus::Approved => summary.approved += 1,
            ReviewStatus::Edited => summary.edited += 1,
            ReviewStatus::Rejected => summary.rejected += 1,
            ReviewStatus::Deferred => summary.deferred += 1,
            ReviewStatus::Applied => summary.applied += 1,
        }
        if item.availability == ProposalAvailability::Obsolete {
            summary.obsolete += 1;
        }
        if item.kind == ReviewKind::Literary {
            summary.literary += 1;
        }
    }
    let _ = project_id;
    Ok(summary)
}

fn canon_summary(layout: &ProjectLayout, manifest: &ProjectFile) -> CanonSummary {
    let characters = match character_engine::CharacterBible::load_json(&layout.characters_file) {
        Ok(bible) => bible,
        Err(_) => character_engine::CharacterBible::new(),
    };
    let glossary = memory_engine::load_glossary(&layout.glossary_file).unwrap_or_default();
    CanonSummary {
        characters: characters.profiles().len(),
        aliases: characters.aliases().len(),
        relationships: characters.relationships().len(),
        glossary_entries: glossary.entries().len(),
        fingerprint: manifest.canon_fingerprint.clone(),
    }
}

fn derive_status_and_action(
    manifest: &ProjectFile,
    review: &ReviewSummary,
    translation: Option<&TranslationSummary>,
    export: Option<&ExportSummary>,
    source_fresh: bool,
) -> (ProjectStatus, NextAction) {
    if manifest.source.is_none() {
        return (ProjectStatus::Empty, NextAction::ImportBook);
    }
    if !source_fresh {
        return (ProjectStatus::Error, NextAction::ReimportSource);
    }
    if manifest.analysis.is_none() {
        return (ProjectStatus::Imported, NextAction::RunAnalysis);
    }
    if review.pending > 0 || review.deferred > 0 {
        return (ProjectStatus::NeedsReview, NextAction::ReviewIntelligence);
    }
    match translation {
        Some(summary) => match summary.state {
            TranslationState::Running => {
                (ProjectStatus::Translating, NextAction::ResumeTranslation)
            }
            TranslationState::Paused => (ProjectStatus::Paused, NextAction::ResumeTranslation),
            TranslationState::Completed => {
                if let Some(export) = export {
                    if export.exists {
                        (ProjectStatus::Exported, NextAction::None)
                    } else {
                        (ProjectStatus::ReadyToExport, NextAction::Export)
                    }
                } else {
                    (ProjectStatus::ReadyToExport, NextAction::Export)
                }
            }
            TranslationState::Failed => (ProjectStatus::Error, NextAction::ResumeTranslation),
            TranslationState::NotStarted => (
                ProjectStatus::ReadyToTranslate,
                NextAction::StartTranslation,
            ),
        },
        None => (
            ProjectStatus::ReadyToTranslate,
            NextAction::StartTranslation,
        ),
    }
}

/// Assemble the full project snapshot from persisted metadata only.
pub fn snapshot(
    layout: &ProjectLayout,
    project_id: &str,
) -> Result<ProjectSnapshot, ApplicationError> {
    let manifest = load_manifest(layout)?;
    let (source, mut warnings) = source_state(layout, &manifest);
    let source_fresh = matches!(
        source.as_ref().map(|summary| summary.state),
        Some(ArtifactState::Fresh)
    );
    let review = review_summary(layout, project_id)?;
    let canon = canon_summary(layout, &manifest);
    let translation = manifest
        .translation
        .as_ref()
        .and_then(|_record| translation_summary(&manifest));
    let export = manifest
        .export
        .as_ref()
        .and_then(|_record| export_summary(&manifest, layout));
    if let Some(translation) = &translation {
        if translation.context_stale {
            warnings.push(
                "canon changed since this translation run started; context is stale".to_string(),
            );
        }
    }
    let (status, next_action) = derive_status_and_action(
        &manifest,
        &review,
        translation.as_ref(),
        export.as_ref(),
        source_fresh,
    );

    Ok(ProjectSnapshot {
        schema_version: super::models::PROJECT_SCHEMA_VERSION,
        project_id: manifest.project_id.clone(),
        name: manifest.name.clone(),
        root: layout.root.display().to_string(),
        created_at: manifest.created_at,
        updated_at: manifest.updated_at,
        target_language: manifest.target_language.clone(),
        source,
        analysis: analysis_state(&manifest),
        advanced: advanced_state(&manifest),
        review,
        canon,
        translation,
        export,
        status,
        next_action,
        warnings,
    })
}

/// Recompute the canon fingerprint from the current Character Bible + Glossary
/// files (or an empty fingerprint when neither exists yet).
pub(crate) fn canon_fingerprint(layout: &ProjectLayout) -> Result<String, ApplicationError> {
    use super::models::content_fingerprint;
    let mut bytes = Vec::new();
    if layout.characters_file.exists() {
        bytes.extend_from_slice(&fs::read(&layout.characters_file).map_err(|error| {
            ApplicationError::PersistenceFailure(format!(
                "failed to read {}: {error}",
                layout.characters_file.display()
            ))
        })?);
    }
    bytes.push(0);
    if layout.glossary_file.exists() {
        bytes.extend_from_slice(&fs::read(&layout.glossary_file).map_err(|error| {
            ApplicationError::PersistenceFailure(format!(
                "failed to read {}: {error}",
                layout.glossary_file.display()
            ))
        })?);
    }
    Ok(content_fingerprint(&bytes))
}
