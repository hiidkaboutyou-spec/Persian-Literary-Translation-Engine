use std::path::Path;

use crate::{load_document, Document, DocumentError};

pub fn load_pdf_file(path: impl AsRef<Path>) -> Result<Document, DocumentError> {
    let path = path.as_ref();
    let text = pdf_extract::extract_text(path).map_err(|error| {
        DocumentError::InvalidDocument(format!(
            "failed to extract text from PDF {}: {error}",
            path.display()
        ))
    })?;

    document_from_pdf_text(path, text)
}

fn document_from_pdf_text(path: &Path, text: String) -> Result<Document, DocumentError> {
    if text.trim().is_empty() {
        return Err(DocumentError::InvalidDocument(format!(
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
    fn rejects_image_only_or_empty_pdf_text() {
        let error = document_from_pdf_text(Path::new("scan.pdf"), "  \n".to_string())
            .expect_err("blank extracted text should be rejected");

        assert!(error.to_string().contains("require OCR"));
    }
}
