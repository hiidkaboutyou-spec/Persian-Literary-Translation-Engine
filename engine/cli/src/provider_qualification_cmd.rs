use crate::OutputFormat;
use literary_evaluation_engine::{
    evaluate_submission, CandidateOutput, CandidateSubmission, LiteraryEvaluationCorpus,
};
use serde::Serialize;
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::sync::Mutex;
use std::time::Instant;
use translation_core::{
    AtriaProvider, EchoProvider, OpenAIProvider, PipelineInput, ProviderError, ProviderRequest,
    ProviderResponse, TranslationPipeline, TranslationProvider,
};

type Result<T> = std::result::Result<T, String>;

#[derive(Debug, Serialize)]
struct ProviderQualificationReport {
    schema_version: u32,
    corpus_id: String,
    provider: String,
    model: Option<String>,
    cases_generated: usize,
    corpus_cases: usize,
    complete_corpus: bool,
    pipeline: String,
    submission_file: String,
    deterministic_anchor_passed: usize,
    deterministic_anchor_total: usize,
    deterministic_anchor_pass_rate: f32,
    provider_requests: usize,
    usage_observed_requests: usize,
    token_usage_complete: bool,
    input_tokens: Option<u64>,
    output_tokens: Option<u64>,
    total_provider_latency_ms: u128,
    human_review_required: bool,
    production_admission: &'static str,
    notes: Vec<&'static str>,
}

#[derive(Debug, Serialize)]
struct BlindComparisonBundle {
    schema_version: u32,
    corpus_id: String,
    cases: Vec<BlindComparisonCase>,
    instructions: Vec<&'static str>,
}

#[derive(Debug, Serialize)]
struct BlindComparisonCase {
    case_id: String,
    title: String,
    source: String,
    context_before: Option<String>,
    context_after: Option<String>,
    dimensions: Vec<literary_evaluation_engine::EvaluationDimension>,
    candidate_a: String,
    candidate_b: String,
}

#[derive(Debug, Serialize)]
struct BlindComparisonKey {
    schema_version: u32,
    corpus_id: String,
    system_one: String,
    system_two: String,
    assignments: Vec<BlindAssignment>,
}

#[derive(Debug, Serialize)]
struct BlindAssignment {
    case_id: String,
    candidate_a_system: String,
    candidate_b_system: String,
}

#[derive(Debug, Clone, Copy, Default)]
struct QualificationTelemetry {
    request_count: usize,
    usage_observed_requests: usize,
    input_tokens: u64,
    output_tokens: u64,
    total_latency_ms: u128,
}

struct ObservedProvider<'a> {
    inner: &'a dyn TranslationProvider,
    telemetry: Mutex<QualificationTelemetry>,
}

impl<'a> ObservedProvider<'a> {
    fn new(inner: &'a dyn TranslationProvider) -> Self {
        Self {
            inner,
            telemetry: Mutex::new(QualificationTelemetry::default()),
        }
    }

    fn snapshot(&self) -> QualificationTelemetry {
        *self
            .telemetry
            .lock()
            .expect("qualification telemetry poisoned")
    }
}

impl TranslationProvider for ObservedProvider<'_> {
    fn name(&self) -> &str {
        self.inner.name()
    }

    fn execute(
        &self,
        request: &ProviderRequest,
    ) -> std::result::Result<ProviderResponse, ProviderError> {
        let started = Instant::now();
        let response = self.inner.execute(request)?;
        let elapsed_ms = started.elapsed().as_millis();

        let mut telemetry = self
            .telemetry
            .lock()
            .expect("qualification telemetry poisoned");
        telemetry.request_count += 1;
        telemetry.total_latency_ms = telemetry.total_latency_ms.saturating_add(elapsed_ms);
        if let Some(usage) = response.usage {
            telemetry.usage_observed_requests += 1;
            telemetry.input_tokens = telemetry.input_tokens.saturating_add(usage.input_tokens);
            telemetry.output_tokens = telemetry.output_tokens.saturating_add(usage.output_tokens);
        }
        Ok(response)
    }
}

