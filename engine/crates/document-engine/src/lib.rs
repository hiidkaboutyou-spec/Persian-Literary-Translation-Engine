//! Document ingestion layer.
//! V1 supports UTF-8 plain-text files and chapter segmentation.

pub mod chapter;

use std::fs;
use std::io;
use std::path::Path;

pub use chapter::{split_into_chapters, Chapter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    pub title: String,
    pub text: String,
}

pub fn load_document(title: impl Into<String>, text: impl Into<String>) -> Document {
    Document {
        title: title.into(),
        text: text.into(),
    }
}

pub fn load_text_file(path: impl AsRef<Path>) -> io::Result<Document> {
    let path = path.as_ref();
    let text = fs::read_to_string(path)?;
    let title = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("Untitled")
        .to_string();

    Ok(load_document(title, text))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_document_preserves_content() {
        let document = load_document("Story", "Hello world");
        assert_eq!(document.title, "Story");
        assert_eq!(document.text, "Hello world");
    }

    #[test]
    fn chapter_api_is_exposed() {
        let chapters = split_into_chapters("Chapter 1\nOne\nChapter 2\nTwo");
        assert_eq!(chapters.len(), 2);
        assert_eq!(chapters[0].index, 0);
        assert_eq!(chapters[1].content, "Two");
    }
}
