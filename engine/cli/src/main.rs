use std::{env, fs, path::Path};

use document_engine::{load_document, split_into_chapters};
use translation_core::{prepare_translation, TranslationContext, TranslationRequest};

fn print_usage() {
    eprintln!("Persian Literary Translation Engine v0.1");
    eprintln!("Usage:");
    eprintln!("  literary-engine inspect <file.txt>");
    eprintln!("  literary-engine prepare <file.txt> [target-language]");
}

fn read_document(path: &str) -> Result<(String, String), String> {
    let text = fs::read_to_string(path)
        .map_err(|err| format!("failed to read {path}: {err}"))?;
    let title = Path::new(path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("untitled")
        .to_owned();
    Ok((title, text))
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        print_usage();
        std::process::exit(2);
    }

    let command = &args[1];
    let path = &args[2];
    let (title, text) = match read_document(path) {
        Ok(document) => document,
        Err(error) => {
            eprintln!("error: {error}");
            std::process::exit(1);
        }
    };

    let document = load_document(title, text);
    let chapters = split_into_chapters(&document.text);

    match command.as_str() {
        "inspect" => {
            println!("title: {}", document.title);
            println!("characters: {}", document.text.chars().count());
            println!("segments: {}", chapters.len());
            for chapter in chapters.iter().take(5) {
                println!("- {} ({} chars)", chapter.title, chapter.content.chars().count());
            }
        }
        "prepare" => {
            let target_language = args.get(3).cloned().unwrap_or_else(|| "fa".to_string());
            let summary = prepare_translation(
                TranslationRequest {
                    source: document.text,
                    target_language,
                },
                TranslationContext {
                    glossary_enabled: true,
                    character_memory_enabled: true,
                },
            );
            println!("{summary}");
            println!("segments: {}", chapters.len());
        }
        _ => {
            print_usage();
            std::process::exit(2);
        }
    }
}
