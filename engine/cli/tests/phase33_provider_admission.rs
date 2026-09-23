#[path = "../src/provider_admission_cmd.rs"]
mod provider_admission_cmd;

use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_dir() -> std::path::PathBuf {
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let path = std::env::temp_dir().join(format!("literary-engine-phase33-{}-{nonce}", std::process::id()));
    fs::create_dir_all(&path).unwrap();
    path
}

#[test]
fn unknown_governance_evidence_blocks_admission_without_touching_selector() {
    let dir = temp_dir();
    let dossier = dir.join("dossier.json");
    let profile = dir.join("profile.json");
    let assessment = dir.join("assessment.json");
    fs::write(&dossier, r#"{
      "schema_version":1,"system_one":"atria","system_two":"openai","automatic_winner":null,
      "human_comparative_evidence_only":true,"production_admission":"not_granted",
      "requires_explicit_admission_decision":true
    }"#).unwrap();
    let unknown = r#"{"status":"unknown","evidence":"No authoritative public evidence verified in this review."}"#;
    fs::write(&profile, format!(r#"{{
      "schema_version":1,"provider_id":"atria","model_id":"Atria-Dawn-Preview","reviewed_at":"2026-09-23",
      "terms_url":"https://api.atria-asi.ai/","privacy_url":"https://api.atria-asi.ai/",
      "data_retention":{0},"training_use":{0},"data_residency":{0},"deletion_process":{0},
      "incident_response":{0},"reliability_failure_behavior":{0},"rate_limits":{{"status":"acceptable","evidence":"Official API documentation publishes 429 and Retry-After behavior."}},
      "cost_limits":{0},"rights_and_confidentiality":{0}
    }}"#, unknown)).unwrap();
    let args = vec!["assess".into(), dossier.to_string_lossy().into_owned(), profile.to_string_lossy().into_owned(), assessment.to_string_lossy().into_owned()];
    provider_admission_cmd::run_provider_admission(&args).unwrap();
    let output: serde_json::Value = serde_json::from_str(&fs::read_to_string(&assessment).unwrap()).unwrap();
    assert_eq!(output["eligible_for_owner_decision"], false);
    assert_eq!(output["production_admission"], "not_granted");
    assert_eq!(output["requires_explicit_owner_authorization"], true);
    assert_eq!(output["selector_changed"], false);
    assert!(output["blocking_items"].as_array().unwrap().len() >= 8);
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn provider_not_present_in_blind_dossier_is_rejected() {
    let dir = temp_dir();
    let dossier = dir.join("dossier.json");
    let profile = dir.join("profile.json");
    let assessment = dir.join("assessment.json");
    fs::write(&dossier, r#"{"schema_version":1,"system_one":"openai","system_two":"echo","automatic_winner":null,"human_comparative_evidence_only":true,"production_admission":"not_granted","requires_explicit_admission_decision":true}"#).unwrap();
    let ok = r#"{"status":"acceptable","evidence":"verified"}"#;
    fs::write(&profile, format!(r#"{{"schema_version":1,"provider_id":"atria","model_id":"Atria-Dawn-Preview","reviewed_at":"2026-09-23","terms_url":"https://example.test/terms","privacy_url":"https://example.test/privacy","data_retention":{0},"training_use":{0},"data_residency":{0},"deletion_process":{0},"incident_response":{0},"reliability_failure_behavior":{0},"rate_limits":{0},"cost_limits":{0},"rights_and_confidentiality":{0}}}"#, ok)).unwrap();
    let args = vec!["assess".into(), dossier.to_string_lossy().into_owned(), profile.to_string_lossy().into_owned(), assessment.to_string_lossy().into_owned()];
    let error = provider_admission_cmd::run_provider_admission(&args).unwrap_err();
    assert!(error.contains("absent from the Phase 32 dossier"));
    let _ = fs::remove_dir_all(dir);
}
