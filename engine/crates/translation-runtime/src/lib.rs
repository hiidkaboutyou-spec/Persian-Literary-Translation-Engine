//! Translation runtime executes translation workflows.
//!
//! Literary intelligence provides understanding.
//! Translation runtime consumes that understanding.
//!
//! This crate intentionally does not own literary analysis.

pub mod context;
pub mod execution;
pub mod pipeline;

pub trait TranslationProvider {
    fn provider_name(&self) -> &str;

    fn translate(
        &self,
        request: &execution::TranslationExecutionRequest,
    ) -> Result<String, TranslationRuntimeError>;
}

pub trait QualityGate {
    fn validate(
        &self,
        output: &execution::TranslationOutput,
    ) -> Result<QualityReport, TranslationRuntimeError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QualityReport {
    pub accepted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TranslationRuntimeError {
    InvalidContext(String),
    ProviderFailure(String),
    ExecutionFailure(String),
    QualityRejected(String),
}
