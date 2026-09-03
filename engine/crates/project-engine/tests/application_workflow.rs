//! Phase 16 — application orchestration layer tests.
//!
//! Everything here goes through `ApplicationService` (the same boundary the
//! CLI and the future desktop app use). No test orchestrates engine crates
//! directly and no test touches engine JSON files except where the test *is*
//! about persistence/crash behavior.

use project_engine::application::{
    silent_sink, AdvancedAnalysisSettings, ApplicationService, ArtifactState, DecisionAction,
    NextAction, ProjectEvent, ProjectStatus, TranslationConfig, TranslationState,
};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Unique temp root per test invocation.
fn temp_root(tag: &str) -> std::path::PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("p16-{tag}-{nonce}"))
}

/// Write a synthetic 3-chapter manuscript (Markdown headings → chapters).
fn write_manuscript(path: &Path, chapters: usize, with_unicode: bool) {
    let mut content = String::new();
    for index in 1..=chapters {
        content.push_str(&format!("# Chapter {index}\n\n"));
        content.push_str(&format!(
            "Shirin walked into the garden of roses. The fountain murmured {}.\n\n",
            if with_unicode {
                "به باغ گل‌ها رفت — «سلام!» — دخترِ خندان پاسخ داد"
            } else {
                "She whispered a quiet greeting to the gardener."
            }
        ));
        content.push_str(&format!(
            "Farhad watched from the terrace, his hands trembling {}.\n\n",
            if with_unicode {
                "دست‌هایش می‌لرزید…"
            } else {
                "with quiet longing."
            }
        ));
        content.push_str("A nightingale sang from the cypress tree.\n\n");
    }
    std::fs::write(path, content).unwrap();
}

/// Build a fresh project and return the opened project plus its root.
fn fresh_project(tag: &str) -> (project_engine::application::Project, std::path::PathBuf) {
    let root = temp_root(tag);
    let mut sink = silent_sink();
    let project = ApplicationService::create_project(&root, "Test Book", None, &mut sink).unwrap();
    (project, root)
}

/// Import a manuscript into the project.
fn import_manuscript(project: &project_engine::application::Project, chapters: usize) {
    let service = ApplicationService;
    let source = project.layout.root.join(format!("input-{chapters}.md"));
    write_manuscript(&source, chapters, false);
    let mut sink = silent_sink();
    service.import_book(project, &source, &mut sink).unwrap();
}

fn run_analysis(project: &project_engine::application::Project) {
    let service = ApplicationService;
    let mut sink = silent_sink();
    service.analyze_book(project, &mut sink).unwrap();
}

fn run_advanced(project: &project_engine::application::Project) {
    let service = ApplicationService;
    let mut sink = silent_sink();
    let settings = AdvancedAnalysisSettings::default();
    service
        .run_advanced_analysis(project, &settings, &mut sink)
        .unwrap();
}

/// Approve every pending review item, returning the approved ids.
///
/// Phase 14 requires terminology approvals to carry an explicit preferred
/// translation (seeds never invent one), so those are supplied here.
fn approve_all_pending(project: &project_engine::application::Project) -> Vec<String> {
    use human_review_workflow::ReviewedTerminology;
    use project_engine::application::ReviewedValue;
    let service = ApplicationService;
    let pending = service
        .list_review_items(project, None, Some("pending"))
        .unwrap();
    let mut ids = Vec::new();
    for item in &pending {
        let mut sink = silent_sink();
        let (action, replacement) = if item.kind == "terminology" {
            // Phase 14: terminology approval must carry an explicit preferred
            // translation, and `Approve` re-validates the proposal value — so
            // supplying the translation is an `Edit` with the replacement.
            (
                DecisionAction::Edit,
                Some(ReviewedValue::Terminology(ReviewedTerminology {
                    source_term: item.subject.clone(),
                    preferred_translation: format!("{} (translated)", item.subject),
                    context: String::new(),
                })),
            )
        } else {
            (DecisionAction::Approve, None)
        };
        service
            .decide_review_item(
                project,
                &item.id,
                action,
                replacement,
                "tester",
                "approve",
                &mut sink,
            )
            .unwrap();
        ids.push(item.id.clone());
    }
    ids
}

