use character_engine::CharacterBible;
use document_engine::{export_persian_docx, ingest_file, Chapter};
use literary_intelligence_engine::{
    AnalysisCanon, DeterministicManuscriptAnalyzer, ManuscriptAnalyzer, ManuscriptIntelligence,
};
use memory_engine::glossary::Glossary;
use memory_engine::{
    build_memory_context, load_glossary, load_translation_memory, MemoryContextConfig,
    TranslationMemory,
};
use quality_engine::{evaluate_translation, TerminologyRule};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use translation_core::{
    prepare_translation, EchoProvider, OpenAIProvider, PipelineInput, TranslationContext,
    TranslationPipeline, TranslationProvider, TranslationRequest,
};

mod review;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum OutputFormat {
    Text,
    Json,
}

impl OutputFormat {
    fn from_arg(value: &str) -> Result<Self, String> {
        match value.to_ascii_lowercase().as_str() {
            "text" | "txt" => Ok(Self::Text),
            "json" => Ok(Self::Json),
            _ => Err(format!(
                "unsupported output format '{value}'; expected text or json"
            )),
        }
    }
}

#[derive(Debug, Default)]
struct RuntimeMemory {
    translation: TranslationMemory,
    glossary: Glossary,
    characters: CharacterBible,
}

#[derive(Serialize)]
struct InspectOutput {
    title: String,
    author: Option<String>,
    total_bytes: usize,
    chapters: Vec<InspectChapter>,
}

#[derive(Serialize)]
struct InspectChapter {
    title: String,
    bytes: usize,
}

#[derive(Serialize)]
struct PrepareOutput {
    document: String,
    chapters: Vec<PrepareChapter>,
}

#[derive(Serialize)]
struct PrepareChapter {
    title: String,
    prepared: String,
}

#[derive(Serialize)]
struct RunManifestOutput {
    document: String,
    target_language: String,
    provider: String,
    resume: bool,
    chapters: Vec<RunChapterOutput>,
    manuscript: Option<String>,
}

#[derive(Serialize)]
struct RunChapterOutput {
    index: usize,
    title: String,
    file: String,
    source_fingerprint: String,
    context_fingerprint: String,
    resumed: bool,
    quality_score: f32,
    quality_warnings: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChapterCheckpoint {
    schema_version: u32,
    source_fingerprint: String,
    context_fingerprint: String,
}

fn usage() {
    println!("Persian Literary Translation Engine v{VERSION}");
    println!("Usage:");
    println!(
        "  literary-engine inspect <file.txt|file.md|file.docx|file.epub|file.pdf> [--format json]"
    );
    println!(
        "  literary-engine analyze <file.txt|file.md|file.docx|file.epub|file.pdf> [--format json]"
    );
    println!("  literary-engine review <sync|list|show|approve|edit|reject|defer|reopen|promote> ... [--format json]");
    println!(
        "  literary-engine prepare <file.txt|file.md|file.docx|file.epub|file.pdf> [target-language] [--format json]"
    );
    println!(
        "  literary-engine run <file.txt|file.md|file.docx|file.epub|file.pdf> [target-language] [output-dir] [--format json]"
    );
    println!(
        "  literary-engine resume <file.txt|file.md|file.docx|file.epub|file.pdf> [target-language] [output-dir] [--format json]"
    );
    println!("  literary-engine --help");
    println!("  literary-engine --version");
    println!();
    println!("Output format:");
    println!("  --format text  Human-readable output (default)");
    println!("  --format json  Machine-readable JSON output");
    println!();
    println!("Provider selection:");
    println!("  - OPENAI_API_KEY set: run/resume uses OpenAI by default");
    println!("  - no OPENAI_API_KEY: run/resume uses the deterministic echo provider");
    println!("  - override with LITERARY_ENGINE_PROVIDER=openai|echo");
    println!("  - override the OpenAI model with OPENAI_MODEL");
    println!();
    println!("Resume behavior:");
    println!("  - resume reuses only chapter outputs with a matching source fingerprint");
    println!("  - reused output must still pass the current quality gate");
    println!("  - changed or invalid chapters are translated again automatically");
    println!();
    println!("Optional persisted project memory:");
    println!("  - LITERARY_ENGINE_MEMORY_FILE=/path/to/translation-memory.json");
    println!("  - LITERARY_ENGINE_GLOSSARY_FILE=/path/to/glossary.json");
    println!("  - LITERARY_ENGINE_CHARACTER_BIBLE_FILE=/path/to/character-bible.json");
    println!("  - LITERARY_ENGINE_REVIEW_FILE=/path/to/intelligence-review.json");
}

fn analyze(path: &str, format: &OutputFormat) -> Result<(), String> {
    let manuscript =
        ingest_file(path).map_err(|error| format!("failed to read {path}: {error}"))?;
    let memory = configured_runtime_memory()?;
    let intelligence = DeterministicManuscriptAnalyzer::default()
        .analyze(
            &manuscript,
            AnalysisCanon {
                characters: &memory.characters,
                glossary: &memory.glossary,
            },
        )
        .map_err(|error| format!("failed to analyze {path}: {error}"))?;

    match format {
        OutputFormat::Text => print_intelligence_summary(&intelligence),
        OutputFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(&intelligence)
                .map_err(|error| format!("failed to serialize analysis: {error}"))?
        ),
    }
    Ok(())
}

