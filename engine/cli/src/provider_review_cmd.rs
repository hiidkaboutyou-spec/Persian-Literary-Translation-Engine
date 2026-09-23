use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

type Result<T> = std::result::Result<T, String>;

const LEDGER_SCHEMA_VERSION: u32 = 1;
const DOSSIER_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Deserialize)]
struct BlindComparisonBundleInput {
    schema_version: u32,
    corpus_id: String,
    cases: Vec<BlindComparisonCaseInput>,
}

#[derive(Debug, Deserialize)]
struct BlindComparisonCaseInput {
    case_id: String,
}

#[derive(Debug, Deserialize)]
struct BlindComparisonKeyInput {
    schema_version: u32,
    corpus_id: String,
    system_one: String,
    system_two: String,
    assignments: Vec<BlindAssignmentInput>,
}

#[derive(Debug, Deserialize)]
struct BlindAssignmentInput {
    case_id: String,
    candidate_a_system: String,
    candidate_b_system: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ReviewDecision {
    Pending,
    CandidateA,
    CandidateB,
    Tie,
    Defer,
}

impl ReviewDecision {
    fn parse(value: &str) -> Result<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "a" | "candidate-a" | "candidate_a" => Ok(Self::CandidateA),
            "b" | "candidate-b" | "candidate_b" => Ok(Self::CandidateB),
            "tie" => Ok(Self::Tie),
            "defer" | "cannot-judge" | "cannot_judge" => Ok(Self::Defer),
            other => Err(format!(
                "unsupported decision '{other}'; expected a, b, tie, or defer"
            )),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct CriterionNotes {
    #[serde(skip_serializing_if = "Option::is_none")]
    adequacy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    voice_style: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    cultural_pragmatic_fit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    continuity: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BlindReviewLedger {
    schema_version: u32,
    corpus_id: String,
    bundle_fingerprint: String,
    reviewer: String,
    cases: Vec<BlindReviewCase>,
    instructions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BlindReviewCase {
    case_id: String,
    decision: ReviewDecision,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
    #[serde(default)]
    notes: CriterionNotes,
}

#[derive(Debug, Serialize)]
struct ProviderReviewDossier {
    schema_version: u32,
    corpus_id: String,
    bundle_fingerprint: String,
    reviewers: Vec<String>,
    review_ledgers: usize,
    cases_total: usize,
    judgments_total: usize,
    preference_judgments: usize,
    ties: usize,
    deferred: usize,
    cases_with_multiple_reviews: usize,
    unanimous_cases: usize,
    disagreement_cases: usize,
    system_preference_counts: BTreeMap<String, usize>,
    system_one: String,
    system_two: String,
    automatic_winner: Option<String>,
    human_comparative_evidence_only: bool,
    production_admission: &'static str,
    requires_explicit_admission_decision: bool,
    reveal_binding: &'static str,
    notes: Vec<&'static str>,
}

fn usage() -> &'static str {
    "Usage:\n  literary-engine blind-review init <blind-bundle.json> <ledger.json> --reviewer <id>\n  literary-engine blind-review record <ledger.json> <case-id> <a|b|tie|defer> --reason <text> [--adequacy-note <text>] [--voice-note <text>] [--culture-note <text>] [--continuity-note <text>]\n  literary-engine blind-review dossier <reveal-key.json> <dossier.json> <ledger.json> [ledger2.json ...]\n  literary-engine blind-review verify <blind-bundle.json> <reveal-key.json> <dossier.json> <ledger.json> [ledger2.json ...]\n\nThe review ledger never receives the reveal key. Verification reconstructs the dossier from local inputs without writing an artifact or granting production admission."
}

pub(crate) fn run_provider_review(args: &[String]) -> Result<()> {
    let command = args.first().map(String::as_str).unwrap_or_default();
    match command {
        "init" => run_init(&args[1..]),
        "record" => run_record(&args[1..]),
        "dossier" => run_dossier(&args[1..]),
        "verify" => run_verify(&args[1..]),
        "--help" | "-h" | "help" => {
            println!("{}", usage());
            Ok(())
        }
        _ => Err(usage().to_string()),
    }
}

fn run_init(args: &[String]) -> Result<()> {
    let bundle_path = positional(args, 0, &["--reviewer"]).ok_or_else(|| usage().to_string())?;
    let ledger_path = positional(args, 1, &["--reviewer"]).ok_or_else(|| usage().to_string())?;
    let reviewer = flag_value(args, "--reviewer")
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "init requires a non-empty --reviewer value".to_string())?;

    ensure_output_distinct(ledger_path, &[bundle_path])?;
    let bundle_text = fs::read_to_string(bundle_path)
        .map_err(|error| format!("failed to read {bundle_path}: {error}"))?;
    let bundle: BlindComparisonBundleInput = serde_json::from_str(&bundle_text)
        .map_err(|error| format!("invalid blind comparison bundle: {error}"))?;
    validate_bundle(&bundle)?;

    let ledger = BlindReviewLedger {
        schema_version: LEDGER_SCHEMA_VERSION,
        corpus_id: bundle.corpus_id,
        bundle_fingerprint: fingerprint(bundle_text.as_bytes()),
        reviewer: reviewer.to_string(),
        cases: bundle
            .cases
            .into_iter()
            .map(|case| BlindReviewCase {
                case_id: case.case_id,
                decision: ReviewDecision::Pending,
                reason: None,
                notes: CriterionNotes::default(),
            })
            .collect(),
        instructions: vec![
            "Keep the reveal key physically/logically separate until all judgments are recorded."
                .to_string(),
            "Judge Candidate A versus Candidate B from source/context only; do not infer provider identity."
                .to_string(),
            "Use tie when no meaningful preference exists; use defer when the case cannot be judged reliably."
                .to_string(),
            "Record a concrete reason for every judgment so later disagreement is auditable."
                .to_string(),
        ],
    };

    write_json_atomic(Path::new(ledger_path), &ledger)?;
    println!("blind review ledger initialized: {ledger_path}");
    println!("cases: {}", ledger.cases.len());
    println!("reviewer: {}", ledger.reviewer);
    println!("reveal key consumed: no");
    Ok(())
}

fn run_record(args: &[String]) -> Result<()> {
    let flags = [
        "--reason",
        "--adequacy-note",
        "--voice-note",
        "--culture-note",
        "--continuity-note",
    ];
    let ledger_path = positional(args, 0, &flags).ok_or_else(|| usage().to_string())?;
    let case_id = positional(args, 1, &flags).ok_or_else(|| usage().to_string())?;
    let decision_text = positional(args, 2, &flags).ok_or_else(|| usage().to_string())?;
    let decision = ReviewDecision::parse(decision_text)?;
    let reason = flag_value(args, "--reason")
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "record requires a non-empty --reason for every judgment".to_string())?;

