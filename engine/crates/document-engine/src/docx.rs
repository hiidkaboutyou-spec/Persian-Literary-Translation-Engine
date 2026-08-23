use std::fs::File;
use std::io::Read;
use std::path::Path;
use zip::ZipArchive;

use crate::models::DocumentFormat;
use crate::parser::{base_source, file_stem_title, BlockKind, ParsedBlock, ParsedDocument};
use crate::{load_document, Document, DocumentError};

pub fn load_docx_file(path: impl AsRef<Path>) -> Result<Document, DocumentError> {
    let path = path.as_ref();
    let file = File::open(path)?;
    let mut archive = ZipArchive::new(file).map_err(DocumentError::Zip)?;
    let mut document_xml = String::new();

    archive
        .by_name("word/document.xml")
        .map_err(DocumentError::Zip)?
        .read_to_string(&mut document_xml)?;

    let text = extract_docx_text(&document_xml);
    if text.trim().is_empty() {
        return Err(DocumentError::EmptyDocument(path.to_path_buf()));
    }

    let title = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("Untitled")
        .to_string();

    Ok(load_document(title, text))
}

pub(crate) fn parse_docx(path: &Path) -> Result<ParsedDocument, DocumentError> {
    let file = File::open(path)?;
    let mut archive = ZipArchive::new(file).map_err(|error| {
        DocumentError::CorruptedFile(format!(
            "{} is not a readable DOCX archive: {error}",
            path.display()
        ))
    })?;
    let mut xml = String::new();
    archive
        .by_name("word/document.xml")
        .map_err(|error| {
            DocumentError::CorruptedFile(format!("DOCX is missing word/document.xml: {error}"))
        })?
        .read_to_string(&mut xml)?;
    let core = archive
        .by_name("docProps/core.xml")
        .ok()
        .and_then(|mut entry| {
            let mut value = String::new();
            entry.read_to_string(&mut value).ok().map(|_| value)
        });
    let title = core
        .as_deref()
        .and_then(|value| extract_element(value, "dc:title"))
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| file_stem_title(path));
    let author = core
        .as_deref()
        .and_then(|value| extract_element(value, "dc:creator"));
    let language = core
        .as_deref()
        .and_then(|value| extract_element(value, "dc:language"));
    let base = base_source(path, DocumentFormat::Docx);
    let mut blocks = Vec::new();
    for (index, paragraph_xml) in collect_elements(&xml, "w:p").into_iter().enumerate() {
        let text = extract_docx_text(paragraph_xml);
        if text.trim().is_empty() {
            continue;
        }
        let style = find_attribute_value(paragraph_xml, "w:pStyle", "w:val");
        let kind = if style
            .as_deref()
            .is_some_and(|value| value.to_ascii_lowercase().starts_with("heading"))
        {
            BlockKind::Heading(1)
        } else if matches!(text.trim(), "***" | "---" | "* * *") {
            BlockKind::SceneBreak
        } else {
            BlockKind::Paragraph
        };
        let mut source = base.clone();
        source.resource = Some("word/document.xml".into());
        source.paragraph = Some(index + 1);
        blocks.push(ParsedBlock { kind, text, source });
    }
    if blocks.is_empty() {
        return Err(DocumentError::EmptyDocument(path.to_path_buf()));
    }
    let mut metadata = std::collections::BTreeMap::new();
    if let Some(value) = &author {
        metadata.insert("author".into(), value.clone());
    }
    if let Some(value) = &language {
        metadata.insert("language".into(), value.clone());
    }
    Ok(ParsedDocument {
        title,
        author,
        language,
        metadata,
        source: base,
        blocks,
    })
}

fn collect_elements<'a>(xml: &'a str, tag: &str) -> Vec<&'a str> {
    let open = format!("<{tag}");
    let close = format!("</{tag}>");
    let mut output = Vec::new();
    let mut cursor = 0;
    while let Some(start_rel) = xml[cursor..].find(&open) {
        let start = cursor + start_rel;
        let Some(end_rel) = xml[start..].find(&close) else {
            break;
        };
        let end = start + end_rel + close.len();
        output.push(&xml[start..end]);
        cursor = end;
    }
    output
}

