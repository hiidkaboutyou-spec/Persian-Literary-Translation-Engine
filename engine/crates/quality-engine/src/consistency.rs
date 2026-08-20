//! Translation consistency checks.
//! Future integration: glossary, character memory and style validation.

#[derive(Debug, Clone)]
pub struct ConsistencyResult {
    pub score: f32,
    pub warnings: Vec<String>,
}

pub fn check_consistency() -> ConsistencyResult {
    ConsistencyResult {
        score: 1.0,
        warnings: Vec::new(),
    }
}
