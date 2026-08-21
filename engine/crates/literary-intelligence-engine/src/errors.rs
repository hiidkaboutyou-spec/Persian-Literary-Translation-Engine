use thiserror::Error;

#[derive(Debug, Error)]
pub enum LiteraryIntelligenceError {
    #[error("invalid literary intelligence data: {0}")]
    Validation(String),
}