/// Promote the approved items. Literary findings are review-only (they have
/// no canonical owner), so they are filtered out of the selection — matching
/// the Phase 15 rule that model findings never reach canon.
fn promote(project: &project_engine::application::Project, ids: &[String]) {
    let service = ApplicationService;
    let all = service.list_review_items(project, None, None).unwrap();
    let literary: std::collections::HashSet<String> = all
        .iter()
        .filter(|item| item.kind == "literary")
        .map(|item| item.id.clone())
        .collect();
    let promotable: Vec<String> = ids
        .iter()
        .filter(|id| !literary.contains(*id))
        .cloned()
        .collect();
    assert!(!promotable.is_empty(), "expected promotable approved items");
    let plan = service
        .preview_promotion(project, &promotable, &[])
        .unwrap();
    let mut sink = silent_sink();
    service
        .apply_promotion(project, &plan, "tester", "apply", &mut sink)
        .unwrap();
}

fn start_echo_translation(
    project: &project_engine::application::Project,
    max_chapters: Option<usize>,
) {
    let service = ApplicationService;
    let mut sink = silent_sink();
    let config = TranslationConfig {
        provider: "echo".to_string(),
        target_language: "fa".to_string(),
        max_chapters,
        ..TranslationConfig::default()
    };
    service
        .start_translation(project, &config, &mut sink)
        .unwrap();
}

// ---------------------------------------------------------------------------
// Project lifecycle
// ---------------------------------------------------------------------------

#[test]
fn project_lifecycle_create_open_import_snapshot() {
    let service = ApplicationService;
    let (project, root) = fresh_project("lifecycle");
    let mut sink = silent_sink();

    // Empty project snapshot.
    let snap = service.snapshot(&project).unwrap();
    assert_eq!(snap.status, ProjectStatus::Empty);
    assert_eq!(snap.next_action, NextAction::ImportBook);
    assert!(snap.source.is_none());
    assert_eq!(snap.review.pending, 0);

    // Directories exist.
    assert!(root.join("project.json").exists());
    assert!(root.join("source").is_dir());
    assert!(root.join("intelligence").is_dir());
    assert!(root.join("review").is_dir());
    assert!(root.join("canon").is_dir());
    assert!(root.join("translation").is_dir());
    assert!(root.join("history").is_dir());
    assert!(root.join("exports").is_dir());

    // Import.
    import_manuscript(&project, 3);
    let snap = service.snapshot(&project).unwrap();
    let source_summary = snap.source.as_ref().unwrap();
    assert_eq!(source_summary.chapters, 3);
    assert_eq!(source_summary.paragraphs, 9);
    assert_eq!(snap.status, ProjectStatus::Imported);
    assert_eq!(snap.next_action, NextAction::RunAnalysis);

    // Source was copied into the project (immutable copy).
    assert!(root.join("source").read_dir().unwrap().next().is_some());

    // Reopen from disk.
    let reopened = ApplicationService::open_project(&root).unwrap();
    let snap2 = service.snapshot(&reopened).unwrap();
    assert_eq!(snap2.project_id, snap.project_id);
    assert_eq!(snap2.source.as_ref().unwrap().chapters, 3);

    // create_project must refuse to silently overwrite an existing project.
    let err = ApplicationService::create_project(&root, "Clobber", None, &mut sink);
    assert!(err.is_err(), "overwriting an existing project must fail");

    // Opening a non-project must error.
    let bogus = temp_root("bogus");
    std::fs::create_dir_all(&bogus).unwrap();
    assert!(ApplicationService::open_project(&bogus).is_err());
}

// ---------------------------------------------------------------------------
// Full workflow — the Phase 16 exit test
// ---------------------------------------------------------------------------

