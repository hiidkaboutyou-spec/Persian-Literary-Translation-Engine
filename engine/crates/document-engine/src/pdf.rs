use std::path::Path;

use crate::models::DocumentFormat;
use crate::parser::{
    base_source, file_stem_title, looks_like_chapter_heading, split_paragraph_text, BlockKind,
    ParsedBlock, ParsedDocument,
};
use crate::{load_document, Document, DocumentError};

pub fn load_pdf_file(path: impl AsRef<Path>) -> Result<Document, DocumentError> {
    let path = path.as_ref();
    let text = pdf_extract::extract_text(path).map_err(|error| {
        DocumentError::ParsingFailure(format!(
            "failed to extract text from PDF {}: {error}",
            path.display()
        ))
    })?;

    document_from_pdf_text(path, text)
}

fn normalize_extracted_pdf_text(text: &str) -> String {
    let normalized_newlines = text.replace("\r\n", "\n").replace('\r', "\n");
    let page_breaks_as_paragraphs = normalized_newlines.replace('\u{000c}', "\n\n");

    let mut output = String::with_capacity(page_breaks_as_paragraphs.len());
    let mut blank_run = 0usize;

    for raw_line in page_breaks_as_paragraphs.lines() {
        let line = raw_line.trim_end_matches([' ', '\t', '\0']);
        if line.trim().is_empty() {
            blank_run += 1;
            if blank_run == 1 && !output.is_empty() {
                output.push('\n');
            }
            continue;
        }

        if !output.is_empty() && !output.ends_with('\n') {
            output.push('\n');
        }
        output.push_str(line.trim_start_matches('\0'));
        output.push('\n');
        blank_run = 0;
    }

    output.trim_end().to_string()
}

fn document_from_pdf_text(path: &Path, text: String) -> Result<Document, DocumentError> {
    let text = normalize_extracted_pdf_text(&text);
    if text.trim().is_empty() {
        return Err(DocumentError::ParsingFailure(format!(
            "PDF {} contains no extractable text; scanned/image-only PDFs require OCR before ingestion",
            path.display()
        )));
    }

    let title = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("Untitled")
        .to_string();

    Ok(load_document(title, text))
}

pub(crate) fn parse_pdf(path: &Path) -> Result<ParsedDocument, DocumentError> {
    let pages = pdf_extract::extract_text_by_pages(path).map_err(|error| {
        DocumentError::ParsingFailure(format!("failed to extract PDF {}: {error}", path.display()))
    })?;
    let root = base_source(path, DocumentFormat::Pdf);
    let mut blocks = Vec::new();
    for (page_index, page) in pages.into_iter().enumerate() {
        let normalized = normalize_extracted_pdf_text(&page);
        for (paragraph_index, text) in split_paragraph_text(&normalized).into_iter().enumerate() {
            let kind = if looks_like_chapter_heading(&text) {
                BlockKind::Heading(1)
            } else if matches!(text.as_str(), "***" | "---" | "* * *") {
                BlockKind::SceneBreak
            } else {
                BlockKind::Paragraph
            };
            let mut source = root.clone();
            source.page = Some((page_index + 1) as u32);
            source.paragraph = Some(paragraph_index + 1);
            blocks.push(ParsedBlock { kind, text, source });
        }
    }
    if blocks.is_empty() {
        return Err(DocumentError::ParsingFailure(format!("PDF {} contains no extractable text; scanned/image-only PDFs require OCR before ingestion", path.display())));
    }
    let mut title = file_stem_title(path);
    let mut author = None;
    let mut metadata = std::collections::BTreeMap::new();
    if let Ok(document) = lopdf::Document::load(path) {
        if let Ok(reference) = document
            .trailer
            .get(b"Info")
            .and_then(lopdf::Object::as_reference)
        {
            if let Ok(info) = document.get_dictionary(reference) {
                if let Some(value) = pdf_string(info.get(b"Title").ok()) {
                    if !value.trim().is_empty() {
                        title = value.clone();
                        metadata.insert("title".into(), value);
                    }
                }
                if let Some(value) = pdf_string(info.get(b"Author").ok()) {
                    author = Some(value.clone());
                    metadata.insert("author".into(), value);
                }
            }
        }
    }
    Ok(ParsedDocument {
        title,
        author,
        language: None,
        metadata,
        source: root,
        blocks,
    })
}

fn pdf_string(value: Option<&lopdf::Object>) -> Option<String> {
    value
        .and_then(|object| object.as_str().ok())
        .map(|bytes| String::from_utf8_lossy(bytes).trim().to_string())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn builds_document_from_extracted_pdf_text() {
        let document = document_from_pdf_text(
            Path::new("The Hunters Moon.pdf"),
            "Chapter 1\nHello".to_string(),
        )
        .expect("non-empty extracted text should produce a document");

        assert_eq!(document.title, "The Hunters Moon");
        assert_eq!(document.text, "Chapter 1\nHello");
    }

    #[test]
    fn normalizes_pdf_line_endings_page_breaks_and_blank_runs() {
        let text = "Chapter 1\r\nFirst line   \r\n\r\n\r\n\u{000c}Chapter 2\0\nSecond line\t";
        let normalized = normalize_extracted_pdf_text(text);

        assert_eq!(
            normalized,
            "Chapter 1\nFirst line\n\nChapter 2\nSecond line"
        );
    }

    #[test]
    fn preserves_leading_indentation_but_removes_nul_noise() {
        let normalized = normalize_extracted_pdf_text("\0  dialogue\n    indented line\0");

        assert_eq!(normalized, "  dialogue\n    indented line");
    }

    #[test]
    fn rejects_image_only_or_empty_pdf_text() {
        let error = document_from_pdf_text(Path::new("scan.pdf"), "  \n\u{000c}\0".to_string())
            .expect_err("blank extracted text should be rejected");

        assert!(error.to_string().contains("require OCR"));
    }
}
