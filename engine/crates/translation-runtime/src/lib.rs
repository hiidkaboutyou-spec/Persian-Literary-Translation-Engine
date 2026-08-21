//! Translation runtime executes translation workflows.
//!
//! Literary intelligence provides understanding.
//! Translation runtime consumes that understanding.
//!
//! This crate intentionally does not own literary analysis.

pub mod pipeline;

pub trait TranslationProvider {
    fn translate(&self, source: &str) -> Result<String, TranslationRuntimeError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TranslationRuntimeError {
    ProviderFailure(String),
}
