use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub mod alignment;
pub mod provider;
pub use alignment::{
    align_embeddings, attach_alignment_evidence, AlignmentBlock, AlignmentConfig, AlignmentError,
    AlignmentInput, AlignmentKind, AlignmentResult, EmbeddedSpan, ALIGNMENT_SCHEMA_VERSION,
};
pub use provider::{
    attach_provider_review, LiteraryReviewProvider, MockReviewProvider, OpenAIReviewProvider,
    ProviderFinding, ProviderReviewResponse, ReviewProviderError, ReviewProviderRequest,
    ReviewProviderResult, ReviewRequestLimits, REVIEW_PROMPT_VERSION,
};

pub const REVIEW_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewDimension {
    OmissionAddition,
    SemanticFidelity,
    CharacterVoice,
    RelationshipRegister,
    PersianNaturalness,
    DialogueSubtext,
    TerminologyContinuity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewSeverity {
    Advisory,
    Warning,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceSource {
    Native,
    SemanticAlignment,
    Hazm,
    Vecalign,
    Lingua,
    Comet,
    Provider,
    Human,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RevisionProposal {
    pub rationale: String,
    pub suggested_text: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReviewFinding {
    pub id: String,
    pub dimension: ReviewDimension,
    pub severity: ReviewSeverity,
    pub evidence_source: EvidenceSource,
    pub summary: String,
    pub source_indices: Vec<usize>,
    pub target_indices: Vec<usize>,
    pub confidence: Option<f32>,
    pub revision_proposal: Option<RevisionProposal>,
}

impl ReviewFinding {
    pub fn new(
        unit_id: &str,
        dimension: ReviewDimension,
        severity: ReviewSeverity,
        evidence_source: EvidenceSource,
        summary: impl Into<String>,
    ) -> Self {
        let summary = summary.into();
        let id = stable_finding_id(unit_id, dimension, evidence_source, &summary);
        Self {
            id,
            dimension,
            severity,
            evidence_source,
            summary,
            source_indices: Vec::new(),
            target_indices: Vec::new(),
            confidence: None,
            revision_proposal: None,
        }
    }

    pub fn with_confidence(mut self, confidence: f32) -> Self {
        if confidence.is_finite() {
            self.confidence = Some(confidence.clamp(0.0, 1.0));
        }
        self
    }

    pub fn with_indices(
        mut self,
        source_indices: impl IntoIterator<Item = usize>,
        target_indices: impl IntoIterator<Item = usize>,
    ) -> Self {
        self.source_indices = source_indices.into_iter().collect();
        self.target_indices = target_indices.into_iter().collect();
        self
    }

    pub fn with_revision_proposal(mut self, rationale: impl Into<String>) -> Self {
        self.revision_proposal = Some(RevisionProposal {
            rationale: rationale.into(),
            suggested_text: None,
        });
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LiteraryReviewReport {
    pub schema_version: u32,
    pub unit_id: String,
    pub source_fingerprint: String,
    pub target_fingerprint: String,
    pub evaluated_dimensions: BTreeSet<ReviewDimension>,
    pub findings: Vec<ReviewFinding>,
    pub advisory_notes: Vec<String>,
}

impl LiteraryReviewReport {
    pub fn requires_attention(&self) -> bool {
        self.findings.iter().any(|finding| {
            matches!(
                finding.severity,
                ReviewSeverity::Warning | ReviewSeverity::Critical
            )
        })
    }

    pub fn dimension_was_evaluated(&self, dimension: ReviewDimension) -> bool {
        self.evaluated_dimensions.contains(&dimension)
    }

    pub fn record_dimension(&mut self, dimension: ReviewDimension) {
        self.evaluated_dimensions.insert(dimension);
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NativeReviewConfig {
    pub min_length_ratio: f32,
    pub max_length_ratio: f32,
    pub paragraph_collapse_ratio: f32,
    pub minimum_source_chars_for_ratio: usize,
}

impl Default for NativeReviewConfig {
    fn default() -> Self {
        Self {
            min_length_ratio: 0.25,
            max_length_ratio: 3.2,
            paragraph_collapse_ratio: 0.5,
            minimum_source_chars_for_ratio: 80,
        }
    }
}

/// Deterministic structural-fidelity evidence only.
///
/// This function deliberately does not claim to evaluate semantic fidelity,
/// voice, register, naturalness, or subtext. Those dimensions stay unevaluated
/// until an appropriate reviewer produces evidence for them.
pub fn review_native(
    unit_id: impl Into<String>,
    source: &str,
    target: &str,
    config: NativeReviewConfig,
) -> LiteraryReviewReport {
    let unit_id = unit_id.into();
    let mut report = LiteraryReviewReport {
        schema_version: REVIEW_SCHEMA_VERSION,
        source_fingerprint: sha256_hex(source.as_bytes()),
        target_fingerprint: sha256_hex(target.as_bytes()),
        unit_id: unit_id.clone(),
        evaluated_dimensions: BTreeSet::from([ReviewDimension::OmissionAddition]),
        findings: Vec::new(),
        advisory_notes: vec![
            "Native review evaluates structural omission/addition signals only; other literary dimensions remain explicitly unevaluated.".into(),
        ],
    };

    if !source.trim().is_empty() && target.trim().is_empty() {
        report.findings.push(
            ReviewFinding::new(
                &unit_id,
                ReviewDimension::OmissionAddition,
                ReviewSeverity::Critical,
                EvidenceSource::Native,
                "translation output is empty while the source unit contains text",
            )
            .with_confidence(1.0)
            .with_revision_proposal(
                "restore the missing translated content before literary review",
            ),
        );
        return report;
    }

    let source_chars = source
        .chars()
        .filter(|character| !character.is_whitespace())
        .count();
    let target_chars = target
        .chars()
        .filter(|character| !character.is_whitespace())
        .count();
    if source_chars >= config.minimum_source_chars_for_ratio && target_chars > 0 {
        let ratio = target_chars as f32 / source_chars as f32;
        if ratio < config.min_length_ratio {
            report.findings.push(
                ReviewFinding::new(
                    &unit_id,
                    ReviewDimension::OmissionAddition,
                    ReviewSeverity::Warning,
                    EvidenceSource::Native,
                    format!(
                        "target/source non-whitespace character ratio is unusually low ({ratio:.3}); possible omission requires review"
                    ),
                )
                .with_confidence(0.72),
            );
        } else if ratio > config.max_length_ratio {
            report.findings.push(
                ReviewFinding::new(
                    &unit_id,
                    ReviewDimension::OmissionAddition,
                    ReviewSeverity::Warning,
                    EvidenceSource::Native,
                    format!(
                        "target/source non-whitespace character ratio is unusually high ({ratio:.3}); possible addition or expansion requires review"
                    ),
                )
                .with_confidence(0.68),
            );
        }
    }

    let source_paragraphs = paragraphs(source);
    let target_paragraphs = paragraphs(target);
    if source_paragraphs.len() >= 3 {
        let ratio = target_paragraphs.len() as f32 / source_paragraphs.len() as f32;
        if ratio < config.paragraph_collapse_ratio {
            report.findings.push(
                ReviewFinding::new(
                    &unit_id,
                    ReviewDimension::OmissionAddition,
                    ReviewSeverity::Warning,
                    EvidenceSource::Native,
                    format!(
                        "paragraph structure collapsed from {} source paragraphs to {} target paragraphs; verify that content was not merged or omitted",
                        source_paragraphs.len(),
                        target_paragraphs.len()
                    ),
                )
                .with_confidence(0.82),
            );
        }
    }

    report
}

fn paragraphs(text: &str) -> Vec<&str> {
    text.split("\n\n")
        .map(str::trim)
        .filter(|paragraph| !paragraph.is_empty())
        .collect()
}

fn stable_finding_id(
    unit_id: &str,
    dimension: ReviewDimension,
    source: EvidenceSource,
    summary: &str,
) -> String {
    sha256_hex(
        format!(
            "literary-review-v1\u{1f}|{unit_id}\u{1f}|{dimension:?}\u{1f}|{source:?}\u{1f}|{summary}"
        )
        .as_bytes(),
    )
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_review_flags_empty_translation_without_claiming_other_dimensions() {
        let report = review_native(
            "chapter-1",
            "A meaningful source paragraph.",
            "",
            Default::default(),
        );
        assert!(report.requires_attention());
        assert!(report.dimension_was_evaluated(ReviewDimension::OmissionAddition));
        assert!(!report.dimension_was_evaluated(ReviewDimension::PersianNaturalness));
        assert_eq!(report.findings[0].severity, ReviewSeverity::Critical);
    }

    #[test]
    fn stable_finding_ids_are_deterministic() {
        let first = ReviewFinding::new(
            "unit",
            ReviewDimension::OmissionAddition,
            ReviewSeverity::Warning,
            EvidenceSource::Native,
            "possible omission",
        );
        let second = ReviewFinding::new(
            "unit",
            ReviewDimension::OmissionAddition,
            ReviewSeverity::Warning,
            EvidenceSource::Native,
            "possible omission",
        );
        assert_eq!(first.id, second.id);
    }

    #[test]
    fn paragraph_collapse_is_evidence_not_auto_rejection() {
        let source = "one\n\ntwo\n\nthree\n\nfour";
        let target = "یک پاراگراف واحد";
        let report = review_native("unit", source, target, Default::default());
        assert!(report
            .findings
            .iter()
            .any(|finding| finding.summary.contains("paragraph structure collapsed")));
    }

    #[test]
    fn review_report_round_trips_json() {
        let report = review_native("u", "hello", "سلام", Default::default());
        let json = serde_json::to_string(&report).unwrap();
        let decoded: LiteraryReviewReport = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, report);
    }
}
