use document_engine::{load_file, split_into_chapters};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use translation_core::{
    prepare_translation, EchoProvider, TranslationContext, TranslationPipeline, TranslationRequest,
};

fn usage() {
    eprintln!("Persian Literary Translation Engine v0.1");
    eprintln!("Usage:");
    eprintln!("  literary-engine inspect <file.txt|file.md|file.docx|file.epub>");
    eprintln!("  literary-engine prepare <file.txt|file.md|file.docx|file.epub> [target-language]");
    eprintln!(
        "  literary-engine run <file.txt|file.md|file.docx|file.epub> [target-language] [output-dir]"
    );
    eprintln!("\nThe run command currently uses the deterministic echo provider to validate the full runtime pipeline without API credentials.");
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

fn safe_file_stem(title: &str, index: usize) -> String {
    let mut stem: String = title
        .chars()
        .map(|character| {
            if character.is_alphanumeric() || character == '-' || character == '_' {
                character
            } else {
                '_'
            }
        })
        .collect();
    stem = stem.trim_matches('_').to_string();
    if stem.is_empty() {
        stem = format!("chapter-{}", index + 1);
    }
    format!("{:03}-{}", index + 1, stem)
}

fn run_pipeline(path: &str, target_language: &str, output_dir: &Path) -> Result<(), String> {
    let document = load_file(path).map_err(|error| format!("failed to read {path}: {error}"))?;
    let chapters = split_into_chapters(&document.text);
    if chapters.is_empty() {
        return Err("document contains no translatable chapters".to_string());
    }

    fs::create_dir_all(output_dir)
        .map_err(|error| format!("failed to create {}: {error}", output_dir.display()))?;

    let provider = EchoProvider;
    let pipeline = TranslationPipeline::default_literary_pipeline();
    let mut manifest = String::new();
    manifest.push_str(&format!("document={}\n", document.title));
    manifest.push_str(&format!("target_language={target_language}\n"));
    manifest.push_str("provider=echo\n");
    manifest.push_str(&format!("chapters={}\n", chapters.len()));

    for chapter in chapters {
        let context = format!(
            "document_title={}\nchapter_title={}\nglossary_enabled=true\ncharacter_memory_enabled=true",
            document.title, chapter.title
        );
        let result = pipeline
            .execute(&provider, &chapter.content, target_language, context)
            .map_err(|error| format!("pipeline failed for {}: {error}", chapter.title))?;

        let stem = safe_file_stem(&chapter.title, chapter.index);
        let output_path = output_dir.join(format!("{stem}.txt"));
        fs::write(&output_path, result.final_text())
            .map_err(|error| format!("failed to write {}: {error}", output_path.display()))?;
        manifest.push_str(&format!(
            "chapter.{}.file={}\n",
            chapter.index + 1,
            output_path.display()
        ));
        println!(
            "{} -> {} ({} bytes, provider={})",
            chapter.title,
            output_path.display(),
            result.final_text().len(),
            result.reviewed.provider
        );
    }

    let manifest_path = output_dir.join("manifest.txt");
    fs::write(&manifest_path, manifest)
        .map_err(|error| format!("failed to write {}: {error}", manifest_path.display()))?;
    println!("manifest -> {}", manifest_path.display());
    Ok(())
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    match args.as_slice() {
        [_, command, path] if command == "inspect" => inspect(path),
        [_, command, path] if command == "prepare" => prepare(path, "fa"),
        [_, command, path, target] if command == "prepare" => prepare(path, target),
        [_, command, path] if command == "run" => {
            run_pipeline(path, "fa", &PathBuf::from("output/runtime"))
        }
        [_, command, path, target] if command == "run" => {
            run_pipeline(path, target, &PathBuf::from("output/runtime"))
        }
        [_, command, path, target, output] if command == "run" => {
            run_pipeline(path, target, Path::new(output))
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_file_stems_are_deterministic_and_safe() {
        assert_eq!(safe_file_stem("Chapter 1: Arrival", 0), "001-Chapter_1__Arrival");
        assert_eq!(safe_file_stem("***", 1), "002-chapter-2");
    }
}
