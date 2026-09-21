//! Phase 28 — privacy-safe whole-book pilot audit and review sampling.
//!
//! The audit is computed locally from existing project artifacts. It never
//! stores manuscript/translation text in the report and never calls a model,
//! network service, or external tracker.

use super::analysis::load_manuscript;
use super::error::ApplicationError;
use super::literary_review;
use super::models::{content_fingerprint, TranslationState};
use super::project::{load_manifest, ProjectLayout};
use super::translation;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const PILOT_AUDIT_SCHEMA_VERSION: u32 = 1;
pub const DEFAULT_MAX_REVIEW_TARGETS: usize = 12;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PilotAuditIssueCode {
    TranslationNotComplete,
    LegacyOrMissingPlanIdentity,
    ProgressSourceMismatch,
    ChapterMissingTranslation,
    ChapterSourceMismatch,
    ChapterPlanMismatch,
    LiteraryReviewMissing,
    LiteraryReviewStale,
    LiteraryReviewAttention,
    ManualRevisionPresent,
    TranslationQualityStale,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PilotAuditIssue {
    pub code: PilotAuditIssueCode,
    pub chapter_index: Option<usize>,
    pub chapter_id: Option<String>,
    /// Blocking means the current artifacts are not mechanically safe for
    /// Phase-27 export. Review-only findings never become automatic quality
    /// verdicts.
    pub blocking: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PilotSampleReason {
    EarlyBook,
    MiddleBook,
    LateBook,
    LongChapter,
    DialogueHeavy,
    ManualRevision,
    LiteraryReviewStale,
    LiteraryReviewAttention,
    TranslationQualityStale,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PilotReviewTarget {
    pub chapter_index: usize,
    pub chapter_id: String,
    /// Stable source paragraph identity only. The audit deliberately carries
    /// no source or translated text.
    pub paragraph_id: Option<String>,
    pub reasons: Vec<PilotSampleReason>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BookPilotAudit {
    pub schema_version: u32,
    pub project_id: String,
    pub source_fingerprint: String,
    pub translation_plan_fingerprint: Option<String>,
    pub total_chapters: usize,
    pub translated_chapters: usize,
    pub total_source_paragraphs: usize,
    pub completed_source_paragraphs: usize,
    pub literary_review_coverage_chapters: usize,
    pub literary_review_stale_chapters: usize,
    pub literary_review_attention_chapters: usize,
    pub manual_revision_chapters: usize,
    pub translation_complete: bool,
    pub mechanically_export_ready: bool,
    /// True only when every translated chapter has a current literary review,
    /// none of those reviews requests attention, and no chapter carries stale
    /// post-edit quality evidence. This is workflow state, not a quality score.
    pub human_review_clear: bool,
    pub issues: Vec<PilotAuditIssue>,
    pub review_targets: Vec<PilotReviewTarget>,
}

#[derive(Debug, Clone)]
struct TargetCandidate {
    chapter_index: usize,
    chapter_id: String,
    paragraph_id: Option<String>,
    reasons: BTreeSet<PilotSampleReason>,
}

impl TargetCandidate {
    fn priority(&self) -> u16 {
        self.reasons
            .iter()
            .map(|reason| match reason {
                PilotSampleReason::LiteraryReviewAttention => 100,
                PilotSampleReason::LiteraryReviewStale => 95,
                PilotSampleReason::ManualRevision => 90,
                PilotSampleReason::TranslationQualityStale => 85,
                PilotSampleReason::DialogueHeavy => 60,
                PilotSampleReason::LongChapter => 50,
                PilotSampleReason::LateBook => 40,
                PilotSampleReason::MiddleBook => 30,
                PilotSampleReason::EarlyBook => 20,
            })
            .max()
            .unwrap_or(0)
    }
}

pub fn build_pilot_audit(
    layout: &ProjectLayout,
    max_review_targets: usize,
) -> Result<BookPilotAudit, ApplicationError> {
    let manifest = load_manifest(layout)?;
    let manuscript = load_manuscript(layout)?;
    let source_record = manifest.source.as_ref().ok_or_else(|| {
        ApplicationError::Internal("pilot audit requires an imported source".to_string())
    })?;
    let progress = translation::get_progress(layout).ok();
    let current_plan = progress
        .as_ref()
        .map(|value| value.translation_plan_fingerprint.trim().to_string())
        .filter(|value| !value.is_empty());

    let mut issues = Vec::new();
    let mut translated_chapters = 0usize;
    let mut completed_source_paragraphs = 0usize;
    let mut review_coverage = 0usize;
    let mut stale_reviews = 0usize;
    let mut attention_reviews = 0usize;
    let mut manual_revision_chapters = 0usize;
    let mut candidates: BTreeMap<usize, TargetCandidate> = BTreeMap::new();
    let mut sample_eligible_chapters = BTreeSet::new();

    let total_source_paragraphs = manuscript.paragraph_count();
    let total_chapters = manuscript.chapters.len();

    if let Some(progress) = progress.as_ref() {
        if progress.state != TranslationState::Completed
            || progress.completed_chapters != total_chapters
            || progress.completed_paragraphs != total_source_paragraphs
        {
            issues.push(PilotAuditIssue {
                code: PilotAuditIssueCode::TranslationNotComplete,
                chapter_index: None,
                chapter_id: None,
                blocking: true,
            });
        }
        if progress.translation_plan_fingerprint.trim().is_empty() {
            issues.push(PilotAuditIssue {
                code: PilotAuditIssueCode::LegacyOrMissingPlanIdentity,
                chapter_index: None,
                chapter_id: None,
                blocking: true,
            });
        }
        if progress.source_fingerprint != source_record.fingerprint {
            issues.push(PilotAuditIssue {
                code: PilotAuditIssueCode::ProgressSourceMismatch,
                chapter_index: None,
                chapter_id: None,
                blocking: true,
            });
        }
    } else {
        issues.push(PilotAuditIssue {
            code: PilotAuditIssueCode::TranslationNotComplete,
            chapter_index: None,
            chapter_id: None,
            blocking: true,
        });
    }

    for chapter in &manuscript.chapters {
        let source_paragraphs = chapter
            .scenes
            .iter()
            .flat_map(|scene| scene.paragraphs.iter())
            .collect::<Vec<_>>();
        let translated = match translation::get_translated_chapter(layout, chapter.index) {
            Ok(value) => Some(value),
            Err(ApplicationError::NoCheckpoint) => None,
            Err(error) => return Err(error),
        };

        if let Some(translated) = translated.as_ref() {
            translated_chapters += 1;
            let expected_source = content_fingerprint(chapter.content.as_bytes());
            let source_ok = translated.chapter_id == chapter.id
                && translated.source_fingerprint == expected_source;
            let plan_ok = current_plan
                .as_deref()
                .is_some_and(|plan| translated.translation_plan_fingerprint == plan);

            if source_ok && plan_ok {
                completed_source_paragraphs += source_paragraphs.len();
                sample_eligible_chapters.insert(chapter.index);
            }
            if !source_ok {
                issues.push(PilotAuditIssue {
                    code: PilotAuditIssueCode::ChapterSourceMismatch,
                    chapter_index: Some(chapter.index),
                    chapter_id: Some(chapter.id.clone()),
                    blocking: true,
                });
            }
            if !plan_ok {
                issues.push(PilotAuditIssue {
                    code: PilotAuditIssueCode::ChapterPlanMismatch,
                    chapter_index: Some(chapter.index),
                    chapter_id: Some(chapter.id.clone()),
                    blocking: true,
                });
            }

            let manual_paragraph = translated
                .paragraphs
                .iter()
                .find(|paragraph| paragraph.origin == "manual" || !paragraph.revisions.is_empty())
                .map(|paragraph| paragraph.paragraph_id.clone());
            if manual_paragraph.is_some() {
                manual_revision_chapters += 1;
                issues.push(PilotAuditIssue {
                    code: PilotAuditIssueCode::ManualRevisionPresent,
                    chapter_index: Some(chapter.index),
                    chapter_id: Some(chapter.id.clone()),
                    blocking: false,
                });
                add_candidate(
                    &mut candidates,
                    chapter,
                    manual_paragraph.clone(),
                    PilotSampleReason::ManualRevision,
                );
            }
            if translated.quality_stale {
                issues.push(PilotAuditIssue {
                    code: PilotAuditIssueCode::TranslationQualityStale,
                    chapter_index: Some(chapter.index),
                    chapter_id: Some(chapter.id.clone()),
                    blocking: false,
                });
                add_candidate(
                    &mut candidates,
                    chapter,
                    manual_paragraph.clone(),
                    PilotSampleReason::TranslationQualityStale,
                );
            }

            match literary_review::get_literary_review(layout, chapter.index) {
                Ok(view) => {
                    review_coverage += 1;
                    if view.stale {
                        stale_reviews += 1;
                        issues.push(PilotAuditIssue {
                            code: PilotAuditIssueCode::LiteraryReviewStale,
                            chapter_index: Some(chapter.index),
                            chapter_id: Some(chapter.id.clone()),
                            blocking: false,
                        });
                        add_candidate(
                            &mut candidates,
                            chapter,
                            None,
                            PilotSampleReason::LiteraryReviewStale,
                        );
                    } else if view.artifact.report.requires_attention() {
                        attention_reviews += 1;
                        issues.push(PilotAuditIssue {
                            code: PilotAuditIssueCode::LiteraryReviewAttention,
                            chapter_index: Some(chapter.index),
                            chapter_id: Some(chapter.id.clone()),
                            blocking: false,
                        });
                        add_candidate(
                            &mut candidates,
                            chapter,
                            None,
                            PilotSampleReason::LiteraryReviewAttention,
                        );
                    }
                }
                Err(ApplicationError::NoCheckpoint) => {
                    issues.push(PilotAuditIssue {
                        code: PilotAuditIssueCode::LiteraryReviewMissing,
                        chapter_index: Some(chapter.index),
                        chapter_id: Some(chapter.id.clone()),
                        blocking: false,
                    });
                }
                Err(error) => return Err(error),
            }
        } else {
            issues.push(PilotAuditIssue {
                code: PilotAuditIssueCode::ChapterMissingTranslation,
                chapter_index: Some(chapter.index),
                chapter_id: Some(chapter.id.clone()),
                blocking: true,
            });
        }
    }

    add_position_and_content_samples(
        &manuscript.chapters,
        &sample_eligible_chapters,
        &mut candidates,
    );

    let mut review_targets = candidates.into_values().collect::<Vec<_>>();
    review_targets.sort_by(|a, b| {
        b.priority()
            .cmp(&a.priority())
            .then_with(|| a.chapter_index.cmp(&b.chapter_index))
    });
    review_targets.truncate(max_review_targets.min(DEFAULT_MAX_REVIEW_TARGETS));
    review_targets.sort_by_key(|target| target.chapter_index);
    let review_targets = review_targets
        .into_iter()
        .map(|candidate| PilotReviewTarget {
            chapter_index: candidate.chapter_index,
            chapter_id: candidate.chapter_id,
            paragraph_id: candidate.paragraph_id,
            reasons: candidate.reasons.into_iter().collect(),
        })
        .collect::<Vec<_>>();

    let translation_complete = progress.as_ref().is_some_and(|progress| {
        progress.state == TranslationState::Completed
            && progress.completed_chapters == total_chapters
            && progress.completed_paragraphs == total_source_paragraphs
    });
    let mechanically_export_ready =
        translation_complete && !issues.iter().any(|issue| issue.blocking);
    let human_review_clear = mechanically_export_ready
        && review_coverage == total_chapters
        && stale_reviews == 0
        && attention_reviews == 0
        && !issues.iter().any(|issue| {
            matches!(
                issue.code,
                PilotAuditIssueCode::LiteraryReviewMissing
                    | PilotAuditIssueCode::TranslationQualityStale
            )
        });

    Ok(BookPilotAudit {
        schema_version: PILOT_AUDIT_SCHEMA_VERSION,
        project_id: manifest.project_id,
        source_fingerprint: source_record.fingerprint.clone(),
        translation_plan_fingerprint: current_plan,
        total_chapters,
        translated_chapters,
        total_source_paragraphs,
        completed_source_paragraphs,
        literary_review_coverage_chapters: review_coverage,
        literary_review_stale_chapters: stale_reviews,
        literary_review_attention_chapters: attention_reviews,
        manual_revision_chapters,
        translation_complete,
        mechanically_export_ready,
        human_review_clear,
        issues,
        review_targets,
    })
}

fn add_candidate(
    candidates: &mut BTreeMap<usize, TargetCandidate>,
    chapter: &document_engine::Chapter,
    paragraph_id: Option<String>,
    reason: PilotSampleReason,
) {
    let entry = candidates
        .entry(chapter.index)
        .or_insert_with(|| TargetCandidate {
            chapter_index: chapter.index,
            chapter_id: chapter.id.clone(),
            paragraph_id: paragraph_id.clone().or_else(|| first_paragraph_id(chapter)),
            reasons: BTreeSet::new(),
        });
    if entry.paragraph_id.is_none() {
        entry.paragraph_id = paragraph_id.or_else(|| first_paragraph_id(chapter));
    }
    entry.reasons.insert(reason);
}

fn add_position_and_content_samples(
    chapters: &[document_engine::Chapter],
    eligible_chapters: &BTreeSet<usize>,
    candidates: &mut BTreeMap<usize, TargetCandidate>,
) {
    let eligible = chapters
        .iter()
        .filter(|chapter| eligible_chapters.contains(&chapter.index))
        .collect::<Vec<_>>();
    if eligible.is_empty() {
        return;
    }

    let last = eligible.len() - 1;
    add_candidate(
        candidates,
        eligible[0],
        first_paragraph_id(eligible[0]),
        PilotSampleReason::EarlyBook,
    );
    let middle = eligible.len() / 2;
    add_candidate(
        candidates,
        eligible[middle],
        first_paragraph_id(eligible[middle]),
        PilotSampleReason::MiddleBook,
    );
    add_candidate(
        candidates,
        eligible[last],
        first_paragraph_id(eligible[last]),
        PilotSampleReason::LateBook,
    );

    if let Some(longest) = eligible
        .iter()
        .copied()
        .max_by_key(|chapter| chapter.content.chars().count())
    {
        add_candidate(
            candidates,
            longest,
            longest_paragraph_id(longest),
            PilotSampleReason::LongChapter,
        );
    }

    if let Some((chapter, paragraph_id, score)) = eligible
        .iter()
        .copied()
        .filter_map(|chapter| {
            most_dialogue_heavy_paragraph(chapter)
                .map(|(paragraph_id, score)| (chapter, paragraph_id, score))
        })
        .max_by(|a, b| a.2.cmp(&b.2).then_with(|| b.0.index.cmp(&a.0.index)))
    {
        if score > 0 {
            add_candidate(
                candidates,
                chapter,
                Some(paragraph_id),
                PilotSampleReason::DialogueHeavy,
            );
        }
    }
}

fn first_paragraph_id(chapter: &document_engine::Chapter) -> Option<String> {
    chapter
        .scenes
        .iter()
        .flat_map(|scene| scene.paragraphs.iter())
        .next()
        .map(|paragraph| paragraph.id.clone())
}

fn longest_paragraph_id(chapter: &document_engine::Chapter) -> Option<String> {
    chapter
        .scenes
        .iter()
        .flat_map(|scene| scene.paragraphs.iter())
        .max_by_key(|paragraph| paragraph.original_text.chars().count())
        .map(|paragraph| paragraph.id.clone())
}

fn most_dialogue_heavy_paragraph(chapter: &document_engine::Chapter) -> Option<(String, usize)> {
    chapter
        .scenes
        .iter()
        .flat_map(|scene| scene.paragraphs.iter())
        .map(|paragraph| {
            let score = dialogue_signal_count(&paragraph.original_text);
            (paragraph.id.clone(), score)
        })
        .max_by(|a, b| a.1.cmp(&b.1).then_with(|| b.0.cmp(&a.0)))
}

fn dialogue_signal_count(text: &str) -> usize {
    text.chars()
        .filter(|character| matches!(character, '"' | '“' | '”' | '«' | '»'))
        .count()
        + text
            .lines()
            .filter(|line| {
                let trimmed = line.trim_start();
                trimmed.starts_with('—') || trimmed.starts_with("- ")
            })
            .count()
}