struct LabProvider {
    provider: Box<dyn TranslationProvider>,
    provider_name: String,
    model: Option<String>,
}

pub(crate) fn run_qualify_provider(args: &[String], format: &OutputFormat) -> Result<()> {
    let corpus_path = positional(args, 0).ok_or_else(|| qualification_usage().to_string())?;
    let submission_path = positional(args, 1).ok_or_else(|| qualification_usage().to_string())?;
    let provider_name = flag_value(args, "--provider").ok_or_else(|| {
        "qualify-provider requires explicit --provider echo|openai|atria".to_string()
    })?;

    let corpus_text = fs::read_to_string(corpus_path)
        .map_err(|error| format!("failed to read {corpus_path}: {error}"))?;
    let corpus = LiteraryEvaluationCorpus::from_json(&corpus_text)
        .map_err(|error| format!("invalid qualification corpus: {error}"))?;

    let max_cases = flag_value(args, "--max-cases")
        .map(|value| {
            value
                .parse::<usize>()
                .map_err(|error| format!("invalid --max-cases '{value}': {error}"))
        })
        .transpose()?
        .unwrap_or(corpus.cases.len());
    if max_cases == 0 {
        return Err("--max-cases must be at least 1".to_string());
    }

    let lab = configured_lab_provider(provider_name, flag_value(args, "--model"))?;
    let selected_cases = max_cases.min(corpus.cases.len());
    let (submission, telemetry) = generate_submission(
        &corpus,
        lab.provider.as_ref(),
        selected_cases,
        qualification_system_id(&lab.provider_name, lab.model.as_deref()),
    )?;

    let json = serde_json::to_string_pretty(&submission)
        .map_err(|error| format!("failed to serialize qualification submission: {error}"))?;
    fs::write(submission_path, &json)
        .map_err(|error| format!("failed to write {submission_path}: {error}"))?;

    let mut evaluated_corpus = corpus.clone();
    evaluated_corpus.cases.truncate(selected_cases);
    let benchmark = evaluate_submission(&evaluated_corpus, &submission)
        .map_err(|error| format!("qualification benchmark failed: {error}"))?;

    let report = ProviderQualificationReport {
        schema_version: 1,
        corpus_id: corpus.corpus_id.clone(),
        provider: lab.provider_name,
        model: lab.model,
        cases_generated: selected_cases,
        corpus_cases: corpus.cases.len(),
        complete_corpus: selected_cases == corpus.cases.len(),
        pipeline: "default-literary-v1".to_string(),
        submission_file: submission_path.to_string(),
        deterministic_anchor_passed: benchmark.anchor_passed,
        deterministic_anchor_total: benchmark.anchor_total,
        deterministic_anchor_pass_rate: benchmark.anchor_pass_rate,
        provider_requests: telemetry.request_count,
        usage_observed_requests: telemetry.usage_observed_requests,
        token_usage_complete: telemetry.request_count > 0
            && telemetry.usage_observed_requests == telemetry.request_count,
        input_tokens: (telemetry.usage_observed_requests > 0).then_some(telemetry.input_tokens),
        output_tokens: (telemetry.usage_observed_requests > 0).then_some(telemetry.output_tokens),
        total_provider_latency_ms: telemetry.total_latency_ms,
        human_review_required: true,
        production_admission: "not_granted",
        notes: vec![
            "Phase 31 qualification never changes the production provider selector.",
            "Deterministic anchors are challenge evidence, not a literary-quality score.",
            "Human blind review remains required before any provider can be proposed for production.",
            "Use only rights-safe qualification corpora; do not use private manuscripts in this command.",
        ],
    };

    match format {
        OutputFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(&report)
                .map_err(|error| format!("failed to serialize qualification report: {error}"))?
        ),
        OutputFormat::Text => {
            println!("Provider qualification (research-only)");
            println!("provider: {}", report.provider);
            println!(
                "model: {}",
                report.model.as_deref().unwrap_or("none / deterministic")
            );
            println!(
                "cases: {}/{}{}",
                report.cases_generated,
                report.corpus_cases,
                if report.complete_corpus {
                    ""
                } else {
                    " (partial)"
                }
            );
            println!(
                "deterministic anchors: {}/{} ({:.1}%)",
                report.deterministic_anchor_passed,
                report.deterministic_anchor_total,
                report.deterministic_anchor_pass_rate * 100.0
            );
            println!(
                "provider calls: {} · measured latency: {} ms",
                report.provider_requests, report.total_provider_latency_ms
            );
            if let (Some(input_tokens), Some(output_tokens)) =
                (report.input_tokens, report.output_tokens)
            {
                println!(
                    "token usage: {input_tokens} input · {output_tokens} output · complete={}",
                    report.token_usage_complete
                );
            } else {
                println!("token usage: unavailable from this provider");
            }
            println!("submission: {}", report.submission_file);
            println!("production admission: NOT GRANTED — human blind review required");
        }
    }

    Ok(())
}

