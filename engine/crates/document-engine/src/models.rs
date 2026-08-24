use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DocumentFormat {
    Pdf,
    Epub,
    Docx,
    Txt,
    Markdown,
}

impl DocumentFormat {
    pub fn from_path(path: &Path) -> Option<Self> {
        match path.extension()?.to_str()?.to_ascii_lowercase().as_str() {
            "pdf" => Some(Self::Pdf),
            "epub" => Some(Self::Epub),
            "docx" => Some(Self::Docx),
            "txt" => Some(Self::Txt),
            "md" | "markdown" => Some(Self::Markdown),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceLocation {
    pub path: PathBuf,
    pub format: DocumentFormat,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chapter: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scene: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paragraph: Option<usize>,
}

impl SourceLocation {
    pub fn new(path: impl Into<PathBuf>, format: DocumentFormat) -> Self {
        Self {
            path: path.into(),
            format,
            page: None,
            resource: None,
            chapter: None,
            scene: None,
            paragraph: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ImportanceMetadata {
    pub score: Option<f32>,
    pub labels: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Book {
    pub id: String,
    pub title: String,
    pub author: Option<String>,
    pub language: Option<String>,
    pub metadata: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Paragraph {
    pub id: String,
    pub scene_id: String,
    pub original_text: String,
    pub position: usize,
    pub source: SourceLocation,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Scene {
    pub id: String,
    pub chapter_id: String,
    pub order: usize,
    pub text: String,
    pub importance: ImportanceMetadata,
    pub paragraphs: Vec<Paragraph>,
    pub source: SourceLocation,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Chapter {
    pub id: String,
    pub title: String,
    pub order: usize,
    /// Compatibility with the original zero-based runtime contract.
    pub index: usize,
    pub content: String,
    pub scenes: Vec<Scene>,
    pub source: SourceLocation,
}

impl Chapter {
    pub fn translated(index: usize, title: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: format!("translated-chapter-{}", index + 1),
            title: title.into(),
            order: index + 1,
            index,
            content: content.into(),
            scenes: Vec::new(),
            source: SourceLocation::new("translated", DocumentFormat::Txt),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Manuscript {
    pub book: Book,
    pub chapters: Vec<Chapter>,
    pub source: SourceLocation,
}

impl Manuscript {
    pub fn chapter_count(&self) -> usize {
        self.chapters.len()
    }
    pub fn paragraph_count(&self) -> usize {
        self.chapters
            .iter()
            .flat_map(|chapter| &chapter.scenes)
            .map(|scene| scene.paragraphs.len())
            .sum()
    }
    pub fn translation_units(&self) -> impl Iterator<Item = &Paragraph> {
        self.chapters
            .iter()
            .flat_map(|chapter| &chapter.scenes)
            .flat_map(|scene| &scene.paragraphs)
    }
}
