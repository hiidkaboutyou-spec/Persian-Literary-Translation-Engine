//! Sequential, bounded orchestration of provider-assisted analysis.
//!
//! No agent swarms, no recursive loops: one pass over bounded units, bounded
//! retries for retryable failures only, deterministic validation of every
//! response, fingerprint-based cache reuse, and cross-unit disagreement
//! surfaced as scope-preserving warnings rather than silent last-write-wins.

use crate::cache::{cache_key, canon_fingerprint, AnalysisCache, CachedUnitResult};
use crate::models::{
    AdvancedAnalysisError, AdvancedAnalysisResult, AdvancedLiteraryFinding,
    AnalysisProviderRequest, CanonContext, ConfidenceLevel, FailedUnit, FindingCategory,
    ProviderMetadata, UsageMetadata, ADVANCED_ANALYSIS_SCHEMA_VERSION,
};
use crate::planner::{AnalysisPlanner, PlannerConfig};
use crate::prompts::build_prompt;
use crate::provider::LiteraryAnalysisProvider;
use crate::validation::validate_unit_response;
use chrono::{DateTime, Utc};
use document_engine::Manuscript;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct AdvancedAnalysisConfig {
    pub planner: PlannerConfig,
    /// Restricted category set (empty = all supported categories).
    pub categories: Vec<FindingCategory>,
    /// Bounded retry attempts for retryable provider failures.
    pub max_retries: usize,
    /// Delay between retry attempts (use Duration::ZERO in tests).
    pub retry_delay: Duration,
    /// Optional cache file for resumable runs.
    pub cache: Option<PathBuf>,
}

impl Default for AdvancedAnalysisConfig {
    fn default() -> Self {
        Self {
            planner: PlannerConfig::default(),
            categories: Vec::new(),
            max_retries: 2,
            retry_delay: Duration::from_millis(250),
            cache: None,
        }
    }
}

impl AdvancedAnalysisConfig {
    pub fn fingerprint(&self) -> String {
        let category_labels = if self.categories.is_empty() {
            "all".to_string()
        } else {
            self.categories
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(",")
        };
        let identity = format!(
            "config\0{}\0{category_labels}\0retries={}",
            self.planner.fingerprint(),
            self.max_retries
        );
        stable_hash(identity.as_bytes())
    }
}