#[test]
fn full_workflow_through_application_apis() {
    let service = ApplicationService;
    let (project, root) = fresh_project("full");
    let mut sink = silent_sink();
    import_manuscript(&project, 3);

    // ---- Deterministic analysis ----
    let record = service.analyze_book(&project, &mut sink).unwrap();
    assert!(record.character_seeds > 0);
    assert!(record.review_items_added > 0);
    let snap = service.snapshot(&project).unwrap();
    assert_eq!(snap.status, ProjectStatus::NeedsReview);
    assert_eq!(snap.next_action, NextAction::ReviewIntelligence);
    assert!(snap.review.pending > 0);

    // ---- Advanced analysis (mock, offline) ----
    run_advanced(&project);
    let snap = service.snapshot(&project).unwrap();
    assert!(snap.advanced.is_some());
    let advanced = snap.advanced.as_ref().unwrap();
    assert_eq!(advanced.state, ArtifactState::Fresh);
    assert!(
        snap.review.literary > 0,
        "advanced findings enter the review queue"
    );

    // ---- Review: approve everything ----
    let approved = approve_all_pending(&project);
    assert!(!approved.is_empty());
    let snap = service.snapshot(&project).unwrap();
    assert_eq!(snap.review.pending, 0);
    assert_eq!(
        snap.review.total,
        snap.review.approved + snap.review.edited + snap.review.rejected + snap.review.deferred
    );

    // ---- Canon promotion ----
    promote(&project, &approved);
    let snap = service.snapshot(&project).unwrap();
    assert!(snap.canon.characters > 0);
    assert!(snap.canon.glossary_entries > 0);
    assert_eq!(snap.next_action, NextAction::StartTranslation);

    // ---- Translation (EchoProvider, offline) ----
    start_echo_translation(&project, None);
    let snap = service.snapshot(&project).unwrap();
    let translation = snap.translation.as_ref().unwrap();
    assert_eq!(translation.state, TranslationState::Completed);
    assert_eq!(translation.completed_chapters, 3);
    assert_eq!(translation.percent, 1.0);

    // ---- Read translated content ----
    let chapter = service.get_translated_chapter(&project, 0).unwrap();
    assert_eq!(chapter.chapter_index, 0);
    assert!(!chapter.paragraphs.is_empty());
    assert!(!service.get_translated_text(&project, 0).unwrap().is_empty());

    // ---- Export ----
    let export = service.export_project(&project, &mut sink).unwrap();
    assert_eq!(export.format, "docx");
    assert!(root.join("exports").join(&export.relative_path).exists());

    // ---- Final snapshot ----
    let snap = service.snapshot(&project).unwrap();
    assert_eq!(snap.status, ProjectStatus::Exported);
    assert_eq!(snap.next_action, NextAction::None);

    // ---- Reopen: everything still there ----
    let reopened = ApplicationService::open_project(&root).unwrap();
    let snap = service.snapshot(&reopened).unwrap();
    assert_eq!(snap.status, ProjectStatus::Exported);
    assert_eq!(snap.translation.as_ref().unwrap().completed_chapters, 3);
    assert!(snap.export.as_ref().unwrap().exists);

    // History tracks the workflow.
    let events = service.history(&reopened).unwrap();
    let names: Vec<&str> = events.iter().map(|event| event.event.as_str()).collect();
    for expected in [
        "project_created",
        "source_imported",
        "analysis_completed",
        "advanced_analysis_completed",
        "canon_promoted",
        "translation_completed",
        "export_completed",
    ] {
        assert!(
            names.contains(&expected),
            "history should contain {expected}; got {names:?}"
        );
    }

    // Source still verifies as fresh.
    assert_eq!(
        service.verify_source(&reopened).unwrap(),
        ArtifactState::Fresh
    );
}

// ---------------------------------------------------------------------------
// Snapshot accuracy across stages
// ---------------------------------------------------------------------------

#[test]
fn snapshot_accuracy_across_stages() {
    let service = ApplicationService;
    let (project, _) = fresh_project("snapacc");

    let snap = service.snapshot(&project).unwrap();
    assert_eq!(snap.status, ProjectStatus::Empty);

    import_manuscript(&project, 3);
    let snap = service.snapshot(&project).unwrap();
    assert_eq!(snap.status, ProjectStatus::Imported);

    run_analysis(&project);
    let snap = service.snapshot(&project).unwrap();
    assert_eq!(snap.status, ProjectStatus::NeedsReview);
    assert!(snap.review.pending >= snap.review.literary);

    run_advanced(&project);
    let snap = service.snapshot(&project).unwrap();
    assert!(snap.review.literary > 0);

    let approved = approve_all_pending(&project);
    promote(&project, &approved);
    let snap = service.snapshot(&project).unwrap();
    assert_eq!(snap.status, ProjectStatus::ReadyToTranslate);

    start_echo_translation(&project, Some(1));
    let snap = service.snapshot(&project).unwrap();
    assert_eq!(snap.status, ProjectStatus::Paused);
    assert_eq!(snap.translation.as_ref().unwrap().completed_chapters, 1);

    // Resume to completion.
    let mut sink = silent_sink();
    let config = TranslationConfig {
        provider: "echo".to_string(),
        target_language: "fa".to_string(),
        max_chapters: None,
        ..TranslationConfig::default()
    };
    service
        .resume_translation(&project, &config, &mut sink)
        .unwrap();
    let snap = service.snapshot(&project).unwrap();
    assert_eq!(snap.status, ProjectStatus::ReadyToExport);

    service.export_project(&project, &mut sink).unwrap();
    let snap = service.snapshot(&project).unwrap();
    assert_eq!(snap.status, ProjectStatus::Exported);
    assert_eq!(snap.next_action, NextAction::None);
}

