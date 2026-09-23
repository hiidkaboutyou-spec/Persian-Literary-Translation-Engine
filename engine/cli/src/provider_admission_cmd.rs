use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

type Result<T> = std::result::Result<T, String>;
const ASSESSMENT_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Deserialize)]
struct ReviewDossier {
    schema_version: u32,
    system_one: String,
    system_two: String,
    automatic_winner: Option<String>,
    human_comparative_evidence_only: bool,
    production_admission: String,
    requires_explicit_admission_decision: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum EvidenceStatus {
    Acceptable,
    Unacceptable,
    Unknown,
}

#[derive(Debug, Deserialize)]
struct ProviderGovernanceProfile {
    schema_version: u32,
    provider_id: String,
    model_id: String,
    reviewed_at: String,
    terms_url: String,
    privacy_url: String,
    data_retention: EvidenceItem,
    training_use: EvidenceItem,
    data_residency: EvidenceItem,
    deletion_process: EvidenceItem,
    incident_response: EvidenceItem,
    reliability_failure_behavior: EvidenceItem,
    rate_limits: EvidenceItem,
    cost_limits: EvidenceItem,
    rights_and_confidentiality: EvidenceItem,
}

#[derive(Debug, Deserialize)]
struct EvidenceItem {
    status: EvidenceStatus,
    evidence: String,
}

#[derive(Debug, Serialize)]
struct AdmissionAssessment {
    schema_version: u32,
    provider_id: String,
    model_id: String,
    reviewed_at: String,
    dossier_fingerprint: String,
    profile_fingerprint: String,
    evidence_complete: bool,
    blocking_items: Vec<String>,
    eligible_for_owner_decision: bool,
    production_admission: &'static str,
    requires_explicit_owner_authorization: bool,
    selector_changed: bool,
    notes: Vec<&'static str>,
}

fn usage() -> &'static str {
    "Usage:\n  literary-engine provider-admission assess <phase32-dossier.json> <governance-profile.json> <assessment.json>\n\nThis command only assesses admission evidence. It never authorizes a provider and never changes the production selector."
}

pub(crate) fn run_provider_admission(args: &[String]) -> Result<()> {
    match args.first().map(String::as_str).unwrap_or_default() {
        "assess" => run_assess(&args[1..]),
        "--help" | "-h" | "help" => {
            println!("{}", usage());
            Ok(())
        }
        _ => Err(usage().to_string()),
    }
}

fn run_assess(args: &[String]) -> Result<()> {
    let [dossier_path, profile_path, output_path] = args else {
        return Err(usage().to_string());
    };
    if output_path == dossier_path || output_path == profile_path {
        return Err(
            "refusing destructive artifact collision: assessment output must differ from inputs"
                .into(),
        );
    }
    let dossier_bytes =
        fs::read(dossier_path).map_err(|e| format!("failed to read {dossier_path}: {e}"))?;
    let profile_bytes =
        fs::read(profile_path).map_err(|e| format!("failed to read {profile_path}: {e}"))?;
    let dossier: ReviewDossier = serde_json::from_slice(&dossier_bytes)
        .map_err(|e| format!("invalid Phase 32 dossier: {e}"))?;
    let profile: ProviderGovernanceProfile = serde_json::from_slice(&profile_bytes)
        .map_err(|e| format!("invalid governance profile: {e}"))?;
    validate_dossier(&dossier)?;
    validate_profile(&profile)?;
    if profile.provider_id != dossier.system_one && profile.provider_id != dossier.system_two {
        return Err(format!(
            "provider '{}' is absent from the Phase 32 dossier",
            profile.provider_id
        ));
    }

    let mut blocking_items = Vec::new();
    for (name, item) in evidence_items(&profile) {
        match item.status {
            EvidenceStatus::Acceptable => {}
            EvidenceStatus::Unknown => blocking_items.push(format!("{name}: unknown")),
            EvidenceStatus::Unacceptable => blocking_items.push(format!("{name}: unacceptable")),
        }
    }
    let evidence_complete = evidence_items(&profile)
        .iter()
        .all(|(_, item)| item.status != EvidenceStatus::Unknown);
    let eligible = blocking_items.is_empty();
    let assessment = AdmissionAssessment {
        schema_version: ASSESSMENT_SCHEMA_VERSION,
        provider_id: profile.provider_id,
        model_id: profile.model_id,
        reviewed_at: profile.reviewed_at,
        dossier_fingerprint: fingerprint(&dossier_bytes),
        profile_fingerprint: fingerprint(&profile_bytes),
        evidence_complete,
        blocking_items,
        eligible_for_owner_decision: eligible,
        production_admission: "not_granted",
        requires_explicit_owner_authorization: true,
        selector_changed: false,
        notes: vec![
            "Human comparative preference is evidence, not provider authorization.",
            "Unknown or unacceptable governance evidence blocks eligibility fail-closed.",
            "This artifact cannot change runtime provider selection.",
            "Private manuscripts must not be sent to a candidate provider before a separate explicit owner authorization and runtime integration stage.",
        ],
    };
    write_json_atomic(Path::new(output_path), &assessment)?;
    println!("provider admission assessment written: {output_path}");
    println!(
        "eligible for owner decision: {}",
        assessment.eligible_for_owner_decision
    );
    println!("production admission: not_granted");
    Ok(())
}