pub(crate) fn run_blind_compare(args: &[String], format: &OutputFormat) -> Result<()> {
    let corpus_path = args
        .first()
        .ok_or_else(|| blind_compare_usage().to_string())?;
    let first_path = args
        .get(1)
        .ok_or_else(|| blind_compare_usage().to_string())?;
    let second_path = args
        .get(2)
        .ok_or_else(|| blind_compare_usage().to_string())?;
    let bundle_path = args
        .get(3)
        .ok_or_else(|| blind_compare_usage().to_string())?;
    let key_path = args
        .get(4)
        .ok_or_else(|| blind_compare_usage().to_string())?;

    let corpus = LiteraryEvaluationCorpus::from_json(
        &fs::read_to_string(corpus_path)
            .map_err(|error| format!("failed to read {corpus_path}: {error}"))?,
    )
    .map_err(|error| format!("invalid comparison corpus: {error}"))?;
    let first = CandidateSubmission::from_json(
        &fs::read_to_string(first_path)
            .map_err(|error| format!("failed to read {first_path}: {error}"))?,
    )
    .map_err(|error| format!("invalid first submission: {error}"))?;
    let second = CandidateSubmission::from_json(
        &fs::read_to_string(second_path)
            .map_err(|error| format!("failed to read {second_path}: {error}"))?,
    )
    .map_err(|error| format!("invalid second submission: {error}"))?;

    // Completeness/identity validation is reused from Phase 21 before blinding.
    evaluate_submission(&corpus, &first)
        .map_err(|error| format!("first submission is not comparable: {error}"))?;
    evaluate_submission(&corpus, &second)
        .map_err(|error| format!("second submission is not comparable: {error}"))?;

    let (bundle, key) = build_blind_comparison(&corpus, &first, &second)?;
    fs::write(
        bundle_path,
        serde_json::to_string_pretty(&bundle)
            .map_err(|error| format!("failed to serialize blind comparison: {error}"))?,
    )
    .map_err(|error| format!("failed to write {bundle_path}: {error}"))?;
    fs::write(
        key_path,
        serde_json::to_string_pretty(&key)
            .map_err(|error| format!("failed to serialize comparison key: {error}"))?,
    )
    .map_err(|error| format!("failed to write {key_path}: {error}"))?;

    match format {
        OutputFormat::Json => println!(
            "{}",
            serde_json::json!({
                "corpus_id": corpus.corpus_id,
                "cases": bundle.cases.len(),
                "blind_bundle": bundle_path,
                "reveal_key": key_path,
                "automatic_winner": null,
                "human_review_required": true
            })
        ),
        OutputFormat::Text => {
            println!("Blind provider comparison prepared.");
            println!("cases: {}", bundle.cases.len());
            println!("review bundle: {bundle_path}");
            println!("reveal key: {key_path}");
            println!(
                "automatic winner: none — keep the reveal key separate until review is complete"
            );
        }
    }
    Ok(())
}

fn blind_compare_usage() -> &'static str {
    "usage: literary-engine blind-compare <corpus.json> <submission-a.json> <submission-b.json> <blind-bundle.json> <reveal-key.json> [--format json]"
}

