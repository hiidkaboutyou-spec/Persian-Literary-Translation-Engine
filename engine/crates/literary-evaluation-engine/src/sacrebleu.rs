use serde::{Deserialize, Serialize};
use std::fmt;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SacreBleuEvaluationItem {
    pub id: String,
    pub translation: String,
    pub reference: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SacreBleuItemScore {
    pub id: String,
    pub chrf2pp: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SacreBleuEvaluationResult {
    pub tool: String,
    pub version: String,
    pub metric: String,
    pub signature: String,
    pub corpus_chrf2pp: f32,
    pub items: Vec<SacreBleuItemScore>,
}

#[derive(Debug, Serialize)]
struct SacreBleuRequest<'a> {
    items: &'a [SacreBleuEvaluationItem],
}

#[derive(Debug)]
pub enum SacreBleuError {
    Io(std::io::Error),
    Json(serde_json::Error),
    SidecarFailed { status: Option<i32>, stderr: String },
    Protocol(String),
}

impl fmt::Display for SacreBleuError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "SacreBLEU sidecar I/O error: {error}"),
            Self::Json(error) => write!(formatter, "SacreBLEU sidecar JSON error: {error}"),
            Self::SidecarFailed { status, stderr } => write!(
                formatter,
                "SacreBLEU sidecar failed with status {}: {}",
                status
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "signal".to_string()),
                stderr.trim()
            ),
            Self::Protocol(message) => write!(formatter, "SacreBLEU protocol error: {message}"),
        }
    }
}

impl std::error::Error for SacreBleuError {}

impl From<std::io::Error> for SacreBleuError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<serde_json::Error> for SacreBleuError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

#[derive(Debug, Clone)]
pub struct SacreBleuSidecar {
    python: PathBuf,
    script: PathBuf,
}

impl SacreBleuSidecar {
    pub fn new(python: impl Into<PathBuf>, script: impl Into<PathBuf>) -> Self {
        Self {
            python: python.into(),
            script: script.into(),
        }
    }

    pub fn python(&self) -> &Path {
        &self.python
    }

    pub fn script(&self) -> &Path {
        &self.script
    }

    pub fn evaluate(
        &self,
        items: &[SacreBleuEvaluationItem],
    ) -> Result<SacreBleuEvaluationResult, SacreBleuError> {
        if items.is_empty() {
            return Err(SacreBleuError::Protocol(
                "at least one evaluation item is required".to_string(),
            ));
        }
        for item in items {
            if item.id.trim().is_empty()
                || item.translation.trim().is_empty()
                || item.reference.trim().is_empty()
            {
                return Err(SacreBleuError::Protocol(
                    "items require non-empty id, translation, and reference".to_string(),
                ));
            }
        }

        let payload = serde_json::to_vec(&SacreBleuRequest { items })?;
        let mut child = Command::new(&self.python)
            .arg(&self.script)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        let mut stdin = child.stdin.take().ok_or_else(|| {
            SacreBleuError::Protocol("failed to open SacreBLEU sidecar stdin".to_string())
        })?;
        stdin.write_all(&payload)?;
        drop(stdin);

        let output = child.wait_with_output()?;
        if !output.status.success() {
            return Err(SacreBleuError::SidecarFailed {
                status: output.status.code(),
                stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            });
        }
        let result: SacreBleuEvaluationResult = serde_json::from_slice(&output.stdout)?;
        if result.tool != "sacrebleu" || result.metric != "chrF2++" {
            return Err(SacreBleuError::Protocol(
                "sidecar returned an unexpected tool or metric".to_string(),
            ));
        }
        if !result.corpus_chrf2pp.is_finite()
            || result.items.len() != items.len()
            || result
                .items
                .iter()
                .any(|item| !item.chrf2pp.is_finite())
        {
            return Err(SacreBleuError::Protocol(
                "sidecar returned invalid score data".to_string(),
            ));
        }
        for (expected, scored) in items.iter().zip(&result.items) {
            if expected.id != scored.id {
                return Err(SacreBleuError::Protocol(format!(
                    "sidecar item order/id mismatch: expected '{}', got '{}'",
                    expected.id, scored.id
                )));
            }
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_batch_is_rejected_without_starting_process() {
        let sidecar = SacreBleuSidecar::new("python3", "evaluate.py");
        assert!(sidecar.evaluate(&[]).is_err());
    }
}
