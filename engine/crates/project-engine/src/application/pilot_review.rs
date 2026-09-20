//! Phase 29 — append-only human pilot review ledger.
//!
//! Phase 28 decides which current translation locations deserve focused human
//! inspection. Phase 29 records the human decision without copying source or
//! translated prose into the ledger. Records are fingerprint-bound and remain
//! as audit history when later edits make them stale.

use super::error::ApplicationError;
use super::pilot_audit::{build_pilot_audit, BookPilotAudit, PilotReviewTarget};
use super::project::{atomic_write_json, load_manifest, ProjectLayout};
use super::translation;
use chrono::{DateTime, Utc};
use literary_review_engine::{ReviewDimension, ReviewSeverity};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fs;

pub const PILOT_REVIEW_LEDGER_SCHEMA_VERSION: u32 = 1;
pub const PILOT_REVIEW_SUMMARY_SCHEMA_VERSION: u32 = 1;
const MAX_PILOT_REVIEW_RECORDS: usize = 5_000;
const MAX_FINDINGS_PER_RECORD: usize = 16;
const MAX_REVIEWER_CHARS: usize = 200;
const MAX_NOTE_CHARS: usize = 4_000;
const MAX_FINDING_NOTE_CHARS: usize = 2_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PilotReviewOutcome {
    Clear,
    AcceptedAsIs,
    NeedsRevision,
}

