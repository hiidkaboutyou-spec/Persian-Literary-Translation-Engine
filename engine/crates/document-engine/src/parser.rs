use crate::models::{
    Book, Chapter, DocumentFormat, ImportanceMetadata, Manuscript, Paragraph, Scene, SourceLocation,
};
use crate::{docx, epub, pdf, text, DocumentError};
use std::collections::{BTreeMap, HashMap};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockKind {
    Heading(u8),
    Paragraph,
    SceneBreak,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedBlock {
    pub kind: BlockKind,
    pub text: String,
    pub source: SourceLocation,
}

#[derive(Debug, Clone)]
pub struct ParsedDocument {
    pub title: String,
    pub author: Option<String>,
    pub language: Option<String>,
    pub metadata: BTreeMap<String, String>,
    pub source: SourceLocation,
    pub blocks: Vec<ParsedBlock>,
}

pub trait ManuscriptParser: Send + Sync {
    fn format(&self) -> DocumentFormat;
    fn parse(&self, path: &Path) -> Result<ParsedDocument, DocumentError>;
}

struct BuiltInParser(DocumentFormat);
impl ManuscriptParser for BuiltInParser {
    fn format(&self) -> DocumentFormat {
        self.0
    }
    fn parse(&self, path: &Path) -> Result<ParsedDocument, DocumentError> {
        match self.0 {
            DocumentFormat::Txt => text::parse_text(path, false),
            DocumentFormat::Markdown => text::parse_text(path, true),
            DocumentFormat::Docx => docx::parse_docx(path),
            DocumentFormat::Epub => epub::parse_epub(path),
            DocumentFormat::Pdf => pdf::parse_pdf(path),
        }
    }
}

pub struct DocumentIngestor {
    parsers: HashMap<DocumentFormat, Box<dyn ManuscriptParser>>,
}
impl Default for DocumentIngestor {
    fn default() -> Self {
        let mut value = Self {
            parsers: HashMap::new(),
        };
        for format in [
            DocumentFormat::Pdf,
            DocumentFormat::Epub,
            DocumentFormat::Docx,
            DocumentFormat::Txt,
            DocumentFormat::Markdown,
        ] {
            value.register(Box::new(BuiltInParser(format)));
        }
        value
    }
}
impl DocumentIngestor {
    pub fn register(&mut self, parser: Box<dyn ManuscriptParser>) {
        self.parsers.insert(parser.format(), parser);
    }
    pub fn ingest(&self, path: impl AsRef<Path>) -> Result<Manuscript, DocumentError> {
        let path = path.as_ref();
        let format = DocumentFormat::from_path(path)
            .ok_or_else(|| DocumentError::UnsupportedFormat(path.to_path_buf()))?;
        let parser = self
            .parsers
            .get(&format)
            .ok_or_else(|| DocumentError::UnsupportedFormat(path.to_path_buf()))?;
        build_manuscript(parser.parse(path)?)
    }
}

pub(crate) fn base_source(path: &Path, format: DocumentFormat) -> SourceLocation {
    SourceLocation::new(path, format)
}
pub(crate) fn file_stem_title(path: &Path) -> String {
    path.file_stem()
        .and_then(|v| v.to_str())
        .unwrap_or("Untitled")
        .to_string()
}
pub(crate) fn looks_like_chapter_heading(text: &str) -> bool {
    let value = text.trim();
    if value.is_empty() || value.len() > 120 {
        return false;
    }
    let lower = value.to_ascii_lowercase();
    lower.starts_with("chapter ")
        || lower.starts_with("part ")
        || lower.starts_with("book ")
        || value.starts_with("فصل ")
        || value.starts_with("بخش ")
}
pub(crate) fn split_paragraph_text(text: &str) -> Vec<String> {
    text.replace("\r\n", "\n")
        .replace('\r', "\n")
        .split("\n\n")
        .map(|part| part.lines().map(str::trim).collect::<Vec<_>>().join(" "))
        .map(|part| part.trim().to_string())
        .filter(|part| !part.is_empty())
        .collect()
}
fn stable_id(parts: &[&str]) -> String {
    let mut hash = 0xcbf29ce484222325_u64;
    for part in parts {
        for byte in part.as_bytes().iter().copied().chain(std::iter::once(0)) {
            hash = (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3);
        }
    }
    format!("doc-{hash:016x}")
}

fn build_manuscript(parsed: ParsedDocument) -> Result<Manuscript, DocumentError> {
    let path_key = parsed.source.path.to_string_lossy();
    let book_id = stable_id(&[&path_key, &parsed.title]);
    let mut groups: Vec<(String, SourceLocation, Vec<ParsedBlock>)> = Vec::new();
    let mut title = None;
    let mut source = parsed.source.clone();
    let mut blocks = Vec::new();
    for block in parsed.blocks {
        let heading = matches!(block.kind, BlockKind::Heading(_))
            || (matches!(block.kind, BlockKind::Paragraph)
                && looks_like_chapter_heading(&block.text));
        if heading {
            if !blocks.is_empty() {
                let fallback = format!("Chapter {}", groups.len() + 1);
                groups.push((
                    title.take().unwrap_or(fallback),
                    source.clone(),
                    std::mem::take(&mut blocks),
                ));
            }
            title = Some(block.text.trim().to_string());
            source = block.source;
        } else {
            if blocks.is_empty() && title.is_none() {
                source = block.source.clone();
            }
            blocks.push(block);
        }
    }
    if !blocks.is_empty() {
        let fallback = format!("Chapter {}", groups.len() + 1);
        groups.push((title.unwrap_or(fallback), source, blocks));
    }
    if groups.is_empty() {
        return Err(DocumentError::EmptyDocument(parsed.source.path));
    }

    let mut chapters = Vec::new();
    for (chapter_index, (title, mut chapter_source, blocks)) in groups.into_iter().enumerate() {
        let order = chapter_index + 1;
        let chapter_id = stable_id(&[&book_id, "chapter", &order.to_string(), &title]);
        chapter_source.chapter = Some(order);
        let mut scene_groups = vec![Vec::new()];
        for block in blocks {
            if matches!(block.kind, BlockKind::SceneBreak) {
                if scene_groups.last().is_some_and(|group| !group.is_empty()) {
                    scene_groups.push(Vec::new());
                }
            } else if let Some(group) = scene_groups.last_mut() {
                group.push(block);
            } else {
                return Err(DocumentError::InvalidStructure(
                    "scene grouping was not initialized".into(),
                ));
            }
        }
        scene_groups.retain(|group| !group.is_empty());
        let mut scenes = Vec::new();
        for (scene_index, scene_blocks) in scene_groups.into_iter().enumerate() {
            let scene_order = scene_index + 1;
            let scene_id = stable_id(&[&chapter_id, "scene", &scene_order.to_string()]);
            let mut paragraphs = Vec::new();
            for block in scene_blocks {
                for original_text in split_paragraph_text(&block.text) {
                    let position = paragraphs.len() + 1;
                    let mut location = block.source.clone();
                    location.chapter = Some(order);
                    location.scene = Some(scene_order);
                    location.paragraph.get_or_insert(position);
                    paragraphs.push(Paragraph {
                        id: stable_id(&[
                            &scene_id,
                            "paragraph",
                            &position.to_string(),
                            &original_text,
                        ]),
                        scene_id: scene_id.clone(),
                        original_text,
                        position,
                        source: location,
                    });
                }
            }
            if !paragraphs.is_empty() {
                let text = paragraphs
                    .iter()
                    .map(|p| p.original_text.as_str())
                    .collect::<Vec<_>>()
                    .join("\n\n");
                let location = paragraphs[0].source.clone();
                scenes.push(Scene {
                    id: scene_id,
                    chapter_id: chapter_id.clone(),
                    order: scene_order,
                    text,
                    importance: ImportanceMetadata::default(),
                    paragraphs,
                    source: location,
                });
            }
        }
        if !scenes.is_empty() {
            let content = scenes
                .iter()
                .map(|s| s.text.as_str())
                .collect::<Vec<_>>()
                .join("\n\n***\n\n");
            chapters.push(Chapter {
                id: chapter_id,
                title,
                order,
                index: chapter_index,
                content,
                scenes,
                source: chapter_source,
            });
        }
    }
    if chapters.is_empty() {
        return Err(DocumentError::InvalidStructure(
            "document has no non-empty chapters".into(),
        ));
    }
    Ok(Manuscript {
        book: Book {
            id: book_id,
            title: parsed.title,
            author: parsed.author,
            language: parsed.language,
            metadata: parsed.metadata,
        },
        chapters,
        source: parsed.source,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn constructs_scenes_and_source_positions() {
        let source = SourceLocation::new("story.txt", DocumentFormat::Txt);
        let parsed = ParsedDocument {
            title: "Story".into(),
            author: None,
            language: None,
            metadata: BTreeMap::new(),
            source: source.clone(),
            blocks: vec![
                ParsedBlock {
                    kind: BlockKind::Heading(1),
                    text: "Chapter 1".into(),
                    source: source.clone(),
                },
                ParsedBlock {
                    kind: BlockKind::Paragraph,
                    text: "First.".into(),
                    source: source.clone(),
                },
                ParsedBlock {
                    kind: BlockKind::SceneBreak,
                    text: "***".into(),
                    source: source.clone(),
                },
                ParsedBlock {
                    kind: BlockKind::Paragraph,
                    text: "Second.".into(),
                    source,
                },
            ],
        };
        let manuscript = build_manuscript(parsed).unwrap();
        assert_eq!(manuscript.chapters[0].scenes.len(), 2);
        assert_eq!(
            manuscript.chapters[0].scenes[1].paragraphs[0].source.scene,
            Some(2)
        );
    }
}
