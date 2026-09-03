use crate::OutputFormat;
use character_engine::CharacterBible;
use chrono::Utc;
use document_engine::ingest_file;
use human_review_workflow::{
    build_promotion_plan, confidence_rank, ConflictResolution, DecisionAction, ReviewItem,
    ReviewKind, ReviewLedger, ReviewStatus, ReviewedValue,
};
use literary_intelligence_engine::{
    AnalysisCanon, DeterministicManuscriptAnalyzer, ManuscriptAnalyzer,
};
use memory_engine::glossary::Glossary;
use project_engine::review_store::{
    apply_promotion, load_character_bible_or_default, load_glossary_or_default, load_review_ledger,
    recover_pending_promotion, save_review_ledger, PromotionApplyResult, PromotionApplyStatus,
    ReviewStorePaths,
};
use serde::Serialize;
use std::collections::BTreeSet;

/// One human-approved/edited advanced literary finding, ready to enrich
/// translation context for the chapters its evidence belongs to.
///
/// Ordering note (never weakened by Phase 15):
/// human-approved canon > human-reviewed literary finding > deterministic
/// manuscript evidence > unreviewed model inference. Only items that have
/// passed the human review boundary are eligible here.
pub(crate) struct LiteraryContextLine {
    pub chapter_ids: Vec<String>,
    pub line: String,
}

/// Load reviewed (approved or edited) `Literary` items from the review ledger
/// configured via `LITERARY_ENGINE_REVIEW_FILE`, when present. Returns an
/// empty list when no ledger is configured or no eligible findings exist.
pub(crate) fn load_reviewed_literary_lines() -> Vec<LiteraryContextLine> {
    let Ok(path) = env::var("LITERARY_ENGINE_REVIEW_FILE") else {
        return Vec::new();
    };
    if path.trim().is_empty() {
        return Vec::new();
    }
    let Ok(ledger) = load_review_ledger(&path) else {
        return Vec::new();
    };
    ledger
        .items
        .iter()
        .filter(|item| item.kind == ReviewKind::Literary)
        .filter(|item| matches!(item.status, ReviewStatus::Approved | ReviewStatus::Edited))
        .filter_map(|item| {
            let ReviewedValue::Literary(value) = item.reviewed_value.as_ref()? else {
                return None;
            };
            let chapter_ids = item
                .latest_proposal
                .evidence()
                .iter()
                .map(|evidence| evidence.chapter_id.clone())
                .collect::<Vec<_>>();
            if chapter_ids.is_empty() {
                return None;
            }
            Some(LiteraryContextLine {
                chapter_ids,
                line: format!(
                    "[{}] {} ({}) — {}",
                    value.finding_category, value.subject, value.scope_label, value.claim
                ),
            })
        })
        .collect()
}
use std::env;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

const REVIEW_OUTPUT_SCHEMA_VERSION: u32 = 1;

#[derive(Serialize)]
struct ReconcileOutput<'a> {
    schema_version: u32,
    review_file: String,
    manuscript_id: &'a str,
    total_items: usize,
    reconciliation: &'a human_review_workflow::ReconciliationRecord,
}

#[derive(Serialize)]
struct QueueOutput<'a> {
    schema_version: u32,
    manuscript_id: &'a str,
    filter: String,
    total: usize,
    items: Vec<&'a ReviewItem>,
}

#[derive(Serialize)]
struct ItemOutput<'a> {
    schema_version: u32,
    item: &'a ReviewItem,
}

#[derive(Serialize)]
struct DecisionOutput<'a> {
    schema_version: u32,
    item: &'a ReviewItem,
}

pub(crate) fn run_review(args: &[String], format: &OutputFormat) -> Result<(), String> {
    let Some(command) = args.first().map(String::as_str) else {
        review_usage();
        return Ok(());
    };
    match command {
        "help" | "--help" | "-h" => {
            review_usage();
            Ok(())
        }
        "sync" => sync(&args[1..], format),
        "list" => list(&args[1..], format),
        "show" => show(&args[1..], format),
        "approve" => decide(&args[1..], format, DecisionAction::Approve),
        "edit" => decide(&args[1..], format, DecisionAction::Edit),
        "reject" => decide(&args[1..], format, DecisionAction::Reject),
        "defer" => decide(&args[1..], format, DecisionAction::Defer),
        "reopen" => decide(&args[1..], format, DecisionAction::Reopen),
        "promote" => promote(&args[1..], format),
        _ => Err(format!("unknown review subcommand '{command}'")),
    }
}

