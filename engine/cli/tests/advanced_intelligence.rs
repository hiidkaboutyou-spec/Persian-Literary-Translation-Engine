//! Phase 15 end-to-end coverage through the real CLI with the deterministic
//! mock analysis provider: no API key, no network, no real model.
//!
//! Covers: bounded unit analysis over a manuscript, structured findings with
//! validated evidence, review-queue integration, approve + preserve across
//! re-analysis, no canon promotion for review-only literary findings, and
//! chapter-scoped selection of approved literary context in translation.

use human_review_workflow::{ReviewKind, ReviewStatus};
use project_engine::review_store::load_review_ledger;
use std::fs;
use std::path::Path;
use std::process::{Command, Output};

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_literary-engine"))
        .args(args)
        .env_remove("OPENAI_API_KEY")
        .env("LITERARY_ENGINE_PROVIDER", "echo")
        .output()
        .expect("run literary-engine")
}

fn run_with_review_file(review_file: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_literary-engine"))
        .args(args)
        .env_remove("OPENAI_API_KEY")
        .env("LITERARY_ENGINE_PROVIDER", "echo")
        .env("LITERARY_ENGINE_REVIEW_FILE", review_file)
        .output()
        .expect("run literary-engine")
}

fn path(path: &Path) -> &str {
    path.to_str().expect("temporary path is UTF-8")
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "command failed\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn story_text() -> &'static str {
    "Chapter 1\nMina met Reza beside the silver door.\n\n\"Reza—wait,\" Mina said quietly, almost to herself.\n\nReza did not answer. He studied the door.\n\nChapter 2\nReza opened the silver door for Mina.\n\n\"After you,\" he said.\n\nMina stepped through without a word.\n"
}

