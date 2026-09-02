//! Literary Intelligence creates understanding.
//!
//! Translation consumes understanding.
//!
//! This crate represents literary meaning before translation begins.
//! It models narrative concepts such as characters, scenes, relationships,
//! literary rules, and translation decisions.
//!
//! This crate does not:
//! - translate text
//! - call AI providers
//! - store persistence data
//! - orchestrate workflows

pub mod errors;
pub mod models;
pub mod traits;

mod manuscript_analyzer;

pub use manuscript_analyzer::{
    AnalysisCanon, AnalysisConfig, DeterministicManuscriptAnalyzer, ManuscriptAnalyzer,
};
pub use models::manuscript_intelligence::*;
