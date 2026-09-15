use serde::{Deserialize, Serialize};
use std::fmt;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CometEvaluationItem {
    pub id: String,
    pub source: String,
    pub translation: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CometScoredItem {
    pub id: String,
    pub score: f32,
    #[serde(default)]
    pub error_spans: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CometEvaluationResult {
    pub model: String,
    pub system_score: f32,
    pub items: Vec<CometScoredItem>,
}

#[derive(Debug, Serialize)]
struct CometRequest<'a> {
    model: &'a str,
    batch_size: usize,
    gpus: usize,
    items: &'a [CometEvaluationItem],
}

#[derive(Debug)]
pub enum CometError {
    Io(std::io::Error),
    Json(serde_json::Error),
    SidecarFailed { status: Option<i32>, stderr: String },
    Protocol(String),
}

impl fmt::Display for CometError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "COMET sidecar I/O error: {error}"),
            Self::Json(error) => write!(formatter, "COMET sidecar JSON error: {error}"),
            Self::SidecarFailed { status, stderr } => write!(
                formatter,
                "COMET sidecar failed with status {}: {}",
                status
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "signal".to_string()),
                stderr.trim()
            ),
            Self::Protocol(message) => write!(formatter, "COMET sidecar protocol error: {message}"),
        }
    }
}

impl std::error::Error for CometError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Json(error) => Some(error),
            Self::SidecarFailed { .. } | Self::Protocol(_) => None,
        }
    }
}

impl From<std::io::Error> for CometError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<serde_json::Error> for CometError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

#[derive(Debug, Clone)]
pub struct CometSidecar {
    python: PathBuf,
    script: PathBuf,
    model: String,
    batch_size: usize,
    gpus: usize,
}

impl CometSidecar {
    pub fn new(
        python: impl Into<PathBuf>,
        script: impl Into<PathBuf>,
        model: impl Into<String>,
    ) -> Self {
        Self {
            python: python.into(),
            script: script.into(),
            model: model.into(),
            batch_size: 8,
            gpus: 0,
        }
    }

    pub fn with_batch_size(mut self, batch_size: usize) -> Self {
        self.batch_size = batch_size.max(1);
        self
    }

    pub fn with_gpus(mut self, gpus: usize) -> Self {
        self.gpus = gpus;
        self
    }

    pub fn python(&self) -> &Path {
        &self.python
    }

    pub fn script(&self) -> &Path {
        &self.script
    }

    pub fn model(&self) -> &str {
        &self.model
    }

    pub fn evaluate(
        &self,
        items: &[CometEvaluationItem],
    ) -> Result<CometEvaluationResult, CometError> {
        if items.is_empty() {
            return Err(CometError::Protocol(
                "at least one evaluation item is required".to_string(),
            ));
        }
        if self.model.trim().is_empty() {
            return Err(CometError::Protocol(
                "COMET model name must not be empty".to_string(),
            ));
        }

        let request = CometRequest {
            model: &self.model,
            batch_size: self.batch_size,
            gpus: self.gpus,
            items,
        };
        let payload = serde_json::to_vec(&request)?;

        let mut child = Command::new(&self.python)
            .arg(&self.script)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        let mut stdin = child.stdin.take().ok_or_else(|| {
            CometError::Protocol("failed to open COMET sidecar stdin".to_string())
        })?;
        stdin.write_all(&payload)?;
        drop(stdin);

        let output = child.wait_with_output()?;
        if !output.status.success() {
            return Err(CometError::SidecarFailed {
                status: output.status.code(),
                stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            });
        }

        let result: CometEvaluationResult = serde_json::from_slice(&output.stdout)?;
        if result.items.len() != items.len() {
            return Err(CometError::Protocol(format!(
                "sidecar returned {} item scores for {} inputs",
                result.items.len(),
                items.len()
            )));
        }
        for (expected, scored) in items.iter().zip(&result.items) {
            if expected.id != scored.id {
                return Err(CometError::Protocol(format!(
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
    fn request_and_response_protocol_round_trip() {
        let item = CometEvaluationItem {
            id: "chapter-1-p1".into(),
            source: "She smiled.".into(),
            translation: "لبخند زد.".into(),
            reference: None,
        };
        let request = CometRequest {
            model: "example/model",
            batch_size: 4,
            gpus: 0,
            items: std::slice::from_ref(&item),
        };
        let json = serde_json::to_string(&request).expect("request should serialize");
        assert!(json.contains("chapter-1-p1"));
        assert!(json.contains("example/model"));

        let response = r#"{
            "model":"example/model",
            "system_score":0.91,
            "items":[{"id":"chapter-1-p1","score":0.91,"error_spans":[]}]
        }"#;
        let parsed: CometEvaluationResult =
            serde_json::from_str(response).expect("response should deserialize");
        assert_eq!(parsed.model, "example/model");
        assert_eq!(parsed.items[0].id, item.id);
    }

    #[test]
    fn empty_batch_is_rejected_before_process_start() {
        let sidecar = CometSidecar::new("python3", "evaluate.py", "example/model");
        let error = sidecar.evaluate(&[]).expect_err("empty batch must fail");
        assert!(error.to_string().contains("at least one evaluation item"));
    }
}
