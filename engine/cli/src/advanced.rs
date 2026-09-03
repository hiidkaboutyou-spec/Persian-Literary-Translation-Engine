//! `literary-engine analyze-advanced <manuscript>` — explicit, provider-assisted
//! literary analysis (Phase 15).
//!
//! The plain `analyze` command stays fully deterministic. Advanced analysis is
//! only ever run here, on request, and every provider finding is validated
//! (evidence identifiers, schema, confidence, bounded fields) before it can
//! become a review proposal. Findings flow into the same Phase 14 review
//! ledger as `Literary` proposals — reviewable, never auto-promoted, never
//! canonized.

use crate::OutputFormat;
use advanced_literary_analysis::{
    run_advanced_analysis, AdvancedAnalysisConfig, AdvancedLiteraryFinding, CanonContext,
    LiteraryAnalysisProvider, MockAnalysisProvider, OpenAIAnalysisProvider,
    ADVANCED_ANALYSIS_SCHEMA_VERSION,
};
use chrono::Utc;
use document_engine::ingest_file;
use human_review_workflow::{LiteraryFindingProposal, ReviewLedger, ReviewProposal};
use literary_intelligence_engine::{
    Confidence, ConfidenceLevel, DeterministicManuscriptAnalyzer, EvidenceRef, ManuscriptAnalyzer,
};
use project_engine::review_store::{
    load_character_bible_or_default, load_glossary_or_default, load_review_ledger,
    save_review_ledger,
};
use serde::Serialize;
use std::env;
use std::path::PathBuf;

#[derive(Serialize)]
struct AnalyzeAdvancedOutput {
    schema_version: u32,
    analysis_id: String,
    manuscript_id: String,
    manuscript_title: String,
    provider: String,
    model: String,
    prompt_version: String,
    analysis_schema_version: u32,
    total_units: usize,
    succeeded_units: usize,
    cached_units: usize,
    failed_units: Vec<advanced_literary_analysis::FailedUnit>,
    findings: Vec<AdvancedLiteraryFinding>,
    findings_by_category: Vec<CategoryCount>,
    warnings: Vec<String>,
    usage: advanced_literary_analysis::UsageMetadata,
    review_file: Option<String>,
    review_proposals_generated: usize,
    review_items_added: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    reconciliation_added: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reconciliation_unchanged: Option<Vec<String>>,
}

#[derive(Serialize)]
struct CategoryCount {
    category: String,
    count: usize,
}

