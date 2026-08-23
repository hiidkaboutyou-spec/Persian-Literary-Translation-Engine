use crate::models::DocumentFormat;
use crate::parser::{
    base_source, file_stem_title, looks_like_chapter_heading, BlockKind, ParsedBlock,
    ParsedDocument,
};
use crate::DocumentError;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

pub(crate) fn parse_text(path: &Path, markdown: bool) -> Result<ParsedDocument, DocumentError> {
    let bytes = fs::read(path)?;
    let content = String::from_utf8(bytes).map_err(|error| {
        DocumentError::ParsingFailure(format!(
            "{} is not valid UTF-8 text: {error}",
            path.display()
        ))
    })?;
    if content.trim().is_empty() {
        return Err(DocumentError::EmptyDocument(path.to_path_buf()));
    }
    let format = if markdown {
        DocumentFormat::Markdown
    } else {
        DocumentFormat::Txt
    };
    let source = base_source(path, format);
    let (metadata, body) = if markdown {
        parse_frontmatter(&content)
    } else {
        (BTreeMap::new(), content.as_str())
    };
    let mut title = metadata
        .get("title")
        .cloned()
        .unwrap_or_else(|| file_stem_title(path));
    let author = metadata.get("author").cloned();
    let language = metadata.get("language").cloned();
    let mut blocks = Vec::new();
    let mut paragraph_lines = Vec::new();
    let mut first_heading = true;
    fn flush(
        lines: &mut Vec<String>,
        blocks: &mut Vec<ParsedBlock>,
        source: &crate::SourceLocation,
    ) {
        if lines.is_empty() {
            return;
        }
        let text = lines.join("\n").trim().to_string();
        lines.clear();
        if text.is_empty() {
            return;
        }
        let kind = if matches!(text.as_str(), "***" | "---" | "* * *") {
            BlockKind::SceneBreak
        } else if looks_like_chapter_heading(&text) {
            BlockKind::Heading(1)
        } else {
            BlockKind::Paragraph
        };
        blocks.push(ParsedBlock {
            kind,
            text,
            source: source.clone(),
        });
    }
    for line in body.lines() {
        let trimmed = line.trim();
        let heading = if markdown {
            let count = trimmed.chars().take_while(|c| *c == '#').count();
            ((1..=6).contains(&count) && trimmed[count..].starts_with(char::is_whitespace))
                .then(|| (count as u8, trimmed[count..].trim().to_string()))
        } else {
            None
        };
        if let Some((level, value)) = heading {
            flush(&mut paragraph_lines, &mut blocks, &source);
            if first_heading && level == 1 && !metadata.contains_key("title") {
                title = value;
                first_heading = false;
                continue;
            }
            first_heading = false;
            blocks.push(ParsedBlock {
                kind: BlockKind::Heading(level),
                text: value,
                source: source.clone(),
            });
        } else if trimmed.is_empty() {
            flush(&mut paragraph_lines, &mut blocks, &source);
        } else if looks_like_chapter_heading(trimmed) && paragraph_lines.is_empty() {
            blocks.push(ParsedBlock {
                kind: BlockKind::Heading(1),
                text: trimmed.to_string(),
                source: source.clone(),
            });
        } else {
            paragraph_lines.push(line.to_string());
        }
    }
    flush(&mut paragraph_lines, &mut blocks, &source);
    Ok(ParsedDocument {
        title,
        author,
        language,
        metadata,
        source,
        blocks,
    })
}

fn parse_frontmatter(content: &str) -> (BTreeMap<String, String>, &str) {
    let normalized = content.strip_prefix('\u{feff}').unwrap_or(content);
    let Some(rest) = normalized.strip_prefix("---\n") else {
        return (BTreeMap::new(), content);
    };
    let Some(end) = rest.find("\n---") else {
        return (BTreeMap::new(), content);
    };
    let mut metadata = BTreeMap::new();
    for line in rest[..end].lines() {
        if let Some((key, value)) = line.split_once(':') {
            let value = value.trim().trim_matches(['\'', '"']);
            if !key.trim().is_empty() && !value.is_empty() {
                metadata.insert(key.trim().to_ascii_lowercase(), value.to_string());
            }
        }
    }
    (metadata, rest[end + 4..].trim_start_matches(['\r', '\n']))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    #[test]
    fn extracts_markdown_metadata() {
        let mut file = tempfile::Builder::new().suffix(".md").tempfile().unwrap();
        write!(file, "---\ntitle: The House\nauthor: A. Writer\nlanguage: en\n---\n\n## Chapter 1\n\nFirst.\n\n***\n\nSecond.").unwrap();
        let parsed = parse_text(file.path(), true).unwrap();
        assert_eq!(parsed.title, "The House");
        assert_eq!(parsed.author.as_deref(), Some("A. Writer"));
        assert!(parsed
            .blocks
            .iter()
            .any(|block| block.kind == BlockKind::SceneBreak));
    }
}
