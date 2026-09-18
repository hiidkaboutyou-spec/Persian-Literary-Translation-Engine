use literary_evaluation_engine::{
    contrastive_sanity_check, evaluate_submission, CandidateSubmission, LiteraryEvaluationCorpus,
};
use std::fs;
use std::path::PathBuf;

fn repo_file(relative: &str) -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../..");
    fs::read_to_string(root.join(relative)).expect("fixture should be readable")
}

#[test]
fn committed_project_owned_corpus_is_valid_and_contrastive() {
    let corpus =
        LiteraryEvaluationCorpus::from_json(&repo_file("benchmarks/phase21/corpus-v1.json"))
            .expect("corpus should be valid");
    assert!(corpus.provenance.rights_safe);
    assert_eq!(corpus.cases.len(), 8);
    contrastive_sanity_check(&corpus).expect("every declared degradation should trigger");
}

#[test]
fn reference_submission_passes_all_deterministic_anchors() {
    let corpus =
        LiteraryEvaluationCorpus::from_json(&repo_file("benchmarks/phase21/corpus-v1.json"))
            .unwrap();
    let submission =
        CandidateSubmission::from_json(&repo_file("benchmarks/phase21/reference-submission.json"))
            .unwrap();
    let report = evaluate_submission(&corpus, &submission).unwrap();
    assert_eq!(report.anchor_passed, report.anchor_total);
    assert_eq!(report.anchor_pass_rate, 1.0);
}

#[test]
fn contrastively_degraded_submission_scores_below_reference() {
    let corpus =
        LiteraryEvaluationCorpus::from_json(&repo_file("benchmarks/phase21/corpus-v1.json"))
            .unwrap();
    let good =
        CandidateSubmission::from_json(&repo_file("benchmarks/phase21/reference-submission.json"))
            .unwrap();
    let bad =
        CandidateSubmission::from_json(&repo_file("benchmarks/phase21/degraded-submission.json"))
            .unwrap();
    let good_report = evaluate_submission(&corpus, &good).unwrap();
    let bad_report = evaluate_submission(&corpus, &bad).unwrap();
    assert!(good_report.anchor_pass_rate > bad_report.anchor_pass_rate);
    assert!(bad_report.anchor_passed < bad_report.anchor_total);
}
