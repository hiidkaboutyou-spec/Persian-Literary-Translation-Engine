//! Deterministic validation gate between provider output and domain state.
//!
//! No provider response reaches the review ledger unchecked. Findings with
//! hallucinated evidence ordinals, unknown categories, invalid confidence, or
//! unbounded fields are rejected with reasons; nothing is silently repaired.

use crate::models::{
    AdvancedLiteraryFinding, AnalysisProviderResponse, AnalysisUnit, ConfidenceLevel,
    DerivedConfidence, FindingCategory, FindingResponse, NarrativeScope, ProviderMetadata,
    ValidatedEvidenceRef,
};
use std::collections::BTreeSet;

pub const MAX_SUBJECT_CHARS: usize = 160;
pub const MAX_CLAIM_CHARS: usize = 700;
pub const MAX_UNCERTAINTY_CHARS: usize = 400;
pub const MAX_ALTERNATIVES: usize = 3;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RejectedFinding {
    pub category: String,
    pub reason: String,
}

#[derive(Debug, Clone, Default)]
pub struct ValidationOutcome {
    pub findings: Vec<AdvancedLiteraryFinding>,
    pub rejected: Vec<RejectedFinding>,
}

pub fn validate_unit_response(
    unit: &AnalysisUnit,
    provider_metadata: &ProviderMetadata,
    response: &AnalysisProviderResponse,
) -> ValidationOutcome {
    let mut outcome = ValidationOutcome::default();
    let mut seen_ids = BTreeSet::new();
    let scope = unit.scene_scope();
    for raw in &response.findings {
        match validate_finding(unit, provider_metadata, raw, &scope) {
            Ok(finding) => {
                if seen_ids.insert(finding.finding_id.clone()) {
                    outcome.findings.push(finding);
                }
            }
            Err(reason) => outcome.rejected.push(RejectedFinding {
                category: raw.category.clone(),
                reason,
            }),
        }
    }
    outcome
}

fn validate_finding(
    unit: &AnalysisUnit,
    provider_metadata: &ProviderMetadata,
    raw: &FindingResponse,
    scope: &NarrativeScope,
) -> Result<AdvancedLiteraryFinding, String> {
    let category = FindingCategory::from_label(raw.category.trim())
        .ok_or_else(|| format!("unknown finding category '{}'", raw.category.trim()))?;

    let subject = raw.subject.trim();
    if subject.is_empty() {
        return Err("empty finding subject".to_string());
    }
    if subject.chars().count() > MAX_SUBJECT_CHARS {
        return Err(format!(
            "finding subject exceeds {MAX_SUBJECT_CHARS} characters"
        ));
    }

    let claim = raw.claim.trim();
    if claim.is_empty() {
        return Err("empty finding claim".to_string());
    }
    if claim.chars().count() > MAX_CLAIM_CHARS {
        return Err(format!(
            "finding claim exceeds {MAX_CLAIM_CHARS} characters"
        ));
    }

    if !raw.confidence.is_finite() || !(0.0..=1.0).contains(&raw.confidence) {
        return Err(format!("invalid confidence value {}", raw.confidence));
    }

    // Evidence ordinals must be supplied tokens that resolve to real document
    // paragraph identifiers. Any fabricated reference rejects the finding.
    let mut cited_paragraph_ids = BTreeSet::new();
    for ordinal in &raw.evidence_ordinals {
        let token = crate::models::normalize_ordinal(ordinal)
            .ok_or_else(|| format!("malformed evidence ordinal '{ordinal}'"))?;
        if !unit.supplied_ordinals().contains(&token) {
            return Err(format!("evidence ordinal '{token}' was not supplied"));
        }
        let paragraph_id = unit
            .resolve_ordinal(&token)
            .ok_or_else(|| format!("evidence ordinal '{token}' does not resolve"))?
            .to_string();
        cited_paragraph_ids.insert((token, paragraph_id));
    }

    let uncertainty = raw
        .uncertainty
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| {
            value
                .chars()
                .take(MAX_UNCERTAINTY_CHARS)
                .collect::<String>()
        });

    let mut alternatives: Vec<String> = raw
        .alternative_interpretations
        .iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .take(MAX_ALTERNATIVES)
        .collect();
    alternatives.dedup();

    let confidence = derive_confidence(raw.confidence, cited_paragraph_ids.len(), unit);
    confidence.validate()?;

    let evidence: Vec<ValidatedEvidenceRef> = {
        let mut refs = Vec::with_capacity(cited_paragraph_ids.len());
        for (token, paragraph_id) in &cited_paragraph_ids {
            let position = token
                .strip_prefix("para-")
                .and_then(|value| value.parse::<usize>().ok())
                .unwrap_or_default();
            let mut source = unit.source.clone();
            source.chapter.get_or_insert(unit.chapter_index + 1);
            if let Some(scene_id) = &unit.scene_id {
                let _ = scene_id;
            }
            source.paragraph.get_or_insert(position);
            refs.push(ValidatedEvidenceRef {
                chapter_id: unit.chapter_id.clone(),
                scene_id: unit
                    .scene_id
                    .clone()
                    .unwrap_or_else(|| unit.chapter_id.clone()),
                paragraph_id: paragraph_id.clone(),
                source,
            });
        }
        refs.sort_by(|left, right| left.paragraph_id.cmp(&right.paragraph_id));
        refs
    };

    let finding_id =
        AdvancedLiteraryFinding::stable_id(&unit.unit_id, category, subject, claim, scope);

    let is_review_eligible = !evidence.is_empty();

    Ok(AdvancedLiteraryFinding {
        finding_id,
        analysis_unit_id: unit.unit_id.clone(),
        category,
        subject: subject.to_string(),
        claim: claim.to_string(),
        confidence,
        evidence,
        scope: scope.clone(),
        provider_metadata: provider_metadata.clone(),
        uncertainty,
        alternative_interpretations: alternatives,
        is_review_eligible,
    })
}