    let mut ledger = read_ledger(ledger_path)?;
    validate_ledger(&ledger)?;
    let entry = ledger
        .cases
        .iter_mut()
        .find(|entry| entry.case_id == case_id)
        .ok_or_else(|| format!("ledger does not contain case '{case_id}'"))?;

    entry.decision = decision;
    entry.reason = Some(reason.to_string());
    entry.notes = CriterionNotes {
        adequacy: optional_flag(args, "--adequacy-note"),
        voice_style: optional_flag(args, "--voice-note"),
        cultural_pragmatic_fit: optional_flag(args, "--culture-note"),
        continuity: optional_flag(args, "--continuity-note"),
    };

    write_json_atomic(Path::new(ledger_path), &ledger)?;
    let remaining = ledger
        .cases
        .iter()
        .filter(|entry| entry.decision == ReviewDecision::Pending)
        .count();
    println!("recorded {case_id}: {}", decision_label(decision));
    println!("pending: {remaining}/{}", ledger.cases.len());
    Ok(())
}

fn run_dossier(args: &[String]) -> Result<()> {
    let key_path = args.first().ok_or_else(|| usage().to_string())?;
    let dossier_path = args.get(1).ok_or_else(|| usage().to_string())?;
    let ledger_paths = args
        .get(2..)
        .filter(|paths| !paths.is_empty())
        .ok_or_else(|| "dossier requires at least one completed blind review ledger".to_string())?;

    let mut protected = Vec::with_capacity(ledger_paths.len() + 1);
    protected.push(key_path.as_str());
    protected.extend(ledger_paths.iter().map(String::as_str));
    ensure_output_distinct(dossier_path, &protected)?;
    ensure_unique_inputs(&protected)?;

    let key_text = fs::read_to_string(key_path)
        .map_err(|error| format!("failed to read {key_path}: {error}"))?;
    let key: BlindComparisonKeyInput =
        serde_json::from_str(&key_text).map_err(|error| format!("invalid reveal key: {error}"))?;
    validate_key(&key)?;

    let ledgers = ledger_paths
        .iter()
        .map(|path| read_ledger(path))
        .collect::<Result<Vec<_>>>()?;
    let dossier = build_dossier(&key, &ledgers)?;
    write_json_atomic(Path::new(dossier_path), &dossier)?;

    println!("provider review dossier written: {dossier_path}");
    println!("reviewers: {}", dossier.reviewers.len());
    println!("judgments: {}", dossier.judgments_total);
    println!(
        "agreement: {} unanimous · {} disagreement cases",
        dossier.unanimous_cases, dossier.disagreement_cases
    );
    println!("automatic winner: none");
    println!("production admission: NOT GRANTED");
    Ok(())
}

fn run_verify(args: &[String]) -> Result<()> {
    let [bundle_path, key_path, dossier_path, ledger_paths @ ..] = args else {
        return Err(usage().to_string());
    };
    if ledger_paths.is_empty() {
        return Err("verify requires at least one completed blind review ledger".into());
    }
    ensure_unique_inputs(&args.iter().map(String::as_str).collect::<Vec<_>>())?;

    let bundle_bytes =
        fs::read(bundle_path).map_err(|error| format!("failed to read {bundle_path}: {error}"))?;
    let bundle: BlindComparisonBundleInput = serde_json::from_slice(&bundle_bytes)
        .map_err(|error| format!("invalid blind comparison bundle: {error}"))?;
    validate_bundle(&bundle)?;

    let key_bytes =
        fs::read(key_path).map_err(|error| format!("failed to read {key_path}: {error}"))?;
    let key: BlindComparisonKeyInput = serde_json::from_slice(&key_bytes)
        .map_err(|error| format!("invalid reveal key: {error}"))?;
    validate_key(&key)?;
    let bundle_cases = bundle
        .cases
        .iter()
        .map(|case| case.case_id.as_str())
        .collect::<BTreeSet<_>>();
    let key_cases = key
        .assignments
        .iter()
        .map(|case| case.case_id.as_str())
        .collect::<BTreeSet<_>>();
    if bundle.corpus_id != key.corpus_id || bundle_cases != key_cases {
        return Err("blind bundle and reveal key have different corpus or case ids".into());
    }

    let ledgers = ledger_paths
        .iter()
        .map(|path| read_ledger(path))
        .collect::<Result<Vec<_>>>()?;
    if ledgers
        .iter()
        .any(|ledger| ledger.bundle_fingerprint != fingerprint(&bundle_bytes))
    {
        return Err("review ledger fingerprint differs from the supplied blind bundle".into());
    }
    let expected = serde_json::to_value(build_dossier(&key, &ledgers)?)
        .map_err(|error| format!("failed to reconstruct dossier: {error}"))?;
    let actual_bytes = fs::read(dossier_path)
        .map_err(|error| format!("failed to read {dossier_path}: {error}"))?;
    let actual: serde_json::Value = serde_json::from_slice(&actual_bytes)
        .map_err(|error| format!("invalid review dossier: {error}"))?;
    if expected != actual {
        return Err(
            "review dossier differs from the supplied blind bundle, reveal key and completed ledgers"
                .into(),
        );
    }
    println!("review dossier verified against supplied local evidence: {dossier_path}");
    println!("production admission: NOT GRANTED");
    Ok(())
}

fn validate_bundle(bundle: &BlindComparisonBundleInput) -> Result<()> {
    if bundle.schema_version != 1 {
        return Err(format!(
            "unsupported blind bundle schema {}; expected 1",
            bundle.schema_version
        ));
    }
    if bundle.corpus_id.trim().is_empty() {
        return Err("blind bundle corpus_id is empty".to_string());
    }
    if bundle.cases.is_empty() {
        return Err("blind bundle contains no cases".to_string());
    }
    let mut ids = BTreeSet::new();
    for case in &bundle.cases {
        if case.case_id.trim().is_empty() {
            return Err("blind bundle contains an empty case_id".to_string());
        }
        if !ids.insert(case.case_id.as_str()) {
            return Err(format!("blind bundle duplicates case '{}'", case.case_id));
        }
    }
    Ok(())
}

fn validate_key(key: &BlindComparisonKeyInput) -> Result<()> {
    if key.schema_version != 1 {
        return Err(format!(
            "unsupported reveal-key schema {}; expected 1",
            key.schema_version
        ));
    }
    if key.corpus_id.trim().is_empty() {
        return Err("reveal key corpus_id is empty".to_string());
    }
    if key.system_one.trim().is_empty() || key.system_two.trim().is_empty() {
        return Err("reveal key contains an empty system id".to_string());
    }
    if key.system_one == key.system_two {
        return Err("reveal key must describe two distinct systems".to_string());
    }
    if key.assignments.is_empty() {
        return Err("reveal key contains no assignments".to_string());
    }
    let allowed = BTreeSet::from([key.system_one.as_str(), key.system_two.as_str()]);
    let mut ids = BTreeSet::new();
    for assignment in &key.assignments {
        if !ids.insert(assignment.case_id.as_str()) {
            return Err(format!(
                "reveal key duplicates case '{}'",
                assignment.case_id
            ));
        }
        if assignment.candidate_a_system == assignment.candidate_b_system {
            return Err(format!(
                "reveal key maps both candidates to one system for case '{}'",
                assignment.case_id
            ));
        }
        if !allowed.contains(assignment.candidate_a_system.as_str())
            || !allowed.contains(assignment.candidate_b_system.as_str())
        {
            return Err(format!(
                "reveal key references an unknown system for case '{}'",
                assignment.case_id
            ));
        }
    }
    Ok(())
}

fn validate_ledger(ledger: &BlindReviewLedger) -> Result<()> {
    if ledger.schema_version != LEDGER_SCHEMA_VERSION {
        return Err(format!(
            "unsupported ledger schema {}; expected {LEDGER_SCHEMA_VERSION}",
            ledger.schema_version
        ));
    }
    if ledger.corpus_id.trim().is_empty()
        || ledger.bundle_fingerprint.trim().is_empty()
        || ledger.reviewer.trim().is_empty()
    {
        return Err("ledger identity fields must not be empty".to_string());
    }
    if ledger.cases.is_empty() {
        return Err("ledger contains no cases".to_string());
    }
    let mut ids = BTreeSet::new();
    for case in &ledger.cases {
        if !ids.insert(case.case_id.as_str()) {
            return Err(format!("ledger duplicates case '{}'", case.case_id));
        }
        match case.decision {
            ReviewDecision::Pending => {
                if case.reason.is_some() {
                    return Err(format!(
                        "pending case '{}' must not carry a judgment reason",
                        case.case_id
                    ));
                }
            }
            _ => {
                if case
                    .reason
                    .as_deref()
                    .map(str::trim)
                    .filter(|v| !v.is_empty())
                    .is_none()
                {
                    return Err(format!(
                        "completed case '{}' is missing a judgment reason",
                        case.case_id
                    ));
                }
            }
        }
    }
    Ok(())
}

fn build_dossier(
    key: &BlindComparisonKeyInput,
    ledgers: &[BlindReviewLedger],
) -> Result<ProviderReviewDossier> {
    validate_key(key)?;
    if ledgers.is_empty() {
        return Err("at least one ledger is required".to_string());
    }

    let assignment_by_case = key
        .assignments
        .iter()
        .map(|assignment| (assignment.case_id.as_str(), assignment))
        .collect::<BTreeMap<_, _>>();
    let expected_case_ids = assignment_by_case.keys().copied().collect::<BTreeSet<_>>();

    let first = &ledgers[0];
    validate_ledger(first)?;
    let expected_fingerprint = first.bundle_fingerprint.clone();
    let mut reviewers = BTreeSet::new();
    let mut decisions_by_case: BTreeMap<String, Vec<ReviewDecision>> = BTreeMap::new();
    let mut preference_counts = BTreeMap::from([
        (key.system_one.clone(), 0usize),
        (key.system_two.clone(), 0usize),
    ]);
    let mut judgments_total = 0usize;
    let mut preference_judgments = 0usize;
    let mut ties = 0usize;
    let mut deferred = 0usize;

    for ledger in ledgers {
        validate_ledger(ledger)?;
        if ledger.corpus_id != key.corpus_id {
            return Err(format!(
                "ledger for reviewer '{}' targets corpus '{}' but key targets '{}'",
                ledger.reviewer, ledger.corpus_id, key.corpus_id
            ));
        }
        if ledger.bundle_fingerprint != expected_fingerprint {
            return Err(format!(
                "ledger for reviewer '{}' was created from a different blind bundle",
                ledger.reviewer
            ));
        }
        if !reviewers.insert(ledger.reviewer.clone()) {
            return Err(format!(
                "reviewer '{}' appears more than once; duplicate ledgers would double-count judgments",
                ledger.reviewer
            ));
        }
        let actual_case_ids = ledger
            .cases
            .iter()
            .map(|case| case.case_id.as_str())
            .collect::<BTreeSet<_>>();
        if actual_case_ids != expected_case_ids {
            return Err(format!(
                "ledger for reviewer '{}' does not match reveal-key case ids",
                ledger.reviewer
            ));
        }
        if let Some(pending) = ledger
            .cases
            .iter()
            .find(|case| case.decision == ReviewDecision::Pending)
        {
            return Err(format!(
                "ledger for reviewer '{}' is incomplete; case '{}' is still pending",
                ledger.reviewer, pending.case_id
            ));
        }

        for case in &ledger.cases {
            judgments_total += 1;
            decisions_by_case
                .entry(case.case_id.clone())
                .or_default()
                .push(case.decision);
            let assignment = assignment_by_case
                .get(case.case_id.as_str())
                .expect("case-id equality checked above");
            match case.decision {
                ReviewDecision::CandidateA => {
                    *preference_counts
                        .get_mut(&assignment.candidate_a_system)
                        .expect("reveal-key system validated") += 1;
                    preference_judgments += 1;
                }
                ReviewDecision::CandidateB => {
                    *preference_counts
                        .get_mut(&assignment.candidate_b_system)
                        .expect("reveal-key system validated") += 1;
                    preference_judgments += 1;
                }
                ReviewDecision::Tie => ties += 1,
                ReviewDecision::Defer => deferred += 1,
                ReviewDecision::Pending => unreachable!("pending decisions rejected above"),
            }
        }
    }

    let mut cases_with_multiple_reviews = 0usize;
    let mut unanimous_cases = 0usize;
    let mut disagreement_cases = 0usize;
    for decisions in decisions_by_case.values() {
        let comparable = decisions
            .iter()
            .copied()
            .filter(|decision| *decision != ReviewDecision::Defer)
            .collect::<Vec<_>>();
        if comparable.len() >= 2 {
            cases_with_multiple_reviews += 1;
            let distinct = comparable.iter().copied().collect::<BTreeSet<_>>();
            if distinct.len() == 1 {
                unanimous_cases += 1;
            } else {
                disagreement_cases += 1;
            }
        }
    }

    Ok(ProviderReviewDossier {
        schema_version: DOSSIER_SCHEMA_VERSION,
        corpus_id: key.corpus_id.clone(),
        bundle_fingerprint: expected_fingerprint,
        reviewers: reviewers.into_iter().collect(),
        review_ledgers: ledgers.len(),
        cases_total: expected_case_ids.len(),
        judgments_total,
        preference_judgments,
        ties,
        deferred,
        cases_with_multiple_reviews,
        unanimous_cases,
        disagreement_cases,
        system_preference_counts: preference_counts,
        system_one: key.system_one.clone(),
        system_two: key.system_two.clone(),
        automatic_winner: None,
        human_comparative_evidence_only: true,
        production_admission: "not_granted",
        requires_explicit_admission_decision: true,
        reveal_binding: "phase31-key-schema-v1: corpus_id + exact case-id set; ledger bundle fingerprint binds reviewers to the same blind bundle",
        notes: vec![
            "Preference counts summarize recorded human comparative judgments; they are not an automatic provider-selection decision.",
            "Ties and deferrals remain first-class evidence and are not forced into a winner.",
            "Disagreement is reported rather than erased because literary translation can admit multiple defensible readings.",
            "Production admission remains separate and must include privacy/retention, cost, reliability, rights, and explicit authorization review.",
            "Do not use private manuscripts in this research workflow.",
        ],
    })
}

fn read_ledger(path: &str) -> Result<BlindReviewLedger> {
    let text =
        fs::read_to_string(path).map_err(|error| format!("failed to read {path}: {error}"))?;
    serde_json::from_str(&text).map_err(|error| format!("invalid review ledger {path}: {error}"))
}

fn write_json_atomic<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    if !parent.is_dir() {
        return Err(format!(
            "output directory does not exist: {}",
            parent.display()
        ));
    }
    let json = serde_json::to_vec_pretty(value)
        .map_err(|error| format!("failed to serialize {}: {error}", path.display()))?;
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("system clock error: {error}"))?
        .as_nanos();
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| {
            format!(
                "output path has no valid UTF-8 file name: {}",
                path.display()
            )
        })?;
    let temp_path = parent.join(format!(".{file_name}.{}.{}.tmp", std::process::id(), nonce));

    let write_result = (|| -> Result<()> {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp_path)
            .map_err(|error| format!("failed to create {}: {error}", temp_path.display()))?;
        file.write_all(&json)
            .map_err(|error| format!("failed to write {}: {error}", temp_path.display()))?;
        file.write_all(b"\n")
            .map_err(|error| format!("failed to finish {}: {error}", temp_path.display()))?;
        file.sync_all()
            .map_err(|error| format!("failed to sync {}: {error}", temp_path.display()))?;
        fs::rename(&temp_path, path).map_err(|error| {
            format!(
                "failed to atomically replace {} with {}: {error}",
                path.display(),
                temp_path.display()
            )
        })?;
        Ok(())
    })();

    if write_result.is_err() {
        let _ = fs::remove_file(&temp_path);
    }
    write_result
}