fn print_intelligence_summary(intelligence: &ManuscriptIntelligence) {
    println!("manuscript: {}", intelligence.manuscript_title);
    println!("schema_version: {}", intelligence.schema_version);
    println!("chapters: {}", intelligence.chapter_maps.len());
    println!("character_seeds: {}", intelligence.character_seeds.len());
    println!(
        "relationship_seeds: {}",
        intelligence.relationship_seeds.len()
    );
    println!(
        "terminology_seeds: {}",
        intelligence.terminology_seeds.len()
    );
    println!(
        "dialogue_density: {:.3}",
        intelligence.literary_profile.observed.dialogue_density
    );
    println!("conflicts: {}", intelligence.initialization.conflicts.len());
    println!(
        "canon_mutated: {}",
        intelligence.initialization.mutates_canon
    );
}

fn inspect(path: &str, format: &OutputFormat) -> Result<(), String> {
    let manuscript =
        ingest_file(path).map_err(|error| format!("failed to read {path}: {error}"))?;

    match format {
        OutputFormat::Text => {
            println!("title: {}", manuscript.book.title);
            println!(
                "author: {}",
                manuscript.book.author.as_deref().unwrap_or("unknown")
            );
            println!(
                "bytes: {}",
                manuscript
                    .chapters
                    .iter()
                    .map(|chapter| chapter.content.len())
                    .sum::<usize>()
            );
            println!("chapters: {}", manuscript.chapters.len());
            println!("paragraphs: {}", manuscript.paragraph_count());
            for chapter in &manuscript.chapters {
                println!("- {}: {} bytes", chapter.title, chapter.content.len());
            }
        }
        OutputFormat::Json => {
            let output = InspectOutput {
                title: manuscript.book.title,
                author: manuscript.book.author,
                total_bytes: manuscript.chapters.iter().map(|c| c.content.len()).sum(),
                chapters: manuscript
                    .chapters
                    .into_iter()
                    .map(|c| InspectChapter {
                        title: c.title,
                        bytes: c.content.len(),
                    })
                    .collect(),
            };
            println!("{}", serde_json::to_string_pretty(&output).unwrap());
        }
    }
    Ok(())
}