// ---------------------------------------------------------------------------
// Review lifecycle integration (Phase 14 rules preserved)
// ---------------------------------------------------------------------------

#[test]
fn review_lifecycle_reject_defer_approve() {
    let service = ApplicationService;
    let (project, _) = fresh_project("review");
    import_manuscript(&project, 3);
    run_analysis(&project);

    let items = service.list_review_items(&project, None, None).unwrap();
    assert!(!items.is_empty());

    // Reject the first item, defer the second, approve the rest.
    let mut deferred_id = None;
    let mut approved = Vec::new();
    for (index, item) in items.iter().enumerate() {
        let mut sink = silent_sink();
        match index {
            0 => {
                service
                    .decide_review_item(
                        &project,
                        &item.id,
                        DecisionAction::Reject,
                        None,
                        "tester",
                        "not needed",
                        &mut sink,
                    )
                    .unwrap();
            }
            1 => {
                service
                    .decide_review_item(
                        &project,
                        &item.id,
                        DecisionAction::Defer,
                        None,
                        "tester",
                        "later",
                        &mut sink,
                    )
                    .unwrap();
                deferred_id = Some(item.id.clone());
            }
            _ => {
                // Phase 14: terminology approvals must carry an explicit
                // preferred translation (an `Edit`); approve everything else
                // as-is.
                let (action, replacement) = if item.kind == "terminology" {
                    use human_review_workflow::ReviewedTerminology;
                    use project_engine::application::ReviewedValue;
                    (
                        DecisionAction::Edit,
                        Some(ReviewedValue::Terminology(ReviewedTerminology {
                            source_term: item.subject.clone(),
                            preferred_translation: format!("{} (translated)", item.subject),
                            context: String::new(),
                        })),
                    )
                } else {
                    (DecisionAction::Approve, None)
                };
                service
                    .decide_review_item(
                        &project,
                        &item.id,
                        action,
                        replacement,
                        "tester",
                        "ok",
                        &mut sink,
                    )
                    .unwrap();
                approved.push(item.id.clone());
            }
        }
    }
    let snap = service.snapshot(&project).unwrap();
    assert_eq!(snap.review.pending, 0);
    assert_eq!(snap.review.rejected, 1);
    assert_eq!(snap.review.deferred, 1);
    assert_eq!(
        snap.review.total,
        snap.review.rejected + snap.review.deferred + snap.review.approved + snap.review.edited
    );

    // Promotion of the approved subset works; the deferred item stays out.
    let plan = service.preview_promotion(&project, &approved, &[]).unwrap();
    assert!(plan
        .promoted_item_ids
        .iter()
        .all(|id| approved.contains(id)));
    let mut sink = silent_sink();
    service
        .apply_promotion(&project, &plan, "tester", "apply", &mut sink)
        .unwrap();

    // Reopen the *rejected* item (Phase 14: Reopen is valid from Rejected,
    // while Deferred items are acted on directly).
    let rejected_id = items[0].id.clone();
    let mut sink = silent_sink();
    service
        .decide_review_item(
            &project,
            &rejected_id,
            DecisionAction::Reopen,
            None,
            "tester",
            "reopen",
            &mut sink,
        )
        .unwrap();
    let snap = service.snapshot(&project).unwrap();
    assert_eq!(snap.review.rejected, 0);
    assert_eq!(snap.review.pending, 1);

    // The deferred item can be approved directly (Deferred -> Approved);
    // terminology still needs an explicit preferred translation.
    let deferred_id = deferred_id.unwrap();
    let mut sink = silent_sink();
    let (action, replacement) = if items[1].kind == "terminology" {
        use human_review_workflow::ReviewedTerminology;
        use project_engine::application::ReviewedValue;
        (
            DecisionAction::Edit,
            Some(ReviewedValue::Terminology(ReviewedTerminology {
                source_term: items[1].subject.clone(),
                preferred_translation: format!("{} (translated)", items[1].subject),
                context: String::new(),
            })),
        )
    } else {
        (DecisionAction::Approve, None)
    };
    service
        .decide_review_item(
            &project,
            &deferred_id,
            action,
            replacement,
            "tester",
            "ok now",
            &mut sink,
        )
        .unwrap();
    let snap = service.snapshot(&project).unwrap();
    assert_eq!(snap.review.deferred, 0);

    // Canon is populated only from approved items.
    assert!(!service.list_characters(&project).unwrap().is_empty());
    assert!(!service.list_glossary_entries(&project).unwrap().is_empty());
}