/// Derive final confidence from the model report plus deterministic evidence
/// coverage. Model self-reports are never trusted as final confidence.
fn derive_confidence(
    model_reported: f32,
    cited_paragraphs: usize,
    unit: &AnalysisUnit,
) -> DerivedConfidence {
    let coverage = if unit.paragraph_ids.is_empty() {
        0.0
    } else {
        (cited_paragraphs as f32) / (unit.paragraph_ids.len() as f32).min(20.0)
    };
    let level = match (model_reported, cited_paragraphs) {
        (confidence, _) if confidence >= 0.8 && cited_paragraphs >= 2 => ConfidenceLevel::High,
        (confidence, _) if confidence >= 0.5 && cited_paragraphs >= 1 => ConfidenceLevel::Moderate,
        _ => ConfidenceLevel::Low,
    };
    DerivedConfidence {
        level,
        model_reported_confidence: Some(model_reported),
        evidence_coverage: coverage.min(1.0),
        supporting_unit_count: 1,
        contradicting_unit_count: 0,
        has_deterministic_signal: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AnalysisUnit, AnalysisUnitType, PROMPT_VERSION};
    use document_engine::{DocumentFormat, SourceLocation};

    fn unit() -> AnalysisUnit {
        AnalysisUnit {
            unit_id: "unit-1".to_string(),
            unit_type: AnalysisUnitType::Scene,
            chapter_id: "chapter-1".to_string(),
            chapter_index: 0,
            scene_id: Some("scene-1".to_string()),
            text_content: "[para-1] one\n\n[para-2] two\n\n[para-3] three".to_string(),
            text_fingerprint: "fp".to_string(),
            paragraph_ids: vec![
                "paragraph-1".to_string(),
                "paragraph-2".to_string(),
                "paragraph-3".to_string(),
            ],
            previous_scene_context: None,
            next_scene_context: None,
            known_character_names: Vec::new(),
            known_glossary_terms: Vec::new(),
            source: SourceLocation::new("synthetic.txt", DocumentFormat::Txt),
        }
    }

    fn metadata() -> ProviderMetadata {
        ProviderMetadata {
            provider_name: "mock".to_string(),
            model: "mock-v1".to_string(),
            prompt_version: PROMPT_VERSION.to_string(),
            analysis_schema_version: 1,
            configuration_fingerprint: "cfg".to_string(),
        }
    }

    fn response(findings: Vec<FindingResponse>) -> AnalysisProviderResponse {
        AnalysisProviderResponse {
            findings,
            usage: None,
        }
    }

    fn raw(category: &str, claim: &str) -> FindingResponse {
        FindingResponse {
            category: category.to_string(),
            subject: "scene".to_string(),
            claim: claim.to_string(),
            confidence: 0.8,
            evidence_ordinals: vec!["para-1".to_string(), "para-2".to_string()],
            uncertainty: None,
            alternative_interpretations: Vec::new(),
        }
    }

    fn validate(response: &AnalysisProviderResponse) -> ValidationOutcome {
        validate_unit_response(&unit(), &metadata(), response)
    }

    #[test]
    fn valid_structured_response_is_accepted() {
        let outcome = validate(&response(vec![raw("tone", "somber narrative tone")]));
        assert_eq!(outcome.findings.len(), 1);
        assert_eq!(outcome.rejected.len(), 0);
        let finding = &outcome.findings[0];
        assert_eq!(finding.category, FindingCategory::Tone);
        assert_eq!(finding.evidence.len(), 2);
        assert!(finding.is_review_eligible);
        assert_eq!(finding.confidence.level, ConfidenceLevel::High);
        assert_eq!(
            finding.evidence[0].paragraph_id, "paragraph-1",
            "evidence resolves to the real document paragraph id"
        );
    }

    #[test]
    fn hallucinated_evidence_ordinal_is_rejected() {
        let mut finding = raw("tone", "claim");
        finding.evidence_ordinals = vec!["para-1".to_string(), "para-99".to_string()];
        let outcome = validate(&response(vec![finding]));
        assert_eq!(outcome.findings.len(), 0);
        assert_eq!(outcome.rejected.len(), 1);
        assert!(outcome.rejected[0].reason.contains("para-99"));
    }

    #[test]
    fn unknown_category_and_invalid_confidence_are_rejected() {
        let mut bad_category = raw("telepathy", "claim");
        bad_category.confidence = 0.9;
        let outcome = validate(&response(vec![bad_category]));
        assert_eq!(outcome.rejected.len(), 1);
        assert!(outcome.rejected[0]
            .reason
            .contains("unknown finding category"));

        let mut bad_confidence = raw("tone", "claim");
        bad_confidence.confidence = 1.5;
        let outcome = validate(&response(vec![bad_confidence]));
        assert_eq!(outcome.rejected.len(), 1);
        assert!(outcome.rejected[0].reason.contains("invalid confidence"));
    }

    #[test]
    fn oversized_and_empty_fields_are_rejected() {
        let mut empty_claim = raw("tone", "");
        let outcome = validate(&response(vec![empty_claim.clone()]));
        assert_eq!(outcome.rejected.len(), 1);
        assert!(outcome.rejected[0].reason.contains("empty finding claim"));

        empty_claim.claim = "x".repeat(MAX_CLAIM_CHARS + 1);
        let outcome = validate(&response(vec![empty_claim]));
        assert_eq!(outcome.rejected.len(), 1);
        assert!(outcome.rejected[0].reason.contains("exceeds"));

        let mut oversized_subject = raw("tone", "claim");
        oversized_subject.subject = "s".repeat(MAX_SUBJECT_CHARS + 1);
        let outcome = validate(&response(vec![oversized_subject]));
        assert_eq!(outcome.rejected.len(), 1);
    }

    #[test]
    fn no_evidence_means_not_review_eligible_but_still_retained() {
        let mut finding = raw("tone", "unsupported claim");
        finding.evidence_ordinals = Vec::new();
        finding.confidence = 0.1;
        let outcome = validate(&response(vec![finding]));
        assert_eq!(outcome.findings.len(), 1);
        assert!(!outcome.findings[0].is_review_eligible);
        assert_eq!(outcome.findings[0].confidence.level, ConfidenceLevel::Low);
    }

    #[test]
    fn duplicate_semantic_findings_collapse_within_a_unit() {
        let outcome = validate(&response(vec![
            raw("tone", "somber narrative tone"),
            raw("tone", "somber narrative tone"),
        ]));
        assert_eq!(outcome.findings.len(), 1);
    }

    #[test]
    fn scene_scope_is_preserved() {
        let outcome = validate(&response(vec![raw(
            "point_of_view",
            "third person limited",
        )]));
        let finding = &outcome.findings[0];
        assert!(matches!(finding.scope, NarrativeScope::Scene { .. }));
    }
}
