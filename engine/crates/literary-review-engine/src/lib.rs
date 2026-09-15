use std::collections::BTreeSet;
use std::fmt;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const REVIEW_SCHEMA_VERSION: u32 = 1;
pub const HAZM_PROTOCOL_VERSION: u32 = 1;

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
            .with_revision_proposal("restore the missing translated content before literary review"),
        );
        return report;
    }

    let source_chars = source.chars().filter(|character| !character.is_whitespace()).count();
    let target_chars = target.chars().filter(|character| !character.is_whitespace()).count();
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HazmDiagnosticRequest {
    pub schema_version: u32,
    pub unit_id: String,
    pub text: String,
}

impl HazmDiagnosticRequest {
    pub fn new(unit_id: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            schema_version: HAZM_PROTOCOL_VERSION,
            unit_id: unit_id.into(),
            text: text.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HazmDiagnosticResponse {
    pub schema_version: u32,
    pub unit_id: String,
    pub tool: String,
    pub tool_version: String,
    pub sentence_count: usize,
    pub word_count: usize,
    pub normalized_changed: bool,
    pub normalization_edit_ratio: f32,
    pub arabic_variant_count: usize,
    pub repeated_whitespace_count: usize,
    pub zwnj_count: usize,
}

#[derive(Debug)]
pub enum HazmSidecarError {
    InvalidRequest(String),
    Io(std::io::Error),
    Timeout { timeout_ms: u64 },
    SidecarFailed { status: Option<i32>, stderr: String },
    Protocol(String),
}

impl fmt::Display for HazmSidecarError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRequest(message) => write!(formatter, "invalid Hazm request: {message}"),
            Self::Io(error) => write!(formatter, "Hazm sidecar I/O error: {error}"),
            Self::Timeout { timeout_ms } => {
                write!(formatter, "Hazm sidecar timed out after {timeout_ms} ms")
            }
            Self::SidecarFailed { status, stderr } => write!(
                formatter,
                "Hazm sidecar failed with status {:?}: {}",
                status,
                stderr.trim()
            ),
            Self::Protocol(message) => write!(formatter, "Hazm protocol error: {message}"),
        }
    }
}

impl std::error::Error for HazmSidecarError {}

impl From<std::io::Error> for HazmSidecarError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

#[derive(Debug, Clone)]
pub struct HazmSidecar {
    executable: PathBuf,
    timeout: Duration,
    max_chars: usize,
}

impl HazmSidecar {
    pub fn new(executable: impl Into<PathBuf>) -> Self {
        Self {
            executable: executable.into(),
            timeout: Duration::from_secs(30),
            max_chars: 40_000,
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout.max(Duration::from_millis(1));
        self
    }

    pub fn with_max_chars(mut self, max_chars: usize) -> Self {
        self.max_chars = max_chars.max(1);
        self
    }

    pub fn executable(&self) -> &Path {
        &self.executable
    }

    pub fn diagnose(
        &self,
        request: &HazmDiagnosticRequest,
    ) -> Result<HazmDiagnosticResponse, HazmSidecarError> {
        validate_hazm_request(request, self.max_chars)?;
        let payload = serde_json::to_vec(request)
            .map_err(|error| HazmSidecarError::Protocol(error.to_string()))?;

        let mut child = Command::new(&self.executable)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| HazmSidecarError::Protocol("sidecar stdin unavailable".into()))?;
        stdin.write_all(&payload)?;
        drop(stdin);

        let started = Instant::now();
        let status = loop {
            if let Some(status) = child.try_wait()? {
                break status;
            }
            if started.elapsed() >= self.timeout {
                let _ = child.kill();
                let _ = child.wait();
                return Err(HazmSidecarError::Timeout {
                    timeout_ms: self.timeout.as_millis().min(u128::from(u64::MAX)) as u64,
                });
            }
            thread::sleep(Duration::from_millis(20));
        };

        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        if let Some(mut pipe) = child.stdout.take() {
            pipe.read_to_end(&mut stdout)?;
        }
        if let Some(mut pipe) = child.stderr.take() {
            pipe.read_to_end(&mut stderr)?;
        }
        if !status.success() {
            return Err(HazmSidecarError::SidecarFailed {
                status: status.code(),
                stderr: String::from_utf8_lossy(&stderr).into_owned(),
            });
        }

        let response: HazmDiagnosticResponse = serde_json::from_slice(&stdout)
            .map_err(|error| HazmSidecarError::Protocol(format!("invalid JSON response: {error}")))?;
        validate_hazm_response(request, &response)?;
        Ok(response)
    }
}

fn validate_hazm_request(
    request: &HazmDiagnosticRequest,
    max_chars: usize,
) -> Result<(), HazmSidecarError> {
    if request.schema_version != HAZM_PROTOCOL_VERSION {
        return Err(HazmSidecarError::InvalidRequest(format!(
            "unsupported schema version {}",
            request.schema_version
        )));
    }
    if request.unit_id.trim().is_empty() {
        return Err(HazmSidecarError::InvalidRequest("unit_id is empty".into()));
    }
    if request.text.chars().count() > max_chars {
        return Err(HazmSidecarError::InvalidRequest(format!(
            "text exceeds configured {max_chars}-character limit"
        )));
    }
    Ok(())
}

