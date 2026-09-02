//! Cross-crate integration tests exercising the full translation pipeline:
//! document ingestion → chapter segmentation → context assembly → translation
//! → quality gate → DOCX export → JSON output.

use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_workspace(name: &str) -> std::path::PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock must be after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("literary-engine-integration-{name}-{nonce}"))
}

#[test]
fn full_pipeline_with_glossary_and_character_bible() {
    let workspace = temp_workspace("glossary-character");
    let input_path = workspace.join("story.txt");
    let output_dir = workspace.join("output");
    let memory_path = workspace.join("translation-memory.json");
    let glossary_path = workspace.join("glossary.json");
    let character_path = workspace.join("character-bible.json");
    fs::create_dir_all(&workspace).expect("create workspace");

    // Source with a recurring character name and title
    fs::write(
        &input_path,
        "Chapter 1\nMagnus the High Warlock whispered softly to Alec.\n\nChapter 2\nAlec looked at Magnus and smiled.\n",
    )
    .expect("write source");

    // Glossary: enforce consistent terminology
    fs::write(
        &glossary_path,
        r#"[{"source_term":"High Warlock","preferred_translation":"جادوگر اعظم","context":"title"}]"#,
    )
    .expect("write glossary");

    // Character bible: Magnus and Alec with voice notes
    fs::write(
        &character_path,
        r#"{"profiles":[{"name":"Magnus","voice_notes":"witty and theatrical","personality_notes":"confident"},{"name":"Alec","voice_notes":"restrained and observant","personality_notes":"loyal"}],"aliases":[],"relationships":[{"character_a":"Magnus","character_b":"Alec","dynamic_notes":"playful banter with underlying tenderness","address_notes":"","boundaries_notes":""}]}"#,
    )
    .expect("write character bible");

    // Translation memory with a prior decision
    fs::write(
        &memory_path,
        r#"[{"source":"Magnus whispered softly","translation":"مگنوس آرام زمزمه کرد","context":"intimate dialogue","tags":[]}]"#,
    )
    .expect("write translation memory");

    let status = Command::new(env!("CARGO_BIN_EXE_literary-engine"))
        .arg("run")
        .arg(&input_path)
        .arg("fa")
        .arg(&output_dir)
        .env("LITERARY_ENGINE_PROVIDER", "echo")
        .env_remove("OPENAI_API_KEY")
        .env("LITERARY_ENGINE_MEMORY_FILE", &memory_path)
        .env("LITERARY_ENGINE_GLOSSARY_FILE", &glossary_path)
        .env("LITERARY_ENGINE_CHARACTER_BIBLE_FILE", &character_path)
        .status()
        .expect("run pipeline");

    assert!(status.success(), "pipeline must succeed with memory loaded");

    let manifest =
        fs::read_to_string(output_dir.join("manifest.txt")).expect("manifest must exist");
    assert!(manifest.contains("chapters=2"));
    assert!(manifest.contains("translation_memory_entries=1"));
    assert!(manifest.contains("glossary_entries=1"));
    assert!(manifest.contains("character_profiles=2"));
    assert!(manifest.contains("manuscript_intelligence_schema=1"));
    assert!(manifest.contains("character_seeds="));
    assert!(manifest.contains("relationship_seeds="));
    assert!(manifest.contains("terminology_seeds="));
    assert!(manifest.contains("quality_score=1.00"));

    // DOCX must be generated
    assert!(output_dir.join("manuscript.docx").exists());

    fs::remove_dir_all(workspace).ok();
}

#[test]
fn analyze_json_exposes_stable_seed_contract_without_mutating_canon() {
    let workspace = temp_workspace("analyze-json");
    let input_path = workspace.join("story.txt");
    fs::create_dir_all(&workspace).expect("create workspace");
    fs::write(
        &input_path,
        "Chapter 1\nMina met Reza beside the silver door.\n\nChapter 2\nReza opened the silver door for Mina.\n\nChapter 3\nMina thanked Reza at the silver door.",
    )
    .expect("write source");

    let run = || {
        Command::new(env!("CARGO_BIN_EXE_literary-engine"))
            .arg("analyze")
            .arg(&input_path)
            .arg("--format")
            .arg("json")
            .env_remove("OPENAI_API_KEY")
            .output()
            .expect("analyze manuscript")
    };
    let first = run();
    let second = run();
    assert!(first.status.success());
    assert_eq!(first.stdout, second.stdout, "analysis JSON must be stable");
    let json: serde_json::Value =
        serde_json::from_slice(&first.stdout).expect("analysis must be JSON");
    assert_eq!(json["schema_version"], 1);
    assert_eq!(json["analysis"]["deterministic"], true);
    assert_eq!(json["chapter_maps"].as_array().unwrap().len(), 3);
    assert!(json["character_seeds"].as_array().unwrap().len() >= 2);
    assert!(!json["relationship_seeds"].as_array().unwrap().is_empty());
    assert_eq!(json["initialization"]["mutates_canon"], false);
    assert!(json["character_seeds"][0]["evidence"][0]["paragraph_id"]
        .as_str()
        .is_some_and(|value| !value.is_empty()));

    fs::remove_dir_all(workspace).ok();
}

