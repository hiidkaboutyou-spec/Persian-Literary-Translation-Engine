use std::fmt;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::{AlignmentConfig, AlignmentKind, AlignmentResult, ALIGNMENT_SCHEMA_VERSION};

const DEFAULT_MAX_TOTAL_CHARS: usize = 500_000;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AlignmentToolRequest {
    pub schema_version: u32,
    pub unit_id: String,
    pub source_segments: Vec<String>,
    pub target_segments: Vec<String>,
    #[serde(default)]
    pub config: AlignmentConfig,
}

impl AlignmentToolRequest {
    pub fn new(
        unit_id: impl Into<String>,
        source_segments: Vec<String>,
        target_segments: Vec<String>,
        config: AlignmentConfig,
    ) -> Result<Self, AlignmentSidecarError> {
        let request = Self {
            schema_version: ALIGNMENT_SCHEMA_VERSION,
            unit_id: unit_id.into(),
            source_segments,
            target_segments,
            config,
        };
        request.validate(DEFAULT_MAX_TOTAL_CHARS)?;
        Ok(request)
    }

    pub fn validate(&self, max_total_chars: usize) -> Result<(), AlignmentSidecarError> {
        if self.schema_version != ALIGNMENT_SCHEMA_VERSION {
            return Err(AlignmentSidecarError::InvalidRequest(format!(
                "unsupported schema version {}; expected {}",
                self.schema_version, ALIGNMENT_SCHEMA_VERSION
            )));
        }
        if self.unit_id.trim().is_empty() {
            return Err(AlignmentSidecarError::InvalidRequest(
                "unit_id must not be empty".into(),
            ));
        }
        if self.config.max_block_size == 0 || self.config.max_block_size > 8 {
            return Err(AlignmentSidecarError::InvalidRequest(
                "max_block_size must be between 1 and 8".into(),
            ));
        }
        if self.source_segments.len() > self.config.max_segments
            || self.target_segments.len() > self.config.max_segments
        {
            return Err(AlignmentSidecarError::InvalidRequest(format!(
                "segment count exceeds configured limit {}",
                self.config.max_segments
            )));
        }
        if self
            .source_segments
            .iter()
            .chain(&self.target_segments)
            .any(|segment| segment.trim().is_empty())
        {
            return Err(AlignmentSidecarError::InvalidRequest(
                "source/target segments must not be empty".into(),
            ));
        }
        let total_chars = self
            .source_segments
            .iter()
            .chain(&self.target_segments)
            .map(|segment| segment.chars().count())
            .sum::<usize>();
        if total_chars > max_total_chars {
            return Err(AlignmentSidecarError::InvalidRequest(format!(
                "alignment request exceeds the {max_total_chars}-character safety limit"
            )));
        }
        for (name, value) in [
            ("gap_penalty", self.config.gap_penalty),
            ("block_penalty", self.config.block_penalty),
        ] {
            if !value.is_finite() || value < 0.0 {
                return Err(AlignmentSidecarError::InvalidRequest(format!(
                    "{name} must be finite and non-negative"
                )));
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum AlignmentSidecarError {
    InvalidRequest(String),
    Io(std::io::Error),
    Timeout { timeout_ms: u64 },
    ToolFailed { status: Option<i32>, stderr: String },
    Protocol(String),
}

impl fmt::Display for AlignmentSidecarError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRequest(message) => {
                write!(formatter, "invalid alignment request: {message}")
            }
            Self::Io(error) => write!(formatter, "alignment tool I/O error: {error}"),
            Self::Timeout { timeout_ms } => {
                write!(formatter, "alignment tool timed out after {timeout_ms} ms")
            }
            Self::ToolFailed { status, stderr } => write!(
                formatter,
                "alignment tool failed with status {:?}: {}",
                status,
                stderr.trim()
            ),
            Self::Protocol(message) => write!(formatter, "alignment protocol error: {message}"),
        }
    }
}

impl std::error::Error for AlignmentSidecarError {}

impl From<std::io::Error> for AlignmentSidecarError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

#[derive(Debug, Clone)]
pub struct AlignmentSidecar {
    executable: PathBuf,
    timeout: Duration,
    max_total_chars: usize,
}

impl AlignmentSidecar {
    pub fn new(executable: impl Into<PathBuf>) -> Self {
        Self {
            executable: executable.into(),
            timeout: Duration::from_secs(90),
            max_total_chars: DEFAULT_MAX_TOTAL_CHARS,
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout.max(Duration::from_millis(1));
        self
    }

    pub fn with_max_total_chars(mut self, max_total_chars: usize) -> Self {
        self.max_total_chars = max_total_chars.max(1);
        self
    }

    pub fn executable(&self) -> &Path {
        &self.executable
    }

    pub fn align(
        &self,
        request: &AlignmentToolRequest,
    ) -> Result<AlignmentResult, AlignmentSidecarError> {
        request.validate(self.max_total_chars)?;
        let payload = serde_json::to_vec(request)
            .map_err(|error| AlignmentSidecarError::Protocol(error.to_string()))?;

        let mut child = Command::new(&self.executable)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| AlignmentSidecarError::Protocol("tool stdin unavailable".into()))?;
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
                return Err(AlignmentSidecarError::Timeout {
                    timeout_ms: self.timeout.as_millis().min(u128::from(u64::MAX)) as u64,
                });
            }
            thread::sleep(Duration::from_millis(20));
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
            return Err(AlignmentSidecarError::ToolFailed {
                status: status.code(),
                stderr: String::from_utf8_lossy(&stderr).into_owned(),
            });
        }

