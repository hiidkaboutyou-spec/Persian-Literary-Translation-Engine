use document_engine::{load_file, split_into_chapters};
use std::env;
use std::process::ExitCode;
use translation_core::{prepare_translation, TranslationContext, TranslationRequest};

fn usage() {
    eprintln!("Persian Literary Translation Engine v0.2");
    eprintln!("Usage:");
    eprintln!("  literary-engine inspect <file.txt|file.md|file.docx>");
    eprintln!("  literary-engine prepare <file.txt|file.md|file.docx> [target-language]");
}

fn inspect(path: &str) -> Result<(), String> {
    let document = load_file(path).map_err(|error| format!("failed to read {path}: {error}"))?;
    let chapters = split_into_chapters(&document.text);

    println!("title: {}", document.title);
    println!("bytes: {}", document.text.len());
    println!("chapters: {}", chapters.len());
    for chapter in chapters {
        println!("- {}: {} bytes", chapter.title, chapter.content.len());
    }
    Ok(())
}

fn prepare(path: &str, target_language: &str) -> Result<(), String> {
    let document = load_file(path).map_err(|error| format!("failed to read {path}: {error}"))?;
    let chapters = split_into_chapters(&document.text);

    println!("document: {}", document.title);
    println!("chapters: {}", chapters.len());

    for chapter in chapters {
        let result = prepare_translation(
            TranslationRequest {
                source: chapter.content,
                target_language: target_language.to_string(),
            },
            TranslationContext {
                glossary_enabled: true,
                character_memory_enabled: true,
            },
        );
        println!("{} -> {}", chapter.title, result);
    }
    Ok(())
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    match args.as_slice() {
        [_, command, path] if command == "inspect" => inspect(path),
        [_, command, path] if command == "prepare" => prepare(path, "fa"),
        [_, command, path, target] if command == "prepare" => prepare(path, target),
        _ => {
            usage();
            Err("invalid arguments".to_string())
        }
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}
