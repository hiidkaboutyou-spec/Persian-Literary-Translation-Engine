use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Current persisted schema version for the canonical publishing model.
///
/// Book IR is intentionally format-neutral: DOCX, EPUB, PDF and the web editor
/// must consume the same semantic document instead of inventing parallel models.
pub const BOOK_IR_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BookIr {
    pub schema_version: u32,
    pub id: String,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(default)]
    pub metadata: BTreeMap<String, String>,
    pub blocks: Vec<BookBlock>,
}

impl BookIr {
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            schema_version: BOOK_IR_SCHEMA_VERSION,
            id: id.into(),
            title: title.into(),
            author: None,
            metadata: BTreeMap::new(),
            blocks: Vec::new(),
        }
    }

    pub fn validate(&self) -> Result<(), BookIrError> {
        if self.schema_version != BOOK_IR_SCHEMA_VERSION {
            return Err(BookIrError::UnsupportedSchema(self.schema_version));
        }
        if self.id.trim().is_empty() {
            return Err(BookIrError::MissingId("book"));
        }

        let mut ids = std::collections::BTreeSet::new();
        for block in &self.blocks {
            let id = block.id();
            if id.trim().is_empty() {
                return Err(BookIrError::MissingId("block"));
            }
            if !ids.insert(id) {
                return Err(BookIrError::DuplicateId(id.to_string()));
            }
            if let BookBlock::Paragraph(paragraph) = block {
                for run in &paragraph.runs {
                    if run.text.is_empty() {
                        return Err(BookIrError::EmptyInlineRun(paragraph.id.clone()));
                    }
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BookBlock {
    ChapterHeading(HeadingBlock),
    Paragraph(ParagraphBlock),
    SceneBreak(SceneBreakBlock),
}

impl BookBlock {
    pub fn id(&self) -> &str {
        match self {
            Self::ChapterHeading(block) => &block.id,
            Self::Paragraph(block) => &block.id,
            Self::SceneBreak(block) => &block.id,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeadingBlock {
    pub id: String,
    pub level: u8,
    pub runs: Vec<InlineRun>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParagraphBlock {
    pub id: String,
    pub runs: Vec<InlineRun>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SceneBreakBlock {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub marker: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InlineRun {
    pub text: String,
    #[serde(default)]
    pub direction: TextDirection,
    #[serde(default)]
    pub protection: RunProtection,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
}

impl InlineRun {
    pub fn persian(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            direction: TextDirection::Rtl,
            protection: RunProtection::Editable,
            lang: Some("fa-IR".to_string()),
        }
    }

    pub fn protected_ltr(text: impl Into<String>, kind: ProtectedKind) -> Self {
        Self {
            text: text.into(),
            direction: TextDirection::Ltr,
            protection: RunProtection::Protected(kind),
            lang: Some("en-US".to_string()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum TextDirection {
    Rtl,
    Ltr,
    #[default]
    Auto,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(tag = "mode", content = "kind", rename_all = "snake_case")]
pub enum RunProtection {
    #[default]
    Editable,
    Protected(ProtectedKind),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProtectedKind {
    Url,
    Email,
    Isbn,
    Identifier,
    Code,
    FilePath,
    ExplicitLatin,
    Provenance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BookIrError {
    UnsupportedSchema(u32),
    MissingId(&'static str),
    DuplicateId(String),
    EmptyInlineRun(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versioned_book_ir_round_trips_without_losing_protected_runs() {
        let mut book = BookIr::new("book-1", "نمونه");
        book.blocks.push(BookBlock::Paragraph(ParagraphBlock {
            id: "p-1".to_string(),
            runs: vec![
                InlineRun::persian("به "),
                InlineRun::protected_ltr("OpenAI.com", ProtectedKind::Url),
                InlineRun::persian(" نگاه کن."),
            ],
        }));

        book.validate().unwrap();
        let json = serde_json::to_string(&book).unwrap();
        let restored: BookIr = serde_json::from_str(&json).unwrap();

        assert_eq!(restored, book);
        assert_eq!(restored.schema_version, BOOK_IR_SCHEMA_VERSION);
        let BookBlock::Paragraph(paragraph) = &restored.blocks[0] else {
            panic!("expected paragraph");
        };
        assert_eq!(paragraph.id, "p-1");
        assert_eq!(paragraph.runs[1].direction, TextDirection::Ltr);
        assert_eq!(
            paragraph.runs[1].protection,
            RunProtection::Protected(ProtectedKind::Url)
        );
    }

    #[test]
    fn validation_rejects_duplicate_stable_block_ids() {
        let mut book = BookIr::new("book-1", "نمونه");
        for _ in 0..2 {
            book.blocks.push(BookBlock::Paragraph(ParagraphBlock {
                id: "same-id".to_string(),
                runs: vec![InlineRun::persian("متن")],
            }));
        }
        assert_eq!(
            book.validate(),
            Err(BookIrError::DuplicateId("same-id".to_string()))
        );
    }

    #[test]
    fn validation_fails_closed_on_unknown_schema_version() {
        let mut book = BookIr::new("book-1", "نمونه");
        book.schema_version = BOOK_IR_SCHEMA_VERSION + 1;
        assert_eq!(
            book.validate(),
            Err(BookIrError::UnsupportedSchema(BOOK_IR_SCHEMA_VERSION + 1))
        );
    }
}
