use character_engine::CharacterBible;
use document_engine::{export_persian_docx, load_file, split_into_chapters, Chapter};
use memory_engine::glossary::Glossary;
use memory_engine::{
    build_memory_context, load_glossary, load_translation_memory, MemoryContextConfig,
    TranslationMemory,
};
use quality_engine::{evaluate_translation, TerminologyRule};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use translation_core::{
    prepare_translation, EchoProvider, OpenAIProvider, PipelineInput, TranslationContext,
    TranslationPipeline, TranslationProvider, TranslationRequest,
};

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Default)]
struct RuntimeMemory {
    translation: TranslationMemory,
    glossary: Glossary,
    characters: CharacterBible,
}

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
    println!("Optional persisted project memory:");
    println!("  - LITERARY_ENGINE_MEMORY_FILE=/path/to/translation-memory.json");
    println!("  - LITERARY_ENGINE_GLOSSARY_FILE=/path/to/glossary.json");
    println!("  - LITERARY_ENGINE_CHARACTER_BIBLE_FILE=/path/to/character-bible.json");
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

fn configured_runtime_memory() -> Result<RuntimeMemory, String> {
    let translation = match env::var("LITERARY_ENGINE_MEMORY_FILE") {
        Ok(path) if !path.trim().is_empty() => load_translation_memory(&path)
            .map_err(|error| format!("failed to load translation memory {path}: {error}"))?,
        _ => TranslationMemory::new(),
    };

    let glossary = match env::var("LITERARY_ENGINE_GLOSSARY_FILE") {
        Ok(path) if !path.trim().is_empty() => load_glossary(&path)
            .map_err(|error| format!("failed to load glossary {path}: {error}"))?,
        _ => Glossary::default(),
    };

    let characters = match env::var("LITERARY_ENGINE_CHARACTER_BIBLE_FILE") {
        Ok(path) if !path.trim().is_empty() => CharacterBible::load_json(&path)
            .map_err(|error| format!("failed to load character bible {path}: {error}"))?,
        _ => CharacterBible::new(),
    };

    Ok(RuntimeMemory {
        translation,
        glossary,
        characters,
    })
}

fn chapter_context(
    document_title: &str,
    chapter_title: &str,
    source_text: &str,
    memory: &RuntimeMemory,
) -> String {
    let mut sections = vec![format!(
        "document_title={document_title}\nchapter_title={chapter_title}"
    )];

    let character_context = memory.characters.context_for_text(source_text);
    if !character_context.trim().is_empty() {
        sections.push(format!(
            "CHARACTER BIBLE — preserve voice and relationship continuity:\n{character_context}"
        ));
    }

    let memory_context = build_memory_context(
        source_text,
        &memory.translation,
        &memory.glossary,
        &MemoryContextConfig::default(),
    );
    if !memory_context.text.trim().is_empty() {
        sections.push(memory_context.text);
    }

    sections.join("\n\n")
}

fn terminology_rules(memory: &RuntimeMemory, source_text: &str) -> Vec<TerminologyRule> {
    memory
        .glossary
        .relevant_to_text(source_text)
        .into_iter()
        .map(|entry| {
            TerminologyRule::new(
                entry.source_term.clone(),
                entry.preferred_translation.clone(),
            )
        })
        .collect()
}