fn extract_element(xml: &str, name: &str) -> Option<String> {
    let open = format!("<{name}");
    let start = xml.find(&open)?;
    let body = start + xml[start..].find('>')? + 1;
    let close = format!("</{name}>");
    let end = body + xml[body..].find(&close)?;
    Some(decode_xml_entities(xml[body..end].trim()))
}

fn find_attribute_value(xml: &str, tag: &str, attribute: &str) -> Option<String> {
    let start = xml.find(&format!("<{tag}"))?;
    let end = start + xml[start..].find('>')?;
    let tag = &xml[start..=end];
    for quote in ['"', '\''] {
        let pattern = format!("{attribute}={quote}");
        if let Some(pos) = tag.find(&pattern) {
            let value = &tag[pos + pattern.len()..];
            return Some(value[..value.find(quote)?].to_string());
        }
    }
    None
}

pub(crate) fn extract_docx_text(xml: &str) -> String {
    let mut output = String::new();
    let mut cursor = 0;

    while let Some(relative_start) = xml[cursor..].find('<') {
        let start = cursor + relative_start;
        let Some(relative_end) = xml[start..].find('>') else {
            break;
        };
        let end = start + relative_end;
        let tag = &xml[start + 1..end];

        if tag == "w:t" || tag.starts_with("w:t ") {
            let text_start = end + 1;
            if let Some(close_rel) = xml[text_start..].find("</w:t>") {
                let text_end = text_start + close_rel;
                output.push_str(&decode_xml_entities(&xml[text_start..text_end]));
                cursor = text_end + "</w:t>".len();
                continue;
            }
        } else if tag == "/w:p" {
            if !output.ends_with('\n') {
                output.push('\n');
            }
        } else if tag == "w:tab/" || tag == "w:tab /" {
            output.push('\t');
        } else if tag == "w:br/" || tag == "w:br /" || tag.starts_with("w:br ") {
            output.push('\n');
        }

        cursor = end + 1;
    }

    output.trim().to_string()
}

fn decode_xml_entities(input: &str) -> String {
    input
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}

#[cfg(test)]
mod tests {
    use super::extract_docx_text;

    #[test]
    fn extracts_paragraphs_and_inline_runs() {
        let xml = r#"<w:document><w:body><w:p><w:r><w:t>Hello </w:t></w:r><w:r><w:t>world</w:t></w:r></w:p><w:p><w:r><w:t>Second line</w:t></w:r></w:p></w:body></w:document>"#;
        assert_eq!(extract_docx_text(xml), "Hello world\nSecond line");
    }

    #[test]
    fn decodes_common_xml_entities() {
        let xml = r#"<w:p><w:r><w:t>Tom &amp; Jerry &lt;3</w:t></w:r></w:p>"#;
        assert_eq!(extract_docx_text(xml), "Tom & Jerry <3");
    }

    #[test]
    fn preserves_tabs_and_line_breaks() {
        let xml = r#"<w:p><w:r><w:t>A</w:t></w:r><w:tab/><w:r><w:t>B</w:t></w:r><w:br/><w:r><w:t>C</w:t></w:r></w:p>"#;
        assert_eq!(extract_docx_text(xml), "A\tB\nC");
    }

    #[test]
    fn accepts_text_nodes_with_xml_space_attribute() {
        let xml = r#"<w:p><w:r><w:t xml:space="preserve"> spaced </w:t></w:r></w:p>"#;
        assert_eq!(extract_docx_text(xml), "spaced");
    }

    #[test]
    fn does_not_confuse_table_tags_with_text_nodes() {
        let xml = r#"<w:tbl><w:tr><w:tc><w:p><w:r><w:t>Cell text</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#;
        assert_eq!(extract_docx_text(xml), "Cell text");
    }
}