fn build_blind_comparison(
    corpus: &LiteraryEvaluationCorpus,
    first: &CandidateSubmission,
    second: &CandidateSubmission,
) -> Result<(BlindComparisonBundle, BlindComparisonKey)> {
    if first.corpus_id != corpus.corpus_id || second.corpus_id != corpus.corpus_id {
        return Err("blind comparison submissions must target the same corpus".to_string());
    }
    if first.system_id == second.system_id {
        return Err("blind comparison requires two distinct system_id values".to_string());
    }

    let first_outputs = first
        .outputs
        .iter()
        .map(|output| (output.case_id.as_str(), output.translation.as_str()))
        .collect::<BTreeMap<_, _>>();
    let second_outputs = second
        .outputs
        .iter()
        .map(|output| (output.case_id.as_str(), output.translation.as_str()))
        .collect::<BTreeMap<_, _>>();

    let mut cases = Vec::with_capacity(corpus.cases.len());
    let mut assignments = Vec::with_capacity(corpus.cases.len());
    for (index, case) in corpus.cases.iter().enumerate() {
        let first_text = first_outputs
            .get(case.id.as_str())
            .ok_or_else(|| format!("first submission is missing case '{}'", case.id))?;
        let second_text = second_outputs
            .get(case.id.as_str())
            .ok_or_else(|| format!("second submission is missing case '{}'", case.id))?;
        let swap = !index.is_multiple_of(2);
        let (candidate_a, candidate_b, a_system, b_system) = if swap {
            (
                (*second_text).to_string(),
                (*first_text).to_string(),
                second.system_id.clone(),
                first.system_id.clone(),
            )
        } else {
            (
                (*first_text).to_string(),
                (*second_text).to_string(),
                first.system_id.clone(),
                second.system_id.clone(),
            )
        };
        cases.push(BlindComparisonCase {
            case_id: case.id.clone(),
            title: case.title.clone(),
            source: case.source.clone(),
            context_before: case.context_before.clone(),
            context_after: case.context_after.clone(),
            dimensions: case.dimensions.iter().copied().collect(),
            candidate_a,
            candidate_b,
        });
        assignments.push(BlindAssignment {
            case_id: case.id.clone(),
            candidate_a_system: a_system,
            candidate_b_system: b_system,
        });
    }

    Ok((
        BlindComparisonBundle {
            schema_version: 1,
            corpus_id: corpus.corpus_id.clone(),
            cases,
            instructions: vec![
                "Keep the reveal key closed until every judgment is recorded.",
                "Compare Candidate A and Candidate B for the listed literary dimensions and overall reading quality.",
                "Do not infer model identity from style; judge only the supplied source/context and translations.",
                "Record ties when differences are not meaningful rather than forcing a winner.",
            ],
        },
        BlindComparisonKey {
            schema_version: 1,
            corpus_id: corpus.corpus_id.clone(),
            system_one: first.system_id.clone(),
            system_two: second.system_id.clone(),
            assignments,
        },
    ))
}

fn qualification_usage() -> &'static str {
    "usage: literary-engine qualify-provider <corpus.json> <submission.json> --provider echo|openai|atria [--model <id>] [--max-cases <n>] [--format json]"
}