// ---------------------------------------------------------------------------
// Character Bible / Glossary APIs
// ---------------------------------------------------------------------------

#[test]
fn character_and_glossary_apis() {
    let service = ApplicationService;
    let (project, _) = fresh_project("canon");
    import_manuscript(&project, 3);
    run_analysis(&project);
    let approved = approve_all_pending(&project);
    promote(&project, &approved);

    let characters = service.list_characters(&project).unwrap();
    assert!(!characters.is_empty());
    let fetched = service
        .get_character(&project, &characters[0].name)
        .unwrap();
    assert!(fetched.is_some());

    let glossary = service.list_glossary_entries(&project).unwrap();
    assert!(!glossary.is_empty());

    // Direct upsert is available for safe canonical edits.
    let profile = character_engine::CharacterProfile {
        name: "بهار".to_string(),
        voice_notes: "gentle, lyrical".to_string(),
        personality_notes: "warm".to_string(),
    };
    service.upsert_character(&project, profile, false).unwrap();
    assert!(service.get_character(&project, "بهار").unwrap().is_some());

    let entry = memory_engine::glossary::GlossaryEntry {
        source_term: "گل".to_string(),
        preferred_translation: "rose".to_string(),
        context: "poetry".to_string(),
    };
    service
        .upsert_glossary_entry(&project, entry, false)
        .unwrap();
    let fetched = service.get_glossary_entry(&project, "گل").unwrap();
    assert_eq!(fetched.unwrap().preferred_translation, "rose");
}

// ---------------------------------------------------------------------------
// Translation lifecycle: progress, pause, resume, manual edits
// ---------------------------------------------------------------------------

#[test]
fn translation_progress_pause_resume_and_manual_edit() {
    let service = ApplicationService;
    let (project, root) = fresh_project("translate");
    import_manuscript(&project, 3);
    run_analysis(&project);
    let approved = approve_all_pending(&project);
    promote(&project, &approved);

    // Start with a 1-chapter bound → pauses with resumable state.
    start_echo_translation(&project, Some(1));
    let progress = service.get_progress(&project).unwrap();
    assert_eq!(progress.state, TranslationState::Paused);
    assert_eq!(progress.completed_chapters, 1);
    assert_eq!(progress.total_chapters, 3);
    assert!((progress.percent - 1.0 / 3.0).abs() < 0.001);
    assert!(progress.completed_paragraphs > 0);
    assert_eq!(progress.completed_paragraphs, progress.total_paragraphs / 3);
    assert!(!progress.run_id.is_empty());

    // Pause request while already paused is a no-op success.
    service.request_pause(&project).unwrap();

    // Resume → completes.
    let mut sink = silent_sink();
    let config = TranslationConfig {
        provider: "echo".to_string(),
        target_language: "fa".to_string(),
        max_chapters: None,
        ..TranslationConfig::default()
    };
    let progress = service
        .resume_translation(&project, &config, &mut sink)
        .unwrap();
    assert_eq!(progress.state, TranslationState::Completed);
    assert_eq!(progress.completed_chapters, 3);
    assert_eq!(progress.percent, 1.0);

    // Monotonic progress: final >= every intermediate.
    assert!(progress.completed_paragraphs >= progress.total_paragraphs / 3);

    // Manual edit with revision history.
    let chapter = service.get_translated_chapter(&project, 0).unwrap();
    let paragraph = &chapter.paragraphs[0];
    let original_text = paragraph.translated.clone();
    let mut sink = silent_sink();
    let revision = service
        .apply_manual_translation_edit(
            &project,
            0,
            &paragraph.paragraph_id,
            "ترجمهٔ بازبینی‌شده — ویرایش انسانی.",
            Some("editor"),
            &mut sink,
        )
        .unwrap();
    assert_eq!(revision.previous, original_text);
    assert_eq!(revision.new, "ترجمهٔ بازبینی‌شده — ویرایش انسانی.");
    assert_eq!(revision.origin, "manual");

    // Reload: edited text visible, original preserved in revision history.
    let chapter = service.get_translated_chapter(&project, 0).unwrap();
    let paragraph = chapter
        .paragraphs
        .iter()
        .find(|paragraph| paragraph.paragraph_id == revision.paragraph_id)
        .unwrap();
    assert_eq!(paragraph.translated, revision.new);
    assert_eq!(paragraph.origin, "manual");
    assert_eq!(paragraph.revisions.len(), 1);
    assert_eq!(paragraph.revisions[0].previous, original_text);
    assert!(paragraph.revisions[0].previous != revision.new);

    // Original source paragraph untouched.
    assert!(!paragraph.source.is_empty());

    // Reopen: everything survives.
    let reopened = ApplicationService::open_project(&root).unwrap();
    let chapter = service.get_translated_chapter(&reopened, 0).unwrap();
    let paragraph = chapter
        .paragraphs
        .iter()
        .find(|paragraph| paragraph.paragraph_id == revision.paragraph_id)
        .unwrap();
    assert_eq!(paragraph.translated, revision.new);
}