impl PilotReviewOutcome {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Clear => "clear",
            Self::AcceptedAsIs => "accepted_as_is",
            Self::NeedsRevision => "needs_revision",
        }
    }

    fn resolves_target(self) -> bool {
        matches!(self, Self::Clear | Self::AcceptedAsIs)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewCharSpan {
    /// Unicode scalar-value offset, not a byte offset.
    pub start_char: usize,
    /// Exclusive Unicode scalar-value offset.
    pub end_char: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HumanPilotFinding {
    pub dimension: ReviewDimension,
    pub severity: ReviewSeverity,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_span: Option<ReviewCharSpan>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_span: Option<ReviewCharSpan>,
    /// Reviewer-authored local note. The engine never copies manuscript or
    /// translation prose into this field automatically.
    #[serde(default)]
    pub note: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PilotReviewSubmission {
    pub reviewer: String,
    pub outcome: PilotReviewOutcome,
    #[serde(default)]
    pub findings: Vec<HumanPilotFinding>,
    #[serde(default)]
    pub note: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PilotReviewRecord {
    pub record_id: String,
    pub target_id: String,
    pub chapter_index: usize,
    pub chapter_id: String,
    pub paragraph_id: Option<String>,
    pub source_fingerprint: String,
    pub translation_fingerprint: String,
    pub translation_context_fingerprint: String,
    pub translation_plan_fingerprint: String,
    pub reviewer: String,
    pub outcome: PilotReviewOutcome,
    pub findings: Vec<HumanPilotFinding>,
    pub note: String,
    pub reviewed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct PilotReviewLedger {
    schema_version: u32,
    project_id: String,
    records: Vec<PilotReviewRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PilotReviewTargetState {
    pub target: PilotReviewTarget,
    pub target_id: String,
    pub source_fingerprint: String,
    pub translation_fingerprint: String,
    pub translation_context_fingerprint: String,
    pub translation_plan_fingerprint: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_record: Option<PilotReviewRecord>,
    pub stale_record_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PilotReviewSummary {
    pub schema_version: u32,
    pub audit: BookPilotAudit,
    pub targets: Vec<PilotReviewTargetState>,
    pub reviewed_current: usize,
    pub resolved_current: usize,
    pub needs_revision_current: usize,
    pub missing_current: usize,
    pub stale_records: usize,
    pub orphaned_records: usize,
    /// True only when Phase-28 mechanical artifacts are current and every
    /// selected target has a current human Clear/AcceptedAsIs record.
    ///
    /// This is a sampled-review workflow state, never a book-quality score.
    pub sample_review_complete: bool,
}

#[derive(Debug)]
struct TargetSnapshot {
    target: PilotReviewTarget,
    target_id: String,
    source_fingerprint: String,
    translation_fingerprint: String,
    translation_context_fingerprint: String,
    translation_plan_fingerprint: String,
    source_chars: usize,
    translation_chars: usize,
}

pub fn review_summary(
    layout: &ProjectLayout,
    max_review_targets: usize,
) -> Result<PilotReviewSummary, ApplicationError> {
    let manifest = load_manifest(layout)?;
    let audit = build_pilot_audit(layout, max_review_targets)?;
    let ledger = load_ledger(layout, &manifest.project_id)?;

    let mut states = Vec::with_capacity(audit.review_targets.len());
    let mut current_target_ids = BTreeSet::new();
    let mut reviewed_current = 0usize;
    let mut resolved_current = 0usize;
    let mut needs_revision_current = 0usize;
    let mut missing_current = 0usize;
    let mut stale_records = 0usize;

    for target in &audit.review_targets {
        let snapshot = resolve_target(layout, &manifest.project_id, target)?;
        current_target_ids.insert(snapshot.target_id.clone());

        let mut latest_current = None;
        let mut stale_for_target = 0usize;
        for record in ledger
            .records
            .iter()
            .filter(|record| record.target_id == snapshot.target_id)
        {
            if record.source_fingerprint == snapshot.source_fingerprint
                && record.translation_fingerprint == snapshot.translation_fingerprint
                && record.translation_context_fingerprint
                    == snapshot.translation_context_fingerprint
                && record.translation_plan_fingerprint == snapshot.translation_plan_fingerprint
            {
                latest_current = Some(record.clone());
            } else {
                stale_for_target += 1;
            }
        }
        stale_records += stale_for_target;

        if let Some(record) = latest_current.as_ref() {
            reviewed_current += 1;
            if record.outcome.resolves_target() {
                resolved_current += 1;
            } else {
                needs_revision_current += 1;
            }
        } else {
            missing_current += 1;
        }

        states.push(PilotReviewTargetState {
            target: snapshot.target,
            target_id: snapshot.target_id,
            source_fingerprint: snapshot.source_fingerprint,
            translation_fingerprint: snapshot.translation_fingerprint,
            translation_context_fingerprint: snapshot.translation_context_fingerprint,
            translation_plan_fingerprint: snapshot.translation_plan_fingerprint,
            current_record: latest_current,
            stale_record_count: stale_for_target,
        });
    }

    let orphaned_records = ledger
        .records
        .iter()
        .filter(|record| !current_target_ids.contains(&record.target_id))
        .count();

    let sample_review_complete = !states.is_empty()
        && audit.mechanically_export_ready
        && missing_current == 0
        && needs_revision_current == 0
        && resolved_current == states.len();

    Ok(PilotReviewSummary {
        schema_version: PILOT_REVIEW_SUMMARY_SCHEMA_VERSION,
        audit,
        targets: states,
        reviewed_current,
        resolved_current,
        needs_revision_current,
        missing_current,
        stale_records,
        orphaned_records,
        sample_review_complete,
    })
}

pub fn record_review(
    layout: &ProjectLayout,
    max_review_targets: usize,
    target_id: &str,
    submission: PilotReviewSubmission,
    reviewed_at: DateTime<Utc>,
) -> Result<PilotReviewRecord, ApplicationError> {
    let manifest = load_manifest(layout)?;
    let audit = build_pilot_audit(layout, max_review_targets)?;
    let snapshot = audit
        .review_targets
        .iter()
        .map(|target| resolve_target(layout, &manifest.project_id, target))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .find(|snapshot| snapshot.target_id == target_id)
        .ok_or_else(|| {
            ApplicationError::PilotReviewUnavailable(
                "target is not part of the current Phase-28 review sample".to_string(),
            )
        })?;

    validate_submission(
        &submission,
        snapshot.source_chars,
        snapshot.translation_chars,
    )?;

    let mut ledger = load_ledger(layout, &manifest.project_id)?;
    if ledger.records.len() >= MAX_PILOT_REVIEW_RECORDS {
        return Err(ApplicationError::PilotReviewUnavailable(format!(
            "pilot review ledger reached its bounded capacity of {MAX_PILOT_REVIEW_RECORDS} records"
        )));
    }

    let record_id = record_id(
        &snapshot.target_id,
        &snapshot.source_fingerprint,
        &snapshot.translation_fingerprint,
        &snapshot.translation_context_fingerprint,
        submission.outcome,
        &submission.reviewer,
        reviewed_at,
        ledger.records.len(),
    );
    let record = PilotReviewRecord {
        record_id,
        target_id: snapshot.target_id,
        chapter_index: snapshot.target.chapter_index,
        chapter_id: snapshot.target.chapter_id,
        paragraph_id: snapshot.target.paragraph_id,
        source_fingerprint: snapshot.source_fingerprint,
        translation_fingerprint: snapshot.translation_fingerprint,
        translation_context_fingerprint: snapshot.translation_context_fingerprint,
        translation_plan_fingerprint: snapshot.translation_plan_fingerprint,
        reviewer: submission.reviewer.trim().to_string(),
        outcome: submission.outcome,
        findings: submission.findings,
        note: submission.note.trim().to_string(),
        reviewed_at,
    };
    ledger.records.push(record.clone());
    atomic_write_json(&layout.pilot_review_file, &ledger)?;
    Ok(record)
}

fn resolve_target(
    layout: &ProjectLayout,
    project_id: &str,
    target: &PilotReviewTarget,
) -> Result<TargetSnapshot, ApplicationError> {
    let translated = translation::get_translated_chapter(layout, target.chapter_index)?;
    if translated.chapter_id != target.chapter_id {
        return Err(ApplicationError::PilotReviewUnavailable(format!(
            "review target chapter {} is stale",
            target.chapter_index
        )));
    }
    if translated.translation_plan_fingerprint.trim().is_empty() {
        return Err(ApplicationError::PilotReviewUnavailable(
            "review target has no current translation-plan identity".to_string(),
        ));
    }

    let (source, translation) = if let Some(paragraph_id) = target.paragraph_id.as_deref() {
        let paragraph = translated
            .paragraphs
            .iter()
            .find(|paragraph| paragraph.paragraph_id == paragraph_id)
            .ok_or_else(|| {
                ApplicationError::PilotReviewUnavailable(format!(
                    "paragraph target '{paragraph_id}' is stale or missing"
                ))
            })?;
        (paragraph.source.clone(), paragraph.translated.clone())
    } else {
        (
            translated
                .paragraphs
                .iter()
                .map(|paragraph| paragraph.source.as_str())
                .collect::<Vec<_>>()
                .join("\n\n"),
            translated
                .paragraphs
                .iter()
                .map(|paragraph| paragraph.translated.as_str())
                .collect::<Vec<_>>()
                .join("\n\n"),
        )
    };

    let source_fingerprint = fingerprint_segments(
        translated
            .paragraphs
            .iter()
            .map(|paragraph| paragraph.source.as_str()),
    );
    let translation_fingerprint = fingerprint_segments(
        translated
            .paragraphs
            .iter()
            .map(|paragraph| paragraph.translated.as_str()),
    );

    Ok(TargetSnapshot {
        target: target.clone(),
        target_id: stable_target_id(
            project_id,
            &target.chapter_id,
            target.paragraph_id.as_deref(),
        ),
        source_fingerprint,
        translation_fingerprint,
        translation_context_fingerprint: translated.context_fingerprint,
        translation_plan_fingerprint: translated.translation_plan_fingerprint,
        source_chars: source.chars().count(),
        translation_chars: translation.chars().count(),
    })
}

fn validate_submission(
    submission: &PilotReviewSubmission,
    source_chars: usize,
    translation_chars: usize,
) -> Result<(), ApplicationError> {
    let reviewer = submission.reviewer.trim();
    if reviewer.is_empty() || reviewer.chars().count() > MAX_REVIEWER_CHARS {
        return Err(ApplicationError::PilotReviewUnavailable(format!(
            "reviewer must contain 1..={MAX_REVIEWER_CHARS} characters"
        )));
    }
    if submission.note.chars().count() > MAX_NOTE_CHARS {
        return Err(ApplicationError::PilotReviewUnavailable(format!(
            "review note exceeds {MAX_NOTE_CHARS} characters"
        )));
    }
    if submission.findings.len() > MAX_FINDINGS_PER_RECORD {
        return Err(ApplicationError::PilotReviewUnavailable(format!(
            "a pilot review record may contain at most {MAX_FINDINGS_PER_RECORD} findings"
        )));
    }

    match submission.outcome {
        PilotReviewOutcome::Clear if !submission.findings.is_empty() => {
            return Err(ApplicationError::PilotReviewUnavailable(
                "a clear target cannot carry unresolved findings".to_string(),
            ));
        }
        PilotReviewOutcome::AcceptedAsIs
            if submission.findings.is_empty() && submission.note.trim().is_empty() =>
        {
            return Err(ApplicationError::PilotReviewUnavailable(
                "accepted_as_is requires a reviewer note or explicit finding".to_string(),
            ));
        }
        PilotReviewOutcome::AcceptedAsIs
            if submission
                .findings
                .iter()
                .any(|finding| finding.severity == ReviewSeverity::Critical) =>
        {
            return Err(ApplicationError::PilotReviewUnavailable(
                "accepted_as_is cannot retain a critical human finding".to_string(),
            ));
        }
        PilotReviewOutcome::NeedsRevision
            if !submission.findings.iter().any(|finding| {
                matches!(
                    finding.severity,
                    ReviewSeverity::Warning | ReviewSeverity::Critical
                )
            }) =>
        {
            return Err(ApplicationError::PilotReviewUnavailable(
                "needs_revision requires at least one warning or critical finding".to_string(),
            ));
        }
        _ => {}
    }

    for finding in &submission.findings {
        if finding.note.chars().count() > MAX_FINDING_NOTE_CHARS {
            return Err(ApplicationError::PilotReviewUnavailable(format!(
                "finding note exceeds {MAX_FINDING_NOTE_CHARS} characters"
            )));
        }
        validate_span("source", finding.source_span, source_chars)?;
        validate_span("translation", finding.target_span, translation_chars)?;
        if finding.source_span.is_none()
            && finding.target_span.is_none()
            && finding.note.trim().is_empty()
        {
            return Err(ApplicationError::PilotReviewUnavailable(
                "a finding needs a source/translation span or reviewer note".to_string(),
            ));
        }
    }

    Ok(())
}

fn validate_span(
    label: &str,
    span: Option<ReviewCharSpan>,
    text_chars: usize,
) -> Result<(), ApplicationError> {
    if let Some(span) = span {
        if span.start_char >= span.end_char || span.end_char > text_chars {
            return Err(ApplicationError::PilotReviewUnavailable(format!(
                "{label} span {}..{} is outside the current {text_chars}-character target",
                span.start_char, span.end_char
            )));
        }
    }
    Ok(())
}

fn load_ledger(
    layout: &ProjectLayout,
    project_id: &str,
) -> Result<PilotReviewLedger, ApplicationError> {
    if !layout.pilot_review_file.exists() {
        return Ok(PilotReviewLedger {
            schema_version: PILOT_REVIEW_LEDGER_SCHEMA_VERSION,
            project_id: project_id.to_string(),
            records: Vec::new(),
        });
    }
    let bytes = fs::read(&layout.pilot_review_file).map_err(|error| {
        ApplicationError::PersistenceFailure(format!(
            "failed to read pilot review ledger {}: {error}",
            layout.pilot_review_file.display()
        ))
    })?;
    let ledger: PilotReviewLedger = serde_json::from_slice(&bytes).map_err(|error| {
        ApplicationError::PersistenceFailure(format!(
            "invalid pilot review ledger {}: {error}",
            layout.pilot_review_file.display()
        ))
    })?;
    if ledger.schema_version != PILOT_REVIEW_LEDGER_SCHEMA_VERSION {
        return Err(ApplicationError::PersistenceFailure(format!(
            "unsupported pilot review ledger schema {}",
            ledger.schema_version
        )));
    }
    if ledger.project_id != project_id {
        return Err(ApplicationError::InvalidProject(
            "pilot review ledger belongs to a different project".to_string(),
        ));
    }
    if ledger.records.len() > MAX_PILOT_REVIEW_RECORDS {
        return Err(ApplicationError::PersistenceFailure(format!(
            "pilot review ledger exceeds bounded capacity of {MAX_PILOT_REVIEW_RECORDS}"
        )));
    }
    Ok(ledger)
}

fn fingerprint_segments<'a>(segments: impl IntoIterator<Item = &'a str>) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"phase29-chapter-fingerprint-v1\0");
    for segment in segments {
        let bytes = segment.as_bytes();
        hasher.update((bytes.len() as u64).to_le_bytes());
        hasher.update(bytes);
    }
    format!("sha256-{:x}", hasher.finalize())
}

fn stable_target_id(project_id: &str, chapter_id: &str, paragraph_id: Option<&str>) -> String {
    let identity = format!(
        "phase29-target-v1\0{project_id}\0{chapter_id}\0{}",
        paragraph_id.unwrap_or("chapter")
    );
    format!("pilot-target-{:x}", Sha256::digest(identity.as_bytes()))
}

fn record_id(
    target_id: &str,
    source_fingerprint: &str,
    translation_fingerprint: &str,
    translation_context_fingerprint: &str,
    outcome: PilotReviewOutcome,
    reviewer: &str,
    reviewed_at: DateTime<Utc>,
    sequence: usize,
) -> String {
    let identity = format!(
        "phase29-record-v1\0{target_id}\0{source_fingerprint}\0{translation_fingerprint}\0{translation_context_fingerprint}\0{}\0{}\0{}\0{sequence}",
        outcome.as_str(),
        reviewer.trim(),
        reviewed_at.to_rfc3339_opts(chrono::SecondsFormat::Nanos, true),
    );
    format!("pilot-review-{:x}", Sha256::digest(identity.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_unicode_char_spans_not_byte_offsets() {
        let submission = PilotReviewSubmission {
            reviewer: "editor".into(),
            outcome: PilotReviewOutcome::NeedsRevision,
            findings: vec![HumanPilotFinding {
                dimension: ReviewDimension::PersianNaturalness,
                severity: ReviewSeverity::Warning,
                source_span: Some(ReviewCharSpan {
                    start_char: 0,
                    end_char: 3,
                }),
                target_span: Some(ReviewCharSpan {
                    start_char: 1,
                    end_char: 4,
                }),
                note: "wording".into(),
            }],
            note: String::new(),
        };
        assert!(validate_submission(&submission, 3, 4).is_ok());
    }

    #[test]
    fn clear_outcome_rejects_unresolved_findings() {
        let submission = PilotReviewSubmission {
            reviewer: "editor".into(),
            outcome: PilotReviewOutcome::Clear,
            findings: vec![HumanPilotFinding {
                dimension: ReviewDimension::SemanticFidelity,
                severity: ReviewSeverity::Warning,
                source_span: None,
                target_span: None,
                note: "issue".into(),
            }],
            note: String::new(),
        };
        assert!(validate_submission(&submission, 10, 10).is_err());
    }
}