fn validate_hazm_response(
    request: &HazmDiagnosticRequest,
    response: &HazmDiagnosticResponse,
) -> Result<(), HazmSidecarError> {
    if response.schema_version != HAZM_PROTOCOL_VERSION {
        return Err(HazmSidecarError::Protocol(format!(
            "response schema version {} does not match {}",
            response.schema_version, HAZM_PROTOCOL_VERSION
        )));
    }
    if response.unit_id != request.unit_id {
        return Err(HazmSidecarError::Protocol(
            "response unit_id does not match request".into(),
        ));
    }
    if response.tool.trim().is_empty() || response.tool_version.trim().is_empty() {
        return Err(HazmSidecarError::Protocol(
            "response tool provenance is missing".into(),
        ));
    }
    if !response.normalization_edit_ratio.is_finite()
        || !(0.0..=1.0).contains(&response.normalization_edit_ratio)
    {
        return Err(HazmSidecarError::Protocol(
            "normalization_edit_ratio must be finite and within 0..=1".into(),
        ));
    }
    Ok(())
}

/// Attach only advisory surface-language evidence from Hazm. The function never
/// rewrites target text and never marks a literary dimension approved.
pub fn attach_hazm_diagnostics(
    report: &mut LiteraryReviewReport,
    diagnostics: &HazmDiagnosticResponse,
) {
    report.record_dimension(ReviewDimension::PersianNaturalness);

    if diagnostics.arabic_variant_count > 0 {
        report.findings.push(
            ReviewFinding::new(
                &report.unit_id,
                ReviewDimension::PersianNaturalness,
                ReviewSeverity::Advisory,
                EvidenceSource::Hazm,
                format!(
                    "Hazm observed {} Arabic-form kaf/yeh characters; review typography without changing intentional quoted text",
                    diagnostics.arabic_variant_count
                ),
            )
            .with_confidence(0.9),
        );
    }

    if diagnostics.repeated_whitespace_count > 0 {
        report.findings.push(
            ReviewFinding::new(
                &report.unit_id,
                ReviewDimension::PersianNaturalness,
                ReviewSeverity::Advisory,
                EvidenceSource::Hazm,
                format!(
                    "Hazm observed {} repeated-whitespace spans; verify whether spacing is intentional",
                    diagnostics.repeated_whitespace_count
                ),
            )
            .with_confidence(0.88),
        );
    }

    if diagnostics.normalization_edit_ratio >= 0.08 {
        report.findings.push(
            ReviewFinding::new(
                &report.unit_id,
                ReviewDimension::PersianNaturalness,
                ReviewSeverity::Advisory,
                EvidenceSource::Hazm,
                format!(
                    "Hazm normalization would alter {:.1}% of the character sequence; inspect orthography/spacing while preserving literary voice",
                    diagnostics.normalization_edit_ratio * 100.0
                ),
            )
            .with_confidence(0.7),
        );
    }

    report.advisory_notes.push(format!(
        "Hazm {} diagnostics are surface evidence only; no normalization was applied to the translation.",
        diagnostics.tool_version
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_review_flags_empty_translation_without_claiming_other_dimensions() {
        let report = review_native("chapter-1", "A meaningful source paragraph.", "", Default::default());
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
    fn hazm_evidence_is_advisory_and_does_not_rewrite() {
        let mut report = review_native("u", "source", "فارسي  متن", Default::default());
        let diagnostics = HazmDiagnosticResponse {
            schema_version: HAZM_PROTOCOL_VERSION,
            unit_id: "u".into(),
            tool: "hazm".into(),
            tool_version: "0.12.1".into(),
            sentence_count: 1,
            word_count: 2,
            normalized_changed: true,
            normalization_edit_ratio: 0.12,
            arabic_variant_count: 1,
            repeated_whitespace_count: 1,
            zwnj_count: 0,
        };
        attach_hazm_diagnostics(&mut report, &diagnostics);
        assert!(report.dimension_was_evaluated(ReviewDimension::PersianNaturalness));
        assert!(report
            .findings
            .iter()
            .filter(|finding| finding.evidence_source == EvidenceSource::Hazm)
            .all(|finding| finding.severity == ReviewSeverity::Advisory));
    }

    #[test]
    fn hazm_request_rejects_unbounded_text() {
        let sidecar = HazmSidecar::new("missing").with_max_chars(3);
        let request = HazmDiagnosticRequest::new("u", "چهار");
        assert!(matches!(
            sidecar.diagnose(&request),
            Err(HazmSidecarError::InvalidRequest(_))
        ));
    }

    #[test]
    fn review_report_round_trips_json() {
        let report = review_native("u", "hello", "سلام", Default::default());
        let json = serde_json::to_string(&report).unwrap();
        let decoded: LiteraryReviewReport = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, report);
    }
}