fn configured_lab_provider(provider: &str, model: Option<&str>) -> Result<LabProvider> {
    match provider.trim().to_ascii_lowercase().as_str() {
        "echo" => Ok(LabProvider {
            provider: Box::new(EchoProvider),
            provider_name: "echo".to_string(),
            model: None,
        }),
        "openai" => {
            let configured = match model.map(str::trim).filter(|value| !value.is_empty()) {
                Some(model) => {
                    let api_key = env::var("OPENAI_API_KEY")
                        .map_err(|_| "OPENAI_API_KEY is not configured".to_string())?;
                    OpenAIProvider::new(api_key, model).map_err(|error| error.to_string())?
                }
                None => OpenAIProvider::from_env().map_err(|error| error.to_string())?,
            };
            let resolved_model = Some(
                model
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(ToOwned::to_owned)
                    .or_else(|| env::var("OPENAI_MODEL").ok())
                    .unwrap_or_else(|| "gpt-5.6".to_string()),
            );
            Ok(LabProvider {
                provider: Box::new(configured),
                provider_name: "openai".to_string(),
                model: resolved_model,
            })
        }
        "atria" => {
            let configured = match model.map(str::trim).filter(|value| !value.is_empty()) {
                Some(model) => {
                    let api_key = env::var("ATRIA_API_KEY")
                        .map_err(|_| "ATRIA_API_KEY is not configured".to_string())?;
                    AtriaProvider::new(api_key, model).map_err(|error| error.to_string())?
                }
                None => AtriaProvider::from_env().map_err(|error| error.to_string())?,
            };
            let resolved_model = Some(configured.model().to_string());
            Ok(LabProvider {
                provider: Box::new(configured),
                provider_name: "atria".to_string(),
                model: resolved_model,
            })
        }
        other => Err(format!(
            "unsupported qualification provider '{other}'; expected echo, openai, or atria"
        )),
    }
}

fn qualification_system_id(provider: &str, model: Option<&str>) -> String {
    format!(
        "phase31:{}:{}:default-literary-v1",
        provider,
        model.unwrap_or("deterministic")
    )
}