fn fingerprint(bytes: &[u8]) -> String {
    const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;
    let hash = bytes.iter().fold(FNV_OFFSET_BASIS, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(FNV_PRIME)
    });
    format!("fnv1a64-{hash:016x}")
}

fn normalized_path(path: &str) -> Result<PathBuf> {
    let path = Path::new(path);
    if path.exists() {
        return fs::canonicalize(path)
            .map_err(|error| format!("failed to resolve {}: {error}", path.display()));
    }
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        env::current_dir()
            .map_err(|error| format!("failed to resolve current directory: {error}"))?
            .join(path)
    };
    let parent = absolute.parent().unwrap_or_else(|| Path::new("."));
    if parent.exists() {
        let canonical_parent = fs::canonicalize(parent)
            .map_err(|error| format!("failed to resolve {}: {error}", parent.display()))?;
        if let Some(name) = absolute.file_name() {
            return Ok(canonical_parent.join(name));
        }
    }
    Ok(absolute)
}

fn ensure_output_distinct(output: &str, inputs: &[&str]) -> Result<()> {
    let output = normalized_path(output)?;
    for input in inputs {
        if output == normalized_path(input)? {
            return Err(format!(
                "refusing destructive artifact collision: output '{}' resolves to input '{}'",
                output.display(),
                input
            ));
        }
    }
    Ok(())
}

