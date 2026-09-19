use std::collections::{BTreeSet, HashSet};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use character_engine::CharacterBible;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const COREFERENCE_PROTOCOL_VERSION: u32 = 1;
const DEFAULT_MAX_SOURCE_CHARS: usize = 500_000;
const DEFAULT_MAX_CLUSTERS: usize = 4_096;
const DEFAULT_MAX_MENTIONS_PER_CLUSTER: usize = 2_048;
const MAX_CONTEXT_LINKS: usize = 24;
const MAX_CONTEXT_MENTION_CHARS: usize = 96;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoreferenceRequest {
    pub schema_version: u32,
    pub unit_id: String,
    pub text: String,
}

impl CoreferenceRequest {
    pub fn new(unit_id: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            schema_version: COREFERENCE_PROTOCOL_VERSION,
            unit_id: unit_id.into(),
            text: text.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoreferenceMention {
    pub id: String,
    pub start_char: usize,
    pub end_char: usize,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoreferenceCluster {
    pub id: String,
    pub mentions: Vec<CoreferenceMention>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoreferenceResponse {
    pub schema_version: u32,
    pub model: String,
    pub clusters: Vec<CoreferenceCluster>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalCoreferenceLink {
    pub cluster_id: String,
    pub canonical_character: String,
    pub anchor_mentions: Vec<String>,
    pub linked_mentions: Vec<CoreferenceMention>,
}

#[derive(Debug, Error)]
pub enum CoreferenceError {
    #[error("invalid coreference request: {0}")]
    InvalidRequest(String),
    #[error("coreference sidecar I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("coreference sidecar timed out after {timeout_ms} ms")]
    Timeout { timeout_ms: u64 },
    #[error("coreference sidecar failed with status {status:?}: {stderr}")]
    SidecarFailed { status: Option<i32>, stderr: String },
    #[error("coreference protocol error: {0}")]
    Protocol(String),
}

#[derive(Debug, Clone)]
pub struct CoreferenceSidecar {
    executable: PathBuf,
    timeout: Duration,
    max_source_chars: usize,
    max_clusters: usize,
    max_mentions_per_cluster: usize,
}

impl CoreferenceSidecar {
    pub fn new(executable: impl Into<PathBuf>) -> Self {
        Self {
            executable: executable.into(),
            timeout: Duration::from_secs(180),
            max_source_chars: DEFAULT_MAX_SOURCE_CHARS,
            max_clusters: DEFAULT_MAX_CLUSTERS,
            max_mentions_per_cluster: DEFAULT_MAX_MENTIONS_PER_CLUSTER,
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout.max(Duration::from_millis(1));
        self
    }

    pub fn with_max_source_chars(mut self, max_source_chars: usize) -> Self {
        self.max_source_chars = max_source_chars.max(1);
        self
    }

    pub fn with_limits(mut self, max_clusters: usize, max_mentions_per_cluster: usize) -> Self {
        self.max_clusters = max_clusters.max(1);
        self.max_mentions_per_cluster = max_mentions_per_cluster.max(1);
        self
    }

    pub fn executable(&self) -> &Path {
        &self.executable
    }

    pub fn resolve(
        &self,
        request: &CoreferenceRequest,
    ) -> Result<CoreferenceResponse, CoreferenceError> {
        validate_request(request, self.max_source_chars)?;
        let payload = serde_json::to_vec(request)
            .map_err(|error| CoreferenceError::Protocol(error.to_string()))?;

        let mut child = Command::new(&self.executable)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| CoreferenceError::Protocol("sidecar stdin was not available".into()))?;
        stdin.write_all(&payload)?;
        drop(stdin);

        let started = Instant::now();
        let status = loop {
            if let Some(status) = child.try_wait()? {
                break status;
            }
            if started.elapsed() >= self.timeout {
                let _ = child.kill();
                let _ = child.wait();
                return Err(CoreferenceError::Timeout {
                    timeout_ms: self.timeout.as_millis().min(u128::from(u64::MAX)) as u64,
                });
            }
            thread::sleep(Duration::from_millis(25));
        };

        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        if let Some(mut pipe) = child.stdout.take() {
            pipe.read_to_end(&mut stdout)?;
        }
        if let Some(mut pipe) = child.stderr.take() {
            pipe.read_to_end(&mut stderr)?;
        }
        if !status.success() {
            return Err(CoreferenceError::SidecarFailed {
                status: status.code(),
                stderr: String::from_utf8_lossy(&stderr).into_owned(),
            });
        }

        let response: CoreferenceResponse = serde_json::from_slice(&stdout)
            .map_err(|error| CoreferenceError::Protocol(format!("invalid JSON response: {error}")))?;
        validate_response(
            &request.text,
            &response,
            self.max_clusters,
            self.max_mentions_per_cluster,
        )?;
        Ok(response)
    }
}

pub fn canonical_coreference_links(
    source_text: &str,
    characters: &CharacterBible,
    response: &CoreferenceResponse,
) -> Result<Vec<CanonicalCoreferenceLink>, CoreferenceError> {
    validate_response(
        source_text,
        response,
        DEFAULT_MAX_CLUSTERS,
        DEFAULT_MAX_MENTIONS_PER_CLUSTER,
    )?;

    let mut links = Vec::new();
    for cluster in &response.clusters {
        let mut canonical_names = BTreeSet::new();
        let mut anchor_ids = Vec::new();

        for mention in &cluster.mentions {
            if let Some(owner) = characters.find_alias_owner(mention.text.trim()) {
                canonical_names.insert(owner.to_string());
                anchor_ids.push(mention.id.clone());
            }
        }

        if canonical_names.len() != 1 {
            continue;
        }
        let canonical_character = canonical_names
            .into_iter()
            .next()
            .expect("exactly one canonical character after len check");
        let anchor_set = anchor_ids.iter().map(String::as_str).collect::<HashSet<_>>();
        let linked_mentions = cluster
            .mentions
            .iter()
            .filter(|mention| !anchor_set.contains(mention.id.as_str()))
            .cloned()
            .collect::<Vec<_>>();
        if linked_mentions.is_empty() {
            continue;
        }

        links.push(CanonicalCoreferenceLink {
            cluster_id: cluster.id.clone(),
            canonical_character,
            anchor_mentions: anchor_ids,
            linked_mentions,
        });
    }
    Ok(links)
}

pub fn model_coreference_context(
    source_text: &str,
    characters: &CharacterBible,
    response: &CoreferenceResponse,
) -> Result<Option<String>, CoreferenceError> {
    let links = canonical_coreference_links(source_text, characters, response)?;
    if links.is_empty() {
        return Ok(None);
    }

    let mut lines = vec![format!(
        "COREFERENCE MAP — optional model evidence ({}) mapped only through explicit canonical anchors; never canon:",
        response.model.trim()
    )];
    let mut emitted = 0usize;
    for link in links {
        for mention in link.linked_mentions {
            if emitted >= MAX_CONTEXT_LINKS {
                break;
            }
            let text = truncate_chars(mention.text.trim(), MAX_CONTEXT_MENTION_CHARS);
            lines.push(format!(
                "- {:?} → {} (cluster={})",
                text, link.canonical_character, link.cluster_id
            ));
            emitted += 1;
        }
        if emitted >= MAX_CONTEXT_LINKS {
            break;
        }
    }
    if emitted == 0 {
        return Ok(None);
    }
    Ok(Some(lines.join("\n")))
}

fn validate_request(
    request: &CoreferenceRequest,
    max_source_chars: usize,
) -> Result<(), CoreferenceError> {
    if request.schema_version != COREFERENCE_PROTOCOL_VERSION {
        return Err(CoreferenceError::InvalidRequest(format!(
            "unsupported schema version {}; expected {}",
            request.schema_version, COREFERENCE_PROTOCOL_VERSION
        )));
    }
    if request.unit_id.trim().is_empty() {
        return Err(CoreferenceError::InvalidRequest(
            "unit_id must not be empty".into(),
        ));
    }
    if request.text.trim().is_empty() {
        return Err(CoreferenceError::InvalidRequest(
            "source text must not be empty".into(),
        ));
    }
    let source_chars = request.text.chars().count();
    if source_chars > max_source_chars {
        return Err(CoreferenceError::InvalidRequest(format!(
            "source has {source_chars} characters; limit is {max_source_chars}"
        )));
    }
    Ok(())
}

pub fn validate_response(
    source_text: &str,
    response: &CoreferenceResponse,
    max_clusters: usize,
    max_mentions_per_cluster: usize,
) -> Result<(), CoreferenceError> {
    if response.schema_version != COREFERENCE_PROTOCOL_VERSION {
        return Err(CoreferenceError::Protocol(format!(
            "response schema version {} does not match {}",
            response.schema_version, COREFERENCE_PROTOCOL_VERSION
        )));
    }
    if response.model.trim().is_empty() {
        return Err(CoreferenceError::Protocol(
            "response model must not be empty".into(),
        ));
    }
    if response.clusters.len() > max_clusters {
        return Err(CoreferenceError::Protocol(format!(
            "response returned {} clusters; limit is {max_clusters}",
            response.clusters.len()
        )));
    }

    let source_chars = source_text.chars().collect::<Vec<_>>();
    let mut cluster_ids = HashSet::new();
    let mut mention_ids = HashSet::new();
    let mut spans = HashSet::new();

    for cluster in &response.clusters {
        if cluster.id.trim().is_empty() || !cluster_ids.insert(cluster.id.as_str()) {
            return Err(CoreferenceError::Protocol(
                "cluster IDs must be non-empty and unique".into(),
            ));
        }
        if cluster.mentions.is_empty() || cluster.mentions.len() > max_mentions_per_cluster {
            return Err(CoreferenceError::Protocol(format!(
                "cluster '{}' must contain 1..={max_mentions_per_cluster} mentions",
                cluster.id
            )));
        }

        for mention in &cluster.mentions {
            if mention.id.trim().is_empty() || !mention_ids.insert(mention.id.as_str()) {
                return Err(CoreferenceError::Protocol(
                    "mention IDs must be non-empty and globally unique".into(),
                ));
            }
            if mention.start_char >= mention.end_char || mention.end_char > source_chars.len() {
                return Err(CoreferenceError::Protocol(format!(
                    "mention '{}' has invalid character offsets",
                    mention.id
                )));
            }
            if !spans.insert((mention.start_char, mention.end_char)) {
                return Err(CoreferenceError::Protocol(format!(
                    "mention '{}' reuses a span already assigned to another cluster",
                    mention.id
                )));
            }
            let actual = source_chars[mention.start_char..mention.end_char]
                .iter()
                .collect::<String>();
            if actual != mention.text {
                return Err(CoreferenceError::Protocol(format!(
                    "mention '{}' text does not match source offsets",
                    mention.id
                )));
            }
        }
    }
    Ok(())
}

fn truncate_chars(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        value.to_string()
    } else {
        value.chars().take(max_chars).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use character_engine::CharacterProfile;

    fn bible() -> CharacterBible {
        let mut bible = CharacterBible::new();
        bible.add(CharacterProfile {
            name: "Mina".into(),
            voice_notes: "quiet".into(),
            personality_notes: "guarded".into(),
        });
        bible.add(CharacterProfile {
            name: "Reza".into(),
            voice_notes: "warm".into(),
            personality_notes: "patient".into(),
        });
        bible.add_alias("Mina", "Min");
        bible
    }

    fn mention(id: &str, start: usize, end: usize, text: &str) -> CoreferenceMention {
        CoreferenceMention {
            id: id.into(),
            start_char: start,
            end_char: end,
            text: text.into(),
        }
    }

    #[test]
    fn canonical_anchor_maps_pronoun_without_creating_canon() {
        let text = "Mina closed the door. She sighed.";
        let response = CoreferenceResponse {
            schema_version: COREFERENCE_PROTOCOL_VERSION,
            model: "synthetic".into(),
            clusters: vec![CoreferenceCluster {
                id: "c1".into(),
                mentions: vec![mention("m1", 0, 4, "Mina"), mention("m2", 22, 25, "She")],
            }],
        };
        let links = canonical_coreference_links(text, &bible(), &response).unwrap();
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].canonical_character, "Mina");
        assert_eq!(links[0].linked_mentions[0].text, "She");
    }

    #[test]
    fn approved_alias_can_anchor_a_cluster() {
        let text = "Min waited. She looked away.";
        let response = CoreferenceResponse {
            schema_version: COREFERENCE_PROTOCOL_VERSION,
            model: "synthetic".into(),
            clusters: vec![CoreferenceCluster {
                id: "c1".into(),
                mentions: vec![mention("m1", 0, 3, "Min"), mention("m2", 12, 15, "She")],
            }],
        };
        let links = canonical_coreference_links(text, &bible(), &response).unwrap();
        assert_eq!(links[0].canonical_character, "Mina");
    }

    #[test]
    fn conflicting_canonical_anchors_fail_closed() {
        let text = "Mina met Reza. They left.";
        let response = CoreferenceResponse {
            schema_version: COREFERENCE_PROTOCOL_VERSION,
            model: "synthetic".into(),
            clusters: vec![CoreferenceCluster {
                id: "c1".into(),
                mentions: vec![
                    mention("m1", 0, 4, "Mina"),
                    mention("m2", 9, 13, "Reza"),
                    mention("m3", 15, 19, "They"),
                ],
            }],
        };
        assert!(canonical_coreference_links(text, &bible(), &response)
            .unwrap()
            .is_empty());
    }

    #[test]
    fn unanchored_cluster_is_omitted() {
        let text = "She waited. The doctor frowned.";
        let response = CoreferenceResponse {
            schema_version: COREFERENCE_PROTOCOL_VERSION,
            model: "synthetic".into(),
            clusters: vec![CoreferenceCluster {
                id: "c1".into(),
                mentions: vec![
                    mention("m1", 0, 3, "She"),
                    mention("m2", 12, 22, "The doctor"),
                ],
            }],
        };
        assert!(canonical_coreference_links(text, &bible(), &response)
            .unwrap()
            .is_empty());
    }

    #[test]
    fn mismatched_offsets_are_rejected() {
        let text = "Mina left.";
        let response = CoreferenceResponse {
            schema_version: COREFERENCE_PROTOCOL_VERSION,
            model: "synthetic".into(),
            clusters: vec![CoreferenceCluster {
                id: "c1".into(),
                mentions: vec![mention("m1", 0, 4, "Reza")],
            }],
        };
        assert!(matches!(
            validate_response(text, &response, 10, 10),
            Err(CoreferenceError::Protocol(_))
        ));
    }

    #[test]
    fn model_context_is_bounded_and_labeled_noncanonical() {
        let text = "Mina closed the door. She sighed.";
        let response = CoreferenceResponse {
            schema_version: COREFERENCE_PROTOCOL_VERSION,
            model: "synthetic".into(),
            clusters: vec![CoreferenceCluster {
                id: "c1".into(),
                mentions: vec![mention("m1", 0, 4, "Mina"), mention("m2", 22, 25, "She")],
            }],
        };
        let context = model_coreference_context(text, &bible(), &response)
            .unwrap()
            .unwrap();
        assert!(context.contains("never canon"));
        assert!(context.contains("\"She\" → Mina"));
    }

    #[cfg(unix)]
    #[test]
    fn sidecar_timeout_is_bounded() {
        use std::fs;
        use std::os::unix::fs::PermissionsExt;

        let script = std::env::temp_dir().join(format!(
            "literary-coreference-timeout-{}",
            std::process::id()
        ));
        fs::write(&script, "#!/bin/sh\nsleep 5\n").unwrap();
        let mut permissions = fs::metadata(&script).unwrap().permissions();
        permissions.set_mode(0o700);
        fs::set_permissions(&script, permissions).unwrap();

        let request = CoreferenceRequest::new("u1", "Mina left.");
        let sidecar = CoreferenceSidecar::new(&script).with_timeout(Duration::from_millis(50));
        let result = sidecar.resolve(&request);
        let _ = fs::remove_file(&script);
        assert!(matches!(result, Err(CoreferenceError::Timeout { .. })));
    }
}