#[test]
fn advanced_analysis_produces_validated_findings_through_review_into_translation_context() {
    let workspace = tempfile::tempdir().unwrap();
    let input = workspace.path().join("story.txt");
    let review_file = workspace.path().join("review.json");
    let characters_file = workspace.path().join("characters.json");
    let glossary_file = workspace.path().join("glossary.json");
    let output_dir = workspace.path().join("output");
    fs::write(&input, story_text()).unwrap();

    // 1. Explicit advanced analysis with the mock provider.
    let advanced = run(&[
        "analyze-advanced",
        path(&input),
        "--provider",
        "mock",
        "--review-file",
        path(&review_file),
        "--format",
        "json",
    ]);
    assert_success(&advanced);
    let first: serde_json::Value = serde_json::from_slice(&advanced.stdout).unwrap();
    assert_eq!(first["provider"], "mock");
    assert_eq!(first["model"], "mock-v1");
    assert!(first["succeeded_units"].as_u64().unwrap() >= 2);
    assert_eq!(first["failed_units"].as_array().unwrap().len(), 0);
    assert_eq!(first["cached_units"].as_u64().unwrap(), 0);

    let findings = first["findings"].as_array().unwrap();
    assert!(!findings.is_empty(), "mock provider must produce findings");
    let finding = &findings[0];
    // Every finding carries validated evidence referencing supplied sources.
    let evidence = finding["evidence"].as_array().unwrap();
    assert!(!evidence.is_empty());
    assert!(!evidence[0]["chapter_id"].as_str().unwrap().is_empty());
    assert!(!evidence[0]["paragraph_id"].as_str().unwrap().is_empty());
    assert!(finding["is_review_eligible"].as_bool().unwrap());
    assert_eq!(
        first["review_items_added"].as_u64().unwrap(),
        findings.len() as u64
    ); // 2. Findings reached the shared Phase 14 review ledger as literary items.
    let ledger = load_review_ledger(&review_file).unwrap();
    assert!(
        ledger
            .items
            .iter()
            .filter(|item| item.kind == ReviewKind::Literary)
            .count()
            >= 2
    );
    // Ledger item ids are sorted hashes; locate the chapter-1 finding by its
    // stable finding id (evidence `source.chapter == 1`) so the approval below
    // is deterministic regardless of temporary directory names.
    let chapter_one_finding = findings
        .iter()
        .find(|finding| {
            finding["evidence"]
                .as_array()
                .and_then(|list| list.first())
                .and_then(|evidence| evidence["source"]["chapter"].as_u64())
                == Some(1)
        })
        .expect("chapter 1 finding exists");
    let chapter_one_finding_id = chapter_one_finding["finding_id"].as_str().unwrap();
    let target = ledger
        .items
        .iter()
        .find(|item| {
            item.kind == ReviewKind::Literary && item.proposal_id == chapter_one_finding_id
        })
        .expect("review item for chapter 1 finding")
        .id
        .clone();
    assert_eq!(ledger.item(&target).unwrap().kind, ReviewKind::Literary);

    // 3. Human approves the chapter-1 literary finding.
    let approved = run(&[
        "review",
        "approve",
        &target,
        "--review-file",
        path(&review_file),
        "--reviewer",
        "editor",
        "--reason",
        "matches reading",
        "--format",
        "json",
    ]);
    assert_success(&approved);

    // 4. Re-analysis reconciles idempotently and preserves the approval.
    let rerun = run(&[
        "analyze-advanced",
        path(&input),
        "--provider",
        "mock",
        "--review-file",
        path(&review_file),
        "--format",
        "json",
    ]);
    assert_success(&rerun);
    let rerun_json: serde_json::Value = serde_json::from_slice(&rerun.stdout).unwrap();
    assert_eq!(
        rerun_json["reconciliation_unchanged"]
            .as_array()
            .unwrap()
            .len(),
        findings.len()
    );
    let ledger = load_review_ledger(&review_file).unwrap();
    let item = ledger.item(&target).unwrap();
    assert_eq!(item.status, ReviewStatus::Approved);
    assert_eq!(
        ledger
            .items
            .iter()
            .filter(|item| item.kind == ReviewKind::Literary)
            .count(),
        findings.len(),
        "identical re-analysis must not duplicate review items"
    );

    // 5. Review-only literary findings can never be promoted to canon.
    let plan = run(&[
        "review",
        "promote",
        "--dry-run",
        "--review-file",
        path(&review_file),
        "--character-bible",
        path(&characters_file),
        "--glossary",
        path(&glossary_file),
        "--format",
        "json",
    ]);
    assert_success(&plan);
    let plan_json: serde_json::Value = serde_json::from_slice(&plan.stdout).unwrap();
    assert!(
        plan_json["selected_item_ids"]
            .as_array()
            .unwrap()
            .iter()
            .all(|selected| selected.as_str().unwrap() != target.as_str()),
        "approved literary findings must never be selected for canon promotion"
    );

    // 6. Deterministic analysis still works offline with zero config.
    let deterministic = run(&["analyze", path(&input), "--format", "json"]);
    assert_success(&deterministic);
    let deterministic_json: serde_json::Value =
        serde_json::from_slice(&deterministic.stdout).unwrap();
    assert!(
        deterministic_json["schema_version"].as_u64().unwrap() >= 1,
        "deterministic Phase 13 analysis remains available"
    );

    // 7. Translation run selects approved literary context per chapter.
    let translated = run_with_review_file(
        &review_file,
        &[
            "run",
            path(&input),
            "fa",
            path(&output_dir),
            "--format",
            "json",
        ],
    );
    assert_success(&translated);
    let manifest = fs::read_to_string(output_dir.join("manifest.txt")).expect("manifest written");
    assert!(
        manifest
            .lines()
            .any(|line| line == "reviewed_literary_findings_available=1"),
        "one approved literary finding must be eligible as translation context\nmanifest:\n{manifest}"
    );
    assert!(
        manifest
            .lines()
            .any(|line| line == "chapter.1.literary_findings_used=1"),
        "chapter 1 owns the approved finding and must receive it\nmanifest:\n{manifest}"
    );
    assert!(
        manifest
            .lines()
            .any(|line| line == "chapter.2.literary_findings_used=0"),
        "chapter 2 must not receive chapter 1's finding\nmanifest:\n{manifest}"
    );
}

#[test]
fn advanced_analysis_without_review_file_is_pure_reporting() {
    let workspace = tempfile::tempdir().unwrap();
    let input = workspace.path().join("story.txt");
    fs::write(&input, story_text()).unwrap();
    let advanced = run(&[
        "analyze-advanced",
        path(&input),
        "--provider",
        "mock",
        "--format",
        "json",
    ]);
    assert_success(&advanced);
    let output: serde_json::Value = serde_json::from_slice(&advanced.stdout).unwrap();
    assert!(output["review_file"].is_null());
    assert_eq!(output["review_items_added"].as_u64().unwrap(), 0);
    assert!(!output["findings"].as_array().unwrap().is_empty());
}
