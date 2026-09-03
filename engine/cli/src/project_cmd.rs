//! `literary-engine project ...` — thin adapter over the Phase 16 application
//! orchestration layer.
//!
//! Every subcommand below calls `project_engine::application::ApplicationService`
//! (the same boundary the future desktop app uses). The CLI never orchestrates
//! engine crates itself here, so `literary-engine project ...` behavior cannot
//! drift from what the desktop app will do.

use crate::OutputFormat;
use human_review_workflow::{ReviewedRelationship, ReviewedTerminology};
use project_engine::application::{
    AdvancedAnalysisSettings, ApplicationService, DecisionAction, Project, ReviewedValue,
    TranslationConfig, VecEventSink,
};
use serde::Serialize;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

type Result<T> = std::result::Result<T, String>;

pub fn run_project(args: &[String], format: &OutputFormat) -> Result<()> {
    let Some(sub) = args.first() else {
        return Err(project_usage());
    };
    let rest = &args[1..];
    let service = ApplicationService;
    match sub.as_str() {
        "create" => create(&service, rest),
        "import" => import(&service, rest),
        "status" | "snapshot" => status(&service, rest, format),
        "analyze" => analyze(&service, rest),
        "analyze-advanced" => analyze_advanced(&service, rest),
        "review" => review(&service, rest, format),
        "translate" => translate(&service, rest),
        "resume" => resume(&service, rest),
        "progress" => progress(&service, rest, format),
        "export" => export(&service, rest),
        "history" => history(&service, rest, format),
        "help" | "--help" | "-h" => {
            println!("{}", project_help());
            Ok(())
        }
        _ => Err(project_usage()),
    }
}

fn project_usage() -> String {
    format!("invalid project command\n{}", project_help())
}

fn project_help() -> &'static str {
    "literary-engine project <command> <dir> [options]\n\
     \n\
       create <dir> [--name <name>] [--source <file>]   create a new translation project\n\
       import <dir> <book>                              import a source book (copy kept inside project)\n\
       status <dir>                                     structured project snapshot (text or --format json)\n\
       analyze <dir>                                    deterministic Phase 13 analysis + review reconcile\n\
       analyze-advanced <dir> [--provider mock|openai]  provider-assisted Phase 15 literary findings\n\
       review <dir> <list|approve-all|promote>          review lifecycle through the application layer\n\
       translate <dir> [--provider echo|auto|openai] [--max-chapters <n>]\n\
       resume <dir>                                     resume an existing translation run\n\
       progress <dir>                                   current translation progress\n\
       export <dir>                                     export translated DOCX\n\
       history <dir>                                    bounded project audit history\n\
     \n\
       The project command is a thin adapter over the same ApplicationService\n\
       the desktop app will call. Deterministic analysis, review, promotion and\n\
       translation are never auto-run: each step is an explicit subcommand.\n\
     Options: --format json is read from the global flag before `project`."
}

fn project_path(args: &[String], required: bool) -> Result<PathBuf> {
    let dir = args.first().ok_or_else(|| {
        "missing project directory (use: literary-engine project <command> <dir>)".to_string()
    })?;
    if required {
        let path = PathBuf::from(dir);
        if !path.join("project.json").exists() {
            return Err(format!(
                "no project found at {} (run 'literary-engine project create <dir>' first)",
                path.display()
            ));
        }
        Ok(path)
    } else {
        Ok(PathBuf::from(dir))
    }
}

fn flag_value<'a>(args: &'a [String], flag: &str) -> Option<&'a str> {
    args.iter()
        .position(|arg| arg == flag)
        .and_then(|index| args.get(index + 1))
        .map(String::as_str)
}

fn open(_service: &ApplicationService, path: &Path) -> Result<Project> {
    ApplicationService::open_project(path)
        .map_err(|error| format!("failed to open project: {error}"))
}

// ---------------------------------------------------------------------------
// Project lifecycle
// ---------------------------------------------------------------------------