fn prepare(path: &str, target_language: &str, format: &OutputFormat) -> Result<(), String> {
    let manuscript =
        ingest_file(path).map_err(|error| format!("failed to read {path}: {error}"))?;

    match format {
        OutputFormat::Text => {
            println!("document: {}", manuscript.book.title);
            println!("chapters: {}", manuscript.chapters.len());
            for chapter in manuscript.chapters {
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
        }
        OutputFormat::Json => {
            let chapters: Vec<PrepareChapter> = manuscript
                .chapters
                .into_iter()
                .map(|chapter| {
                    let prepared = prepare_translation(
                        TranslationRequest {
                            source: chapter.content,
                            target_language: target_language.to_string(),
                        },
                        TranslationContext {
                            glossary_enabled: true,
                            character_memory_enabled: true,
                        },
                    );
                    PrepareChapter {
                        title: chapter.title,
                        prepared,
                    }
                })
                .collect();
            let output = PrepareOutput {
                document: manuscript.book.title,
                chapters,
            };
            println!("{}", serde_json::to_string_pretty(&output).unwrap());
        }
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

fn source_fingerprint(text: &str) -> String {
    const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;
    let hash = text.as_bytes().iter().fold(FNV_OFFSET_BASIS, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(FNV_PRIME)
    });
    format!("fnv1a64-{hash:016x}")
}

fn resumable_translation(
    output_path: &Path,
    checkpoint_path: &Path,
    source_text: &str,
    expected_context_fingerprint: &str,
) -> Result<Option<String>, String> {
    if !output_path.is_file() || !checkpoint_path.is_file() {
        return Ok(None);
    }

    let expected_source = source_fingerprint(source_text);
    let stored = fs::read_to_string(checkpoint_path)
        .map_err(|error| format!("failed to read {}: {error}", checkpoint_path.display()))?;
    let checkpoint = match serde_json::from_str::<ChapterCheckpoint>(&stored) {
        Ok(checkpoint) if checkpoint.schema_version == 1 => checkpoint,
        Ok(checkpoint) => {
            eprintln!(
                "resume checkpoint {} uses unsupported schema {}; translating again",
                checkpoint_path.display(),
                checkpoint.schema_version
            );
            return Ok(None);
        }
        Err(_) => {
            if stored.trim() == expected_source {
                eprintln!(
                    "legacy resume checkpoint {} has no context fingerprint; translating again",
                    checkpoint_path.display()
                );
            }
            return Ok(None);
        }
    };
    if checkpoint.source_fingerprint != expected_source
        || checkpoint.context_fingerprint != expected_context_fingerprint
    {
        return Ok(None);
    }

    let translated = fs::read_to_string(output_path)
        .map_err(|error| format!("failed to read {}: {error}", output_path.display()))?;
    Ok(Some(translated))
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
    seed_context: Option<&str>,
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

    if let Some(seed_context) = seed_context.filter(|value| !value.trim().is_empty()) {
        sections.push(seed_context.to_string());
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

fn run_pipeline(
    path: &str,
    target_language: &str,
    output_dir: &Path,
    resume: bool,
    format: &OutputFormat,
) -> Result<(), String> {
    let manuscript =
        ingest_file(path).map_err(|error| format!("failed to read {path}: {error}"))?;
    let runtime_memory = configured_runtime_memory()?;
    let intelligence = DeterministicManuscriptAnalyzer::default()
        .analyze(
            &manuscript,
            AnalysisCanon {
                characters: &runtime_memory.characters,
                glossary: &runtime_memory.glossary,
            },
        )
        .map_err(|error| format!("failed to analyze {path}: {error}"))?;
    let document_title = manuscript.book.title.clone();
    let chapters = manuscript.chapters;
    if chapters.is_empty() {
        return Err("document contains no translatable chapters".to_string());
    }

    fs::create_dir_all(output_dir)
        .map_err(|error| format!("failed to create {}: {error}", output_dir.display()))?;

    let provider = configured_provider()?;
    let provider_name = provider.name().to_string();
    let pipeline = TranslationPipeline::default_literary_pipeline();
    let mut translated_chapters = Vec::with_capacity(chapters.len());
    let mut manifest_chapters: Vec<RunChapterOutput> = Vec::with_capacity(chapters.len());
    let mut manifest_text = String::new();
    manifest_text.push_str(&format!("document={}\n", document_title));
    manifest_text.push_str(&format!("target_language={target_language}\n"));
    manifest_text.push_str(&format!("provider={provider_name}\n"));
    manifest_text.push_str(&format!("resume={resume}\n"));
    manifest_text.push_str(&format!("chapters={}\n", chapters.len()));
    manifest_text.push_str(&format!(
        "translation_memory_entries={}\n",
        runtime_memory.translation.entries().len()
    ));
    manifest_text.push_str(&format!(
        "glossary_entries={}\n",
        runtime_memory.glossary.entries().len()
    ));
    manifest_text.push_str(&format!(
        "character_profiles={}\n",
        runtime_memory.characters.profiles().len()
    ));
    manifest_text.push_str(&format!(
        "manuscript_intelligence_schema={}\n",
        intelligence.schema_version
    ));
    manifest_text.push_str(&format!(
        "character_seeds={}\nrelationship_seeds={}\nterminology_seeds={}\n",
        intelligence.character_seeds.len(),
        intelligence.relationship_seeds.len(),
        intelligence.terminology_seeds.len()
    ));

    if *format == OutputFormat::Text {
        println!("provider: {provider_name}");
        println!(
            "memory: {} translation entries, {} glossary entries, {} character profiles",
            runtime_memory.translation.entries().len(),
            runtime_memory.glossary.entries().len(),
            runtime_memory.characters.profiles().len()
        );
    }

    for chapter in &chapters {
        let source_text = chapter.content.clone();
        let rules = terminology_rules(&runtime_memory, &source_text);
        let stem = safe_file_stem(&chapter.title, chapter.index);
        let output_path = output_dir.join(format!("{stem}.txt"));
        let checkpoint_path = output_dir.join(format!("{stem}.source-fingerprint"));
        let fingerprint = source_fingerprint(&source_text);
        let context = chapter_context(
            &document_title,
            &chapter.title,
            &source_text,
            &runtime_memory,
            intelligence.initialization.context_for_chapter(&chapter.id),
        );
        let context_fingerprint = source_fingerprint(&context);

        if resume {
            if let Some(existing) = resumable_translation(
                &output_path,
                &checkpoint_path,
                &source_text,
                &context_fingerprint,
            )? {
                let quality = evaluate_translation(&source_text, &existing, &rules);
                if quality.passes() {
                    translated_chapters.push(Chapter::translated(
                        chapter.index,
                        chapter.title.clone(),
                        existing.clone(),
                    ));
                    let chapter_output = RunChapterOutput {
                        index: chapter.index,
                        title: chapter.title.clone(),
                        file: output_path.display().to_string(),
                        source_fingerprint: fingerprint.clone(),
                        context_fingerprint: context_fingerprint.clone(),
                        resumed: true,
                        quality_score: quality.score,
                        quality_warnings: quality.warnings.clone(),
                    };
                    manifest_chapters.push(chapter_output);
                    manifest_text.push_str(&format!(
                        "chapter.{}.file={}\n",
                        chapter.index + 1,
                        output_path.display()
                    ));
                    manifest_text.push_str(&format!(
                        "chapter.{}.source_fingerprint={}\n",
                        chapter.index + 1,
                        fingerprint
                    ));
                    manifest_text.push_str(&format!(
                        "chapter.{}.context_fingerprint={}\n",
                        chapter.index + 1,
                        context_fingerprint
                    ));
                    manifest_text
                        .push_str(&format!("chapter.{}.resumed=true\n", chapter.index + 1));
                    manifest_text.push_str(&format!(
                        "chapter.{}.quality_score={:.2}\n",
                        chapter.index + 1,
                        quality.score
                    ));
                    manifest_text.push_str(&format!(
                        "chapter.{}.quality_warnings={}\n",
                        chapter.index + 1,
                        quality.warnings.len()
                    ));
                    for (warning_index, warning) in quality.warnings.iter().enumerate() {
                        manifest_text.push_str(&format!(
                            "chapter.{}.warning.{}={}\n",
                            chapter.index + 1,
                            warning_index + 1,
                            manifest_value(warning)
                        ));
                        eprintln!("quality warning [{}]: {}", chapter.title, warning);
                    }
                    if *format == OutputFormat::Text {
                        println!(
                            "{} -> {} (reused checkpoint, {} bytes, quality={:.2})",
                            chapter.title,
                            output_path.display(),
                            existing.len(),
                            quality.score
                        );
                    }
                    continue;
                }

                eprintln!(
                    "resume checkpoint [{}] no longer passes quality; translating again",
                    chapter.title
                );
            }
        }

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

        let quality = evaluate_translation(&source_text, &output.quality_review, &rules);
        if !quality.passes() {
            return Err(format!(
                "quality gate blocked {}: {}",
                chapter.title,
                quality.blocking_errors.join("; ")
            ));
        }

        fs::write(&output_path, &output.quality_review)
            .map_err(|error| format!("failed to write {}: {error}", output_path.display()))?;
        let checkpoint = serde_json::to_string_pretty(&ChapterCheckpoint {
            schema_version: 1,
            source_fingerprint: fingerprint.clone(),
            context_fingerprint: context_fingerprint.clone(),
        })
        .map_err(|error| format!("failed to serialize chapter checkpoint: {error}"))?;
        fs::write(&checkpoint_path, checkpoint)
            .map_err(|error| format!("failed to write {}: {error}", checkpoint_path.display()))?;
        translated_chapters.push(Chapter::translated(
            chapter.index,
            chapter.title.clone(),
            output.quality_review.clone(),
        ));
        let chapter_output = RunChapterOutput {
            index: chapter.index,
            title: chapter.title.clone(),
            file: output_path.display().to_string(),
            source_fingerprint: fingerprint.clone(),
            context_fingerprint: context_fingerprint.clone(),
            resumed: false,
            quality_score: quality.score,
            quality_warnings: quality.warnings.clone(),
        };
        manifest_chapters.push(chapter_output);
        manifest_text.push_str(&format!(
            "chapter.{}.file={}\n",
            chapter.index + 1,
            output_path.display()
        ));
        manifest_text.push_str(&format!(
            "chapter.{}.source_fingerprint={}\n",
            chapter.index + 1,
            fingerprint
        ));
        manifest_text.push_str(&format!(
            "chapter.{}.context_fingerprint={}\n",
            chapter.index + 1,
            context_fingerprint
        ));
        manifest_text.push_str(&format!("chapter.{}.resumed=false\n", chapter.index + 1));
        manifest_text.push_str(&format!(
            "chapter.{}.quality_score={:.2}\n",
            chapter.index + 1,
            quality.score
        ));
        manifest_text.push_str(&format!(
            "chapter.{}.quality_warnings={}\n",
            chapter.index + 1,
            quality.warnings.len()
        ));
        for (warning_index, warning) in quality.warnings.iter().enumerate() {
            manifest_text.push_str(&format!(
                "chapter.{}.warning.{}={}\n",
                chapter.index + 1,
                warning_index + 1,
                manifest_value(warning)
            ));
            eprintln!("quality warning [{}]: {}", chapter.title, warning);
        }
        if *format == OutputFormat::Text {
            println!(
                "{} -> {} ({} bytes, provider={}, quality={:.2})",
                chapter.title,
                output_path.display(),
                output.quality_review.len(),
                output.provider,
                quality.score
            );
        }
    }

    let manuscript_path = output_dir.join("manuscript.docx");
    export_persian_docx(&manuscript_path, &document_title, &translated_chapters)
        .map_err(|error| format!("failed to export {}: {error}", manuscript_path.display()))?;
    manifest_text.push_str(&format!("manuscript={}\n", manuscript_path.display()));
    if *format == OutputFormat::Text {
        println!("manuscript -> {}", manuscript_path.display());
    }

    let manuscript_path_string = manuscript_path.display().to_string();

    match format {
        OutputFormat::Text => {
            let manifest_path = output_dir.join("manifest.txt");
            fs::write(&manifest_path, manifest_text)
                .map_err(|error| format!("failed to write {}: {error}", manifest_path.display()))?;
            println!("manifest -> {}", manifest_path.display());
        }
        OutputFormat::Json => {
            let json_manifest = RunManifestOutput {
                document: document_title,
                target_language: target_language.to_string(),
                provider: provider_name,
                resume,
                chapters: manifest_chapters,
                manuscript: Some(manuscript_path_string),
            };
            let json_path = output_dir.join("manifest.json");
            let json = serde_json::to_string_pretty(&json_manifest).unwrap();
            fs::write(&json_path, &json)
                .map_err(|error| format!("failed to write {}: {error}", json_path.display()))?;
            println!("{json}");

            // Also write text manifest for backward compatibility
            let text_path = output_dir.join("manifest.txt");
            fs::write(&text_path, manifest_text)
                .map_err(|error| format!("failed to write {}: {error}", text_path.display()))?;
        }
    }

    Ok(())
}

fn parse_args() -> (String, Vec<String>, OutputFormat) {
    let args: Vec<String> = env::args().collect();
    let mut format = OutputFormat::Text;
    let mut positional: Vec<String> = Vec::new();

    for arg in &args[1..] {
        if arg == "--format" || arg == "-f" {
            // handled below via peek
            continue;
        }
        if let Some(prev) = args.iter().position(|a| a == arg) {
            if prev > 0 && (args[prev - 1] == "--format" || args[prev - 1] == "-f") {
                format = OutputFormat::from_arg(arg).unwrap_or_else(|error| {
                    eprintln!("error: {error}");
                    std::process::exit(1);
                });
                continue;
            }
        }
        positional.push(arg.clone());
    }

    // Handle --format that appears after its value
    if positional.iter().any(|a| a == "--format" || a == "-f") {
        let pos = positional.iter().position(|a| a == "--format" || a == "-f");
        if let Some(idx) = pos {
            if idx + 1 < positional.len() {
                let value = positional.remove(idx + 1);
                positional.remove(idx);
                format = OutputFormat::from_arg(&value).unwrap_or_else(|error| {
                    eprintln!("error: {error}");
                    std::process::exit(1);
                });
            }
        }
    }

    // Reconstruct the command name
    let command = positional.first().cloned().unwrap_or_default();
    let rest: Vec<String> = positional.into_iter().skip(1).collect();

    (command, rest, format)
}

fn run() -> Result<(), String> {
    let (command, args, format) = parse_args();
    match (command.as_str(), args.as_slice()) {
        ("--help", _) | ("-h", _) => {
            usage();
            Ok(())
        }
        ("--version", _) | ("-V", _) => {
            println!("literary-engine {VERSION}");
            Ok(())
        }
        ("inspect", [path, ..]) => inspect(path, &format),
        ("analyze", [path, ..]) => analyze(path, &format),
        ("review", args) => review::run_review(args, &format),
        ("prepare", [path]) => prepare(path, "fa", &format),
        ("prepare", [path, target]) => prepare(path, target, &format),
        ("run", [path]) => {
            run_pipeline(path, "fa", &PathBuf::from("output/runtime"), false, &format)
        }
        ("run", [path, target]) => run_pipeline(
            path,
            target,
            &PathBuf::from("output/runtime"),
            false,
            &format,
        ),
        ("run", [path, target, output]) => {
            run_pipeline(path, target, Path::new(output), false, &format)
        }
        ("resume", [path]) => {
            run_pipeline(path, "fa", &PathBuf::from("output/runtime"), true, &format)
        }
        ("resume", [path, target]) => run_pipeline(
            path,
            target,
            &PathBuf::from("output/runtime"),
            true,
            &format,
        ),
        ("resume", [path, target, output]) => {
            run_pipeline(path, target, Path::new(output), true, &format)
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
    fn source_fingerprint_is_stable_and_sensitive_to_changes() {
        assert_eq!(
            source_fingerprint("سلام دنیا"),
            source_fingerprint("سلام دنیا")
        );
        assert_ne!(
            source_fingerprint("سلام دنیا"),
            source_fingerprint("سلام دنیا!")
        );
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
            None,
        );

        assert!(context.contains("CHARACTER BIBLE"));
        assert!(context.contains("witty and affectionate"));
        assert!(context.contains("tender banter"));
        assert!(context.contains("High Warlock => جادوگر اعظم"));
        assert!(context.contains("TRANSLATION MEMORY"));
        assert!(context.contains("مگنوس آرام زمزمه کرد"));
    }

    #[test]
    fn approved_context_precedes_manuscript_seed_context() {
        let mut runtime = RuntimeMemory::default();
        runtime.characters.add(CharacterProfile {
            name: "Mina".into(),
            voice_notes: "approved voice".into(),
            personality_notes: String::new(),
        });
        let context = chapter_context(
            "Book",
            "Chapter 1",
            "Mina met Reza.",
            &runtime,
            Some("MANUSCRIPT SEEDS — inferred evidence only:\n- character candidate: Reza"),
        );

        assert!(
            context.find("approved voice").unwrap() < context.find("MANUSCRIPT SEEDS").unwrap()
        );
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

    #[test]
    fn output_format_parses_correctly() {
        assert_eq!(OutputFormat::from_arg("json").unwrap(), OutputFormat::Json);
        assert_eq!(OutputFormat::from_arg("text").unwrap(), OutputFormat::Text);
        assert_eq!(OutputFormat::from_arg("txt").unwrap(), OutputFormat::Text);
        assert!(OutputFormat::from_arg("yaml").is_err());
    }
}
