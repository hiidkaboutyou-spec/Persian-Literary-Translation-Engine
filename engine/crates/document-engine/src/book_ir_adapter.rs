use crate::book_ir::{
    BookBlock, BookIr, BookIrError, HeadingBlock, InlineRun, ParagraphBlock, ProtectedKind,
    RunProtection, SceneBreakBlock, TextDirection,
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
            push_legacy_content(&mut book, chapter, manuscript.book.language.as_deref());
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
    book.blocks.push(BookBlock::ChapterHeading(HeadingBlock {
        id: format!("{}:heading", chapter.id),
        level: 1,
        runs: protected_runs(&title, language),
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
                runs: protected_runs(&paragraph.original_text, language),
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
                runs: protected_runs(&text, language),
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

/// Split only high-confidence technical tokens out of literary prose.
///
/// Generic Latin words deliberately remain editable: the input manuscript is
/// normally English, so treating every ASCII span as protected would prevent
/// translation. The scanner is dependency-free, byte-preserving and
/// conservative. Unsupported/ambiguous tokens fall back to editable prose.
fn protected_runs(text: &str, language: Option<&str>) -> Vec<InlineRun> {
    if text.is_empty() {
        return Vec::new();
    }

    let mut spans: Vec<(usize, usize, ProtectedKind)> = Vec::new();
    let mut cursor = 0;

    while cursor < text.len() {
        if let Some((end, kind)) = protected_span_at(text, cursor) {
            if let Some((_, last_end, _)) = spans.last() {
                if *last_end > cursor {
                    cursor = *last_end;
                    continue;
                }
            }
            spans.push((cursor, end, kind));
            cursor = end;
        } else {
            cursor += text[cursor..].chars().next().map_or(1, char::len_utf8);
        }
    }

    if spans.is_empty() {
        return vec![editable_run(text.to_string(), language)];
    }

    let mut runs = Vec::new();
    let mut start = 0;
    for (span_start, span_end, kind) in spans {
        if span_start > start {
            runs.push(editable_run(text[start..span_start].to_string(), language));
        }
        runs.push(InlineRun::protected_ltr(
            text[span_start..span_end].to_string(),
            kind,
        ));
        start = span_end;
    }
    if start < text.len() {
        runs.push(editable_run(text[start..].to_string(), language));
    }
    runs
}

fn protected_span_at(text: &str, start: usize) -> Option<(usize, ProtectedKind)> {
    if !is_token_boundary(text, start) {
        return None;
    }

    if text[start..].starts_with('`') {
        let content_start = start + 1;
        let close = text[content_start..].find('`')? + content_start;
        if close > content_start {
            return Some((close + 1, ProtectedKind::Code));
        }
    }

    if let Some(end) = isbn_span_end(text, start) {
        return Some((end, ProtectedKind::Isbn));
    }

    let raw_end = text[start..]
        .char_indices()
        .find(|(_, character)| character.is_whitespace())
        .map_or(text.len(), |(offset, _)| start + offset);
    let (core_start, core_end) = trim_token_edges(text, start, raw_end);
    if core_start != start || core_end <= core_start {
        return None;
    }
    let token = &text[core_start..core_end];
    let kind = classify_technical_token(token)?;
    Some((core_end, kind))
}

fn is_token_boundary(text: &str, index: usize) -> bool {
    index == 0
        || text[..index]
            .chars()
            .next_back()
            .is_none_or(|character| character.is_whitespace() || is_opening_delimiter(character))
}

fn is_opening_delimiter(character: char) -> bool {
    matches!(
        character,
        '(' | '[' | '{' | '<' | '«' | '“' | '‘' | '\'' | '"'
    )
}

fn trim_token_edges(text: &str, start: usize, end: usize) -> (usize, usize) {
    let mut core_start = start;
    let mut core_end = end;

    while core_start < core_end {
        let character = text[core_start..core_end].chars().next().unwrap();
        if is_opening_delimiter(character) {
            core_start += character.len_utf8();
        } else {
            break;
        }
    }
    while core_start < core_end {
        let character = text[core_start..core_end].chars().next_back().unwrap();
        if matches!(
            character,
            ')' | ']' | '}' | '>' | '»' | '”' | '’' | ',' | ';' | ':' | '!' | '?' | '،' | '؛' | '؟'
        ) {
            core_end -= character.len_utf8();
        } else if character == '.' && !text[core_start..core_end - 1].ends_with('.') {
            core_end -= 1;
        } else {
            break;
        }
    }
    (core_start, core_end)
}

fn classify_technical_token(token: &str) -> Option<ProtectedKind> {
    if !token.is_ascii() {
        return None;
    }
    let lower = token.to_ascii_lowercase();

    if (lower.starts_with("https://") || lower.starts_with("http://") || lower.starts_with("www."))
        && token.contains('.')
    {
        return Some(ProtectedKind::Url);
    }
    if looks_like_email(token) {
        return Some(ProtectedKind::Email);
    }
    if looks_like_file_path(token) {
        return Some(ProtectedKind::FilePath);
    }
    if looks_like_identifier(token) {
        return Some(ProtectedKind::Identifier);
    }
    None
}

fn looks_like_email(token: &str) -> bool {
    let Some((local, domain)) = token.split_once('@') else {
        return false;
    };
    !local.is_empty()
        && !domain.is_empty()
        && !domain.contains('@')
        && domain.split('.').all(|part| !part.is_empty())
        && domain.contains('.')
}

fn looks_like_file_path(token: &str) -> bool {
    let has_ascii_alphanumeric = token
        .chars()
        .any(|character| character.is_ascii_alphanumeric());
    ((token.starts_with("./") || token.starts_with("../") || token.starts_with('/'))
        || token.contains('\\'))
        && has_ascii_alphanumeric
}

fn looks_like_identifier(token: &str) -> bool {
    let has_letter = token.bytes().any(|byte| byte.is_ascii_alphabetic());
    let has_digit = token.bytes().any(|byte| byte.is_ascii_digit());
    has_letter
        && has_digit
        && token
            .bytes()
            .any(|byte| matches!(byte, b'_' | b'-' | b':' | b'#'))
}

fn isbn_span_end(text: &str, start: usize) -> Option<usize> {
    let rest = &text[start..];
    if !rest
        .get(..4)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("isbn"))
    {
        return None;
    }
    let mut end = start + 4;
    while end < text.len() {
        let character = text[end..].chars().next()?;
        if character.is_ascii_digit() || matches!(character, '-' | ':' | ' ' | '\u{00a0}') {
            end += character.len_utf8();
        } else {
            break;
        }
    }
    while end > start + 4 {
        let character = text[start + 4..end].chars().next_back()?;
        if matches!(character, '-' | ':' | ' ' | '\u{00a0}') {
            end -= character.len_utf8();
        } else {
            break;
        }
    }
    let digits = text[start + 4..end]
        .bytes()
        .filter(|byte| byte.is_ascii_digit())
        .count();
    matches!(digits, 10 | 13).then_some(end)
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
        let edited = manuscript_to_book_ir(&build("Rewritten.\n\n***\n\nAlso rewritten.")).unwrap();

        assert_eq!(
            original
                .blocks
                .iter()
                .map(BookBlock::id)
                .collect::<Vec<_>>(),
            edited.blocks.iter().map(BookBlock::id).collect::<Vec<_>>()
        );
        assert_eq!(
            original
                .blocks
                .iter()
                .map(BookBlock::id)
                .collect::<Vec<_>>(),
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

    #[test]
    fn splits_high_confidence_tokens_without_changing_text() {
        let text = "می\u{200c}روم https://example.com، به editor@example.org؛ ISBN 978-1-4028-9462-6 و `fn main()` در ./src/main.rs با REF-42.";
        let runs = protected_runs(text, Some("en-US"));

        assert_eq!(
            runs.iter().map(|run| run.text.as_str()).collect::<String>(),
            text
        );
        assert_eq!(
            runs.iter()
                .filter_map(|run| match &run.protection {
                    RunProtection::Protected(kind) => Some(*kind),
                    RunProtection::Editable => None,
                })
                .collect::<Vec<_>>(),
            vec![
                ProtectedKind::Url,
                ProtectedKind::Email,
                ProtectedKind::Isbn,
                ProtectedKind::Code,
                ProtectedKind::FilePath,
                ProtectedKind::Identifier,
            ]
        );
        assert!(runs
            .iter()
            .filter(|run| matches!(&run.protection, RunProtection::Protected(_)))
            .all(|run| {
                run.direction == TextDirection::Ltr && run.lang.as_deref() == Some("en-US")
            }));
    }

    #[test]
    fn leaves_ordinary_english_prose_editable() {
        let text = "She crossed the quiet room and closed the door.";
        let runs = protected_runs(text, Some("en-US"));

        assert_eq!(runs, vec![editable_run(text.to_string(), Some("en-US"))]);
    }
}
