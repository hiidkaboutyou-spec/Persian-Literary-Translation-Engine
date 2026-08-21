use document_engine::{load_file, split_into_chapters};
use project_engine::{ChapterRecord, ChapterState, ProjectManifest};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn usage() {
    println!("Persian Literary Translation Project CLI");
    println!("Usage:");
    println!("  literary-project init <source-file> [target-language] [project-dir]");
    println!("  literary-project status <project-manifest.json>");
}

fn source_fingerprint(text: &str) -> String {
    const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;
    let hash = text.as_bytes().iter().fold(FNV_OFFSET_BASIS, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(FNV_PRIME)
    });
    format!("fnv1a64-{hash:016x}")
}

fn init_project(source: &str, target_language: &str, project_dir: &Path) -> Result<(), String> {
    let document = load_file(source).map_err(|error| format!("failed to read {source}: {error}"))?;
    let chapters = split_into_chapters(&document.text);
    if chapters.is_empty() {
        return Err("document contains no chapters".to_string());
    }

    fs::create_dir_all(project_dir)
        .map_err(|error| format!("failed to create {}: {error}", project_dir.display()))?;
    let output_dir = project_dir.join("output");
    fs::create_dir_all(&output_dir)
        .map_err(|error| format!("failed to create {}: {error}", output_dir.display()))?;

    let mut manifest = ProjectManifest::new(
        document.title.clone(),
        source,
        target_language,
        output_dir.to_string_lossy(),
    );
    manifest.memory.translation_memory_path = Some(
        project_dir
            .join("translation-memory.json")
            .to_string_lossy()
            .into_owned(),
    );
    manifest.memory.glossary_path = Some(
        project_dir
            .join("glossary.json")
            .to_string_lossy()
            .into_owned(),
    );
    manifest.memory.character_bible_path = Some(
        project_dir
            .join("character-bible.json")
            .to_string_lossy()
            .into_owned(),
    );
    manifest.chapters = chapters
        .into_iter()
        .map(|chapter| ChapterRecord {
            index: chapter.index,
            title: chapter.title,
            state: ChapterState::Pending,
            source_fingerprint: Some(source_fingerprint(&chapter.content)),
            last_error: None,
        })
        .collect();

    let manifest_path = project_dir.join("project.json");
    manifest
        .save_json(&manifest_path)
        .map_err(|error| format!("failed to write {}: {error}", manifest_path.display()))?;

    println!("project: {}", manifest.project_name);
    println!("source: {}", manifest.source_path);
    println!("target_language: {}", manifest.target_language);
    println!("chapters: {}", manifest.chapters.len());
    println!("manifest: {}", manifest_path.display());
    Ok(())
}

fn project_status(manifest_path: &Path) -> Result<(), String> {
    let manifest = ProjectManifest::load_json(manifest_path)
        .map_err(|error| format!("failed to load {}: {error}", manifest_path.display()))?;
    let counts = manifest.chapter_counts();

    println!("project: {}", manifest.project_name);
    println!("source: {}", manifest.source_path);
    println!("target_language: {}", manifest.target_language);
    println!("chapters: {}", manifest.chapters.len());
    println!("pending: {}", counts.pending);
    println!("translating: {}", counts.translating);
    println!("review_needed: {}", counts.review_needed);
    println!("approved: {}", counts.approved);
    println!("blocked: {}", counts.blocked);
    println!("exported: {}", counts.exported);
    Ok(())
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    match args.as_slice() {
        [_, flag] if flag == "--help" || flag == "-h" => {
            usage();
            Ok(())
        }
        [_, command, source] if command == "init" => {
            init_project(source, "fa", Path::new(".literary-project"))
        }
        [_, command, source, target] if command == "init" => {
            init_project(source, target, Path::new(".literary-project"))
        }
        [_, command, source, target, project_dir] if command == "init" => {
            init_project(source, target, Path::new(project_dir))
        }
        [_, command, manifest] if command == "status" => project_status(Path::new(manifest)),
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
    fn fingerprints_are_stable_for_persian_text() {
        assert_eq!(source_fingerprint("سلام دنیا"), source_fingerprint("سلام دنیا"));
        assert_ne!(source_fingerprint("سلام دنیا"), source_fingerprint("سلام جهان"));
    }
}
