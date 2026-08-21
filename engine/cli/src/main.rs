use document_engine::{load_file, split_into_chapters};
use memory_engine::glossary::Glossary;
use memory_engine::{
    build_memory_context, load_glossary, load_translation_memory, save_translation_memory,
    MemoryContextConfig, MemoryEntry, TranslationMemory,
};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use translation_core::{
    prepare_translation, EchoProvider, OpenAIProvider, PipelineInput, TranslationContext,
    TranslationPipeline, TranslationProvider, TranslationRequest,
};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const DEFAULT_MEMORY_PATH: &str = "translation_memory/runtime.json";
const DEFAULT_GLOSSARY_PATH: &str = "glossary/runtime.json";

fn usage() {
    println!("Persian Literary Translation Engine v{VERSION}");
    println!("Usage:");
    println!("  literary-engine inspect <file.txt|file.md|file.docx|file.epub|file.pdf>");
    println!(
        "  literary-engine prepare <file.txt|file.md|file.docx|file.epub|file.pdf> [target-language]"
    );
    println!(
        "  literary-engine run <file.txt|file.md|file.docx|file.epub|file.pdf> [target-language] [output-dir]"
    );
    println!("  literary-engine --help");
    println!("  literary-engine --version");
    println!();
    println!("Provider selection:");
    println!("  - OPENAI_API_KEY set: run uses OpenAI by default");
    println!("  - no OPENAI_API_KEY: run uses the deterministic echo provider");
    println!("  - override with LITERARY_ENGINE_PROVIDER=openai|echo");
    println!("  - override the OpenAI model with OPENAI_MODEL");
    println!();
    println!("Project memory:");
    println!("  - translation memory: {DEFAULT_MEMORY_PATH}");
    println!("  - glossary: {DEFAULT_GLOSSARY_PATH}");
    println!("  - override with LITERARY_ENGINE_MEMORY_PATH and LITERARY_ENGINE_GLOSSARY_PATH");
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

fn selected_provider_name(
    explicit_provider: Option<&str>,
    has_openai_key: bool,
) -> Result<&'static str, String> {
    match explicit_provider
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        Some(value) if value.eq_ignore_ascii_case("echo") => Ok("echo"),
        Some(value) if value.eq_ignore_ascii_case("openai") => Ok("openai"),
        Some(value) => Err(format!(
            "unsupported provider '{value}'; expected openai or echo"
        )),
        None if has_openai_key => Ok("openai"),
        None => Ok("echo"),
    }
}

fn configured_provider() -> Result<Box<dyn TranslationProvider>, String> {
    let explicit_provider = env::var("LITERARY_ENGINE_PROVIDER").ok();
    let has_openai_key = env::var_os("OPENAI_API_KEY").is_some();
    match selected_provider_name(explicit_provider.as_deref(), has_openai_key)? {
        "openai" => OpenAIProvider::from_env()
            .map(|provider| Box::new(provider) as Box<dyn TranslationProvider>)
            .map_err(|error| error.to_string()),
        "echo" => Ok(Box::new(EchoProvider)),
        _ => unreachable!("provider selection only returns supported providers"),
    }
}

