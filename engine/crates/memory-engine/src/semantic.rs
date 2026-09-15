use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

pub const SEMANTIC_PROTOCOL_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SemanticMode {
    Dense,
    Rerank,
    #[default]
    DenseThenRerank,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticCandidate {
    pub id: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SemanticRerankRequest {
    pub schema_version: u32,
    pub query: String,
    pub candidates: Vec<SemanticCandidate>,
    pub max_results: usize,
    pub mode: SemanticMode,
}

impl SemanticRerankRequest {
    pub fn new(
        query: impl Into<String>,
        candidates: Vec<SemanticCandidate>,
        max_results: usize,
    ) -> Self {
        Self {
            schema_version: SEMANTIC_PROTOCOL_VERSION,
            query: query.into(),
            candidates,
            max_results,
            mode: SemanticMode::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SemanticScore {
    pub id: String,
    pub score: f32,
    pub rank: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SemanticRerankResponse {
    pub schema_version: u32,
    pub model: String,
    pub mode: SemanticMode,
    pub scores: Vec<SemanticScore>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ReciprocalRankFusionConfig {
    pub rank_constant: f32,
    pub deterministic_weight: f32,
    pub semantic_weight: f32,
}

impl Default for ReciprocalRankFusionConfig {
    fn default() -> Self {
        Self {
            rank_constant: 60.0,
            deterministic_weight: 1.0,
            semantic_weight: 1.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FusedRank {
    pub id: String,
    pub score: f32,
    pub deterministic_rank: Option<usize>,
    pub semantic_rank: Option<usize>,
}

/// Fuse deterministic/native ranking with optional semantic ranking using RRF.
/// Raw model scores are intentionally not mixed with deterministic scores because
/// their calibration changes across embedding/reranker model versions.
pub fn reciprocal_rank_fusion(
    deterministic_ids: &[String],
    semantic_scores: &[SemanticScore],
    config: ReciprocalRankFusionConfig,
) -> Vec<FusedRank> {
    let rank_constant = if config.rank_constant.is_finite() {
        config.rank_constant.max(1.0)
    } else {
        60.0
    };
    let deterministic_weight = finite_nonnegative(config.deterministic_weight);
    let semantic_weight = finite_nonnegative(config.semantic_weight);

    let mut state: HashMap<String, FusedRank> = HashMap::new();
    for (zero_rank, id) in deterministic_ids.iter().enumerate() {
        if id.trim().is_empty() {
            continue;
        }
        let rank = zero_rank + 1;
        let entry = state.entry(id.clone()).or_insert(FusedRank {
            id: id.clone(),
            score: 0.0,
            deterministic_rank: None,
            semantic_rank: None,
        });
        if entry.deterministic_rank.is_none() {
            entry.deterministic_rank = Some(rank);
            entry.score += deterministic_weight / (rank_constant + rank as f32);
        }
    }

    let mut semantic = semantic_scores
        .iter()
        .filter(|item| !item.id.trim().is_empty() && item.score.is_finite())
        .collect::<Vec<_>>();
    semantic.sort_by(|a, b| {
        a.rank
            .cmp(&b.rank)
            .then_with(|| b.score.total_cmp(&a.score))
            .then_with(|| a.id.cmp(&b.id))
    });

    let mut seen_semantic = HashSet::new();
    for (position, item) in semantic.into_iter().enumerate() {
        if !seen_semantic.insert(item.id.clone()) {
            continue;
        }
        let rank = if item.rank == 0 {
            position + 1
        } else {
            item.rank
        };
        let entry = state.entry(item.id.clone()).or_insert(FusedRank {
            id: item.id.clone(),
            score: 0.0,
            deterministic_rank: None,
            semantic_rank: None,
        });
        entry.semantic_rank = Some(rank);
        entry.score += semantic_weight / (rank_constant + rank as f32);
    }

    let mut fused = state.into_values().collect::<Vec<_>>();
    fused.sort_by(|a, b| {
        b.score
            .total_cmp(&a.score)
            .then_with(|| compare_optional_rank(a.deterministic_rank, b.deterministic_rank))
            .then_with(|| compare_optional_rank(a.semantic_rank, b.semantic_rank))
            .then_with(|| a.id.cmp(&b.id))
    });
    fused
}

fn compare_optional_rank(a: Option<usize>, b: Option<usize>) -> Ordering {
    match (a, b) {
        (Some(a), Some(b)) => a.cmp(&b),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

fn finite_nonnegative(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

#[derive(Debug)]
pub enum SemanticError {
    InvalidRequest(String),
    Io(std::io::Error),
    Timeout { timeout_ms: u64 },
    SidecarFailed { status: Option<i32>, stderr: String },
    Protocol(String),
}

impl fmt::Display for SemanticError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRequest(message) => {
                write!(formatter, "invalid semantic request: {message}")
            }
            Self::Io(error) => write!(formatter, "semantic sidecar I/O error: {error}"),
            Self::Timeout { timeout_ms } => write!(
                formatter,
                "semantic sidecar timed out after {timeout_ms} ms"
            ),
            Self::SidecarFailed { status, stderr } => write!(
                formatter,
                "semantic sidecar failed with status {:?}: {}",
                status,
                stderr.trim()
            ),
            Self::Protocol(message) => {
                write!(formatter, "semantic sidecar protocol error: {message}")
            }
        }
    }
}

impl std::error::Error for SemanticError {}

impl From<std::io::Error> for SemanticError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

#[derive(Debug, Clone)]
pub struct SemanticSidecar {
    executable: PathBuf,
    max_candidates: usize,
    timeout: Duration,
}

impl SemanticSidecar {
    pub fn new(executable: impl Into<PathBuf>) -> Self {
        Self {
            executable: executable.into(),
            max_candidates: 512,
            timeout: Duration::from_secs(180),
        }
    }

    pub fn with_max_candidates(mut self, max_candidates: usize) -> Self {
        self.max_candidates = max_candidates.max(1);
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout.max(Duration::from_millis(1));
        self
    }

    pub fn executable(&self) -> &Path {
        &self.executable
    }

    pub fn rerank(
        &self,
        request: &SemanticRerankRequest,
    ) -> Result<SemanticRerankResponse, SemanticError> {
        validate_request(request, self.max_candidates)?;
        let payload = serde_json::to_vec(request)
            .map_err(|error| SemanticError::Protocol(error.to_string()))?;

        let mut child = Command::new(&self.executable)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| SemanticError::Protocol("sidecar stdin was not available".into()))?;
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
                return Err(SemanticError::Timeout {
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
            return Err(SemanticError::SidecarFailed {
                status: status.code(),
                stderr: String::from_utf8_lossy(&stderr).into_owned(),
            });
        }

        let response: SemanticRerankResponse = serde_json::from_slice(&stdout)
            .map_err(|error| SemanticError::Protocol(format!("invalid JSON response: {error}")))?;
        validate_response(request, &response)?;
        Ok(response)
    }
}

fn validate_request(
    request: &SemanticRerankRequest,
    max_candidates: usize,
) -> Result<(), SemanticError> {
    if request.schema_version != SEMANTIC_PROTOCOL_VERSION {
        return Err(SemanticError::InvalidRequest(format!(
            "unsupported schema version {}",
            request.schema_version
        )));
    }
    if request.query.trim().is_empty() {
        return Err(SemanticError::InvalidRequest("query is empty".into()));
    }
    if request.candidates.is_empty() {
        return Err(SemanticError::InvalidRequest(
            "candidate list is empty".into(),
        ));
    }
    if request.candidates.len() > max_candidates {
        return Err(SemanticError::InvalidRequest(format!(
            "{} candidates exceeds configured limit {max_candidates}",
            request.candidates.len()
        )));
    }
    if request.max_results == 0 || request.max_results > request.candidates.len() {
        return Err(SemanticError::InvalidRequest(
            "max_results must be between 1 and candidate count".into(),
        ));
    }

    let mut ids = HashSet::new();
    for candidate in &request.candidates {
        if candidate.id.trim().is_empty() || candidate.text.trim().is_empty() {
            return Err(SemanticError::InvalidRequest(
                "candidate IDs and texts must be non-empty".into(),
            ));
        }
        if !ids.insert(candidate.id.as_str()) {
            return Err(SemanticError::InvalidRequest(format!(
                "duplicate candidate id {}",
                candidate.id
            )));
        }
    }
    Ok(())
}

fn validate_response(
    request: &SemanticRerankRequest,
    response: &SemanticRerankResponse,
) -> Result<(), SemanticError> {
    if response.schema_version != SEMANTIC_PROTOCOL_VERSION {
        return Err(SemanticError::Protocol(format!(
            "response schema version {} does not match {}",
            response.schema_version, SEMANTIC_PROTOCOL_VERSION
        )));
    }
    if response.mode != request.mode {
        return Err(SemanticError::Protocol(
            "response mode does not match request".into(),
        ));
    }
    if response.model.trim().is_empty() {
        return Err(SemanticError::Protocol("response model is empty".into()));
    }
    if response.scores.len() > request.max_results {
        return Err(SemanticError::Protocol(
            "response returned more scores than requested".into(),
        ));
    }

    let requested = request
        .candidates
        .iter()
        .map(|candidate| candidate.id.as_str())
        .collect::<HashSet<_>>();
    let mut returned = HashSet::new();
    for item in &response.scores {
        if !requested.contains(item.id.as_str()) {
            return Err(SemanticError::Protocol(format!(
                "response returned unknown id {}",
                item.id
            )));
        }
        if !returned.insert(item.id.as_str()) {
            return Err(SemanticError::Protocol(format!(
                "response returned duplicate id {}",
                item.id
            )));
        }
        if !item.score.is_finite() || item.rank == 0 {
            return Err(SemanticError::Protocol(format!(
                "invalid score/rank for {}",
                item.id
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    #[test]
    fn sidecar_timeout_is_bounded() {
        use std::fs;
        use std::os::unix::fs::PermissionsExt;

        let script = std::env::temp_dir().join(format!(
            "literary-engine-semantic-timeout-{}-{}",
            std::process::id(),
            std::thread::current().name().unwrap_or("test")
        ));
        fs::write(&script, "#!/bin/sh\nsleep 5\n").unwrap();
        let mut permissions = fs::metadata(&script).unwrap().permissions();
        permissions.set_mode(0o700);
        fs::set_permissions(&script, permissions).unwrap();

        let request = SemanticRerankRequest::new(
            "query",
            vec![SemanticCandidate {
                id: "known".into(),
                text: "candidate".into(),
            }],
            1,
        );
        let sidecar = SemanticSidecar::new(&script).with_timeout(Duration::from_millis(50));
        let result = sidecar.rerank(&request);
        let _ = fs::remove_file(&script);
        assert!(matches!(result, Err(SemanticError::Timeout { .. })));
    }

    #[test]
    fn request_protocol_round_trips() {
        let request = SemanticRerankRequest::new(
            "He avoided her gaze again",
            vec![SemanticCandidate {
                id: "m1".into(),
                text: "He could not look her in the eye".into(),
            }],
            1,
        );
        let encoded = serde_json::to_vec(&request).unwrap();
        let decoded: SemanticRerankRequest = serde_json::from_slice(&encoded).unwrap();
        assert_eq!(decoded, request);
    }

    #[test]
    fn semantic_response_cannot_invent_candidate_ids() {
        let request = SemanticRerankRequest::new(
            "query",
            vec![SemanticCandidate {
                id: "known".into(),
                text: "candidate".into(),
            }],
            1,
        );
        let response = SemanticRerankResponse {
            schema_version: SEMANTIC_PROTOCOL_VERSION,
            model: "test".into(),
            mode: SemanticMode::DenseThenRerank,
            scores: vec![SemanticScore {
                id: "invented".into(),
                score: 0.9,
                rank: 1,
            }],
        };
        assert!(matches!(
            validate_response(&request, &response),
            Err(SemanticError::Protocol(_))
        ));
    }

    #[test]
    fn fusion_preserves_native_order_when_semantic_is_absent() {
        let ids = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let fused = reciprocal_rank_fusion(&ids, &[], ReciprocalRankFusionConfig::default());
        assert_eq!(
            fused
                .iter()
                .map(|item| item.id.as_str())
                .collect::<Vec<_>>(),
            vec!["a", "b", "c"]
        );
    }

    #[test]
    fn fusion_rewards_agreement_and_is_rank_based() {
        let ids = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let semantic = vec![
            SemanticScore {
                id: "b".into(),
                score: 0.99,
                rank: 1,
            },
            SemanticScore {
                id: "a".into(),
                score: 0.51,
                rank: 2,
            },
            SemanticScore {
                id: "c".into(),
                score: 0.10,
                rank: 3,
            },
        ];
        let fused = reciprocal_rank_fusion(&ids, &semantic, ReciprocalRankFusionConfig::default());
        assert_eq!(fused[0].id, "a");
        assert_eq!(fused[1].id, "b");
    }

    #[test]
    fn request_rejects_duplicates_and_unbounded_input() {
        let duplicate = SemanticRerankRequest::new(
            "query",
            vec![
                SemanticCandidate {
                    id: "same".into(),
                    text: "one".into(),
                },
                SemanticCandidate {
                    id: "same".into(),
                    text: "two".into(),
                },
            ],
            1,
        );
        assert!(validate_request(&duplicate, 10).is_err());

        let bounded = SemanticRerankRequest::new(
            "query",
            vec![
                SemanticCandidate {
                    id: "a".into(),
                    text: "one".into(),
                },
                SemanticCandidate {
                    id: "b".into(),
                    text: "two".into(),
                },
            ],
            1,
        );
        assert!(validate_request(&bounded, 1).is_err());
    }
}
