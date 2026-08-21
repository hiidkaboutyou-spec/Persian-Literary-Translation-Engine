use crate::errors::LiteraryIntelligenceError;
use crate::models::understanding::LiteraryUnderstanding;

/// Domain contract for future sources of literary understanding.
pub trait LiteraryUnderstandingProvider {
    fn build_understanding(&self) -> Result<LiteraryUnderstanding, LiteraryIntelligenceError>;
}