        let result: AlignmentResult = serde_json::from_slice(&stdout).map_err(|error| {
            AlignmentSidecarError::Protocol(format!("invalid JSON response: {error}"))
        })?;
        validate_result(request, &result)?;
        Ok(result)
    }
}

pub fn validate_result(
    request: &AlignmentToolRequest,
    result: &AlignmentResult,
) -> Result<(), AlignmentSidecarError> {
    if result.schema_version != ALIGNMENT_SCHEMA_VERSION {
        return Err(AlignmentSidecarError::Protocol(format!(
            "response schema version {} does not match {}",
            result.schema_version, ALIGNMENT_SCHEMA_VERSION
        )));
    }
    if result.unit_id != request.unit_id {
        return Err(AlignmentSidecarError::Protocol(
            "response unit_id does not match request".into(),
        ));
    }
    if result.source_count != request.source_segments.len()
        || result.target_count != request.target_segments.len()
    {
        return Err(AlignmentSidecarError::Protocol(
            "response segment counts do not match request".into(),
        ));
    }
    if result.embedding_model.trim().is_empty() {
        return Err(AlignmentSidecarError::Protocol(
            "response embedding-model provenance is missing".into(),
        ));
    }
    if !result.total_cost.is_finite() || result.total_cost < 0.0 {
        return Err(AlignmentSidecarError::Protocol(
            "response total_cost must be finite and non-negative".into(),
        ));
    }
    if result
        .mean_matched_similarity
        .is_some_and(|value| !value.is_finite() || !(-1.0..=1.0).contains(&value))
    {
        return Err(AlignmentSidecarError::Protocol(
            "mean_matched_similarity must be within [-1, 1]".into(),
        ));
    }

    let (mut expected_source, mut expected_target) = (0usize, 0usize);
    for block in &result.blocks {
        let expected_source_indices =
            (expected_source..expected_source + block.source_indices.len()).collect::<Vec<_>>();
        let expected_target_indices =
            (expected_target..expected_target + block.target_indices.len()).collect::<Vec<_>>();
        if block.source_indices != expected_source_indices
            || block.target_indices != expected_target_indices
        {
            return Err(AlignmentSidecarError::Protocol(
                "alignment blocks are not contiguous, monotonic, and gap-free as a path".into(),
            ));
        }
        match block.kind {
            AlignmentKind::Matched => {
                if block.source_indices.is_empty() || block.target_indices.is_empty() {
                    return Err(AlignmentSidecarError::Protocol(
                        "matched block must consume both source and target segments".into(),
                    ));
                }
                let similarity = block.similarity.ok_or_else(|| {
                    AlignmentSidecarError::Protocol(
                        "matched block is missing semantic similarity".into(),
                    )
                })?;
                if !similarity.is_finite() || !(-1.0..=1.0).contains(&similarity) {
                    return Err(AlignmentSidecarError::Protocol(
                        "matched block similarity must be within [-1, 1]".into(),
                    ));
                }
            }
            AlignmentKind::SourceOnly => {
                if block.source_indices.is_empty()
                    || !block.target_indices.is_empty()
                    || block.similarity.is_some()
                {
                    return Err(AlignmentSidecarError::Protocol(
                        "source-only block has invalid target/similarity data".into(),
                    ));
                }
            }
            AlignmentKind::TargetOnly => {
                if block.target_indices.is_empty()
                    || !block.source_indices.is_empty()
                    || block.similarity.is_some()
                {
                    return Err(AlignmentSidecarError::Protocol(
                        "target-only block has invalid source/similarity data".into(),
                    ));
                }
            }
        }
        expected_source += block.source_indices.len();
        expected_target += block.target_indices.len();
        if expected_source > request.source_segments.len()
            || expected_target > request.target_segments.len()
        {
            return Err(AlignmentSidecarError::Protocol(
                "alignment response contains out-of-range segment indices".into(),
            ));
        }
    }
    if expected_source != request.source_segments.len()
        || expected_target != request.target_segments.len()
    {
        return Err(AlignmentSidecarError::Protocol(
            "alignment response does not cover every requested segment exactly once".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AlignmentBlock;

    fn request() -> AlignmentToolRequest {
        AlignmentToolRequest::new(
            "chapter-1",
            vec!["source one".into(), "source two".into()],
            vec!["هدف یک".into()],
            AlignmentConfig::default(),
        )
        .unwrap()
    }

    #[test]
    fn rejects_invented_or_non_monotonic_indices() {
        let request = request();
        let result = AlignmentResult {
            schema_version: ALIGNMENT_SCHEMA_VERSION,
            unit_id: request.unit_id.clone(),
            embedding_model: "test-model".into(),
            source_count: 2,
            target_count: 1,
            total_cost: 0.2,
            mean_matched_similarity: Some(0.9),
            blocks: vec![AlignmentBlock {
                kind: AlignmentKind::Matched,
                source_indices: vec![1],
                target_indices: vec![0],
                similarity: Some(0.9),
            }],
        };
        assert!(matches!(
            validate_result(&request, &result),
            Err(AlignmentSidecarError::Protocol(_))
        ));
    }

    #[test]
    fn accepts_a_complete_monotonic_path_with_a_source_gap() {
        let request = request();
        let result = AlignmentResult {
            schema_version: ALIGNMENT_SCHEMA_VERSION,
            unit_id: request.unit_id.clone(),
            embedding_model: "test-model".into(),
            source_count: 2,
            target_count: 1,
            total_cost: 0.2,
            mean_matched_similarity: Some(0.9),
            blocks: vec![
                AlignmentBlock {
                    kind: AlignmentKind::Matched,
                    source_indices: vec![0],
                    target_indices: vec![0],
                    similarity: Some(0.9),
                },
                AlignmentBlock {
                    kind: AlignmentKind::SourceOnly,
                    source_indices: vec![1],
                    target_indices: vec![],
                    similarity: None,
                },
            ],
        };
        validate_result(&request, &result).unwrap();
    }

    #[test]
    fn request_rejects_unbounded_text_before_spawning_tool() {
        let request = AlignmentToolRequest {
            schema_version: ALIGNMENT_SCHEMA_VERSION,
            unit_id: "u".into(),
            source_segments: vec!["x".repeat(20)],
            target_segments: vec!["y".into()],
            config: AlignmentConfig::default(),
        };
        assert!(matches!(
            request.validate(10),
            Err(AlignmentSidecarError::InvalidRequest(_))
        ));
    }
}
