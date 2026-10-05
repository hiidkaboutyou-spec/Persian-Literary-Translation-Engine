use crate::book_ir::{
    BookBlock, BookIr, BookIrError, HeadingBlock, InlineRun, ParagraphBlock, RunProtection,
    SceneBreakBlock, TextDirection,
};
use crate::models::{Chapter, Manuscript, Scene};
use crate::parser::split_paragraph_text;

const DEFAULT_SCENE_BREAK: &str = "***";
const LEGACY_SCENE_SEPARATOR: &str = "\n\n***\n\n";

/// Convert the current ingestion model into the canonical, format-neutral Book IR.
///
/// Parser-owned paragraph IDs are preserved verbatim. Structural blocks that do
/// not exist in `Manuscript` receive deterministic IDs derived from existing
/// chapter/scene identity. Legacy chapters without structured scenes use
/// ordinal IDs so editing their text cannot silently change block identity.
pub fn manuscript_to_book_ir(manuscript: &Manuscript) -> Result<BookIr, BookIrError> {
    let mut book = BookIr::new(&manuscript.book.id, &manuscript.book.title);
    book.author.clone_from(&manuscript.book.author);
    book.metadata.clone_from(&manuscript.book.metadata);

    for chapter in &manuscript.chapters {
        push_heading(&mut book, chapter, manuscript.book.language.as_deref());

        let populated_scenes: Vec<&Scene> = chapter
            .scenes
            .iter()
            .filter(|scene| !scene.paragraphs.is_empty())
            .collect();

        if populated_scenes.is_empty() {
            push_legacy_content(
                &mut book,
                chapter,
                manuscript.book.language.as_deref(),
            );
        } else {
            push_structured_scenes(
                &mut book,
                chapter,
                &populated_scenes,
                manuscript.book.language.as_deref(),
            );
        }
    }

    book.validate()?;
    Ok(book)
}

fn push_heading(book: &mut BookIr, chapter: &Chapter, language: Option<&str>) {
    let title = if chapter.title.trim().is_empty() {
        format!("Chapter {}", chapter.order)
    } else {
        chapter.title.clone()
    };
    book.blocks
        .push(BookBlock::ChapterHeading(HeadingBlock {
            id: format!("{}:heading", chapter.id),
            level: 1,
            runs: vec![editable_run(title, language)],
        }));
}

fn push_structured_scenes(
    book: &mut BookIr,
    chapter: &Chapter,
    scenes: &[&Scene],
    language: Option<&str>,
) {
    for (scene_index, scene) in scenes.iter().enumerate() {
        if scene_index > 0 {
            book.blocks.push(BookBlock::SceneBreak(SceneBreakBlock {
                id: format!("{}:scene-break:{}", chapter.id, scene.id),
                marker: Some(DEFAULT_SCENE_BREAK.to_string()),
            }));
        }

        for paragraph in &scene.paragraphs {
            book.blocks.push(BookBlock::Paragraph(ParagraphBlock {
                id: paragraph.id.clone(),
                runs: vec![editable_run(paragraph.original_text.clone(), language)],
            }));
        }
    }
}

fn push_legacy_content(book: &mut BookIr, chapter: &Chapter, language: Option<&str>) {
    let normalized = chapter.content.replace("\r\n", "\n").replace('\r', "\n");
    let mut paragraph_ordinal = 0;

    for (scene_index, scene_text) in normalized.split(LEGACY_SCENE_SEPARATOR).enumerate() {
        let paragraphs = split_paragraph_text(scene_text);
        if paragraphs.is_empty() {
            continue;
        }

        if paragraph_ordinal > 0 {
            book.blocks.push(BookBlock::SceneBreak(SceneBreakBlock {
                id: format!("{}:scene-break:{}", chapter.id, scene_index + 1),
                marker: Some(DEFAULT_SCENE_BREAK.to_string()),
            }));
        }

        for text in paragraphs {
            paragraph_ordinal += 1;
            book.blocks.push(BookBlock::Paragraph(ParagraphBlock {
                id: format!("{}:paragraph:{}", chapter.id, paragraph_ordinal),
                runs: vec![editable_run(text, language)],
            }));
        }
    }
}

