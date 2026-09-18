use crate::OutputFormat;
use literary_evaluation_engine::{
    evaluate_submission, CandidateSubmission, LiteraryEvaluationCorpus,
};
use std::fs;
use std::path::Path;

type Result<T> = std::result::Result<T, String>;

pub fn run_benchmark(args: &[String], format: &OutputFormat) -> Result<()> {
    let (corpus_path, submission_path) = match args {
        [corpus, submission, ..] => (Path::new(corpus), Path::new(submission)),
        _ => {
            return Err(
                "usage: literary-engine benchmark <corpus.json> <submission.json>".to_string(),
            )
        }
    };

    let corpus_text = fs::read_to_string(corpus_path)
        .map_err(|error| format!("failed to read {}: {error}", corpus_path.display()))?;
    let submission_text = fs::read_to_string(submission_path)
        .map_err(|error| format!("failed to read {}: {error}", submission_path.display()))?;
    let corpus = LiteraryEvaluationCorpus::from_json(&corpus_text)
        .map_err(|error| format!("invalid benchmark corpus: {error}"))?;
    let submission = CandidateSubmission::from_json(&submission_text)
        .map_err(|error| format!("invalid benchmark submission: {error}"))?;
    let report = evaluate_submission(&corpus, &submission)
        .map_err(|error| format!("benchmark evaluation failed: {error}"))?;

    match format {
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(&report)
                    .map_err(|error| format!("failed to serialize benchmark report: {error}"))?
            );
        }
        OutputFormat::Text => {
            println!("Corpus: {}", report.corpus_id);
            println!("System: {}", report.system_id);
            println!("Cases:  {}", report.cases);
            println!(
                "Deterministic anchors: {}/{} ({:.1}%)",
                report.anchor_passed,
                report.anchor_total,
                report.anchor_pass_rate * 100.0
            );
            for (dimension, coverage) in &report.by_dimension {
                println!(
                    "  {:?}: {}/{}",
                    dimension, coverage.passed, coverage.total
                );
            }
            for note in &report.advisory_notes {
                println!("note: {note}");
            }
        }
    }
    Ok(())
}