#[test]
fn translation_requires_analysis_before_start() {
    let service = ApplicationService;
    let (project, _) = fresh_project("gating");
    import_manuscript(&project, 3);

    // No analysis yet → cannot translate (deterministic intelligence is
    // required for chapter context).
    let config = TranslationConfig {
        provider: "echo".to_string(),
        target_language: "fa".to_string(),
        max_chapters: None,
        ..TranslationConfig::default()
    };
    let mut sink = silent_sink();
    assert!(service
        .start_translation(&project, &config, &mut sink)
        .is_err());

    // After analysis, translation may start even before canon promotion
    // (canon is optional context; analysis is the hard gate).
    run_analysis(&project);
    let progress = service
        .start_translation(&project, &config, &mut sink)
        .unwrap();
    assert_eq!(progress.state, TranslationState::Completed);
}

// ---------------------------------------------------------------------------
// Staleness: source mismatch and canon revision
// ---------------------------------------------------------------------------

#[test]
fn source_mismatch_is_detected() {
    let service = ApplicationService;
    let (project, root) = fresh_project("sourcestale");
    import_manuscript(&project, 3);
    run_analysis(&project);
    let approved = approve_all_pending(&project);
    promote(&project, &approved);
    start_echo_translation(&project, None);

    // Tamper with the project's immutable source copy (size change).
    let stored = root
        .join("source")
        .read_dir()
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let mut bytes = std::fs::read(&stored).unwrap();
    bytes.push(b'x');
    std::fs::write(&stored, bytes).unwrap();

    // Snapshot surfaces the warning + Error status with ReimportSource action.
    let snap = service.snapshot(&project).unwrap();
    assert_eq!(snap.status, ProjectStatus::Error);
    assert_eq!(snap.next_action, NextAction::ReimportSource);
    assert!(!snap.warnings.is_empty());
    assert_eq!(
        service.verify_source(&project).unwrap(),
        ArtifactState::Stale
    );

    // Restore size → fresh again.
    let mut bytes = std::fs::read(&stored).unwrap();
    bytes.pop();
    std::fs::write(&stored, bytes).unwrap();
    assert_eq!(
        service.verify_source(&project).unwrap(),
        ArtifactState::Fresh
    );
}

#[test]
fn canon_revision_surfaces_context_staleness() {
    let service = ApplicationService;
    let (project, _) = fresh_project("canonstale");
    import_manuscript(&project, 3);
    run_analysis(&project);
    let approved = approve_all_pending(&project);
    promote(&project, &approved);
    start_echo_translation(&project, None);

    // Canon unchanged → context fresh.
    let snap = service.snapshot(&project).unwrap();
    assert!(!snap.translation.as_ref().unwrap().context_stale);

    // Promote additional canon AFTER the run started → context stale.
    let profile = character_engine::CharacterProfile {
        name: "Newcomer".to_string(),
        voice_notes: "".to_string(),
        personality_notes: "".to_string(),
    };
    service.upsert_character(&project, profile, false).unwrap();
    let snap = service.snapshot(&project).unwrap();
    assert!(snap.translation.as_ref().unwrap().context_stale);
}

// ---------------------------------------------------------------------------
// Locking
// ---------------------------------------------------------------------------

#[test]
fn project_locking_blocks_second_writer() {
    let (project, _) = fresh_project("lock");
    import_manuscript(&project, 3);
    run_analysis(&project);

    let layout = project.layout.clone();
    let guard =
        project_engine::application::project::ProjectLock::acquire(&layout, "writer-a").unwrap();

    // Second writer must be blocked.
    let err = project_engine::application::project::ProjectLock::acquire(&layout, "writer-b");
    match err {
        Err(project_engine::application::ApplicationError::ProjectLocked) => {}
        Err(other) => panic!("expected ProjectLocked, got {other}"),
        Ok(_) => panic!("second writer unexpectedly acquired the lock"),
    }

    drop(guard);

    // After release, a new writer succeeds.
    let _guard2 =
        project_engine::application::project::ProjectLock::acquire(&layout, "writer-c").unwrap();
}

