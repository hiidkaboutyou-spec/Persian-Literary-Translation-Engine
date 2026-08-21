use std::fs::File;
use std::io::Read;
use std::path::Path;
use zip::ZipArchive;

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
        return Err(DocumentError::InvalidDocument(
            "DOCX contains no readable paragraph text".to_string(),
        ));
    }

    let title = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("Untitled")
        .to_string();

    Ok(load_document(title, text))
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
