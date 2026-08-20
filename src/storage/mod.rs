//! Storage abstraction layer.
//!
//! The translation engine keeps storage behind traits so the Rust core
//! remains independent from SQLite, Supabase, or other backends.

pub mod traits;

pub use traits::{GlossaryStore, TranslationMemoryStore};