pub fn run_advanced_analysis(
    manuscript: &Manuscript,
    canon: &CanonContext,
    provider: &dyn LiteraryAnalysisProvider,
    config: &AdvancedAnalysisConfig,
    analyzed_at: DateTime<Utc>,
) -> Result<AdvancedAnalysisResult, AdvancedAnalysisError> {
    let planner = AnalysisPlanner::new(config.planner.clone());
    let units = planner.plan(manuscript, canon)?;

    let configuration_fingerprint = config.fingerprint();
    let provider_metadata = ProviderMetadata {
        provider_name: provider.name().to_string(),
        model: provider.model().to_string(),
        prompt_version: crate::models::PROMPT_VERSION.to_string(),
        analysis_schema_version: ADVANCED_ANALYSIS_SCHEMA_VERSION,
        configuration_fingerprint: configuration_fingerprint.clone(),
    };
    let canon_fp = canon_fingerprint(&canon.character_names, &canon.glossary_terms);
    let mut cache = AnalysisCache::new(config.cache.clone());
    let mut findings = Vec::new();
    let mut failed_units = Vec::new();
    let mut warnings = Vec::new();
    let mut usage = UsageMetadata::default();
    let mut succeeded_units = 0usize;
    let mut cached_units = 0usize;

    for unit in &units {
        let prompt = build_prompt(unit, canon, &config.categories)?;
        let key = cache_key(
            ADVANCED_ANALYSIS_SCHEMA_VERSION,
            &prompt.prompt_version,
            provider.name(),
            provider.model(),
            &configuration_fingerprint,
            &canon_fp,
            &unit.unit_id,
        );

        let response = if let Some(cached) = cache.get(&key) {
            cached_units += 1;
            succeeded_units += 1;
            findings.extend(cached.findings.iter().cloned());
            continue;
        } else {
            match call_with_retry(provider, unit, prompt.clone(), canon, config) {
                Ok(response) => {
                    succeeded_units += 1;
                    if let Some(unit_usage) = &response.usage {
                        usage.merge(unit_usage);
                    }
                    response
                }
                Err((error, retryable)) => {
                    failed_units.push(FailedUnit {
                        unit_id: unit.unit_id.clone(),
                        error: error.to_string(),
                        retryable,
                    });
                    continue;
                }
            }
        };

        let outcome = validate_unit_response(unit, &provider_metadata, &response);
        let rejected_count = outcome.rejected.len();
        if rejected_count > 0 {
            let mut reasons = outcome
                .rejected
                .iter()
                .take(3)
                .map(|value| value.reason.clone())
                .collect::<Vec<_>>();
            reasons.sort();
            warnings.push(format!(
                "unit {}: rejected {} provider finding(s) ({})",
                unit.unit_id,
                rejected_count,
                reasons.join("; ")
            ));
        }
        findings.extend(outcome.findings.clone());
        cache.insert(CachedUnitResult {
            key,
            unit_id: unit.unit_id.clone(),
            provider: provider.name().to_string(),
            model: provider.model().to_string(),
            prompt_version: prompt.prompt_version.clone(),
            findings: outcome.findings,
            stored_at: analyzed_at,
        });
    }

    adjust_cross_unit_disagreement(&mut findings, &mut warnings);

    let result = AdvancedAnalysisResult {
        schema_version: ADVANCED_ANALYSIS_SCHEMA_VERSION,
        analysis_id: AdvancedAnalysisResult::analysis_id(
            &manuscript.book.id,
            provider.name(),
            provider.model(),
        ),
        manuscript_id: manuscript.book.id.clone(),
        manuscript_title: manuscript.book.title.clone(),
        provider_metadata,
        findings,
        failed_units,
        total_units: units.len(),
        succeeded_units,
        cached_units,
        warnings,
        usage,
        analyzed_at,
    };

    cache.persist()?;
    Ok(result)
}

/// Bounded retry only for clearly retryable failures (rate limit / transport).
/// Schema, auth, and permanent failures are never retried.
fn call_with_retry(
    provider: &dyn LiteraryAnalysisProvider,
    unit: &crate::models::AnalysisUnit,
    prompt: crate::models::ProviderPrompt,
    canon: &CanonContext,
    config: &AdvancedAnalysisConfig,
) -> Result<crate::models::AnalysisProviderResponse, (crate::provider::AnalysisProviderError, bool)>
{
    let request = AnalysisProviderRequest {
        unit: unit.clone(),
        prompt: prompt.clone(),
        categories: config.categories.clone(),
        canon_context: canon.clone(),
    };
    let mut last_error = None;
    for attempt in 0..=config.max_retries {
        match provider.analyze(&request) {
            Ok(response) => return Ok(response),
            Err(error) if error.is_retryable() && attempt < config.max_retries => {
                if !config.retry_delay.is_zero() {
                    std::thread::sleep(config.retry_delay);
                }
                last_error = Some(error);
            }
            Err(error) => {
                let retryable = error.is_retryable();
                return Err((error, retryable));
            }
        }
    }
    Err((
        last_error.unwrap_or(crate::provider::AnalysisProviderError::Failed(
            "provider retry budget exhausted".to_string(),
        )),
        true,
    ))
}