fn editable_run(text: String, language: Option<&str>) -> InlineRun {
    InlineRun {
        text,
        direction: TextDirection::Auto,
        protection: RunProtection::Editable,
        lang: language.map(str::to_string),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        Book, DocumentFormat, ImportanceMetadata, Paragraph, Scene, SourceLocation,
    };
    use std::collections::BTreeMap;

    fn source() -> SourceLocation {
        SourceLocation::new("story.epub", DocumentFormat::Epub)
    }

    fn book(metadata: BTreeMap<String, String>) -> Book {
        Book {
            id: "book-1".to_string(),
            title: "Story".to_string(),
            author: Some("Author".to_string()),
            language: Some("en-US".to_string()),
            metadata,
        }
    }

    #[test]
    fn preserves_parser_ids_order_and_import_metadata() {
        let mut metadata = BTreeMap::new();
        metadata.insert(
            "source_url".to_string(),
            "https://archiveofourown.org/works/123".to_string(),
        );
        metadata.insert("source_kind".to_string(), "ao3".to_string());

        let chapter_id = "chapter-1".to_string();
        let scene_one_id = "scene-1".to_string();
        let scene_two_id = "scene-2".to_string();
        let manuscript = Manuscript {
            book: book(metadata.clone()),
            chapters: vec![Chapter {
                id: chapter_id.clone(),
                title: "Chapter 1".to_string(),
                order: 1,
                index: 0,
                content: "First.\n\n***\n\nSecond.".to_string(),
                scenes: vec![
                    Scene {
                        id: scene_one_id.clone(),
                        chapter_id: chapter_id.clone(),
                        order: 1,
                        text: "First.".to_string(),
                        importance: ImportanceMetadata::default(),
                        paragraphs: vec![Paragraph {
                            id: "paragraph-source-1".to_string(),
                            scene_id: scene_one_id,
                            original_text: "First.".to_string(),
                            position: 1,
                            source: source(),
                        }],
                        source: source(),
                    },
                    Scene {
                        id: scene_two_id.clone(),
                        chapter_id: chapter_id.clone(),
                        order: 2,
                        text: "Second.".to_string(),
                        importance: ImportanceMetadata::default(),
                        paragraphs: vec![Paragraph {
                            id: "paragraph-source-2".to_string(),
                            scene_id: scene_two_id.clone(),
                            original_text: "Second.".to_string(),
                            position: 1,
                            source: source(),
                        }],
                        source: source(),
                    },
                ],
                source: source(),
            }],
            source: source(),
        };

        let ir = manuscript_to_book_ir(&manuscript).unwrap();

        assert_eq!(ir.author.as_deref(), Some("Author"));
        assert_eq!(ir.metadata, metadata);
        assert_eq!(
            ir.blocks.iter().map(BookBlock::id).collect::<Vec<_>>(),
            vec![
                "chapter-1:heading",
                "paragraph-source-1",
                "chapter-1:scene-break:scene-2",
                "paragraph-source-2",
            ]
        );
        let BookBlock::Paragraph(paragraph) = &ir.blocks[1] else {
            panic!("expected paragraph");
        };
        assert_eq!(paragraph.runs[0].lang.as_deref(), Some("en-US"));
        assert_eq!(paragraph.runs[0].direction, TextDirection::Auto);
        assert_eq!(paragraph.runs[0].protection, RunProtection::Editable);
        ir.validate().unwrap();
    }

    #[test]
    fn legacy_fallback_ids_do_not_depend_on_text() {
        let build = |content: &str| Manuscript {
            book: book(BTreeMap::new()),
            chapters: vec![Chapter {
                id: "legacy-chapter".to_string(),
                title: "Chapter 1".to_string(),
                order: 1,
                index: 0,
                content: content.to_string(),
                scenes: Vec::new(),
                source: source(),
            }],
            source: source(),
        };

        let original = manuscript_to_book_ir(&build("First.\n\n***\n\nSecond.")).unwrap();
        let edited =
            manuscript_to_book_ir(&build("Rewritten.\n\n***\n\nAlso rewritten.")).unwrap();

        assert_eq!(
            original.blocks.iter().map(BookBlock::id).collect::<Vec<_>>(),
            edited.blocks.iter().map(BookBlock::id).collect::<Vec<_>>()
        );
        assert_eq!(
            original.blocks.iter().map(BookBlock::id).collect::<Vec<_>>(),
            vec![
                "legacy-chapter:heading",
                "legacy-chapter:paragraph:1",
                "legacy-chapter:scene-break:2",
                "legacy-chapter:paragraph:2",
            ]
        );
    }

    #[test]
    fn duplicate_parser_ids_fail_closed() {
        let duplicate = |text: &str, scene_id: &str| Scene {
            id: scene_id.to_string(),
            chapter_id: "chapter-1".to_string(),
            order: 1,
            text: text.to_string(),
            importance: ImportanceMetadata::default(),
            paragraphs: vec![Paragraph {
                id: "duplicate".to_string(),
                scene_id: scene_id.to_string(),
                original_text: text.to_string(),
                position: 1,
                source: source(),
            }],
            source: source(),
        };
        let manuscript = Manuscript {
            book: book(BTreeMap::new()),
            chapters: vec![Chapter {
                id: "chapter-1".to_string(),
                title: "Chapter 1".to_string(),
                order: 1,
                index: 0,
                content: String::new(),
                scenes: vec![duplicate("One.", "scene-1"), duplicate("Two.", "scene-2")],
                source: source(),
            }],
            source: source(),
        };

        assert_eq!(
            manuscript_to_book_ir(&manuscript),
            Err(BookIrError::DuplicateId("duplicate".to_string()))
        );
    }
}