fn create(service: &ApplicationService, args: &[String]) -> Result<()> {
    let dir = project_path(args, false)?;
    let name = flag_value(args, "--name")
        .unwrap_or("Untitled Project")
        .to_string();
    let source = flag_value(args, "--source").map(PathBuf::from);
    if dir.join("project.json").exists() {
        return Err(format!(
            "refusing to overwrite existing project at {}",
            dir.display()
        ));
    }
    let mut sink = VecEventSink::new();
    let project = ApplicationService::create_project(&dir, name, source.as_deref(), &mut sink)
        .map_err(|error| format!("failed to create project: {error}"))?;
    let snapshot = service
        .snapshot(&project)
        .map_err(|error| format!("failed to read project state: {error}"))?;
    println!(
        "created project '{}' at {} (id {})",
        snapshot.name,
        dir.display(),
        snapshot.project_id
    );
    println!("next action: {:?}", snapshot.next_action);
    Ok(())
}

fn import(service: &ApplicationService, args: &[String]) -> Result<()> {
    let (dir, book) = match args {
        [dir, book, ..] => (dir, book),
        _ => return Err("usage: literary-engine project import <dir> <book>".to_string()),
    };
    let project = open(service, Path::new(dir))?;
    let mut sink = VecEventSink::new();
    let record = service
        .import_book(&project, Path::new(book), &mut sink)
        .map_err(|error| format!("import failed: {error}"))?;
    println!(
        "imported '{}' ({} chapters, {} scenes, {} paragraphs)",
        record.title, record.chapter_count, record.scene_count, record.paragraph_count
    );
    println!("fingerprint: {}", record.fingerprint);
    println!("next: run 'literary-engine project analyze <dir>'");
    Ok(())
}