fn project_memory_paths() -> (PathBuf, PathBuf) {
    let memory = env::var_os("LITERARY_ENGINE_MEMORY_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_MEMORY_PATH));
    let glossary = env::var_os("LITERARY_ENGINE_GLOSSARY_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_GLOSSARY_PATH));
    (memory, glossary)
}

fn load_project_memory(
    memory_path: &Path,
    glossary_path: &Path,
) -> Result<(TranslationMemory, Glossary), String> {
    let memory = if memory_path.exists() {
        load_translation_memory(memory_path).map_err(|error| {
            format!(
                "failed to load translation memory {}: {error}",
                memory_path.display()
            )
        })?
    } else {
        TranslationMemory::new()
    };

    let glossary = if glossary_path.exists() {
        load_glossary(glossary_path).map_err(|error| {
            format!(
                "failed to load glossary {}: {error}",
                glossary_path.display()
            )
        })?
    } else {
        Glossary::default()
    };

    Ok((memory, glossary))
}

fn runtime_context(
    document_title: &str,
    chapter_title: &str,
    source_text: &str,
    memory: &TranslationMemory,
    glossary: &Glossary,
) -> (String, usize, usize) {
    let memory_context = build_memory_context(
        source_text,
        memory,
        glossary,
        &MemoryContextConfig::default(),
    );
    let mut context = format!(
        "document_title={document_title}\nchapter_title={chapter_title}\nglossary_enabled=true\ncharacter_memory_enabled=true"
    );
    if !memory_context.text.is_empty() {
        context.push_str("\n\n");
        context.push_str(&memory_context.text);
    }
    (
        context,
        memory_context.memory_hits,
        memory_context.glossary_hits,
    )
}

fn run_pipeline(path: &str, target_language: &str, output_dir: &Path) -> Result<(), String> {
    let document = load_file(path).map_err(|error| format!("failed to read {path}: {error}"))?;
    let chapters = split_into_chapters(&document.text);
    if chapters.is_empty() {
        return Err("document contains no translatable chapters".to_string());
    }

    fs::create_dir_all(output_dir)
        .map_err(|error| format!("failed to create {}: {error}", output_dir.display()))?;

    let provider = configured_provider()?;
    let provider_name = provider.name().to_string();
    let pipeline = TranslationPipeline::default_literary_pipeline();
    let (memory_path, glossary_path) = project_memory_paths();
    let (mut memory, glossary) = load_project_memory(&memory_path, &glossary_path)?;
    let persist_outputs = provider_name != "echo";

    let mut manifest = String::new();
    manifest.push_str(&format!("document={}\n", document.title));
    manifest.push_str(&format!("target_language={target_language}\n"));
    manifest.push_str(&format!("provider={provider_name}\n"));
    manifest.push_str(&format!("chapters={}\n", chapters.len()));
    manifest.push_str(&format!("memory_path={}\n", memory_path.display()));
    manifest.push_str(&format!("glossary_path={}\n", glossary_path.display()));
    manifest.push_str(&format!(
        "memory_entries_before={}\n",
        memory.entries().len()
    ));
    manifest.push_str(&format!("glossary_entries={}\n", glossary.entries().len()));

    println!("provider: {provider_name}");
    println!("memory: {} entries", memory.entries().len());
    println!("glossary: {} entries", glossary.entries().len());

    for chapter in chapters {
        let (context, memory_hits, glossary_hits) = runtime_context(
            &document.title,
            &chapter.title,
            &chapter.content,
            &memory,
            &glossary,
        );
        let source_text = chapter.content;
        let output = pipeline
            .execute(
                provider.as_ref(),
                PipelineInput {
                    source_text: source_text.clone(),
                    target_language: target_language.to_string(),
                    context,
                },
            )
            .map_err(|error| format!("pipeline failed for {}: {error}", chapter.title))?;

        let stem = safe_file_stem(&chapter.title, chapter.index);
        let output_path = output_dir.join(format!("{stem}.txt"));
        fs::write(&output_path, &output.quality_review)
            .map_err(|error| format!("failed to write {}: {error}", output_path.display()))?;

        if persist_outputs {
            memory.add(MemoryEntry::new(
                source_text,
                output.quality_review.clone(),
                format!(
                    "document={} | chapter={} | provider={}",
                    document.title, chapter.title, provider_name
                ),
            ));
            save_translation_memory(&memory_path, &memory).map_err(|error| {
                format!(
                    "failed to save translation memory {}: {error}",
                    memory_path.display()
                )
            })?;
        }

        manifest.push_str(&format!(
            "chapter.{}.file={}\nchapter.{}.memory_hits={}\nchapter.{}.glossary_hits={}\n",
            chapter.index + 1,
            output_path.display(),
            chapter.index + 1,
            memory_hits,
            chapter.index + 1,
            glossary_hits
        ));
        println!(
            "{} -> {} ({} bytes, provider={}, memory_hits={}, glossary_hits={})",
            chapter.title,
            output_path.display(),
            output.quality_review.len(),
            output.provider,
            memory_hits,
            glossary_hits
        );
    }

    manifest.push_str(&format!(
        "memory_entries_after={}\n",
        memory.entries().len()
    ));
    manifest.push_str(&format!("memory_persisted={}\n", persist_outputs));
    let manifest_path = output_dir.join("manifest.txt");
    fs::write(&manifest_path, manifest)
        .map_err(|error| format!("failed to write {}: {error}", manifest_path.display()))?;
    println!("manifest -> {}", manifest_path.display());
    Ok(())
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    match args.as_slice() {
        [_, flag] if flag == "--help" || flag == "-h" => {
            usage();
            Ok(())
        }
        [_, flag] if flag == "--version" || flag == "-V" => {
            println!("literary-engine {VERSION}");
            Ok(())
        }
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
    use memory_engine::glossary::GlossaryEntry;

    #[test]
    fn output_file_stems_are_deterministic_and_safe() {
        assert_eq!(
            safe_file_stem("Chapter 1: Arrival", 0),
            "001-Chapter_1__Arrival"
        );
        assert_eq!(safe_file_stem("***", 1), "002-chapter-2");
    }

    #[test]
    fn provider_selection_prefers_openai_when_key_is_available() {
        assert_eq!(selected_provider_name(None, true).unwrap(), "openai");
        assert_eq!(selected_provider_name(None, false).unwrap(), "echo");
    }

    #[test]
    fn explicit_provider_overrides_automatic_selection() {
        assert_eq!(selected_provider_name(Some("echo"), true).unwrap(), "echo");
        assert_eq!(
            selected_provider_name(Some("OpenAI"), false).unwrap(),
            "openai"
        );
    }

    #[test]
    fn unsupported_provider_is_rejected() {
        let error = selected_provider_name(Some("unknown"), true).unwrap_err();
        assert!(error.contains("unsupported provider"));
    }

    #[test]
    fn runtime_context_includes_relevant_memory_and_glossary() {
        let mut memory = TranslationMemory::new();
        memory.add(MemoryEntry::new(
            "The High Warlock whispered softly.".into(),
            "جادوگر اعظم آرام زمزمه کرد.".into(),
            "voice reference".into(),
        ));
        let mut glossary = Glossary::default();
        glossary.add(GlossaryEntry {
            source_term: "High Warlock".into(),
            preferred_translation: "جادوگر اعظم".into(),
            context: "title".into(),
        });

        let (context, memory_hits, glossary_hits) = runtime_context(
            "Book",
            "Chapter 2",
            "The High Warlock whispered softly.",
            &memory,
            &glossary,
        );

        assert_eq!(memory_hits, 1);
        assert_eq!(glossary_hits, 1);
        assert!(context.contains("GLOSSARY"));
        assert!(context.contains("TRANSLATION MEMORY"));
        assert!(context.contains("جادوگر اعظم"));
    }
}
