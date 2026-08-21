//! Literary Intelligence creates understanding.
//!
//! Translation consumes understanding.
//!
//! The Literary Understanding Pipeline provides structured literary context before translation.
//! It models narrative concepts such as characters, scenes, relationships, literary rules,
//! translation decisions, and the aggregate understanding state that future translation layers
//! can consume.
//!
//! This crate does not:
//! - translate text
//! - generate translations
//! - call AI providers
//! - persist data
//! - orchestrate workflows

pub mod errors;
pub mod models;
pub mod traits;