fn manifest_value(value: &str) -> String {
    value.replace(['\r', '\n'], " ")
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
    let runtime_memory = configured_runtime_memory()?;
    let pipeline = TranslationPipeline::default_literary_pipeline();
    let mut translated_chapters = Vec::with_capacity(chapters.len());
    let mut manifest = String::new();
    manifest.push_str(&format!("document={}\n", document.title));
    manifest.push_str(&format!("target_language={target_language}\n"));
    manifest.push_str(&format!("provider={provider_name}\n"));
    manifest.push_str(&format!("chapters={}\n", chapters.len()));
    manifest.push_str(&format!(
        "translation_memory_entries={}\n",
        runtime_memory.translation.entries().len()
    ));
    manifest.push_str(&format!(
        "glossary_entries={}\n",
        runtime_memory.glossary.entries().len()
    ));
    manifest.push_str(&format!(
        "character_profiles={}\n",
        runtime_memory.characters.profiles().len()
    ));

    println!("provider: {provider_name}");
    println!(
        "memory: {} translation entries, {} glossary entries, {} character profiles",
        runtime_memory.translation.entries().len(),
        runtime_memory.glossary.entries().len(),
        runtime_memory.characters.profiles().len()
    );

    for chapter in chapters {
        let source_text = chapter.content.clone();
        let context = chapter_context(
            &document.title,
            &chapter.title,
            &source_text,
            &runtime_memory,
        );
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

        let rules = terminology_rules(&runtime_memory, &source_text);
        let quality = evaluate_translation(&source_text, &output.quality_review, &rules);
        if !quality.passes() {
            return Err(format!(
                "quality gate blocked {}: {}",
                chapter.title,
                quality.blocking_errors.join("; ")
            ));
        }

        let stem = safe_file_stem(&chapter.title, chapter.index);
        let output_path = output_dir.join(format!("{stem}.txt"));
        fs::write(&output_path, &output.quality_review)
            .map_err(|error| format!("failed to write {}: {error}", output_path.display()))?;
        translated_chapters.push(Chapter {
            index: chapter.index,
            title: chapter.title.clone(),
            content: output.quality_review.clone(),
        });
        manifest.push_str(&format!(
            "chapter.{}.file={}\n",
            chapter.index + 1,
            output_path.display()
        ));
        manifest.push_str(&format!(
            "chapter.{}.quality_score={:.2}\n",
            chapter.index + 1,
            quality.score
        ));
        manifest.push_str(&format!(
            "chapter.{}.quality_warnings={}\n",
            chapter.index + 1,
            quality.warnings.len()
        ));
        for (warning_index, warning) in quality.warnings.iter().enumerate() {
            manifest.push_str(&format!(
                "chapter.{}.warning.{}={}\n",
                chapter.index + 1,
                warning_index + 1,
                manifest_value(warning)
            ));
            eprintln!("quality warning [{}]: {}", chapter.title, warning);
        }
        println!(
            "{} -> {} ({} bytes, provider={}, quality={:.2})",
            chapter.title,
            output_path.display(),
            output.quality_review.len(),
            output.provider,
            quality.score
        );
    }

    let manuscript_path = output_dir.join("manuscript.docx");
    export_persian_docx(&manuscript_path, &document.title, &translated_chapters)
        .map_err(|error| format!("failed to export {}: {error}", manuscript_path.display()))?;
    manifest.push_str(&format!("manuscript={}\n", manuscript_path.display()));
    println!("manuscript -> {}", manuscript_path.display());

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
    use character_engine::{CharacterProfile, RelationshipProfile};
    use memory_engine::glossary::GlossaryEntry;
    use memory_engine::MemoryEntry;

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
    fn chapter_context_combines_character_glossary_and_translation_memory() {
        let mut runtime = RuntimeMemory::default();
        runtime.translation.add(MemoryEntry::new(
            "Magnus whispered softly".into(),
            "مگنوس آرام زمزمه کرد".into(),
            "intimate dialogue".into(),
        ));
        runtime.glossary.add(GlossaryEntry {
            source_term: "High Warlock".into(),
            preferred_translation: "جادوگر اعظم".into(),
            context: "title".into(),
        });
        runtime.characters.add(CharacterProfile {
            name: "Magnus".into(),
            voice_notes: "witty and affectionate".into(),
            personality_notes: "confident".into(),
        });
        let mut relationship = RelationshipProfile::new("Magnus", "Alec");
        relationship.dynamic_notes = "tender banter".into();
        runtime.characters.add(CharacterProfile {
            name: "Alec".into(),
            voice_notes: "restrained".into(),
            personality_notes: "loyal".into(),
        });
        runtime.characters.add_relationship(relationship);

        let context = chapter_context(
            "Book",
            "Chapter 1",
            "Magnus, the High Warlock, whispered softly to Alec.",
            &runtime,
        );

        assert!(context.contains("CHARACTER BIBLE"));
        assert!(context.contains("witty and affectionate"));
        assert!(context.contains("tender banter"));
        assert!(context.contains("High Warlock => جادوگر اعظم"));
        assert!(context.contains("TRANSLATION MEMORY"));
        assert!(context.contains("مگنوس آرام زمزمه کرد"));
    }

    #[test]
    fn terminology_rules_only_include_terms_present_in_source() {
        let mut runtime = RuntimeMemory::default();
        runtime.glossary.add(GlossaryEntry {
            source_term: "High Warlock".into(),
            preferred_translation: "جادوگر اعظم".into(),
            context: "title".into(),
        });
        runtime.glossary.add(GlossaryEntry {
            source_term: "Parabatai".into(),
            preferred_translation: "پاراباتای".into(),
            context: "title".into(),
        });

        let rules = terminology_rules(&runtime, "The High Warlock smiled.");
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].source_term, "High Warlock");
        assert_eq!(rules[0].preferred_translation, "جادوگر اعظم");
    }
}
