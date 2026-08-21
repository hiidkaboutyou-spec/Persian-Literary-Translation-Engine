//! Quality gates validate translation execution outputs.
//!
//! This layer does not rewrite translations.
//! It checks whether runtime output preserved required guarantees.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QualityReport {
    pub checks: Vec<QualityCheck>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QualityCheck {
    pub name: String,
    pub passed: bool,
    pub details: String,
}

impl QualityReport {
    pub fn passed(&self) -> bool {
        self.checks.iter().all(|check| check.passed)
    }
}
