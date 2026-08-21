use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SourceFormat { Pdf, Epub, Docx, Txt, Markdown }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: Uuid,
    pub title: String,
    pub author: String,
    pub language: String,
    pub source_format: SourceFormat,
    pub file_reference: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manuscript {
    pub id: Uuid,
    pub document_id: Uuid,
    pub title: String,
    pub chapters: Vec<Chapter>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    pub id: Uuid,
    pub number: u32,
    pub title: String,
    pub paragraphs: Vec<Paragraph>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paragraph {
    pub id: Uuid,
    pub chapter_id: Uuid,
    pub position: u32,
    pub content: String,
    pub paragraph_type: ParagraphType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ParagraphType { Narrative, Dialogue, Heading, Unknown }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneBoundary {
    pub id: Uuid,
    pub chapter_id: Uuid,
    pub start_paragraph: u32,
    pub end_paragraph: u32,
    pub metadata: serde_json::Value,
}

pub trait DocumentImporter {
    fn import(&self, document: Document) -> Result<Manuscript, DocumentError>;
}

#[derive(Debug, Error)]
pub enum DocumentError {
    #[error("missing title")]
    MissingTitle,
    #[error("unsupported format")]
    UnsupportedFormat,
    #[error("invalid document structure")]
    InvalidStructure,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn document_serializes() {
        let doc = Document {
            id: Uuid::new_v4(), title: "Book".into(), author: "Author".into(),
            language: "en".into(), source_format: SourceFormat::Txt,
            file_reference: "book.txt".into(), metadata: serde_json::json!({}),
            created_at: Utc::now(),
        };
        assert!(serde_json::to_string(&doc).is_ok());
    }
}
