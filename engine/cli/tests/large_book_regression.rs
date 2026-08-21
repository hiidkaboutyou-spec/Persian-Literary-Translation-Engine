use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_workspace() -> std::path::PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock must be after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("literary-engine-large-book-{nonce}"))
}

#[test]
fn processes_large_multi_chapter_book_end_to_end_with_echo_provider() {
    const CHAPTERS: usize = 120;
    let workspace = temp_workspace();
    let input_path = workspace.join("large-book.txt");
    let output_dir = workspace.join("output");
    fs::create_dir_all(&workspace).expect("create temporary workspace");

    let mut source = String::new();
    for index in 1..=CHAPTERS {
        source.push_str(&format!(
            "Chapter {index}\nThis is chapter {index}. The characters exchange dialogue, move through the scene, and preserve narrative continuity.\n\n"
        ));
    }
    fs::write(&input_path, source).expect("write synthetic large book");

    let status = Command::new(env!("CARGO_BIN_EXE_literary-engine"))
        .arg("run")
        .arg(&input_path)
        .arg("fa")
        .arg(&output_dir)
        .env("LITERARY_ENGINE_PROVIDER", "echo")
        .env_remove("OPENAI_API_KEY")
        .status()
        .expect("run literary-engine");

    assert!(status.success(), "large-book runtime must complete successfully");

    let manifest = fs::read_to_string(output_dir.join("manifest.txt"))
        .expect("runtime must emit a manifest");
    assert!(manifest.contains(&format!("chapters={CHAPTERS}")));
    assert!(manifest.contains("provider=echo"));
    assert!(manifest.contains(&format!("chapter.{CHAPTERS}.quality_score=")));

    let chapter_outputs = fs::read_dir(&output_dir)
        .expect("read runtime output")
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "txt"))
        .filter(|entry| entry.file_name() != "manifest.txt")
        .count();
    assert_eq!(chapter_outputs, CHAPTERS);

    fs::remove_dir_all(workspace).ok();
}
