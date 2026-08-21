//! Document ingestion layer.
//! V1 supports UTF-8 plain text, DOCX, EPUB, and text-based PDF files, plus chapter segmentation.

pub mod chapter;
pub mod docx;
pub mod epub;
pub mod pdf;

use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub use chapter::{split_into_chapters, Chapter};
pub use docx::load_docx_file;
pub use epub::load_epub_file;
pub use pdf::load_pdf_file;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    pub title: String,
    pub text: String,
}

#[derive(Debug)]
pub enum DocumentError {
    Io(io::Error),
    Zip(zip::result::ZipError),
    UnsupportedFormat(PathBuf),
    InvalidDocument(String),
}

impl fmt::Display for DocumentError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "I/O error: {error}"),
            Self::Zip(error) => write!(formatter, "document archive error: {error}"),
            Self::UnsupportedFormat(path) => write!(
                formatter,
                "unsupported document format for {} (supported: .txt, .md, .docx, .epub, .pdf)",
                path.display()
            ),
            Self::InvalidDocument(message) => write!(formatter, "invalid document: {message}"),
        }
    }
}

impl std::error::Error for DocumentError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Zip(error) => Some(error),
            Self::UnsupportedFormat(_) | Self::InvalidDocument(_) => None,
        }
    }
}

impl From<io::Error> for DocumentError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
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

pub fn load_file(path: impl AsRef<Path>) -> Result<Document, DocumentError> {
    let path = path.as_ref();
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .map(str::to_ascii_lowercase);

    match extension.as_deref() {
        Some("txt") | Some("md") => load_text_file(path).map_err(DocumentError::Io),
        Some("docx") => load_docx_file(path),
        Some("epub") => load_epub_file(path),
        Some("pdf") => load_pdf_file(path),
        _ => Err(DocumentError::UnsupportedFormat(path.to_path_buf())),
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
    fn rejects_unsupported_extensions() {
        let error = load_file("story.rtf").expect_err("RTF is not supported");
        assert!(matches!(error, DocumentError::UnsupportedFormat(_)));
        assert!(error.to_string().contains(".txt, .md, .docx, .epub, .pdf"));
    }
}
