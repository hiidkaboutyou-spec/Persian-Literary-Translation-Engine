use human_review_workflow::{
    ReviewKind, ReviewLedger, ReviewStatus, ReviewedCharacter, ReviewedTerminology, ReviewedValue,
};
use project_engine::review_store::{
    load_character_bible_or_default, load_glossary_or_default, load_review_ledger,
};
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

fn write_replacement(path: &Path, value: &ReviewedValue) {
    fs::write(path, serde_json::to_vec_pretty(value).unwrap()).unwrap();
}

#[test]
fn reviewed_proposals_promote_to_canon_and_drive_safe_resume() {
    let workspace = tempfile::tempdir().unwrap();
    let input = workspace.path().join("story.txt");
    let review_file = workspace.path().join("review.json");
    let characters_file = workspace.path().join("characters.json");
    let glossary_file = workspace.path().join("glossary.json");
    let output_dir = workspace.path().join("output");
    fs::write(
        &input,
        "Chapter 1\nMina met Reza beside the silver door.\n\n\"Reza—wait,\" Mina said.\n\nChapter 2\nReza opened the silver door for Mina.\n\nChapter 3\nMina thanked Reza at the silver door.\n",
    )
    .unwrap();

    let sync = run(&[
        "review",
        "sync",
        path(&input),
        "--review-file",
        path(&review_file),
        "--character-bible",
        path(&characters_file),
        "--glossary",
        path(&glossary_file),
        "--format",
        "json",
    ]);
    assert_success(&sync);
    let sync_json: serde_json::Value = serde_json::from_slice(&sync.stdout).unwrap();
    assert_eq!(sync_json["schema_version"], 1);
    assert!(sync_json["total_items"].as_u64().unwrap() >= 4);

    let ledger = load_review_ledger(&review_file).unwrap();
    let mina = ledger
        .items
        .iter()
        .find(|item| {
            item.kind == ReviewKind::Character
                && serde_json::to_string(&item.latest_proposal)
                    .unwrap()
                    .contains("Mina")
        })
        .unwrap()
        .id
        .clone();
    let reza = ledger
        .items
        .iter()
        .find(|item| {
            item.kind == ReviewKind::Character
                && serde_json::to_string(&item.latest_proposal)
                    .unwrap()
                    .contains("Reza")
        })
        .unwrap()
        .id
        .clone();
    let relationship = ledger
        .items
        .iter()
        .find(|item| item.kind == ReviewKind::Relationship)
        .unwrap()
        .id
        .clone();
    let term = ledger
        .items
        .iter()
        .find(|item| {
            item.kind == ReviewKind::Terminology
                && serde_json::to_string(&item.latest_proposal)
                    .unwrap()
                    .contains("silver door")
        })
        .unwrap()
        .id
        .clone();

    let character_edit = workspace.path().join("character-edit.json");
    write_replacement(
        &character_edit,
        &ReviewedValue::Character(ReviewedCharacter {
            canonical_name: "Mina Farahani".into(),
            aliases: vec!["Mina".into(), "مینا".into()],
            voice_notes: "quiet, with curly ‘quotes’ and an em dash — when hesitant".into(),
            personality_notes: "observant".into(),
        }),
    );
    let edit = run(&[
        "review",
        "edit",
        &mina,
        "--review-file",
        path(&review_file),
        "--replacement",
        path(&character_edit),
        "--reviewer",
        "editor@example.test",
        "--reason",
        "Correct full identity and aliases",
        "--format",
        "json",
    ]);
    assert_success(&edit);

    let term_edit = workspace.path().join("term-edit.json");
    write_replacement(
        &term_edit,
        &ReviewedValue::Terminology(ReviewedTerminology {
            source_term: "silver door".into(),
            preferred_translation: "درِ نقره‌ای".into(),
            context: "recurring مکان/شیء".into(),
        }),
    );
    assert_success(&run(&[
        "review",
        "edit",
        &term,
        "--review-file",
        path(&review_file),
        "--replacement",
        path(&term_edit),
        "--reviewer",
        "editor@example.test",
        "--reason",
        "Set Persian house style",
    ]));
    assert_success(&run(&[
        "review",
        "reject",
        &reza,
        "--review-file",
        path(&review_file),
        "--reviewer",
        "editor@example.test",
        "--reason",
        "Not enough evidence for a separate identity",
    ]));
    assert_success(&run(&[
        "review",
        "defer",
        &relationship,
        "--review-file",
        path(&review_file),
        "--reviewer",
        "editor@example.test",
        "--reason",
        "Relationship meaning remains unresolved",
    ]));

    let review_before = fs::read(&review_file).unwrap();
    let preview = run(&[
        "review",
        "promote",
        "--dry-run",
        "--review-file",
        path(&review_file),
        "--character-bible",
        path(&characters_file),
        "--glossary",
        path(&glossary_file),
        "--item",
        &mina,
        "--item",
        &term,
        "--format",
        "json",
    ]);
    assert_success(&preview);
    assert_eq!(fs::read(&review_file).unwrap(), review_before);
    assert!(!characters_file.exists());
    assert!(!glossary_file.exists());
    let preview_json: serde_json::Value = serde_json::from_slice(&preview.stdout).unwrap();
    assert_eq!(preview_json["blocked"], false);
    let plan_id = preview_json["plan_id"].as_str().unwrap();

    let apply = run(&[
        "review",
        "promote",
        "--apply",
        "--plan-id",
        plan_id,
        "--review-file",
        path(&review_file),
        "--character-bible",
        path(&characters_file),
        "--glossary",
        path(&glossary_file),
        "--item",
        &mina,
        "--item",
        &term,
        "--reviewer",
        "editor@example.test",
        "--reason",
        "Apply reviewed canon",
        "--format",
        "json",
    ]);
    assert_success(&apply);
    let apply_json: serde_json::Value = serde_json::from_slice(&apply.stdout).unwrap();
    assert_eq!(apply_json["status"], "applied");

    let bible = load_character_bible_or_default(&characters_file).unwrap();
    assert!(bible.find_profile("Mina Farahani").is_some());
    assert_eq!(bible.find_alias_owner("Mina"), Some("Mina Farahani"));
    let glossary = load_glossary_or_default(&glossary_file).unwrap();
    assert_eq!(
        glossary
            .find_exact_term("silver door")
            .unwrap()
            .preferred_translation,
        "درِ نقره‌ای"
    );
    let applied_ledger: ReviewLedger = load_review_ledger(&review_file).unwrap();
    assert_eq!(
        applied_ledger.item(&mina).unwrap().status,
        ReviewStatus::Applied
    );
    assert_eq!(
        applied_ledger.item(&term).unwrap().status,
        ReviewStatus::Applied
    );
    assert_eq!(
        applied_ledger.item(&reza).unwrap().status,
        ReviewStatus::Rejected
    );
    assert_eq!(
        applied_ledger.item(&relationship).unwrap().status,
        ReviewStatus::Deferred
    );
    assert_eq!(applied_ledger.promotions.len(), 1);
    assert!(!applied_ledger.promotions[0].plan.operations.is_empty());

    let second_apply = run(&[
        "review",
        "promote",
        "--apply",
        "--plan-id",
        plan_id,
        "--review-file",
        path(&review_file),
        "--character-bible",
        path(&characters_file),
        "--glossary",
        path(&glossary_file),
        "--reviewer",
        "editor@example.test",
        "--reason",
        "Retry safely",
        "--format",
        "json",
    ]);
    assert_success(&second_apply);
    let second_json: serde_json::Value = serde_json::from_slice(&second_apply.stdout).unwrap();
    assert_eq!(second_json["status"], "already_applied");

    let sync_again = run(&[
        "review",
        "sync",
        path(&input),
        "--review-file",
        path(&review_file),
        "--character-bible",
        path(&characters_file),
        "--glossary",
        path(&glossary_file),
        "--format",
        "json",
    ]);
    assert_success(&sync_again);
    let reconciled = load_review_ledger(&review_file).unwrap();
    assert_eq!(
        reconciled.item(&reza).unwrap().status,
        ReviewStatus::Rejected
    );

    let first_run = Command::new(env!("CARGO_BIN_EXE_literary-engine"))
        .arg("run")
        .arg(&input)
        .arg("fa")
        .arg(&output_dir)
        .env("LITERARY_ENGINE_PROVIDER", "echo")
        .env_remove("OPENAI_API_KEY")
        .env("LITERARY_ENGINE_CHARACTER_BIBLE_FILE", &characters_file)
        .env("LITERARY_ENGINE_GLOSSARY_FILE", &glossary_file)
        .output()
        .unwrap();
    assert_success(&first_run);
    let first_manifest = fs::read_to_string(output_dir.join("manifest.txt")).unwrap();
    assert!(first_manifest.contains("character_profiles=1"));
    assert!(first_manifest.contains("glossary_entries=1"));
    assert!(first_manifest.contains("context_fingerprint="));

    let mut changed_glossary = fs::read_to_string(&glossary_file).unwrap();
    changed_glossary = changed_glossary.replace("recurring مکان/شیء", "updated approved context");
    fs::write(&glossary_file, changed_glossary).unwrap();
    let resume = Command::new(env!("CARGO_BIN_EXE_literary-engine"))
        .arg("resume")
        .arg(&input)
        .arg("fa")
        .arg(&output_dir)
        .env("LITERARY_ENGINE_PROVIDER", "echo")
        .env_remove("OPENAI_API_KEY")
        .env("LITERARY_ENGINE_CHARACTER_BIBLE_FILE", &characters_file)
        .env("LITERARY_ENGINE_GLOSSARY_FILE", &glossary_file)
        .output()
        .unwrap();
    assert_success(&resume);
    let resume_manifest = fs::read_to_string(output_dir.join("manifest.txt")).unwrap();
    assert!(resume_manifest.contains("resumed=false"));

    let show = run(&[
        "review",
        "show",
        &term,
        "--review-file",
        path(&review_file),
        "--format",
        "json",
    ]);
    assert_success(&show);
    let show_json: serde_json::Value = serde_json::from_slice(&show.stdout).unwrap();
    assert_eq!(show_json["schema_version"], 1);
}
