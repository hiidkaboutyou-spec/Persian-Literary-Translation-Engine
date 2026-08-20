//! Document ingestion layer.
//! Future support: PDF, EPUB, DOCX parsing.

pub mod chapter;

pub use chapter::{split_into_chapters, Chapter};

#[derive(Debug, Clone)]
pub struct Document {
    pub title: String,
    pub text: String,
}

pub fn load_document(title: impl Into<String>, text: impl Into<String>) -> Document {
    Document { title: title.into(), text: text.into() }
}
