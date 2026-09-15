from pathlib import Path
import re


def replace_once(path: Path, old: str, new: str) -> None:
    text = path.read_text()
    if new in text:
        return
    if old not in text:
        raise SystemExit(f"missing patch anchor in {path}: {old[:80]!r}")
    path.write_text(text.replace(old, new, 1))


service = Path("engine/crates/project-engine/src/application/service.rs")
replace_once(
    service,
    "use super::project::{\n",
    "use super::literary_review as literary_review_ops;\nuse super::project::{\n",
)
service_methods = """    // ------------------------------------------------------------------
    // Phase 19 literary translation review
    // ------------------------------------------------------------------

    pub fn review_translation(
        &self,
        project: &Project,
        settings: &literary_review_ops::LiteraryReviewSettings,
    ) -> Result<literary_review_ops::LiteraryReviewRunSummary, ApplicationError> {
        let _lock = ProjectLock::acquire(&project.layout, "literary-review")?;
        literary_review_ops::run_literary_review(&project.layout, settings)
    }

    pub fn get_literary_review(
        &self,
        project: &Project,
        chapter_index: usize,
    ) -> Result<literary_review_ops::LiteraryReviewArtifactView, ApplicationError> {
        literary_review_ops::get_literary_review(&project.layout, chapter_index)
    }

"""
replace_once(
    service,
    "    // ------------------------------------------------------------------\n    // Translation lifecycle\n    // ------------------------------------------------------------------\n",
    service_methods
    + "    // ------------------------------------------------------------------\n    // Translation lifecycle\n    // ------------------------------------------------------------------\n",
)

cli = Path("engine/cli/src/project_cmd.rs")
replace_once(
    cli,
    "    AdvancedAnalysisSettings, ApplicationService, DecisionAction, Project, ReviewedValue,\n    TranslationConfig, VecEventSink,\n",
    "    AdvancedAnalysisSettings, ApplicationService, DecisionAction, LiteraryReviewSettings, Project,\n    ReviewedValue, TranslationConfig, VecEventSink,\n",
)
replace_once(
    cli,
    '        "review" => review(&service, rest, format),\n        "translate" => translate(&service, rest),\n',
    '        "review" => review(&service, rest, format),\n        "review-translation" => review_translation(&service, rest, format),\n        "translate" => translate(&service, rest),\n',
)
replace_once(
    cli,
    "       review <dir> <list|approve-all|promote>          review lifecycle through the application layer\\n\\\n       translate <dir> [--provider echo|auto|openai] [--max-chapters <n>]\\n\\\n",
    "       review <dir> <list|approve-all|promote>          review lifecycle through the application layer\\n\\\n       review-translation <dir> [--provider none|mock|openai] [--no-alignment] [--max-chapters <n>]\\n\\\n       translate <dir> [--provider echo|auto|openai] [--max-chapters <n>]\\n\\\n",
)
review_fn = """fn review_translation(
    service: &ApplicationService,
    args: &[String],
    format: &OutputFormat,
) -> Result<()> {
    let dir = project_path(args, true)?;
    let project = open(service, &dir)?;
    let max_chapters = match flag_value(args, "--max-chapters") {
        Some(value) => Some(
            value
                .parse::<usize>()
                .map_err(|error| format!("invalid --max-chapters '{value}': {error}"))?,
        ),
        None => None,
    };
    let settings = LiteraryReviewSettings {
        provider: flag_value(args, "--provider").unwrap_or("none").to_string(),
        model: flag_value(args, "--model").map(str::to_string),
        semantic_alignment: !args.iter().any(|arg| arg == "--no-alignment"),
        max_chapters,
    };
    let summary = service
        .review_translation(&project, &settings)
        .map_err(|error| format!("literary review failed: {error}"))?;
    match format {
        OutputFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(&summary)
                .map_err(|error| format!("serialization failed: {error}"))?
        ),
        OutputFormat::Text => {
            println!(
                "literary review: {} chapters, {} findings, {} chapters requiring attention",
                summary.reviewed_chapters,
                summary.findings,
                summary.chapters_requiring_attention
            );
            println!(
                "optional evidence failures: alignment {}, provider {}",
                summary.alignment_failures, summary.provider_failures
            );
            for artifact in &summary.artifacts {
                println!("- {artifact}");
            }
        }
    }
    Ok(())
}

"""
replace_once(
    cli,
    "// ---------------------------------------------------------------------------\n// Translation lifecycle\n// ---------------------------------------------------------------------------\n\nfn translation_config",
    "// ---------------------------------------------------------------------------\n// Phase 19 translation review\n// ---------------------------------------------------------------------------\n\n"
    + review_fn
    + "// ---------------------------------------------------------------------------\n// Translation lifecycle\n// ---------------------------------------------------------------------------\n\nfn translation_config",
)

main = Path("engine/cli/src/main.rs")
replace_once(
    main,
    "project <create|import|status|analyze|analyze-advanced|review|translate|resume|progress|export|history>",
    "project <create|import|status|analyze|analyze-advanced|review|review-translation|translate|resume|progress|export|history>",
)

literary = Path("engine/crates/project-engine/src/application/literary_review.rs")
text = literary.read_text()
text, count = re.subn(
    r"\npub fn artifact_path_for_display\(.*?\n\}\n\n#\[cfg\(test\)\]",
    "\n#[cfg(test)]",
    text,
    count=1,
    flags=re.S,
)
if count not in (0, 1):
    raise SystemExit("unexpected artifact_path_for_display patch count")
literary.write_text(text)