pub(crate) fn run_analyze_advanced(args: &[String], format: &OutputFormat) -> Result<(), String> {
    let Some(manuscript_path) = positional(args, 0) else {
        return Err("analyze-advanced requires a manuscript path".to_string());
    };
    let review_file = optional_path(args, "--review-file", "LITERARY_ENGINE_REVIEW_FILE");
    let character_file = optional_path(
        args,
        "--character-bible",
        "LITERARY_ENGINE_CHARACTER_BIBLE_FILE",
    );
    let glossary_file = optional_path(args, "--glossary", "LITERARY_ENGINE_GLOSSARY_FILE");
    let cache_path = optional_path(args, "--cache", "LITERARY_ENGINE_ANALYSIS_CACHE");

    let characters = match character_file {
        Some(path) => load_character_bible_or_default(path).map_err(|error| error.to_string())?,
        None => character_engine::CharacterBible::new(),
    };
    let glossary = match glossary_file {
        Some(path) => load_glossary_or_default(path).map_err(|error| error.to_string())?,
        None => memory_engine::glossary::Glossary::default(),
    };

    let manuscript = ingest_file(manuscript_path)
        .map_err(|error| format!("failed to read {manuscript_path}: {error}"))?;

    // Deterministic analysis first (Phase 13) — always runs, always offline.
    // It binds the review ledger to this manuscript and supplies the canon
    // snapshot used to frame provider requests.
    let intelligence = DeterministicManuscriptAnalyzer::default()
        .analyze(
            &manuscript,
            literary_intelligence_engine::AnalysisCanon {
                characters: &characters,
                glossary: &glossary,
            },
        )
        .map_err(|error| format!("failed to analyze {manuscript_path}: {error}"))?;

    let provider = configured_analysis_provider(args)?;

    let max_units = flag_value(args, "--max-units")
        .map(|value| {
            value
                .parse::<usize>()
                .map_err(|error| format!("invalid --max-units '{value}': {error}"))
        })
        .transpose()?;
    let mut config = AdvancedAnalysisConfig {
        cache: cache_path,
        ..Default::default()
    };
    if let Some(max_units) = max_units {
        config.planner.max_units = max_units;
    }

    let canon = CanonContext {
        character_names: characters
            .profiles()
            .iter()
            .map(|profile| profile.name.clone())
            .collect(),
        character_aliases: characters
            .aliases()
            .iter()
            .map(|alias| alias.alias.clone())
            .collect(),
        glossary_terms: glossary
            .entries()
            .iter()
            .map(|entry| entry.source_term.clone())
            .collect(),
        relationship_pairs: Vec::new(),
    };

    let analyzed_at = Utc::now();
    let result =
        run_advanced_analysis(&manuscript, &canon, provider.as_ref(), &config, analyzed_at)
            .map_err(|error| format!("advanced analysis failed: {error}"))?;

    // Map validated, review-eligible findings into Phase 14 Literary proposals.
    let proposals = result
        .findings
        .iter()
        .filter(|finding| finding.is_review_eligible)
        .filter_map(literary_proposal_from_finding)
        .collect::<Vec<_>>();

    let mut review_items_added = 0usize;
    let mut reconciliation_added = None;
    let mut reconciliation_unchanged = None;
    let review_file_label = review_file.as_ref().map(|path| path.display().to_string());

    if let Some(path) = &review_file {
        let mut ledger = if path.exists() {
            load_review_ledger(path).map_err(|error| error.to_string())?
        } else {
            ReviewLedger::new(&intelligence)
        };
        // Bind/refresh the ledger to this manuscript via the deterministic
        // pass (scope-safe: it never obsoletes Literary items).
        ledger
            .reconcile(&intelligence, analyzed_at)
            .map_err(|error| format!("failed to reconcile review ledger: {error}"))?;
        let record = ledger
            .reconcile_advanced(
                proposals,
                ADVANCED_ANALYSIS_SCHEMA_VERSION,
                format!("advanced:{}/{}", provider.name(), provider.model()),
                result.provider_metadata.prompt_version.clone(),
                analyzed_at,
            )
            .map_err(|error| format!("failed to reconcile advanced findings: {error}"))?;
        review_items_added = record.added.len();
        reconciliation_added = Some(record.added.clone());
        reconciliation_unchanged = Some(record.unchanged.clone());
        save_review_ledger(path, &ledger).map_err(|error| error.to_string())?;
    }

    let mut category_counts: Vec<CategoryCount> = Vec::new();
    for finding in &result.findings {
        let label = finding.category.to_string();
        if let Some(existing) = category_counts
            .iter_mut()
            .find(|count| count.category == label)
        {
            existing.count += 1;
        } else {
            category_counts.push(CategoryCount {
                category: label,
                count: 1,
            });
        }
    }
    category_counts.sort_by_key(|count| std::cmp::Reverse(count.count));

    let output = AnalyzeAdvancedOutput {
        schema_version: ADVANCED_ANALYSIS_SCHEMA_VERSION,
        analysis_id: result.analysis_id.clone(),
        manuscript_id: result.manuscript_id.clone(),
        manuscript_title: result.manuscript_title.clone(),
        provider: result.provider_metadata.provider_name.clone(),
        model: result.provider_metadata.model.clone(),
        prompt_version: result.provider_metadata.prompt_version.clone(),
        analysis_schema_version: result.provider_metadata.analysis_schema_version,
        total_units: result.total_units,
        succeeded_units: result.succeeded_units,
        cached_units: result.cached_units,
        failed_units: result.failed_units.clone(),
        findings: result.findings.clone(),
        findings_by_category: category_counts,
        warnings: result.warnings.clone(),
        usage: result.usage.clone(),
        review_file: review_file_label,
        review_proposals_generated: result
            .findings
            .iter()
            .filter(|f| f.is_review_eligible)
            .count(),
        review_items_added,
        reconciliation_added,
        reconciliation_unchanged,
    };

    print_advanced_output(format, &output, &result, review_items_added)
}