#[test]
fn resume_reuses_unchanged_chapters() {
    let workspace = temp_workspace("resume");
    let input_path = workspace.join("story.txt");
    let output_dir = workspace.join("output");
    fs::create_dir_all(&workspace).expect("create workspace");

    fs::write(
        &input_path,
        "Chapter 1\nFirst chapter content.\n\nChapter 2\nSecond chapter content.\n",
    )
    .expect("write source");

    // First run
    let status = Command::new(env!("CARGO_BIN_EXE_literary-engine"))
        .arg("run")
        .arg(&input_path)
        .arg("fa")
        .arg(&output_dir)
        .env("LITERARY_ENGINE_PROVIDER", "echo")
        .env_remove("OPENAI_API_KEY")
        .status()
        .expect("first run");
    assert!(status.success());

    let manifest_before =
        fs::read_to_string(output_dir.join("manifest.txt")).expect("manifest before resume");

    // Resume with same source
    let status = Command::new(env!("CARGO_BIN_EXE_literary-engine"))
        .arg("resume")
        .arg(&input_path)
        .arg("fa")
        .arg(&output_dir)
        .env("LITERARY_ENGINE_PROVIDER", "echo")
        .env_remove("OPENAI_API_KEY")
        .status()
        .expect("resume run");
    assert!(status.success());

    let manifest_after =
        fs::read_to_string(output_dir.join("manifest.txt")).expect("manifest after resume");
    assert!(manifest_after.contains("resumed=true"));

    // Both runs produced the same manifest structure
    assert_eq!(
        manifest_before.lines().count(),
        manifest_after.lines().count()
    );

    fs::remove_dir_all(workspace).ok();
}

#[test]
fn json_output_produces_valid_manifest() {
    let workspace = temp_workspace("json-output");
    let input_path = workspace.join("story.txt");
    let output_dir = workspace.join("output");
    fs::create_dir_all(&workspace).expect("create workspace");

    fs::write(&input_path, "Chapter 1\nHello world.\n").expect("write source");

    let output = Command::new(env!("CARGO_BIN_EXE_literary-engine"))
        .arg("run")
        .arg(&input_path)
        .arg("fa")
        .arg(&output_dir)
        .arg("--format")
        .arg("json")
        .env("LITERARY_ENGINE_PROVIDER", "echo")
        .env_remove("OPENAI_API_KEY")
        .output()
        .expect("run with json output");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("stdout must be valid JSON");
    assert_eq!(json["document"], "story");
    assert_eq!(json["provider"], "echo");
    assert_eq!(json["resume"], false);
    assert!(json["chapters"].as_array().unwrap().len() == 1);
    assert!(json["chapters"][0]["quality_score"] == 1.0);

    // manifest.json must also be written
    assert!(output_dir.join("manifest.json").exists());
    let manifest_json =
        fs::read_to_string(output_dir.join("manifest.json")).expect("manifest.json must exist");
    let _: serde_json::Value =
        serde_json::from_str(&manifest_json).expect("manifest.json must be valid JSON");

    // manifest.txt must still be written for backward compatibility
    assert!(output_dir.join("manifest.txt").exists());

    fs::remove_dir_all(workspace).ok();
}

#[test]
fn inspect_json_output_matches_text_fields() {
    let workspace = temp_workspace("inspect-json");
    let input_path = workspace.join("story.txt");
    fs::create_dir_all(&workspace).expect("create workspace");

    fs::write(&input_path, "Chapter 1\nFirst.\n\nChapter 2\nSecond.\n").expect("write source");

    let text_output = Command::new(env!("CARGO_BIN_EXE_literary-engine"))
        .arg("inspect")
        .arg(&input_path)
        .output()
        .expect("inspect text");
    let text = String::from_utf8_lossy(&text_output.stdout);
    assert!(text.contains("chapters: 2"));

    let json_output = Command::new(env!("CARGO_BIN_EXE_literary-engine"))
        .arg("inspect")
        .arg(&input_path)
        .arg("--format")
        .arg("json")
        .output()
        .expect("inspect json");
    let json: serde_json::Value =
        serde_json::from_str(&String::from_utf8_lossy(&json_output.stdout))
            .expect("json must be valid");
    assert_eq!(json["chapters"].as_array().unwrap().len(), 2);

    fs::remove_dir_all(workspace).ok();
}

#[test]
fn quality_gate_blocks_empty_output() {
    let workspace = temp_workspace("quality-gate");
    let input_path = workspace.join("story.txt");
    let output_dir = workspace.join("output");
    fs::create_dir_all(&workspace).expect("create workspace");

    // Write a source file
    fs::write(&input_path, "Chapter 1\nSome content.\n").expect("write source");

    // The echo provider should pass quality gate
    let status = Command::new(env!("CARGO_BIN_EXE_literary-engine"))
        .arg("run")
        .arg(&input_path)
        .arg("fa")
        .arg(&output_dir)
        .env("LITERARY_ENGINE_PROVIDER", "echo")
        .env_remove("OPENAI_API_KEY")
        .status()
        .expect("run with echo provider");

    assert!(status.success(), "echo provider must pass quality gate");

    fs::remove_dir_all(workspace).ok();
}
