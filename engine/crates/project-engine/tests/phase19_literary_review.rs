//! Phase 19 application-level regression coverage.
//!
//! These tests exercise literary review only through `ApplicationService`, the
//! same boundary used by CLI/product surfaces. External evidence stays
//! optional and review artifacts must become stale after a manual edit.

use project_engine::application::{
    silent_sink, ApplicationService, EvidenceRunState, LiteraryReviewSettings, TranslationConfig,
};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_root(tag: &str) -> std::path::PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("p19-{tag}-{nonce}"))
}

fn write_manuscript(path: &Path) {
    std::fs::write(
        path,
        "# Chapter 1\n\nShirin entered the garden.\n\nFarhad did not answer her.\n\nThe nightingale sang softly.\n",
    )
    .unwrap();
}

fn translated_project(tag: &str) -> project_engine::application::Project {
    let root = temp_root(tag);
    let mut sink = silent_sink();
    let project = ApplicationService::create_project(&root, "Phase 19 Test", None, &mut sink)
        .expect("project should be created");
    let source = root.join("input.md");
    write_manuscript(&source);

    let service = ApplicationService;
    service
        .import_book(&project, &source, &mut sink)
        .expect("source should import");
    service
        .analyze_book(&project, &mut sink)
        .expect("deterministic analysis should succeed");
    service
        .start_translation(
            &project,
            &TranslationConfig {
                provider: "echo".into(),
                target_language: "fa".into(),
                max_chapters: None,
                ..TranslationConfig::default()
            },
            &mut sink,
        )
        .expect("offline echo translation should succeed");
    project
}

#[test]
fn literary_review_is_non_mutating_persisted_and_staleness_aware() {
    let service = ApplicationService;
    let project = translated_project("artifact");
    let review_before = service.snapshot(&project).unwrap().review;

    let summary = service
        .review_translation(
            &project,
            &LiteraryReviewSettings {
                provider: "none".into(),
                model: None,
                semantic_alignment: false,
                max_chapters: None,
            },
        )
        .expect("native review should succeed without external tools");

    assert_eq!(summary.reviewed_chapters, 1);
    assert_eq!(summary.alignment_failures, 0);
    assert_eq!(summary.provider_failures, 0);
    assert_eq!(summary.artifacts.len(), 1);

    let initial = service
        .get_literary_review(&project, 0)
        .expect("persisted review should be readable");
    assert!(!initial.stale);
    assert_eq!(
        initial.artifact.semantic_alignment.state,
        EvidenceRunState::NotRequested
    );
    assert_eq!(
        initial.artifact.provider_review.state,
        EvidenceRunState::NotRequested
    );

    // Running literary review must not mutate the intelligence review/canon
    // lifecycle. It only writes a translation-review artifact.
    assert_eq!(service.snapshot(&project).unwrap().review, review_before);

    let chapter = service.get_translated_chapter(&project, 0).unwrap();
    let first = chapter
        .paragraphs
        .first()
        .expect("translated chapter should contain a paragraph");
    let edited = format!("{} — ویرایش انسانی", first.translated);
    let mut sink = silent_sink();
    service
        .apply_manual_translation_edit(
            &project,
            0,
            &first.paragraph_id,
            &edited,
            Some("test-reviewer"),
            &mut sink,
        )
        .expect("manual edit should succeed");

    let stale = service
        .get_literary_review(&project, 0)
        .expect("old artifact should remain readable after an edit");
    assert!(stale.stale, "manual edits must invalidate old review evidence");
}

#[test]
fn mock_provider_evidence_is_recorded_without_becoming_human_approval() {
    let service = ApplicationService;
    let project = translated_project("mock-provider");
    let review_before = service.snapshot(&project).unwrap().review;

    let summary = service
        .review_translation(
            &project,
            &LiteraryReviewSettings {
                provider: "mock".into(),
                model: None,
                semantic_alignment: false,
                max_chapters: Some(1),
            },
        )
        .expect("mock provider review should succeed offline");

    assert_eq!(summary.reviewed_chapters, 1);
    assert_eq!(summary.provider_failures, 0);
    let artifact = service.get_literary_review(&project, 0).unwrap();
    assert_eq!(artifact.artifact.provider_review.state, EvidenceRunState::Completed);
    assert!(!artifact.stale);
    assert_eq!(service.snapshot(&project).unwrap().review, review_before);
}