fn review_usage() {
    println!("Literary intelligence review commands:");
    println!("  review sync <manuscript> --review-file <path> [canon paths]");
    println!("  review list --review-file <path> [--status <filter>] [--kind <all|character|relationship|terminology|literary>]");
    println!("  review show <id> --review-file <path>");
    println!("  review <approve|reject|defer|reopen> <id> --review-file <path> --reviewer <name> --reason <text>");
    println!("  review edit <id> --review-file <path> --replacement <file|-> --reviewer <name> --reason <text>");
    println!("  review promote --dry-run --review-file <path> [canon paths] [--item <id> ...]");
    println!("  review promote --apply --plan-id <id> --review-file <path> --character-bible <path> --glossary <path> --reviewer <name> --reason <text>");
    println!("Canon paths: --character-bible <path> --glossary <path>");
    println!("Structured operations support --format json.");
}

fn sync(args: &[String], format: &OutputFormat) -> Result<(), String> {
    let manuscript_path = positional(args, 0).ok_or("review sync requires a manuscript path")?;
    let review_file = required_path(args, "--review-file", "LITERARY_ENGINE_REVIEW_FILE")?;
    let character_file = optional_path(
        args,
        "--character-bible",
        "LITERARY_ENGINE_CHARACTER_BIBLE_FILE",
    );
    let glossary_file = optional_path(args, "--glossary", "LITERARY_ENGINE_GLOSSARY_FILE");
    let characters = match character_file {
        Some(path) => load_character_bible_or_default(path).map_err(|error| error.to_string())?,
        None => CharacterBible::new(),
    };
    let glossary = match glossary_file {
        Some(path) => load_glossary_or_default(path).map_err(|error| error.to_string())?,
        None => Glossary::default(),
    };
    let manuscript = ingest_file(manuscript_path)
        .map_err(|error| format!("failed to read {manuscript_path}: {error}"))?;
    let intelligence = DeterministicManuscriptAnalyzer::default()
        .analyze(
            &manuscript,
            AnalysisCanon {
                characters: &characters,
                glossary: &glossary,
            },
        )
        .map_err(|error| format!("failed to analyze {manuscript_path}: {error}"))?;
    let mut ledger = if review_file.exists() {
        load_review_ledger(&review_file).map_err(|error| error.to_string())?
    } else {
        ReviewLedger::new(&intelligence)
    };
    let result = ledger
        .reconcile(&intelligence, Utc::now())
        .map_err(|error| error.to_string())?;
    save_review_ledger(&review_file, &ledger).map_err(|error| error.to_string())?;
    let output = ReconcileOutput {
        schema_version: REVIEW_OUTPUT_SCHEMA_VERSION,
        review_file: review_file.display().to_string(),
        manuscript_id: &ledger.manuscript_id,
        total_items: ledger.items.len(),
        reconciliation: &result,
    };
    print_output(format, &output, || {
        format!(
            "review queue synchronized: {} items ({} new, {} changed, {} obsolete)",
            ledger.items.len(),
            result.added.len(),
            result.changed.len(),
            result.obsolete.len()
        )
    })
}

