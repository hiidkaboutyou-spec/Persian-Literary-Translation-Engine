//! CLI end-to-end test: `literary-engine project ...` drives the Phase 16
//! ApplicationService, so the CLI behavior is the application behavior.
//! Full offline flow: create → import → analyze → analyze-advanced (mock) →
//! review approve-all → promote → translate (echo) → export → reopen.

use std::path::PathBuf;
use std::process::Command;

fn binary() -> PathBuf {
    // Cargo sets CARGO_BIN_EXE_literary-engine for integration tests.
    PathBuf::from(env!("CARGO_BIN_EXE_literary-engine"))
}

fn temp_dir(tag: &str) -> PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("p16-cli-{tag}-{nonce}"))
}

fn write_book(path: &PathBuf) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(
        path,
        "# Chapter 1\n\nShirin walked into the garden of roses. The fountain murmured softly.\n\nFarhad watched from the terrace, his hands trembling with quiet longing.\n\nA nightingale sang from the cypress tree.\n\n# Chapter 2\n\nShirin sat by the fountain, remembering the nightingale's song.\n\nFarhad approached with a single red rose in his trembling hands.\n\nThe garden held its breath between them.\n\n# Chapter 3\n\nShirin and Farhad stood beneath the cypress tree at dusk.\n\nThe nightingale sang again, and this time it was a promise.\n\nFarhad placed the rose in Shirin's hands without a word.\n",
    )
    .unwrap();
}

fn run(args: &[&str]) -> (bool, String) {
    let output = Command::new(binary())
        .args(args)
        .output()
        .expect("run binary");
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    (output.status.success(), format!("{stdout}\n{stderr}"))
}

#[test]
fn project_commands_run_the_full_offline_workflow() {
    let root = temp_dir("wf");
    let book = root.join("book.md");
    write_book(&book);
    let project = root.join("proj");

    // create
    let (ok, out) = run(&[
        "project",
        "create",
        project.to_str().unwrap(),
        "--name",
        "E2E",
    ]);
    assert!(ok, "create failed:\n{out}");
    // refuse to overwrite
    let (ok, _) = run(&["project", "create", project.to_str().unwrap()]);
    assert!(!ok, "second create on the same project must fail");

    // import
    let (ok, out) = run(&[
        "project",
        "import",
        project.to_str().unwrap(),
        book.to_str().unwrap(),
    ]);
    assert!(ok, "import failed:\n{out}");
    assert!(out.contains("3 chapters"));

    // status: Imported
    let (ok, out) = run(&[
        "project",
        "status",
        project.to_str().unwrap(),
        "--format",
        "json",
    ]);
    assert!(ok, "status failed:\n{out}");
    let snapshot: serde_json::Value = serde_json::from_str(&out).expect("valid json snapshot");
    assert_eq!(snapshot["status"], "imported");
    assert_eq!(snapshot["source"]["chapters"], 3);

    // analyze
    let (ok, out) = run(&["project", "analyze", project.to_str().unwrap()]);
    assert!(ok, "analyze failed:\n{out}");
    assert!(out.contains("review queue"));

    // analyze-advanced (offline mock)
    let (ok, out) = run(&["project", "analyze-advanced", project.to_str().unwrap()]);
    assert!(ok, "analyze-advanced failed:\n{out}");
    assert!(out.contains("advanced analysis complete"));

    // status: NeedsReview
    let (ok, out) = run(&[
        "project",
        "status",
        project.to_str().unwrap(),
        "--format",
        "json",
    ]);
    assert!(ok);
    let snapshot: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(snapshot["status"], "needs_review");
    assert!(snapshot["review"]["pending"].as_u64().unwrap() > 0);

    // review list
    let (ok, out) = run(&["project", "review", project.to_str().unwrap(), "list"]);
    assert!(ok, "review list failed:\n{out}");
    assert!(out.contains("items"));

    // approve-all + promote
    let (ok, out) = run(&[
        "project",
        "review",
        project.to_str().unwrap(),
        "approve-all",
    ]);
    assert!(ok, "approve-all failed:\n{out}");
    let (ok, out) = run(&["project", "review", project.to_str().unwrap(), "promote"]);
    assert!(ok, "promote failed:\n{out}");
    assert!(out.contains("promoted"));

    // translate (echo), export
    let (ok, out) = run(&[
        "project",
        "translate",
        project.to_str().unwrap(),
        "--provider",
        "echo",
    ]);
    assert!(ok, "translate failed:\n{out}");
    assert!(out.contains("Completed"));
    let (ok, out) = run(&["project", "export", project.to_str().unwrap()]);
    assert!(ok, "export failed:\n{out}");
    assert!(out.contains("exported docx"));

    // status: Exported
    let (ok, out) = run(&[
        "project",
        "status",
        project.to_str().unwrap(),
        "--format",
        "json",
    ]);
    assert!(ok);
    let snapshot: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(snapshot["status"], "exported");
    assert_eq!(snapshot["next_action"], "none");
    assert!(snapshot["canon"]["characters"].as_u64().unwrap() > 0);
    assert_eq!(snapshot["translation"]["completed_chapters"], 3);

    // history has the full lifecycle
    let (ok, out) = run(&[
        "project",
        "history",
        project.to_str().unwrap(),
        "--format",
        "json",
    ]);
    assert!(ok);
    let events: serde_json::Value = serde_json::from_str(&out).unwrap();
    let names: Vec<&str> = events
        .as_array()
        .unwrap()
        .iter()
        .map(|event| event["event"].as_str().unwrap())
        .collect();
    for expected in [
        "project_created",
        "source_imported",
        "analysis_completed",
        "advanced_analysis_completed",
        "canon_promoted",
        "translation_completed",
        "export_completed",
    ] {
        assert!(
            names.contains(&expected),
            "history missing {expected}: {names:?}"
        );
    }

    // Reopen after close: status still coherent.
    let (ok, out) = run(&["project", "progress", project.to_str().unwrap()]);
    assert!(ok, "progress after reopen failed:\n{out}");
    assert!(out.contains("Completed"));
}

#[test]
fn project_pause_resume_via_max_chapters() {
    let root = temp_dir("pause");
    let book = root.join("book.md");
    write_book(&book);
    let project = root.join("proj");

    for args in [
        vec!["project", "create", project.to_str().unwrap()],
        vec![
            "project",
            "import",
            project.to_str().unwrap(),
            book.to_str().unwrap(),
        ],
        vec!["project", "analyze", project.to_str().unwrap()],
    ] {
        let (ok, out) = run(&args);
        assert!(ok, "step failed: {out}");
    }
    let (ok, out) = run(&[
        "project",
        "review",
        project.to_str().unwrap(),
        "approve-all",
    ]);
    assert!(ok, "{out}");
    let (ok, out) = run(&["project", "review", project.to_str().unwrap(), "promote"]);
    assert!(ok, "{out}");

    // One chapter per run → Paused; then resume → Completed.
    let (ok, out) = run(&[
        "project",
        "translate",
        project.to_str().unwrap(),
        "--provider",
        "echo",
        "--max-chapters",
        "1",
    ]);
    assert!(ok, "{out}");
    assert!(out.contains("Paused"), "expected pause state:\n{out}");

    let (ok, out) = run(&[
        "project",
        "resume",
        project.to_str().unwrap(),
        "--provider",
        "echo",
    ]);
    assert!(ok, "{out}");
    assert!(
        out.contains("Completed"),
        "expected completion after resume:\n{out}"
    );
}