// ---------------------------------------------------------------------------
// Crash recovery: corrupt manifest and stale lock
// ---------------------------------------------------------------------------

#[test]
fn corrupt_manifest_is_detected_not_silently_accepted() {
    let (project, root) = fresh_project("corrupt");
    import_manuscript(&project, 3);

    // Simulate a torn/corrupt manifest write.
    std::fs::write(root.join("project.json"), b"{ not valid json !!").unwrap();

    let err = ApplicationService::open_project(&root);
    assert!(
        err.is_err(),
        "corrupt manifest must produce an error, not silent state"
    );

    // Opening must not panic and must produce a typed error payload.
    let payload = ApplicationService::error_payload(&err.unwrap_err());
    assert!(!payload.code.is_empty());
    assert_ne!(
        payload.recovery_hint,
        project_engine::application::RecoveryHint::None
    );
}

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

#[test]
fn events_are_emitted_through_the_application_boundary() {
    let service = ApplicationService;
    let (project, _) = fresh_project("events");
    let mut sink = silent_sink();
    let source = project.layout.root.join("events.md");
    write_manuscript(&source, 3, false);
    service.import_book(&project, &source, &mut sink).unwrap();
    service.analyze_book(&project, &mut sink).unwrap();

    let event_names: Vec<&str> = sink
        .events
        .iter()
        .map(|event: &ProjectEvent| match event {
            ProjectEvent::ImportCompleted { .. } => "import",
            ProjectEvent::AnalysisStarted { .. } => "analysis_started",
            ProjectEvent::AnalysisCompleted { .. } => "analysis_completed",
            _ => "other",
        })
        .collect();
    assert!(event_names.contains(&"import"));
    assert!(event_names.contains(&"analysis_started"));
    assert!(event_names.contains(&"analysis_completed"));
}

// ---------------------------------------------------------------------------
// Capabilities & provider configuration
// ---------------------------------------------------------------------------

#[test]
fn capabilities_and_provider_configuration() {
    let capabilities = ApplicationService::capabilities();
    assert!(capabilities.import_formats.contains(&"md".to_string()));
    assert!(capabilities.export_formats.contains(&"docx".to_string()));
    assert!(capabilities.advanced_analysis_available);
    assert!(capabilities.pause_supported);
    assert!(capabilities.manual_edit_supported);

    // echo/mock need no credentials; openai without key must fail cleanly.
    assert!(ApplicationService::test_provider_configuration("echo", "mock").is_ok());
    if std::env::var_os("OPENAI_API_KEY").is_none() {
        assert!(ApplicationService::test_provider_configuration("openai", "mock").is_err());
        assert!(ApplicationService::test_provider_configuration("echo", "openai").is_err());
    }
    assert!(ApplicationService::test_provider_configuration("bogus", "mock").is_err());
}

// ---------------------------------------------------------------------------
// Advanced analysis integration: cache reuse
// ---------------------------------------------------------------------------

#[test]
fn advanced_analysis_cache_reuses_completed_units() {
    let service = ApplicationService;
    let (project, _) = fresh_project("advcache");
    import_manuscript(&project, 3);
    run_advanced(&project);

    // Second run reuses the cache (mock provider is deterministic, so all
    // units are cached and zero new provider calls happen).
    let settings = AdvancedAnalysisSettings::default();
    let mut sink = silent_sink();
    service
        .run_advanced_analysis(&project, &settings, &mut sink)
        .unwrap();
    let snap = service.snapshot(&project).unwrap();
    let advanced = snap.advanced.as_ref().unwrap();
    assert!(
        advanced.detail.contains("cached"),
        "advanced detail should mention cache: {}",
        advanced.detail
    );
}

// ---------------------------------------------------------------------------
// JSON serialization of UI-facing models
// ---------------------------------------------------------------------------