fn list(args: &[String], format: &OutputFormat) -> Result<(), String> {
    let review_file = required_path(args, "--review-file", "LITERARY_ENGINE_REVIEW_FILE")?;
    let ledger = load_review_ledger(&review_file).map_err(|error| error.to_string())?;
    let status_filter = flag_value(args, "--status").unwrap_or("all");
    let kind_filter = flag_value(args, "--kind");
    if ![
        "all",
        "pending",
        "deferred",
        "approved-not-applied",
        "approved_not_applied",
        "conflicted",
        "rejected",
        "applied",
        "obsolete",
    ]
    .contains(&status_filter)
    {
        return Err(format!(
            "unsupported review status filter '{status_filter}'"
        ));
    }
    if kind_filter.is_some_and(|value| {
        ![
            "all",
            "character",
            "relationship",
            "terminology",
            "literary",
        ]
        .contains(&value)
    }) {
        return Err(format!(
            "unsupported review kind filter '{}'",
            kind_filter.unwrap_or_default()
        ));
    }
    let conflict_ids = current_conflicted_item_ids(args, &ledger)?;
    let mut items = ledger
        .items
        .iter()
        .filter(|item| matches_status(item, status_filter, &conflict_ids))
        .filter(|item| matches_kind(item, kind_filter))
        .collect::<Vec<_>>();
    items.sort_by(|left, right| {
        conflict_ids
            .contains(&right.id)
            .cmp(&conflict_ids.contains(&left.id))
            .then_with(|| {
                confidence_rank(left.latest_proposal.confidence().level)
                    .cmp(&confidence_rank(right.latest_proposal.confidence().level))
            })
            .then_with(|| left.kind.cmp(&right.kind))
            .then_with(|| source_order(left).cmp(&source_order(right)))
            .then_with(|| left.id.cmp(&right.id))
    });
    let output = QueueOutput {
        schema_version: REVIEW_OUTPUT_SCHEMA_VERSION,
        manuscript_id: &ledger.manuscript_id,
        filter: format!(
            "status={status_filter},kind={}",
            kind_filter.unwrap_or("all")
        ),
        total: items.len(),
        items,
    };
    print_output(format, &output, || {
        let mut text = format!("{} review items", output.total);
        for item in &output.items {
            text.push_str(&format!(
                "\n{}\t{:?}\t{:?}\trevision {}",
                item.id, item.kind, item.status, item.revision
            ));
        }
        text
    })
}

fn show(args: &[String], format: &OutputFormat) -> Result<(), String> {
    let id = positional(args, 0).ok_or("review show requires an item ID")?;
    let review_file = required_path(args, "--review-file", "LITERARY_ENGINE_REVIEW_FILE")?;
    let ledger = load_review_ledger(&review_file).map_err(|error| error.to_string())?;
    let item = ledger.item(id).map_err(|error| error.to_string())?;
    let output = ItemOutput {
        schema_version: REVIEW_OUTPUT_SCHEMA_VERSION,
        item,
    };
    print_output(format, &output, || format!("{item:#?}"))
}

fn decide(args: &[String], format: &OutputFormat, action: DecisionAction) -> Result<(), String> {
    let id = positional(args, 0).ok_or("review decision requires an item ID")?;
    let review_file = required_path(args, "--review-file", "LITERARY_ENGINE_REVIEW_FILE")?;
    let reviewer = required_flag(args, "--reviewer")?.to_string();
    let reason = required_flag(args, "--reason")?.to_string();
    let replacement = if action == DecisionAction::Edit {
        let input = required_flag(args, "--replacement")?;
        Some(read_json_input::<ReviewedValue>(input)?)
    } else {
        None
    };
    let mut ledger = load_review_ledger(&review_file).map_err(|error| error.to_string())?;
    ledger
        .decide(id, action, replacement, reviewer, reason, Utc::now())
        .map_err(|error| error.to_string())?;
    save_review_ledger(&review_file, &ledger).map_err(|error| error.to_string())?;
    let item = ledger.item(id).map_err(|error| error.to_string())?;
    let output = DecisionOutput {
        schema_version: REVIEW_OUTPUT_SCHEMA_VERSION,
        item,
    };
    print_output(format, &output, || {
        format!(
            "{} is now {:?} (revision {})",
            item.id, item.status, item.revision
        )
    })
}

