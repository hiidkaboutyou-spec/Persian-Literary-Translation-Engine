//! Phase 19 literary fidelity / Persian-naturalness review orchestration.
//!
//! Review is deliberately post-translation and non-mutating. Deterministic
//! structural checks always run. Optional semantic alignment and provider
//! critics can add evidence, but their absence/failure leaves dimensions
//! explicitly unevaluated rather than silently passing them.

use super::analysis::load_manuscript;
use super::error::ApplicationError;
use super::models::content_fingerprint;
use super::project::{atomic_write_json, ProjectLayout};
use super::review::{load_canon, load_ledger};
use super::translation;
use chrono::{DateTime, Utc};
use human_review_workflow::{ReviewKind, ReviewStatus, ReviewedValue};
use literary_review_engine::{
    attach_alignment_evidence, attach_native_persian_typography, attach_provider_review,
    review_native, AlignmentConfig, AlignmentSidecar, AlignmentToolRequest, LiteraryReviewProvider,
    LiteraryReviewReport, MockReviewProvider, OpenAIReviewProvider, ReviewDimension,
    ReviewProviderRequest, ReviewRequestLimits,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

pub const LITERARY_REVIEW_ARTIFACT_SCHEMA_VERSION: u32 = 1;
const APPROVED_CONTEXT_CHAR_LIMIT: usize = 12_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiteraryReviewSettings {
    /// `none`, `mock`, or `openai`. Provider review is explicit; default is offline.
    pub provider: String,
    pub model: Option<String>,
    /// Use the optional semantic-alignment tool when configured through
    /// `LITERARY_ENGINE_ALIGNMENT_TOOL`.
    pub semantic_alignment: bool,
    /// Optional bound for interactive/CI runs.
    pub max_chapters: Option<usize>,
}

impl Default for LiteraryReviewSettings {
    fn default() -> Self {
        Self {
            provider: "none".into(),
            model: None,
            semantic_alignment: true,
            max_chapters: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceRunState {
    NotRequested,
    NotConfigured,
    Completed,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceRunStatus {
    pub state: EvidenceRunState,
    pub tool: String,
    pub model: Option<String>,
    pub error: Option<String>,
}

impl EvidenceRunStatus {
    fn not_requested(tool: impl Into<String>) -> Self {
        Self {
            state: EvidenceRunState::NotRequested,
            tool: tool.into(),
            model: None,
            error: None,
        }
    }

    fn not_configured(tool: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            state: EvidenceRunState::NotConfigured,
            tool: tool.into(),
            model: None,
            error: Some(message.into()),
        }
    }

    fn completed(tool: impl Into<String>, model: Option<String>) -> Self {
        Self {
            state: EvidenceRunState::Completed,
            tool: tool.into(),
            model,
            error: None,
        }
    }

    fn failed(tool: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            state: EvidenceRunState::Failed,
            tool: tool.into(),
            model: None,
            error: Some(error.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LiteraryReviewArtifact {
    pub schema_version: u32,
    pub chapter_index: usize,
    pub chapter_id: String,
    pub title: String,
    pub source_fingerprint: String,
    pub translation_fingerprint: String,
    pub translation_context_fingerprint: String,
    pub reviewed_at: DateTime<Utc>,
    pub report: LiteraryReviewReport,
    pub semantic_alignment: EvidenceRunStatus,
    pub provider_review: EvidenceRunStatus,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LiteraryReviewArtifactView {
    pub artifact: LiteraryReviewArtifact,
    /// True when the current translated text/source no longer matches what the
    /// stored review evaluated (for example after a manual editor revision).
    pub stale: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiteraryReviewRunSummary {
    pub reviewed_chapters: usize,
    pub findings: usize,
    pub chapters_requiring_attention: usize,
    pub alignment_failures: usize,
    pub provider_failures: usize,
    pub artifacts: Vec<String>,
}

pub fn run_literary_review(
    layout: &ProjectLayout,
    settings: &LiteraryReviewSettings,
) -> Result<LiteraryReviewRunSummary, ApplicationError> {
    let manuscript = load_manuscript(layout)?;
    let (characters, glossary) = load_canon(layout)?;
    let provider = configured_provider(settings)?;
    let alignment_sidecar = configured_alignment_sidecar(settings);

    let review_dir = review_dir(layout);
    fs::create_dir_all(&review_dir).map_err(|error| {
        ApplicationError::PersistenceFailure(format!(
            "failed to create literary review directory {}: {error}",
            review_dir.display()
        ))
    })?;

    let mut summary = LiteraryReviewRunSummary {
        reviewed_chapters: 0,
        findings: 0,
        chapters_requiring_attention: 0,
        alignment_failures: 0,
        provider_failures: 0,
        artifacts: Vec::new(),
    };

    let max_chapters = settings.max_chapters.unwrap_or(usize::MAX);
    for chapter in manuscript.chapters.iter().take(max_chapters) {
        let translated = match translation::get_translated_chapter(layout, chapter.index) {
            Ok(value) => value,
            Err(ApplicationError::NoCheckpoint) => continue,
            Err(error) => return Err(error),
        };
        let target = translated_text(&translated.paragraphs);
        let source = chapter.content.as_str();
        let mut report = review_native(chapter.id.clone(), source, &target, Default::default());
        attach_native_persian_typography(&mut report, &target);

        let (semantic_alignment, alignment_failed) = if !settings.semantic_alignment {
            (
                EvidenceRunStatus::not_requested("semantic_alignment"),
                false,
            )
        } else if let Some(sidecar) = alignment_sidecar.as_ref() {
            let source_segments = paragraph_segments(source);
            let target_segments = paragraph_segments(&target);
            match AlignmentToolRequest::new(
                chapter.id.clone(),
                source_segments,
                target_segments,
                AlignmentConfig::default(),
            ) {
                Ok(request) => match sidecar.align(&request) {
                    Ok(result) => {
                        let model = result.embedding_model.clone();
                        match attach_alignment_evidence(&mut report, &result) {
                            Ok(()) => (
                                EvidenceRunStatus::completed("semantic_alignment", Some(model)),
                                false,
                            ),
                            Err(error) => {
                                let message = error.to_string();
                                report.advisory_notes.push(format!(
                                    "Semantic alignment evidence was discarded: {message}"
                                ));
                                (
                                    EvidenceRunStatus::failed("semantic_alignment", message),
                                    true,
                                )
                            }
                        }
                    }
                    Err(error) => {
                        let message = error.to_string();
                        report.advisory_notes.push(format!(
                            "Semantic alignment unavailable; omission/addition review remains on deterministic structural evidence only: {message}"
                        ));
                        (
                            EvidenceRunStatus::failed("semantic_alignment", message),
                            true,
                        )
                    }
                },
                Err(error) => {
                    let message = error.to_string();
                    report
                        .advisory_notes
                        .push(format!("Semantic alignment request was not run: {message}"));
                    (
                        EvidenceRunStatus::failed("semantic_alignment", message),
                        true,
                    )
                }
            }
        } else {
            (
                EvidenceRunStatus::not_configured(
                    "semantic_alignment",
                    "LITERARY_ENGINE_ALIGNMENT_TOOL is not configured",
                ),
                false,
            )
        };
        summary.alignment_failures += usize::from(alignment_failed);

        let approved_context =
            approved_context(layout, &chapter.id, source, &characters, &glossary);
        let (provider_review, provider_failed) = if let Some(provider) = provider.as_deref() {
            let dimensions = provider_dimensions(&translated.style_profile);
            match ReviewProviderRequest::from_text(
                chapter.id.clone(),
                source,
                &target,
                approved_context,
                dimensions,
                ReviewRequestLimits::default(),
            ) {
                Ok(request) => match provider.review(&request) {
                    Ok(result) => {
                        let provider_name = result.provider.clone();
                        let model = result.model.clone();
                        attach_provider_review(&mut report, &result);
                        (
                            EvidenceRunStatus::completed(provider_name, Some(model)),
                            false,
                        )
                    }
                    Err(error) => {
                        let message = error.to_string();
                        report.advisory_notes.push(format!(
                            "Provider critic failed; its requested literary dimensions remain unevaluated: {message}"
                        ));
                        (EvidenceRunStatus::failed(provider.name(), message), true)
                    }
                },
                Err(error) => {
                    let message = error.to_string();
                    report.advisory_notes.push(format!(
                        "Provider critic request was not run; its requested literary dimensions remain unevaluated: {message}"
                    ));
                    (EvidenceRunStatus::failed(provider.name(), message), true)
                }
            }
        } else {
            (EvidenceRunStatus::not_requested("provider_critic"), false)
        };
        summary.provider_failures += usize::from(provider_failed);

        let translation_fingerprint = content_fingerprint(target.as_bytes());
        let artifact = LiteraryReviewArtifact {
            schema_version: LITERARY_REVIEW_ARTIFACT_SCHEMA_VERSION,
            chapter_index: chapter.index,
            chapter_id: chapter.id.clone(),
            title: chapter.title.clone(),
            source_fingerprint: content_fingerprint(source.as_bytes()),
            translation_fingerprint,
            translation_context_fingerprint: translated.context_fingerprint.clone(),
            reviewed_at: Utc::now(),
            report,
            semantic_alignment,
            provider_review,
        };
        let path = artifact_path(layout, chapter.index);
        atomic_write_json(&path, &artifact)?;
        let relative = path
            .strip_prefix(&layout.root)
            .unwrap_or(&path)
            .to_string_lossy()
            .to_string();
        summary.artifacts.push(relative);
        summary.reviewed_chapters += 1;
        summary.findings += artifact.report.findings.len();
        summary.chapters_requiring_attention += usize::from(artifact.report.requires_attention());
    }

    if summary.reviewed_chapters == 0 {
        return Err(ApplicationError::NoCheckpoint);
    }
    Ok(summary)
}

pub fn get_literary_review(
    layout: &ProjectLayout,
    chapter_index: usize,
) -> Result<LiteraryReviewArtifactView, ApplicationError> {
    let path = artifact_path(layout, chapter_index);
    let bytes = fs::read(&path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            ApplicationError::NoCheckpoint
        } else {
            ApplicationError::PersistenceFailure(format!(
                "failed to read literary review {}: {error}",
                path.display()
            ))
        }
    })?;
    let artifact: LiteraryReviewArtifact = serde_json::from_slice(&bytes).map_err(|error| {
        ApplicationError::PersistenceFailure(format!(
            "invalid literary review artifact {}: {error}",
            path.display()
        ))
    })?;
    if artifact.schema_version != LITERARY_REVIEW_ARTIFACT_SCHEMA_VERSION {
        return Err(ApplicationError::PersistenceFailure(format!(
            "unsupported literary review artifact schema {}",
            artifact.schema_version
        )));
    }

    let manuscript = load_manuscript(layout)?;
    let chapter = manuscript
        .chapters
        .get(chapter_index)
        .ok_or_else(|| ApplicationError::Internal("chapter index out of range".into()))?;
    let translated = translation::get_translated_chapter(layout, chapter_index)?;
    let target = translated_text(&translated.paragraphs);
    let stale = artifact.chapter_id != chapter.id
        || artifact.source_fingerprint != content_fingerprint(chapter.content.as_bytes())
        || artifact.translation_fingerprint != content_fingerprint(target.as_bytes())
        || artifact.translation_context_fingerprint != translated.context_fingerprint;

    Ok(LiteraryReviewArtifactView { artifact, stale })
}

fn configured_provider(
    settings: &LiteraryReviewSettings,
) -> Result<Option<Box<dyn LiteraryReviewProvider>>, ApplicationError> {
    match settings.provider.trim().to_ascii_lowercase().as_str() {
        "" | "none" | "off" => Ok(None),
        "mock" => Ok(Some(Box::new(MockReviewProvider::no_findings(
            provider_dimensions("literary"),
        )))),
        "openai" => {
            let provider = if let Some(model) = settings.model.as_deref() {
                let api_key = std::env::var("OPENAI_API_KEY").map_err(|_| {
                    ApplicationError::ProviderNotConfigured(
                        "OPENAI_API_KEY is not configured for literary review".into(),
                    )
                })?;
                OpenAIReviewProvider::new(api_key, model).map_err(map_provider_config_error)?
            } else {
                OpenAIReviewProvider::from_env().map_err(map_provider_config_error)?
            };
            Ok(Some(Box::new(provider)))
        }
        other => Err(ApplicationError::ProviderNotConfigured(format!(
            "unsupported literary review provider '{other}'; expected none, mock, or openai"
        ))),
    }
}

fn map_provider_config_error(
    error: literary_review_engine::ReviewProviderError,
) -> ApplicationError {
    match error {
        literary_review_engine::ReviewProviderError::Authentication(message) => {
            ApplicationError::ProviderAuthenticationFailed(message)
        }
        other => ApplicationError::ProviderNotConfigured(other.to_string()),
    }
}

fn configured_alignment_sidecar(settings: &LiteraryReviewSettings) -> Option<AlignmentSidecar> {
    if !settings.semantic_alignment {
        return None;
    }
    std::env::var("LITERARY_ENGINE_ALIGNMENT_TOOL")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .map(AlignmentSidecar::new)
}

fn provider_dimensions(style_profile: &str) -> Vec<ReviewDimension> {
    let mut dimensions = vec![
        ReviewDimension::SemanticFidelity,
        ReviewDimension::CharacterVoice,
        ReviewDimension::RelationshipRegister,
        ReviewDimension::PersianNaturalness,
        ReviewDimension::DialogueSubtext,
        ReviewDimension::TerminologyContinuity,
    ];
    if style_profile == "adult-intimacy" {
        dimensions.push(ReviewDimension::IntimacyFidelity);
    }
    dimensions
}

fn paragraph_segments(text: &str) -> Vec<String> {
    let paragraphs = text
        .split("\n\n")
        .map(str::trim)
        .filter(|paragraph| !paragraph.is_empty())
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    if paragraphs.is_empty() && !text.trim().is_empty() {
        vec![text.trim().to_string()]
    } else {
        paragraphs
    }
}

fn translated_text(paragraphs: &[super::models::TranslatedParagraph]) -> String {
    paragraphs
        .iter()
        .map(|paragraph| paragraph.translated.as_str())
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn approved_context(
    layout: &ProjectLayout,
    chapter_id: &str,
    source: &str,
    characters: &character_engine::CharacterBible,
    glossary: &memory_engine::glossary::Glossary,
) -> String {
    let mut sections = Vec::new();
    let character_context = characters.context_for_text(source);
    if !character_context.trim().is_empty() {
        sections.push(character_context);
    }
    let glossary_lines = glossary
        .relevant_to_text(source)
        .into_iter()
        .map(|entry| {
            if entry.context.trim().is_empty() {
                format!(
                    "Glossary: {} => {}",
                    entry.source_term, entry.preferred_translation
                )
            } else {
                format!(
                    "Glossary: {} => {} ({})",
                    entry.source_term, entry.preferred_translation, entry.context
                )
            }
        })
        .collect::<Vec<_>>();
    if !glossary_lines.is_empty() {
        sections.push(glossary_lines.join("\n"));
    }
    if let Ok(ledger) = load_ledger(layout) {
        let literary = ledger
            .items
            .iter()
            .filter(|item| item.kind == ReviewKind::Literary)
            .filter(|item| matches!(item.status, ReviewStatus::Approved | ReviewStatus::Edited))
            .filter_map(|item| {
                let ReviewedValue::Literary(value) = item.reviewed_value.as_ref()? else {
                    return None;
                };
                let relevant = item
                    .latest_proposal
                    .evidence()
                    .iter()
                    .any(|evidence| evidence.chapter_id == chapter_id);
                relevant.then(|| {
                    format!(
                        "Reviewed literary canon [{}] {} ({}) — {}",
                        value.finding_category, value.subject, value.scope_label, value.claim
                    )
                })
            })
            .collect::<Vec<_>>();
        if !literary.is_empty() {
            sections.push(literary.join("\n"));
        }
    }
    truncate_chars(&sections.join("\n\n"), APPROVED_CONTEXT_CHAR_LIMIT)
}

fn truncate_chars(text: &str, limit: usize) -> String {
    if text.chars().count() <= limit {
        text.to_string()
    } else {
        text.chars().take(limit).collect()
    }
}

fn review_dir(layout: &ProjectLayout) -> PathBuf {
    layout.translation_dir.join("reviews")
}

fn artifact_path(layout: &ProjectLayout, chapter_index: usize) -> PathBuf {
    review_dir(layout).join(format!("{:03}.literary-review.json", chapter_index + 1))
}

pub fn artifact_exists(layout: &ProjectLayout, chapter_index: usize) -> bool {
    artifact_path(layout, chapter_index).is_file()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_review_is_offline_but_allows_optional_alignment() {
        let settings = LiteraryReviewSettings::default();
        assert_eq!(settings.provider, "none");
        assert!(settings.semantic_alignment);
    }

    #[test]
    fn paragraph_segmentation_preserves_order() {
        assert_eq!(
            paragraph_segments("one\n\ntwo\n\nthree"),
            vec!["one", "two", "three"]
        );
    }

    #[test]
    fn approved_context_truncation_is_unicode_safe() {
        let value = "الف".repeat(20);
        let truncated = truncate_chars(&value, 7);
        assert_eq!(truncated.chars().count(), 7);
        assert!(std::str::from_utf8(truncated.as_bytes()).is_ok());
    }

    #[test]
    fn provider_dimensions_leave_structural_omission_to_native_alignment_stack() {
        let dimensions = provider_dimensions("literary");
        assert!(!dimensions.contains(&ReviewDimension::OmissionAddition));
        assert!(dimensions.contains(&ReviewDimension::PersianNaturalness));
        assert!(dimensions.contains(&ReviewDimension::DialogueSubtext));
    }
}