#[test]
fn ui_facing_models_roundtrip_through_json() {
    let service = ApplicationService;
    let (project, _) = fresh_project("json");
    import_manuscript(&project, 3);
    run_analysis(&project);
    run_advanced(&project);

    let snap = service.snapshot(&project).unwrap();
    let json = serde_json::to_string(&snap).unwrap();
    let back: project_engine::application::models::ProjectSnapshot =
        serde_json::from_str(&json).unwrap();
    assert_eq!(back.project_id, snap.project_id);
    assert_eq!(back.status, snap.status);
    assert_eq!(back.review.total, snap.review.total);

    // Progress round-trips too.
    let progress = service.get_progress(&project);
    if let Ok(progress) = progress {
        let json = serde_json::to_string(&progress).unwrap();
        let back: project_engine::application::models::TranslationProgress =
            serde_json::from_str(&json).unwrap();
        assert_eq!(back.run_id, progress.run_id);
    }
}

// ---------------------------------------------------------------------------
// Unicode (Persian / Arabic script) through the whole pipeline
// ---------------------------------------------------------------------------

#[test]
fn unicode_survives_the_full_pipeline() {
    let service = ApplicationService;
    let (project, _) = fresh_project("unicode");
    let source = project.layout.root.join("unicode.md");
    write_manuscript(&source, 2, true);
    let mut sink = silent_sink();
    service.import_book(&project, &source, &mut sink).unwrap();
    run_analysis(&project);
    let approved = approve_all_pending(&project);
    promote(&project, &approved);
    start_echo_translation(&project, None);

    let chapter = service.get_translated_chapter(&project, 0).unwrap();
    assert!(chapter.paragraphs.iter().any(|p| p.source.contains("گل")));
}

// ---------------------------------------------------------------------------
// Large-book regression: 120 chapters, no provider calls, no panics
// ---------------------------------------------------------------------------

#[test]
fn large_book_regression_120_chapters() {
    let service = ApplicationService;
    let root = temp_root("large");
    let mut sink = silent_sink();
    let project = ApplicationService::create_project(&root, "Big Book", None, &mut sink).unwrap();

    let source = root.join("big.md");
    write_manuscript(&source, 120, false);
    service.import_book(&project, &source, &mut sink).unwrap();

    let snap = service.snapshot(&project).unwrap();
    assert_eq!(snap.source.as_ref().unwrap().chapters, 120);

    run_analysis(&project);
    let snap = service.snapshot(&project).unwrap();
    assert_eq!(snap.status, ProjectStatus::NeedsReview);
    assert!(snap.review.pending > 0);
    assert_eq!(snap.review.total, snap.review.pending);

    let approved = approve_all_pending(&project);
    promote(&project, &approved);
    let snap = service.snapshot(&project).unwrap();
    assert!(snap.canon.characters > 0);
    assert_eq!(snap.next_action, NextAction::StartTranslation);

    // Snapshot stays cheap: it never re-reads the manuscript, so repeated
    // snapshots remain consistent and fast.
    for _ in 0..5 {
        let snap = service.snapshot(&project).unwrap();
        assert_eq!(snap.source.as_ref().unwrap().paragraphs, 120 * 3);
        assert!(snap.translation.is_none());
    }

    // A 120-chapter translation representation works without any model.
    let mut sink = silent_sink();
    let config = TranslationConfig {
        provider: "echo".to_string(),
        target_language: "fa".to_string(),
        max_chapters: Some(1),
        ..TranslationConfig::default()
    };
    service
        .start_translation(&project, &config, &mut sink)
        .unwrap();
    let progress = service.get_progress(&project).unwrap();
    assert_eq!(progress.total_chapters, 120);
    assert_eq!(progress.completed_chapters, 1);
    assert!(progress.percent > 0.0 && progress.percent < 0.01);

    // Reopen and confirm stable ordering/counts.
    let reopened = ApplicationService::open_project(&root).unwrap();
    let snap = service.snapshot(&reopened).unwrap();
    assert_eq!(snap.source.as_ref().unwrap().chapters, 120);
    let progress = service.get_progress(&reopened).unwrap();
    assert_eq!(progress.total_chapters, 120);
}

// ---------------------------------------------------------------------------
// Next-action determinism
// ---------------------------------------------------------------------------

#[test]
fn next_action_is_deterministic_per_state() {
    let service = ApplicationService;
    let (project, _) = fresh_project("nextaction");
    import_manuscript(&project, 3);

    let snap = service.snapshot(&project).unwrap();
    assert_eq!(snap.next_action, NextAction::RunAnalysis);

    run_analysis(&project);
    let snap = service.snapshot(&project).unwrap();
    assert_eq!(snap.next_action, NextAction::ReviewIntelligence);

    let approved = approve_all_pending(&project);
    promote(&project, &approved);
    let snap = service.snapshot(&project).unwrap();
    assert_eq!(snap.next_action, NextAction::StartTranslation);
}