fn validate_dossier(d: &ReviewDossier) -> Result<()> {
    if d.schema_version != 1
        || !d.human_comparative_evidence_only
        || d.production_admission != "not_granted"
        || !d.requires_explicit_admission_decision
        || d.automatic_winner.is_some()
    {
        return Err("dossier does not preserve the Phase 32 non-admission contract".into());
    }
    Ok(())
}

fn validate_profile(p: &ProviderGovernanceProfile) -> Result<()> {
    if p.schema_version != 1 {
        return Err(format!(
            "unsupported governance profile schema {}",
            p.schema_version
        ));
    }
    for (name, value) in [
        ("provider_id", p.provider_id.as_str()),
        ("model_id", p.model_id.as_str()),
        ("reviewed_at", p.reviewed_at.as_str()),
        ("terms_url", p.terms_url.as_str()),
        ("privacy_url", p.privacy_url.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(format!("governance profile requires non-empty {name}"));
        }
    }
    for (name, item) in evidence_items(p) {
        if item.evidence.trim().is_empty() {
            return Err(format!("governance evidence for {name} must be non-empty, including when status is unknown"));
        }
    }
    Ok(())
}

fn evidence_items(p: &ProviderGovernanceProfile) -> Vec<(&'static str, &EvidenceItem)> {
    vec![
        ("data_retention", &p.data_retention),
        ("training_use", &p.training_use),
        ("data_residency", &p.data_residency),
        ("deletion_process", &p.deletion_process),
        ("incident_response", &p.incident_response),
        (
            "reliability_failure_behavior",
            &p.reliability_failure_behavior,
        ),
        ("rate_limits", &p.rate_limits),
        ("cost_limits", &p.cost_limits),
        ("rights_and_confidentiality", &p.rights_and_confidentiality),
    ]
}

fn write_json_atomic<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    if !parent.is_dir() {
        return Err(format!(
            "output directory does not exist: {}",
            parent.display()
        ));
    }
    if path.exists() {
        return Err(format!(
            "refusing to overwrite existing admission artifact: {}",
            path.display()
        ));
    }
    let json = serde_json::to_vec_pretty(value)
        .map_err(|e| format!("failed to serialize assessment: {e}"))?;
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("system clock error: {e}"))?
        .as_nanos();
    let name = path
        .file_name()
        .and_then(|v| v.to_str())
        .ok_or_else(|| "invalid output filename".to_string())?;
    let tmp = parent.join(format!(".{name}.{}.{}.tmp", std::process::id(), nonce));
    let result = (|| -> Result<()> {
        let mut f = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&tmp)
            .map_err(|e| format!("failed to create temp assessment: {e}"))?;
        f.write_all(&json)
            .and_then(|_| f.write_all(b"\n"))
            .and_then(|_| f.sync_all())
            .map_err(|e| format!("failed to persist assessment: {e}"))?;
        fs::rename(&tmp, path)
            .map_err(|e| format!("failed to atomically install assessment: {e}"))?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&tmp);
    }
    result
}

fn fingerprint(bytes: &[u8]) -> String {
    const OFFSET: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x100000001b3;
    let hash = bytes.iter().fold(OFFSET, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(PRIME)
    });
    format!("fnv1a64-{hash:016x}")
}

#[cfg(test)]
mod tests {
    use super::*;
    fn item(status: EvidenceStatus) -> EvidenceItem {
        EvidenceItem {
            status,
            evidence: "reviewed evidence".into(),
        }
    }
    fn profile(status: EvidenceStatus) -> ProviderGovernanceProfile {
        ProviderGovernanceProfile {
            schema_version: 1,
            provider_id: "atria".into(),
            model_id: "Atria-Dawn-Preview".into(),
            reviewed_at: "2026-09-23".into(),
            terms_url: "https://example.test/terms".into(),
            privacy_url: "https://example.test/privacy".into(),
            data_retention: item(status),
            training_use: item(status),
            data_residency: item(status),
            deletion_process: item(status),
            incident_response: item(status),
            reliability_failure_behavior: item(status),
            rate_limits: item(status),
            cost_limits: item(status),
            rights_and_confidentiality: item(status),
        }
    }
    #[test]
    fn unknown_evidence_is_not_complete() {
        let p = profile(EvidenceStatus::Unknown);
        assert!(!evidence_items(&p)
            .iter()
            .all(|(_, i)| i.status != EvidenceStatus::Unknown));
    }
    #[test]
    fn unacceptable_evidence_is_distinct_from_unknown() {
        let p = profile(EvidenceStatus::Unacceptable);
        assert!(evidence_items(&p)
            .iter()
            .all(|(_, i)| i.status == EvidenceStatus::Unacceptable));
    }
    #[test]
    fn complete_acceptable_profile_validates() {
        assert!(validate_profile(&profile(EvidenceStatus::Acceptable)).is_ok());
    }
}