fn status(service: &ApplicationService, args: &[String], format: &OutputFormat) -> Result<()> {
    let dir = project_path(args, true)?;
    let project = open(service, &dir)?;
    let snapshot = service
        .snapshot(&project)
        .map_err(|error| format!("failed to read project state: {error}"))?;
    match format {
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&snapshot)
                .map_err(|error| format!("failed to serialize snapshot: {error}"))?;
            println!("{json}");
        }
        OutputFormat::Text => {
            println!("Project: {} (id {})", snapshot.name, snapshot.project_id);
            println!("Status:  {:?}", snapshot.status);
            println!("Next:    {:?}", snapshot.next_action);
            if let Some(source) = &snapshot.source {
                println!(
                    "Source:  {} — {} ({:?})",
                    source.title, source.format, source.state
                );
                println!(
                    "         {} chapters, {} scenes, {} paragraphs",
                    source.chapters, source.scenes, source.paragraphs
                );
            } else {
                println!("Source:  none imported");
            }
            if let Some(analysis) = &snapshot.analysis {
                println!("Analysis: {:?} — {}", analysis.state, analysis.detail);
            } else {
                println!("Analysis: not run");
            }
            if let Some(advanced) = &snapshot.advanced {
                println!("Advanced: {:?} — {}", advanced.state, advanced.detail);
            } else {
                println!("Advanced: not run");
            }
            println!(
                "Review:  {} total ({} pending, {} approved, {} edited, {} rejected, {} deferred, {} applied, {} literary)",
                snapshot.review.total,
                snapshot.review.pending,
                snapshot.review.approved,
                snapshot.review.edited,
                snapshot.review.rejected,
                snapshot.review.deferred,
                snapshot.review.applied,
                snapshot.review.literary
            );
            println!(
                "Canon:   {} characters, {} relationships, {} glossary entries",
                snapshot.canon.characters,
                snapshot.canon.relationships,
                snapshot.canon.glossary_entries
            );
            if let Some(translation) = &snapshot.translation {
                println!(
                    "Translation: {:?} — {}/{} chapters ({:.0}%) via {}",
                    translation.state,
                    translation.completed_chapters,
                    translation.total_chapters,
                    translation.percent * 100.0,
                    translation.provider
                );
                if translation.context_stale {
                    println!("  warning: canon changed after this run started; context is stale");
                }
            } else {
                println!("Translation: not started");
            }
            if let Some(export) = &snapshot.export {
                println!(
                    "Export:  {} ({})",
                    export.relative_path,
                    if export.exists { "ready" } else { "missing" }
                );
            } else {
                println!("Export:  not created");
            }
            for warning in &snapshot.warnings {
                println!("warning: {warning}");
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Analysis
// ---------------------------------------------------------------------------

fn analyze(service: &ApplicationService, args: &[String]) -> Result<()> {
    let dir = project_path(args, true)?;
    let project = open(service, &dir)?;
    let mut sink = VecEventSink::new();
    let record = service
        .analyze_book(&project, &mut sink)
        .map_err(|error| format!("analysis failed: {error}"))?;
    println!(
        "deterministic analysis complete (analyzer {}): {} character seeds, {} relationship seeds, {} terminology seeds",
        record.analyzer, record.character_seeds, record.relationship_seeds, record.terminology_seeds
    );
    println!(
        "review queue: {} added, {} unchanged",
        record.review_items_added, record.review_items_unchanged
    );
    println!("next: run 'literary-engine project review <dir> list'");
    Ok(())
}

fn analyze_advanced(service: &ApplicationService, args: &[String]) -> Result<()> {
    let dir = project_path(args, true)?;
    let project = open(service, &dir)?;
    let provider = flag_value(args, "--provider").unwrap_or("mock");
    let max_units = flag_value(args, "--max-units").map(|value| {
        value
            .parse::<usize>()
            .map_err(|error| format!("invalid --max-units '{value}': {error}"))
    });
    let max_units = match max_units {
        Some(inner) => Some(inner?),
        None => None,
    };
    let settings = AdvancedAnalysisSettings {
        provider: provider.to_string(),
        model: flag_value(args, "--model").map(str::to_string),
        max_units,
        cache: true,
    };
    let mut sink = VecEventSink::new();
    let record = service
        .run_advanced_analysis(&project, &settings, &mut sink)
        .map_err(|error| format!("advanced analysis failed: {error}"))?;
    println!(
        "advanced analysis complete via {}/{}: {} findings in {} units ({} cached, {} failed)",
        record.provider,
        record.model,
        record.findings,
        record.total_units,
        record.cached_units,
        record.failed_units
    );
    println!(
        "review queue now includes literary findings: 'literary-engine project review <dir> list'"
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// Review + canon (thin adapter: Phase 14 rules live in the application layer)
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct ReviewItemRow {
    id: String,
    kind: String,
    status: String,
    availability: String,
    revision: u32,
    subject: String,
    confidence: String,
    evidence_count: usize,
}

fn review(service: &ApplicationService, args: &[String], format: &OutputFormat) -> Result<()> {
    let (dir, sub) = match args {
        [dir, sub, ..] => (dir, sub),
        _ => {
            return Err(
                "usage: literary-engine project review <dir> <list|approve-all|promote>"
                    .to_string(),
            )
        }
    };
    let project = open(service, Path::new(dir))?;
    match sub.as_str() {
        "list" => {
            let items = service
                .list_review_items(&project, None, None)
                .map_err(|error| format!("failed to list review items: {error}"))?;
            match format {
                OutputFormat::Json => {
                    let rows: Vec<ReviewItemRow> = items
                        .iter()
                        .map(|item| ReviewItemRow {
                            id: item.id.clone(),
                            kind: item.kind.clone(),
                            status: item.status.clone(),
                            availability: item.availability.clone(),
                            revision: item.revision,
                            subject: item.subject.clone(),
                            confidence: item.confidence.clone(),
                            evidence_count: item.evidence_count,
                        })
                        .collect();
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&rows)
                            .map_err(|error| format!("serialization failed: {error}"))?
                    );
                }
                OutputFormat::Text => {
                    for item in &items {
                        println!(
                            "{}  {}  {}  {}  rev{}  {}  ({})",
                            item.id,
                            item.kind,
                            item.status,
                            item.availability,
                            item.revision,
                            item.subject,
                            item.evidence_count
                        );
                    }
                    println!("{} items", items.len());
                }
            }
            Ok(())
        }
        "approve-all" => {
            let pending = service
                .list_review_items(&project, None, Some("pending"))
                .map_err(|error| format!("failed to list review items: {error}"))?;
            let mut count = 0usize;
            for item in &pending {
                if item.kind == "literary" {
                    // Literary findings are reviewable but have no canonical
                    // owner; approving records the human review decision.
                    let mut sink = VecEventSink::new();
                    service
                        .decide_review_item(
                            &project,
                            &item.id,
                            DecisionAction::Approve,
                            None,
                            "cli",
                            "approve all",
                            &mut sink,
                        )
                        .map_err(|error| format!("failed to approve {}: {error}", item.id))?;
                } else if item.kind == "terminology" {
                    // Phase 14: terminology needs an explicit translation.
                    let value = ReviewedValue::Terminology(ReviewedTerminology {
                        source_term: item.subject.clone(),
                        preferred_translation: format!("{} (translated)", item.subject),
                        context: String::new(),
                    });
                    let mut sink = VecEventSink::new();
                    service
                        .decide_review_item(
                            &project,
                            &item.id,
                            DecisionAction::Edit,
                            Some(value),
                            "cli",
                            "approve all with default translation",
                            &mut sink,
                        )
                        .map_err(|error| format!("failed to edit {}: {error}", item.id))?;
                } else if item.kind == "relationship" {
                    // Phase 14: relationship seeds carry no invented dynamics;
                    // the human records the observed dynamic at review time.
                    let parts: Vec<&str> = item.subject.split(" / ").collect();
                    let value = ReviewedValue::Relationship(ReviewedRelationship {
                        character_a: parts.first().copied().unwrap_or("").to_string(),
                        character_b: parts.get(1).copied().unwrap_or("").to_string(),
                        dynamic_notes: "relationship observed in manuscript; refine during review"
                            .to_string(),
                        address_notes: String::new(),
                        boundaries_notes: String::new(),
                    });
                    let mut sink = VecEventSink::new();
                    service
                        .decide_review_item(
                            &project,
                            &item.id,
                            DecisionAction::Edit,
                            Some(value),
                            "cli",
                            "approve all with observed relationship",
                            &mut sink,
                        )
                        .map_err(|error| format!("failed to edit {}: {error}", item.id))?;
                } else {
                    let mut sink = VecEventSink::new();
                    service
                        .decide_review_item(
                            &project,
                            &item.id,
                            DecisionAction::Approve,
                            None,
                            "cli",
                            "approve all",
                            &mut sink,
                        )
                        .map_err(|error| format!("failed to approve {}: {error}", item.id))?;
                }
                count += 1;
            }
            println!("approved/edited {count} pending items");
            println!("next: 'literary-engine project review <dir> promote'");
            Ok(())
        }
        "promote" => {
            let all = service
                .list_review_items(&project, None, None)
                .map_err(|error| format!("failed to list review items: {error}"))?;
            let literary: BTreeSet<&str> = all
                .iter()
                .filter(|item| item.kind == "literary")
                .map(|item| item.id.as_str())
                .collect();
            // Explicit selection excludes literary findings (review-only) —
            // matching the Phase 15 rule that model findings never reach canon.
            let selected: Vec<String> = all
                .iter()
                .filter(|item| {
                    matches!(item.status.as_str(), "approved" | "edited")
                        && !literary.contains(item.id.as_str())
                })
                .map(|item| item.id.clone())
                .collect();
            if selected.is_empty() {
                return Err("nothing to promote: no approved/edited items".to_string());
            }
            let plan = service
                .preview_promotion(&project, &selected, &[])
                .map_err(|error| format!("promotion preview blocked: {error}"))?;
            if plan.blocked {
                for conflict in &plan.conflicts {
                    println!(
                        "conflict: [{:?}] {} — {}",
                        conflict.severity, conflict.affected_target, conflict.message
                    );
                }
                return Err(
                    "promotion plan has unresolved blocking conflicts; resolve them and retry"
                        .to_string(),
                );
            }
            let mut sink = VecEventSink::new();
            let result = service
                .apply_promotion(&project, &plan, "cli", "promote approved canon", &mut sink)
                .map_err(|error| format!("promotion apply failed: {error}"))?;
            println!(
                "promoted {} items to canon ({} applied ids)",
                plan.promoted_item_ids.len(),
                result.applied_item_ids.len()
            );
            Ok(())
        }
        _ => Err(
            "usage: literary-engine project review <dir> <list|approve-all|promote>".to_string(),
        ),
    }
}

// ---------------------------------------------------------------------------
// Translation lifecycle
// ---------------------------------------------------------------------------

fn translation_config(args: &[String]) -> Result<TranslationConfig> {
    let provider = flag_value(args, "--provider").unwrap_or("echo").to_string();
    let max_chapters = match flag_value(args, "--max-chapters") {
        Some(value) => Some(
            value
                .parse::<usize>()
                .map_err(|error| format!("invalid --max-chapters '{value}': {error}"))?,
        ),
        None => None,
    };
    let target = flag_value(args, "--target").unwrap_or("fa").to_string();
    Ok(TranslationConfig {
        provider,
        model: flag_value(args, "--model").map(str::to_string),
        target_language: target,
        max_chapters,
    })
}

fn print_progress(
    progress: &project_engine::application::TranslationProgress,
    format: &OutputFormat,
) {
    match format {
        OutputFormat::Json => {
            if let Ok(json) = serde_json::to_string_pretty(progress) {
                println!("{json}");
            }
        }
        OutputFormat::Text => {
            println!(
                "run {}: {:?} — {}/{} chapters ({:.0}%), {} paragraphs translated via {}",
                progress.run_id,
                progress.state,
                progress.completed_chapters,
                progress.total_chapters,
                progress.percent * 100.0,
                progress.completed_paragraphs,
                progress.provider
            );
        }
    }
}

fn translate(service: &ApplicationService, args: &[String]) -> Result<()> {
    let dir = project_path(args, true)?;
    let project = open(service, &dir)?;
    let config = translation_config(args)?;
    let mut sink = VecEventSink::new();
    let progress = service
        .start_translation(&project, &config, &mut sink)
        .map_err(|error| format!("translation failed: {error}"))?;
    print_progress(&progress, &OutputFormat::Text);
    println!("next: 'literary-engine project status <dir>' or 'resume/export'");
    Ok(())
}

fn resume(service: &ApplicationService, args: &[String]) -> Result<()> {
    let dir = project_path(args, true)?;
    let project = open(service, &dir)?;
    let config = translation_config(args)?;
    let mut sink = VecEventSink::new();
    let progress = service
        .resume_translation(&project, &config, &mut sink)
        .map_err(|error| format!("resume failed: {error}"))?;
    print_progress(&progress, &OutputFormat::Text);
    Ok(())
}

fn progress(service: &ApplicationService, args: &[String], format: &OutputFormat) -> Result<()> {
    let dir = project_path(args, true)?;
    let project = open(service, &dir)?;
    let progress = service
        .get_progress(&project)
        .map_err(|error| format!("failed to read progress: {error}"))?;
    print_progress(&progress, format);
    Ok(())
}

fn export(service: &ApplicationService, args: &[String]) -> Result<()> {
    let dir = project_path(args, true)?;
    let project = open(service, &dir)?;
    let mut sink = VecEventSink::new();
    let record = service
        .export_project(&project, &mut sink)
        .map_err(|error| format!("export failed: {error}"))?;
    println!(
        "exported {} ({} chapters) to {}",
        record.format,
        record.chapters,
        dir.join(&record.relative_path).display()
    );
    Ok(())
}

fn history(service: &ApplicationService, args: &[String], format: &OutputFormat) -> Result<()> {
    let dir = project_path(args, true)?;
    let project = open(service, &dir)?;
    let events = service
        .history(&project)
        .map_err(|error| format!("failed to read history: {error}"))?;
    match format {
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&events)
                .map_err(|error| format!("serialization failed: {error}"))?;
            println!("{json}");
        }
        OutputFormat::Text => {
            for event in &events {
                println!("{}  {}  {}", event.timestamp, event.event, event.detail);
            }
        }
    }
    Ok(())
}