fn promote(args: &[String], format: &OutputFormat) -> Result<(), String> {
    let review_file = required_path(args, "--review-file", "LITERARY_ENGINE_REVIEW_FILE")?;
    let mut ledger = load_review_ledger(&review_file).map_err(|error| error.to_string())?;
    let apply = has_flag(args, "--apply");
    let dry_run = has_flag(args, "--dry-run");
    if apply == dry_run {
        return Err("review promote requires exactly one of --dry-run or --apply".into());
    }
    let plan_id = flag_value(args, "--plan-id");
    if apply {
        let expected = plan_id.ok_or("--apply requires --plan-id from a dry run")?;
        if ledger.already_applied(expected) {
            let result = PromotionApplyResult {
                schema_version: 1,
                plan_id: expected.into(),
                status: PromotionApplyStatus::AlreadyApplied,
                applied_item_ids: ledger
                    .promotions
                    .iter()
                    .find(|record| record.plan_id == expected)
                    .map(|record| record.item_ids.clone())
                    .unwrap_or_default(),
                operation_count: 0,
            };
            return print_output(format, &result, || {
                format!("promotion {} was already applied", result.plan_id)
            });
        }
    }
    let character_file = optional_path(
        args,
        "--character-bible",
        "LITERARY_ENGINE_CHARACTER_BIBLE_FILE",
    );
    let glossary_file = optional_path(args, "--glossary", "LITERARY_ENGINE_GLOSSARY_FILE");
    if apply && (character_file.is_none() || glossary_file.is_none()) {
        return Err(
            "--apply requires Character Bible and Glossary paths via flags or environment".into(),
        );
    }
    let characters = match &character_file {
        Some(path) => load_character_bible_or_default(path).map_err(|error| error.to_string())?,
        None => CharacterBible::new(),
    };
    let glossary = match &glossary_file {
        Some(path) => load_glossary_or_default(path).map_err(|error| error.to_string())?,
        None => Glossary::default(),
    };
    let selected = repeated_flag_values(args, "--item");
    let resolutions = flag_value(args, "--resolutions")
        .map(read_json_input::<Vec<ConflictResolution>>)
        .transpose()?
        .unwrap_or_default();
    let plan = build_promotion_plan(&ledger, &characters, &glossary, &selected, &resolutions)
        .map_err(|error| error.to_string())?;
    if !apply {
        return print_output(format, &plan, || promotion_text(&plan));
    }
    let expected = plan_id.ok_or("--apply requires --plan-id")?;
    if expected != plan.plan_id {
        return Err(format!(
            "promotion plan is stale: expected {expected}, current plan is {}",
            plan.plan_id
        ));
    }
    if plan.blocked {
        return Err("promotion plan has unresolved blocking conflicts".into());
    }
    let reviewer = required_flag(args, "--reviewer")?;
    let reason = required_flag(args, "--reason")?;
    let character_bible_file =
        character_file.ok_or("--apply requires Character Bible path via flag or environment")?;
    let glossary_file =
        glossary_file.ok_or("--apply requires Glossary path via flag or environment")?;
    let paths = ReviewStorePaths {
        review_file,
        character_bible_file,
        glossary_file,
    };
    recover_pending_promotion(&paths).map_err(|error| error.to_string())?;
    let result = apply_promotion(&paths, &ledger, &plan, reviewer, reason, Utc::now())
        .map_err(|error| error.to_string())?;
    ledger = load_review_ledger(&paths.review_file).map_err(|error| error.to_string())?;
    let _ = ledger;
    print_output(format, &result, || {
        format!(
            "promotion {} applied: {} items, {} operations",
            result.plan_id,
            result.applied_item_ids.len(),
            result.operation_count
        )
    })
}

fn current_conflicted_item_ids(
    args: &[String],
    ledger: &ReviewLedger,
) -> Result<BTreeSet<String>, String> {
    let character_file = optional_path(
        args,
        "--character-bible",
        "LITERARY_ENGINE_CHARACTER_BIBLE_FILE",
    );
    let glossary_file = optional_path(args, "--glossary", "LITERARY_ENGINE_GLOSSARY_FILE");
    let characters = character_file
        .map(load_character_bible_or_default)
        .transpose()
        .map_err(|error| error.to_string())?
        .unwrap_or_default();
    let glossary = glossary_file
        .map(load_glossary_or_default)
        .transpose()
        .map_err(|error| error.to_string())?
        .unwrap_or_default();
    let plan = build_promotion_plan(ledger, &characters, &glossary, &[], &[])
        .map_err(|error| error.to_string())?;
    Ok(plan
        .conflicts
        .into_iter()
        .map(|conflict| conflict.item_id)
        .collect())
}