/// Conflicting model findings on the exact same scope are surfaced as
/// disagreement, never collapsed into whichever claim arrived last. Findings
/// with distinct scopes (for example a relationship that evolves across
/// chapters) are preserved separately without contradiction counts.
fn adjust_cross_unit_disagreement(
    findings: &mut [AdvancedLiteraryFinding],
    warnings: &mut Vec<String>,
) {
    let mut groups: BTreeMap<(FindingCategory, String, String), Vec<usize>> = BTreeMap::new();
    for (index, finding) in findings.iter().enumerate() {
        let key = (
            finding.category,
            crate::models::normalize_for_identity(&finding.subject),
            finding.scope.key(),
        );
        groups.entry(key).or_default().push(index);
    }
    for ((category, subject, scope), indexes) in &groups {
        if indexes.len() < 2 {
            continue;
        }
        let mut claims = BTreeSet::new();
        for index in indexes {
            claims.insert(crate::models::normalize_for_identity(
                &findings[*index].claim,
            ));
        }
        if claims.len() < 2 {
            continue;
        }
        warnings.push(format!(
            "conflicting model findings for {category} on '{subject}' at {scope}: \
             {} distinct readings retained with scope (human review required)",
            claims.len()
        ));
        for index in indexes {
            let finding = &mut findings[*index];
            finding.confidence.contradicting_unit_count = claims.len() - 1;
            if finding.confidence.level == ConfidenceLevel::High {
                finding.confidence.level = ConfidenceLevel::Moderate;
            }
        }
    }
}