fn print_advanced_output(
    format: &OutputFormat,
    output: &AnalyzeAdvancedOutput,
    result: &advanced_literary_analysis::AdvancedAnalysisResult,
    review_items_added: usize,
) -> Result<(), String> {
    match format {
        OutputFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(output).map_err(|error| format!(
                "failed to serialize advanced analysis output: {error}"
            ))?
        ),
        OutputFormat::Text => {
            println!("manuscript: {}", output.manuscript_title);
            println!("provider: {} ({})", output.provider, output.model);
            println!(
                "analysis units: {} total, {} completed, {} failed, {} cached",
                output.total_units,
                output.succeeded_units,
                output.failed_units.len(),
                output.cached_units
            );
            if output.usage.request_count > 0 {
                println!(
                    "usage: {} requests, {} input tokens, {} output tokens",
                    output.usage.request_count,
                    output.usage.input_tokens,
                    output.usage.output_tokens
                );
            }
            println!("findings: {}", output.findings.len());
            for count in &output.findings_by_category {
                println!("  {}: {}", count.category, count.count);
            }
            if !output.failed_units.is_empty() {
                println!("failed units:");
                for failed in &output.failed_units {
                    println!("  {}: {}", failed.unit_id, failed.error);
                }
            }
            for warning in &output.warnings {
                println!("warning: {warning}");
            }
            match (&output.review_file, review_items_added) {
                (Some(file), added) => {
                    println!("review: {added} new literary proposal(s) reconciled into {file}")
                }
                (None, _) => println!("review: no --review-file, findings not queued for review"),
            }
            let _ = result;
        }
    }
    Ok(())
}

fn literary_proposal_from_finding(finding: &AdvancedLiteraryFinding) -> Option<ReviewProposal> {
    if !finding.is_review_eligible {
        return None;
    }
    let level = match finding.confidence.level {
        advanced_literary_analysis::ConfidenceLevel::Low => ConfidenceLevel::Low,
        advanced_literary_analysis::ConfidenceLevel::Moderate => ConfidenceLevel::Moderate,
        advanced_literary_analysis::ConfidenceLevel::High => ConfidenceLevel::High,
    };
    Some(ReviewProposal::Literary(Box::new(
        LiteraryFindingProposal {
            id: finding.finding_id.clone(),
            finding_category: finding.category.to_string(),
            subject: finding.subject.clone(),
            claim: finding.claim.clone(),
            scope_label: finding.scope.label(),
            confidence: Confidence {
                level,
                supporting_evidence: finding.evidence.len(),
            },
            evidence: finding
                .evidence
                .iter()
                .map(|reference| EvidenceRef {
                    chapter_id: reference.chapter_id.clone(),
                    scene_id: reference.scene_id.clone(),
                    paragraph_id: reference.paragraph_id.clone(),
                    source: reference.source.clone(),
                })
                .collect(),
            analysis_unit_id: finding.analysis_unit_id.clone(),
            alternative_interpretations: finding.alternative_interpretations.clone(),
        },
    )))
}

fn configured_analysis_provider(
    args: &[String],
) -> Result<Box<dyn LiteraryAnalysisProvider>, String> {
    let explicit = flag_value(args, "--provider")
        .map(str::to_string)
        .or_else(|| env::var("LITERARY_ENGINE_ANALYSIS_PROVIDER").ok());
    match explicit.as_deref() {
        Some("openai") => Ok(Box::new(
            OpenAIAnalysisProvider::from_env().map_err(|error| error.to_string())?,
        )),
        Some("mock") | None => Ok(Box::new(MockAnalysisProvider::with_default_findings())),
        Some(other) => Err(format!(
            "unsupported analysis provider '{other}'; expected 'mock' or 'openai'"
        )),
    }
}

fn positional(args: &[String], wanted: usize) -> Option<&str> {
    let flags_with_values = [
        "--review-file",
        "--character-bible",
        "--glossary",
        "--provider",
        "--cache",
        "--max-units",
    ];
    let mut values = Vec::new();
    let mut index = 0;
    while index < args.len() {
        if flags_with_values.contains(&args[index].as_str()) {
            index += 2;
        } else if args[index].starts_with("--") {
            index += 1;
        } else {
            values.push(args[index].as_str());
            index += 1;
        }
    }
    values.get(wanted).copied()
}

fn optional_path(args: &[String], flag: &str, variable: &str) -> Option<PathBuf> {
    flag_value(args, flag).map(PathBuf::from).or_else(|| {
        env::var(variable)
            .ok()
            .filter(|value| !value.trim().is_empty())
            .map(PathBuf::from)
    })
}

fn flag_value<'a>(args: &'a [String], flag: &str) -> Option<&'a str> {
    args.windows(2)
        .find(|pair| pair[0] == flag)
        .map(|pair| pair[1].as_str())
}