fn generate_submission(
    corpus: &LiteraryEvaluationCorpus,
    provider: &dyn TranslationProvider,
    max_cases: usize,
    system_id: String,
) -> Result<(CandidateSubmission, QualificationTelemetry)> {
    let pipeline = TranslationPipeline::default_literary_pipeline();
    let observed = ObservedProvider::new(provider);
    let outputs = corpus
        .cases
        .iter()
        .take(max_cases)
        .map(|case| {
            let context = qualification_context(
                case.context_before.as_deref(),
                case.context_after.as_deref(),
            );
            let output = pipeline
                .execute(
                    &observed,
                    PipelineInput {
                        source_text: case.source.clone(),
                        target_language: corpus.target_language.clone(),
                        context,
                    },
                )
                .map_err(|error| {
                    format!(
                        "provider qualification failed for case '{}': {error}",
                        case.id
                    )
                })?;
            Ok(CandidateOutput {
                case_id: case.id.clone(),
                translation: output.quality_review,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok((
        CandidateSubmission {
            schema_version: literary_evaluation_engine::SUBMISSION_SCHEMA_VERSION,
            corpus_id: corpus.corpus_id.clone(),
            system_id,
            outputs,
        },
        observed.snapshot(),
    ))
}

fn qualification_context(before: Option<&str>, after: Option<&str>) -> String {
    let mut sections = vec![
        "RIGHTS-SAFE PROVIDER QUALIFICATION CASE. Translate only the PASSAGE supplied by the pipeline. Context is evidence for voice, register, subtext, terminology, and continuity; do not translate the context itself."
            .to_string(),
    ];
    if let Some(before) = before.filter(|value| !value.trim().is_empty()) {
        sections.push(format!("CONTEXT BEFORE\n{before}"));
    }
    if let Some(after) = after.filter(|value| !value.trim().is_empty()) {
        sections.push(format!("CONTEXT AFTER\n{after}"));
    }
    sections.join("\n\n")
}

fn positional(args: &[String], wanted: usize) -> Option<&str> {
    let flags_with_values = ["--provider", "--model", "--max-cases"];
    let mut values = Vec::new();
    let mut index = 0usize;
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

fn flag_value<'a>(args: &'a [String], flag: &str) -> Option<&'a str> {
    args.windows(2)
        .find(|pair| pair[0] == flag)
        .map(|pair| pair[1].as_str())
}

#[cfg(test)]
mod tests {
    use super::*;
    use literary_evaluation_engine::{
        AnchorExpectation, AnchorMode, CorpusProvenance, EvaluationCase, EvaluationDimension,
        CORPUS_SCHEMA_VERSION,
    };
    use std::collections::BTreeSet;

    fn corpus() -> LiteraryEvaluationCorpus {
        LiteraryEvaluationCorpus {
            schema_version: CORPUS_SCHEMA_VERSION,
            corpus_id: "phase31-test".into(),
            title: "Phase 31 Test".into(),
            source_language: "en".into(),
            target_language: "fa".into(),
            provenance: CorpusProvenance {
                owner: "project".into(),
                license: "CC0-1.0".into(),
                rights_safe: true,
                creation_note: "synthetic".into(),
            },
            cases: vec![EvaluationCase {
                id: "case-1".into(),
                title: "Context".into(),
                source: "I did not answer.".into(),
                reference: "جواب ندادم.".into(),
                context_before: Some("She waited.".into()),
                context_after: None,
                dimensions: BTreeSet::from([EvaluationDimension::SemanticFidelity]),
                anchors: vec![AnchorExpectation {
                    id: "negation".into(),
                    dimension: EvaluationDimension::SemanticFidelity,
                    mode: AnchorMode::AnyPresent,
                    values: vec!["ندادم".into()],
                    rationale: "keep polarity".into(),
                }],
                contrastive_variants: Vec::new(),
            }],
        }
    }

    #[test]
    fn echo_can_exercise_qualification_path_without_network() {
        let corpus = corpus();
        let (submission, telemetry) = generate_submission(
            &corpus,
            &EchoProvider,
            corpus.cases.len(),
            qualification_system_id("echo", None),
        )
        .unwrap();
        assert_eq!(submission.outputs.len(), 1);
        assert_eq!(submission.outputs[0].translation, "I did not answer.");
        assert!(submission.system_id.contains("default-literary-v1"));
        assert_eq!(telemetry.request_count, 3);
        assert_eq!(telemetry.usage_observed_requests, 0);
    }

    #[test]
    fn blind_comparison_counterbalances_system_labels_and_hides_ids_from_bundle() {
        let corpus = LiteraryEvaluationCorpus {
            cases: vec![
                corpus().cases[0].clone(),
                EvaluationCase {
                    id: "case-2".into(),
                    title: "Second".into(),
                    source: "Wait.".into(),
                    reference: "صبر کن.".into(),
                    context_before: None,
                    context_after: None,
                    dimensions: BTreeSet::from([EvaluationDimension::Readability]),
                    anchors: Vec::new(),
                    contrastive_variants: Vec::new(),
                },
            ],
            ..corpus()
        };
        let first = CandidateSubmission {
            schema_version: literary_evaluation_engine::SUBMISSION_SCHEMA_VERSION,
            corpus_id: corpus.corpus_id.clone(),
            system_id: "system-one".into(),
            outputs: vec![
                CandidateOutput {
                    case_id: "case-1".into(),
                    translation: "اول".into(),
                },
                CandidateOutput {
                    case_id: "case-2".into(),
                    translation: "یک".into(),
                },
            ],
        };
        let second = CandidateSubmission {
            schema_version: literary_evaluation_engine::SUBMISSION_SCHEMA_VERSION,
            corpus_id: corpus.corpus_id.clone(),
            system_id: "system-two".into(),
            outputs: vec![
                CandidateOutput {
                    case_id: "case-1".into(),
                    translation: "دوم".into(),
                },
                CandidateOutput {
                    case_id: "case-2".into(),
                    translation: "دو".into(),
                },
            ],
        };

        let (bundle, key) = build_blind_comparison(&corpus, &first, &second).unwrap();
        assert_eq!(bundle.cases[0].candidate_a, "اول");
        assert_eq!(bundle.cases[1].candidate_a, "دو");
        let blind_json = serde_json::to_string(&bundle).unwrap();
        assert!(!blind_json.contains("system-one"));
        assert!(!blind_json.contains("system-two"));
        assert_eq!(key.assignments[0].candidate_a_system, "system-one");
        assert_eq!(key.assignments[1].candidate_a_system, "system-two");
    }

    #[test]
    fn qualification_context_keeps_neighbors_separate_from_passage() {
        let context = qualification_context(Some("before"), Some("after"));
        assert!(context.contains("CONTEXT BEFORE\nbefore"));
        assert!(context.contains("CONTEXT AFTER\nafter"));
        assert!(context.contains("do not translate the context itself"));
    }
}