fn stable_hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::{MockAnalysisProvider, MockAnalysisProviderConfig, MockFindingTemplate};
    use document_engine::{Book, Chapter, DocumentFormat, Paragraph, Scene, SourceLocation};

    fn paragraph(position: usize, text: &str) -> Paragraph {
        Paragraph {
            id: format!("paragraph-{position}"),
            scene_id: "scene-1".to_string(),
            original_text: text.to_string(),
            position,
            source: SourceLocation::new("synthetic.txt", DocumentFormat::Txt),
        }
    }

    fn scene(paragraphs: Vec<Paragraph>) -> Scene {
        Scene {
            id: "scene-1".to_string(),
            chapter_id: "chapter-1".to_string(),
            order: 1,
            text: paragraphs
                .iter()
                .map(|paragraph| paragraph.original_text.clone())
                .collect::<Vec<_>>()
                .join("\n\n"),
            importance: Default::default(),
            paragraphs,
            source: SourceLocation::new("synthetic.txt", DocumentFormat::Txt),
        }
    }

    fn manuscript_with_scenes(scenes: Vec<Scene>) -> Manuscript {
        Manuscript {
            book: Book {
                id: "book-1".to_string(),
                title: "Synthetic".to_string(),
                author: None,
                language: None,
                metadata: Default::default(),
            },
            chapters: vec![Chapter {
                id: "chapter-1".to_string(),
                title: "Chapter 1".to_string(),
                order: 1,
                index: 0,
                content: String::new(),
                scenes,
                source: SourceLocation::new("synthetic.txt", DocumentFormat::Txt),
            }],
            source: SourceLocation::new("synthetic.txt", DocumentFormat::Txt),
        }
    }

    fn manuscript() -> Manuscript {
        manuscript_with_scenes(vec![scene(vec![
            paragraph(1, "Reza waited in the cold rain."),
            paragraph(2, "Mina arrived at last."),
        ])])
    }

    fn config_with_cache(cache: Option<PathBuf>) -> AdvancedAnalysisConfig {
        AdvancedAnalysisConfig {
            max_retries: 1,
            retry_delay: Duration::ZERO,
            cache,
            ..Default::default()
        }
    }

    #[test]
    fn partial_failure_preserves_successful_units_and_reports_failures() {
        let mut first_scene = scene(vec![paragraph(1, "alpha")]);
        first_scene.id = "scene-1".to_string();
        let mut second_scene = scene(vec![paragraph(1, "beta")]);
        second_scene.id = "scene-2".to_string();
        let book = manuscript_with_scenes(vec![first_scene, second_scene]);

        // Fail exactly one unit. Which unit prefix to use is discovered by
        // planning first; here we use a provider that fails when the unit
        // text mentions "beta".
        struct SelectiveFail;
        impl LiteraryAnalysisProvider for SelectiveFail {
            fn name(&self) -> &str {
                "mock"
            }
            fn model(&self) -> &str {
                "mock-v1"
            }
            fn analyze(
                &self,
                request: &AnalysisProviderRequest,
            ) -> Result<
                crate::models::AnalysisProviderResponse,
                crate::provider::AnalysisProviderError,
            > {
                if request.unit.text_content.contains("beta") {
                    return Err(crate::provider::AnalysisProviderError::Failed(
                        "simulated failure".to_string(),
                    ));
                }
                Ok(crate::models::AnalysisProviderResponse {
                    findings: Vec::new(),
                    usage: None,
                })
            }
        }
        let result = run_advanced_analysis(
            &book,
            &CanonContext::default(),
            &SelectiveFail,
            &config_with_cache(None),
            Utc::now(),
        )
        .unwrap();
        assert_eq!(result.total_units, 2);
        assert_eq!(result.succeeded_units, 1);
        assert_eq!(result.failed_units.len(), 1);
        assert!(!result.failed_units[0].retryable);
        assert!(result.findings.is_empty());
    }

    #[test]
    fn retryable_failure_is_retried_and_eventually_succeeds() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static CALLS: AtomicUsize = AtomicUsize::new(0);
        struct Flaky;
        impl LiteraryAnalysisProvider for Flaky {
            fn name(&self) -> &str {
                "mock"
            }
            fn model(&self) -> &str {
                "mock-v1"
            }
            fn analyze(
                &self,
                _request: &AnalysisProviderRequest,
            ) -> Result<
                crate::models::AnalysisProviderResponse,
                crate::provider::AnalysisProviderError,
            > {
                let attempt = CALLS.fetch_add(1, Ordering::SeqCst);
                if attempt == 0 {
                    Err(crate::provider::AnalysisProviderError::RateLimited)
                } else {
                    Ok(crate::models::AnalysisProviderResponse {
                        findings: Vec::new(),
                        usage: None,
                    })
                }
            }
        }
        CALLS.store(0, Ordering::SeqCst);
        let result = run_advanced_analysis(
            &manuscript(),
            &CanonContext::default(),
            &Flaky,
            &config_with_cache(None),
            Utc::now(),
        )
        .unwrap();
        assert_eq!(result.failed_units.len(), 0);
        assert_eq!(result.succeeded_units, 1);
        assert!(CALLS.load(Ordering::SeqCst) >= 2);
    }

    #[test]
    fn unchanged_input_is_reused_from_cache_and_changed_unit_is_recomputed() {
        let directory = tempfile::tempdir().unwrap();
        let cache_path = directory.path().join("cache.json");
        let provider = MockAnalysisProvider::with_default_findings();
        let canon = CanonContext::default();

        let first = run_advanced_analysis(
            &manuscript(),
            &canon,
            &provider,
            &config_with_cache(Some(cache_path.clone())),
            Utc::now(),
        )
        .unwrap();
        assert_eq!(first.cached_units, 0);
        assert_eq!(first.succeeded_units, 1);
        assert!(!first.findings.is_empty());

        let second = run_advanced_analysis(
            &manuscript(),
            &canon,
            &provider,
            &config_with_cache(Some(cache_path.clone())),
            Utc::now(),
        )
        .unwrap();
        assert_eq!(
            second.cached_units, 1,
            "identical input reuses the cached result"
        );

        let changed = manuscript_with_scenes(vec![scene(vec![
            paragraph(1, "Reza waited in the cold rain, frowning."),
            paragraph(2, "Mina arrived at last."),
        ])]);
        let third = run_advanced_analysis(
            &changed,
            &canon,
            &provider,
            &config_with_cache(Some(cache_path.clone())),
            Utc::now(),
        )
        .unwrap();
        assert_eq!(
            third.cached_units, 0,
            "edited scene invalidates only the affected unit (here: the only unit)"
        );
    }

    #[test]
    fn conflicting_findings_on_one_scope_are_surfaced_not_collapsed() {
        // Mock emits two tone templates with contradictory claims in the same
        // unit; both are retained and flagged, and neither stays High.
        let provider = MockAnalysisProvider::new(MockAnalysisProviderConfig {
            findings_per_unit: vec![
                MockFindingTemplate {
                    category: "tone".to_string(),
                    subject: "scene".to_string(),
                    claim: "tense hostile tone".to_string(),
                    confidence: 0.9,
                    ..Default::default()
                },
                MockFindingTemplate {
                    category: "tone".to_string(),
                    subject: "scene".to_string(),
                    claim: "warm affectionate tone".to_string(),
                    confidence: 0.9,
                    ..Default::default()
                },
            ],
            ..Default::default()
        });
        let result = run_advanced_analysis(
            &manuscript(),
            &CanonContext::default(),
            &provider,
            &config_with_cache(None),
            Utc::now(),
        )
        .unwrap();
        assert_eq!(result.findings.len(), 2);
        assert!(result
            .warnings
            .iter()
            .any(|warning| warning.contains("conflicting model findings")));
        assert!(result
            .findings
            .iter()
            .all(|finding| finding.confidence.level != ConfidenceLevel::High));
        assert!(result
            .findings
            .iter()
            .all(|finding| { finding.confidence.contradicting_unit_count == 1 }));
    }

    #[test]
    fn findings_never_duplicate_across_identical_runs() {
        let provider = MockAnalysisProvider::with_default_findings();
        let first = run_advanced_analysis(
            &manuscript(),
            &CanonContext::default(),
            &provider,
            &config_with_cache(None),
            Utc::now(),
        )
        .unwrap();
        let second = run_advanced_analysis(
            &manuscript(),
            &CanonContext::default(),
            &provider,
            &config_with_cache(None),
            Utc::now(),
        )
        .unwrap();
        assert_eq!(first.findings, second.findings);
        assert_eq!(first.findings[0].finding_id, second.findings[0].finding_id);
    }

    #[test]
    fn usage_metadata_is_accumulated() {
        let provider = MockAnalysisProvider::new(MockAnalysisProviderConfig {
            usage: UsageMetadata {
                input_tokens: 100,
                output_tokens: 40,
                total_tokens: 140,
                request_count: 1,
            },
            ..Default::default()
        });
        let result = run_advanced_analysis(
            &manuscript(),
            &CanonContext::default(),
            &provider,
            &config_with_cache(None),
            Utc::now(),
        )
        .unwrap();
        assert_eq!(result.usage.input_tokens, 100);
        assert_eq!(result.usage.total_tokens, 140);
        assert_eq!(result.usage.request_count, 1);
    }

    #[test]
    fn unicode_manuscript_text_is_handled_without_byte_indexing() {
        let book = manuscript_with_scenes(vec![scene(vec![
            paragraph(1, "رضا در باران سرد منتظر ماند."),
            paragraph(2, "«مینا، تو بالاخره رسیدی.» — کیم گفت."),
        ])]);
        let provider = MockAnalysisProvider::new(MockAnalysisProviderConfig {
            findings_per_unit: vec![MockFindingTemplate {
                category: "dialogue_function".to_string(),
                subject: "scene".to_string(),
                claim: "greeting relieves tension".to_string(),
                confidence: 0.6,
                ..Default::default()
            }],
            ..Default::default()
        });
        let result = run_advanced_analysis(
            &book,
            &CanonContext::default(),
            &provider,
            &config_with_cache(None),
            Utc::now(),
        )
        .unwrap();
        assert_eq!(result.succeeded_units, 1);
        assert_eq!(result.findings.len(), 1);
    }
}