fn ensure_unique_inputs(inputs: &[&str]) -> Result<()> {
    let mut seen = BTreeSet::new();
    for input in inputs {
        let normalized = normalized_path(input)?;
        if !seen.insert(normalized.clone()) {
            return Err(format!(
                "refusing duplicate input '{}'; it would double-count or alias review evidence",
                normalized.display()
            ));
        }
    }
    Ok(())
}

fn positional<'a>(
    args: &'a [String],
    wanted: usize,
    flags_with_values: &[&str],
) -> Option<&'a str> {
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

fn optional_flag(args: &[String], flag: &str) -> Option<String> {
    flag_value(args, flag)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn decision_label(decision: ReviewDecision) -> &'static str {
    match decision {
        ReviewDecision::Pending => "pending",
        ReviewDecision::CandidateA => "candidate_a",
        ReviewDecision::CandidateB => "candidate_b",
        ReviewDecision::Tie => "tie",
        ReviewDecision::Defer => "defer",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn key() -> BlindComparisonKeyInput {
        BlindComparisonKeyInput {
            schema_version: 1,
            corpus_id: "phase32-test".into(),
            system_one: "system-one".into(),
            system_two: "system-two".into(),
            assignments: vec![
                BlindAssignmentInput {
                    case_id: "case-1".into(),
                    candidate_a_system: "system-one".into(),
                    candidate_b_system: "system-two".into(),
                },
                BlindAssignmentInput {
                    case_id: "case-2".into(),
                    candidate_a_system: "system-two".into(),
                    candidate_b_system: "system-one".into(),
                },
            ],
        }
    }

    fn ledger(reviewer: &str, first: ReviewDecision, second: ReviewDecision) -> BlindReviewLedger {
        BlindReviewLedger {
            schema_version: 1,
            corpus_id: "phase32-test".into(),
            bundle_fingerprint: "fnv1a64-same".into(),
            reviewer: reviewer.into(),
            cases: vec![
                BlindReviewCase {
                    case_id: "case-1".into(),
                    decision: first,
                    reason: Some("reason one".into()),
                    notes: CriterionNotes::default(),
                },
                BlindReviewCase {
                    case_id: "case-2".into(),
                    decision: second,
                    reason: Some("reason two".into()),
                    notes: CriterionNotes::default(),
                },
            ],
            instructions: Vec::new(),
        }
    }

    #[test]
    fn dossier_reveals_counts_without_selecting_a_winner() {
        let dossier = build_dossier(
            &key(),
            &[ledger(
                "reviewer-1",
                ReviewDecision::CandidateA,
                ReviewDecision::CandidateA,
            )],
        )
        .unwrap();
        assert_eq!(dossier.system_preference_counts["system-one"], 1);
        assert_eq!(dossier.system_preference_counts["system-two"], 1);
        assert!(dossier.automatic_winner.is_none());
        assert_eq!(dossier.production_admission, "not_granted");
    }

    #[test]
    fn disagreement_is_reported_instead_of_collapsed() {
        let dossier = build_dossier(
            &key(),
            &[
                ledger(
                    "reviewer-1",
                    ReviewDecision::CandidateA,
                    ReviewDecision::Tie,
                ),
                ledger(
                    "reviewer-2",
                    ReviewDecision::CandidateB,
                    ReviewDecision::Tie,
                ),
            ],
        )
        .unwrap();
        assert_eq!(dossier.cases_with_multiple_reviews, 2);
        assert_eq!(dossier.unanimous_cases, 1);
        assert_eq!(dossier.disagreement_cases, 1);
    }

    #[test]
    fn incomplete_ledgers_cannot_be_unblinded_into_a_dossier() {
        let mut incomplete = ledger(
            "reviewer-1",
            ReviewDecision::CandidateA,
            ReviewDecision::CandidateB,
        );
        incomplete.cases[1].decision = ReviewDecision::Pending;
        incomplete.cases[1].reason = None;
        let error = build_dossier(&key(), &[incomplete]).unwrap_err();
        assert!(error.contains("still pending"));
    }

    #[test]
    fn duplicate_reviewer_ledgers_are_rejected() {
        let first = ledger(
            "reviewer-1",
            ReviewDecision::CandidateA,
            ReviewDecision::CandidateB,
        );
        let second = ledger(
            "reviewer-1",
            ReviewDecision::CandidateB,
            ReviewDecision::CandidateA,
        );
        let error = build_dossier(&key(), &[first, second]).unwrap_err();
        assert!(error.contains("appears more than once"));
    }

    #[test]
    fn artifact_collision_is_fail_closed() {
        let dir = tempdir().unwrap();
        let input = dir.path().join("bundle.json");
        fs::write(&input, "{}").unwrap();
        let input_text = input.to_string_lossy().to_string();
        let error = ensure_output_distinct(&input_text, &[&input_text]).unwrap_err();
        assert!(error.contains("destructive artifact collision"));
    }

    #[test]
    fn fingerprint_is_stable_and_sensitive() {
        assert_eq!(fingerprint(b"same"), fingerprint(b"same"));
        assert_ne!(fingerprint(b"same"), fingerprint(b"different"));
    }

    #[test]
    fn verify_reconstructs_dossier_and_rejects_changed_evidence() {
        let dir = tempdir().unwrap();
        let bundle = dir.path().join("bundle.json");
        let key = dir.path().join("key.json");
        let ledger = dir.path().join("ledger.json");
        let dossier = dir.path().join("dossier.json");
        let paths =
            [&bundle, &key, &dossier, &ledger].map(|path| path.to_string_lossy().into_owned());
        fs::write(
            &bundle,
            r#"{"schema_version":1,"corpus_id":"synthetic","cases":[{"case_id":"c1"}]}"#,
        )
        .unwrap();
        fs::write(
            &key,
            r#"{"schema_version":1,"corpus_id":"synthetic","system_one":"one","system_two":"two","assignments":[{"case_id":"c1","candidate_a_system":"one","candidate_b_system":"two"}]}"#,
        )
        .unwrap();
        run_provider_review(&[
            "init".into(),
            paths[0].clone(),
            paths[3].clone(),
            "--reviewer".into(),
            "reviewer".into(),
        ])
        .unwrap();
        run_provider_review(&[
            "record".into(),
            paths[3].clone(),
            "c1".into(),
            "a".into(),
            "--reason".into(),
            "Synthetic reason".into(),
        ])
        .unwrap();
        run_provider_review(&[
            "dossier".into(),
            paths[1].clone(),
            paths[2].clone(),
            paths[3].clone(),
        ])
        .unwrap();
        let verify = || {
            run_provider_review(&[
                "verify".into(),
                paths[0].clone(),
                paths[1].clone(),
                paths[2].clone(),
                paths[3].clone(),
            ])
        };
        verify().unwrap();

        let original_dossier = fs::read(&dossier).unwrap();
        let mut changed: serde_json::Value = serde_json::from_slice(&original_dossier).unwrap();
        changed["system_preference_counts"]["one"] = serde_json::json!(0);
        changed["system_preference_counts"]["two"] = serde_json::json!(1);
        fs::write(&dossier, serde_json::to_vec(&changed).unwrap()).unwrap();
        assert!(verify().unwrap_err().contains("differs"));
        fs::write(&dossier, &original_dossier).unwrap();

        let original_ledger = fs::read(&ledger).unwrap();
        let mut changed: serde_json::Value = serde_json::from_slice(&original_ledger).unwrap();
        changed["cases"][0]["decision"] = serde_json::json!("candidate_b");
        fs::write(&ledger, serde_json::to_vec(&changed).unwrap()).unwrap();
        assert!(verify().unwrap_err().contains("differs"));
        fs::write(&ledger, &original_ledger).unwrap();

        let original_key = fs::read(&key).unwrap();
        let mut changed: serde_json::Value = serde_json::from_slice(&original_key).unwrap();
        changed["assignments"][0]["candidate_a_system"] = serde_json::json!("two");
        changed["assignments"][0]["candidate_b_system"] = serde_json::json!("one");
        fs::write(&key, serde_json::to_vec(&changed).unwrap()).unwrap();
        assert!(verify().unwrap_err().contains("differs"));
        fs::write(&key, &original_key).unwrap();

        let original_bundle = fs::read(&bundle).unwrap();
        fs::write(&bundle, [original_bundle.as_slice(), b"\n"].concat()).unwrap();
        assert!(verify().unwrap_err().contains("fingerprint differs"));
        fs::write(&bundle, &original_bundle).unwrap();
        verify().unwrap();
    }

    #[test]
    fn verify_rejects_duplicate_paths_and_mismatched_bundle_key() {
        let dir = tempdir().unwrap();
        let bundle = dir.path().join("bundle.json");
        let key = dir.path().join("key.json");
        let dossier = dir.path().join("dossier.json");
        let ledger = dir.path().join("ledger.json");
        fs::write(
            &bundle,
            r#"{"schema_version":1,"corpus_id":"synthetic","cases":[{"case_id":"c1"}]}"#,
        )
        .unwrap();
        fs::write(
            &key,
            r#"{"schema_version":1,"corpus_id":"synthetic","system_one":"one","system_two":"two","assignments":[{"case_id":"c2","candidate_a_system":"one","candidate_b_system":"two"}]}"#,
        )
        .unwrap();
        let paths =
            [&bundle, &key, &dossier, &ledger].map(|path| path.to_string_lossy().into_owned());
        let args = [
            "verify".into(),
            paths[0].clone(),
            paths[1].clone(),
            paths[2].clone(),
            paths[3].clone(),
        ];
        assert!(run_provider_review(&args)
            .unwrap_err()
            .contains("different corpus or case ids"));
        let alias_args = [
            "verify".into(),
            paths[0].clone(),
            paths[1].clone(),
            paths[2].clone(),
            paths[2].clone(),
        ];
        assert!(run_provider_review(&alias_args)
            .unwrap_err()
            .contains("duplicate input"));
    }
}