fn matches_status(item: &ReviewItem, filter: &str, conflicts: &BTreeSet<String>) -> bool {
    match filter {
        "all" => true,
        "pending" => {
            item.status == ReviewStatus::Pending
                && item.availability == human_review_workflow::ProposalAvailability::Active
        }
        "deferred" => {
            item.status == ReviewStatus::Deferred
                && item.availability == human_review_workflow::ProposalAvailability::Active
        }
        "approved-not-applied" | "approved_not_applied" => {
            matches!(item.status, ReviewStatus::Approved | ReviewStatus::Edited)
                && item.availability == human_review_workflow::ProposalAvailability::Active
        }
        "conflicted" => conflicts.contains(&item.id),
        "rejected" => item.status == ReviewStatus::Rejected,
        "applied" => item.status == ReviewStatus::Applied,
        "obsolete" => item.availability == human_review_workflow::ProposalAvailability::Obsolete,
        _ => false,
    }
}

fn matches_kind(item: &ReviewItem, filter: Option<&str>) -> bool {
    match filter.unwrap_or("all") {
        "all" => true,
        "character" => item.kind == ReviewKind::Character,
        "relationship" => item.kind == ReviewKind::Relationship,
        "terminology" => item.kind == ReviewKind::Terminology,
        "literary" => item.kind == ReviewKind::Literary,
        _ => false,
    }
}

fn source_order(item: &ReviewItem) -> (usize, usize, usize) {
    item.latest_proposal
        .evidence()
        .first()
        .map(|evidence| {
            (
                evidence.source.chapter.unwrap_or(usize::MAX),
                evidence.source.scene.unwrap_or(usize::MAX),
                evidence.source.paragraph.unwrap_or(usize::MAX),
            )
        })
        .unwrap_or((usize::MAX, usize::MAX, usize::MAX))
}

fn promotion_text(plan: &human_review_workflow::CanonPromotionPlan) -> String {
    format!(
        "promotion plan: {}\nitems: {}\noperations: {}\nconflicts: {}\nblocked: {}",
        plan.plan_id,
        plan.promoted_item_ids.len(),
        plan.operations.len(),
        plan.conflicts.len(),
        plan.blocked
    )
}

fn print_output<T: Serialize>(
    format: &OutputFormat,
    value: &T,
    text: impl FnOnce() -> String,
) -> Result<(), String> {
    match format {
        OutputFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(value)
                .map_err(|error| format!("failed to serialize review output: {error}"))?
        ),
        OutputFormat::Text => println!("{}", text()),
    }
    Ok(())
}

fn read_json_input<T: serde::de::DeserializeOwned>(input: &str) -> Result<T, String> {
    let text = if input == "-" {
        let mut value = String::new();
        io::stdin()
            .read_to_string(&mut value)
            .map_err(|error| format!("failed to read JSON from stdin: {error}"))?;
        value
    } else {
        fs::read_to_string(input)
            .map_err(|error| format!("failed to read JSON input {input}: {error}"))?
    };
    serde_json::from_str(&text).map_err(|error| format!("invalid JSON input: {error}"))
}

fn positional(args: &[String], wanted: usize) -> Option<&str> {
    let flags_with_values = [
        "--review-file",
        "--character-bible",
        "--glossary",
        "--status",
        "--kind",
        "--reviewer",
        "--reason",
        "--replacement",
        "--resolutions",
        "--plan-id",
        "--item",
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

fn required_path(args: &[String], flag: &str, variable: &str) -> Result<PathBuf, String> {
    optional_path(args, flag, variable).ok_or_else(|| format!("{flag} or {variable} is required"))
}

fn optional_path(args: &[String], flag: &str, variable: &str) -> Option<PathBuf> {
    flag_value(args, flag).map(PathBuf::from).or_else(|| {
        env::var(variable)
            .ok()
            .filter(|value| !value.trim().is_empty())
            .map(PathBuf::from)
    })
}

fn required_flag<'a>(args: &'a [String], flag: &str) -> Result<&'a str, String> {
    flag_value(args, flag).ok_or_else(|| format!("{flag} is required"))
}

fn flag_value<'a>(args: &'a [String], flag: &str) -> Option<&'a str> {
    args.windows(2)
        .find(|pair| pair[0] == flag)
        .map(|pair| pair[1].as_str())
}

fn repeated_flag_values(args: &[String], flag: &str) -> Vec<String> {
    args.windows(2)
        .filter(|pair| pair[0] == flag)
        .map(|pair| pair[1].clone())
        .collect()
}

fn has_flag(args: &[String], flag: &str) -> bool {
    args.iter().any(|value| value == flag)
}
