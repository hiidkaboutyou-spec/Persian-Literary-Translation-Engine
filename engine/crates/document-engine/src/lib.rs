//! Document ingestion layer.
//! Supports UTF-8 plain text/Markdown plus DOCX extraction and chapter segmentation.

pub mod chapter;
mod docx;

use std::fs;
use std::io;
use std::path::Path;

pub use chapter::{split_into_chapters, Chapter};
pub use docx::extract_docx_text;

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

fn title_from_path(path: &Path) -> String {
    path.file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("Untitled")
        .to_string()
}

pub fn load_text_file(path: impl AsRef<Path>) -> io::Result<Document> {
    let path = path.as_ref();
    let text = fs::read_to_string(path)?;
    Ok(load_document(title_from_path(path), text))
}

pub fn load_file(path: impl AsRef<Path>) -> io::Result<Document> {
    let path = path.as_ref();
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .map(str::to_ascii_lowercase)
        .unwrap_or_default();

    match extension.as_str() {
        "txt" | "md" | "markdown" => load_text_file(path),
        "docx" => {
            let text = extract_docx_text(path)?;
            Ok(load_document(title_from_path(path), text))
        }
        "pdf" | "epub" => Err(io::Error::new(
            io::ErrorKind::Unsupported,
            format!(".{extension} ingestion is not implemented yet"),
        )),
        "" => load_text_file(path),
        _ => Err(io::Error::new(
            io::ErrorKind::Unsupported,
            format!("unsupported document type: .{extension}"),
        )),
    }
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

    #[test]
    fn unsupported_formats_fail_explicitly() {
        let error = load_file("story.epub").unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::Unsupported);
    }
}
