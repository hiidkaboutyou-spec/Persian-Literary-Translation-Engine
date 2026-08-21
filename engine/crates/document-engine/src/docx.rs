use quick_xml::events::Event;
use quick_xml::Reader;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use zip::ZipArchive;

fn xml_error(error: impl std::fmt::Display) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error.to_string())
}

pub fn extract_docx_text(path: impl AsRef<Path>) -> io::Result<String> {
    let file = File::open(path)?;
    let mut archive = ZipArchive::new(file).map_err(xml_error)?;
    let mut document_xml = String::new();
    archive
        .by_name("word/document.xml")
        .map_err(xml_error)?
        .read_to_string(&mut document_xml)?;

    extract_text_from_document_xml(&document_xml)
}

pub(crate) fn extract_text_from_document_xml(xml: &str) -> io::Result<String> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);

    let mut output = String::new();
    let mut in_text = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(event)) => match event.local_name().as_ref() {
                b"t" => in_text = true,
                b"tab" => output.push('\t'),
                b"br" | b"cr" => output.push('\n'),
                _ => {}
            },
            Ok(Event::Empty(event)) => match event.local_name().as_ref() {
                b"tab" => output.push('\t'),
                b"br" | b"cr" => output.push('\n'),
                _ => {}
            },
            Ok(Event::Text(text)) if in_text => {
                let decoded = text.decode().map_err(xml_error)?;
                output.push_str(&decoded);
            }
            Ok(Event::End(event)) => match event.local_name().as_ref() {
                b"t" => in_text = false,
                b"p" => {
                    if !output.ends_with('\n') {
                        output.push('\n');
                    }
                }
                _ => {}
            },
            Ok(Event::Eof) => break,
            Err(error) => return Err(xml_error(error)),
            _ => {}
        }
    }

    Ok(output.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_paragraphs_tabs_and_breaks() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
            <w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
              <w:body>
                <w:p><w:r><w:t>Chapter 1</w:t></w:r></w:p>
                <w:p><w:r><w:t>Hello</w:t><w:tab/><w:t>world</w:t><w:br/><w:t>again</w:t></w:r></w:p>
              </w:body>
            </w:document>"#;

        let text = extract_text_from_document_xml(xml).unwrap();
        assert_eq!(text, "Chapter 1\nHello\tworld\nagain");
    }

    #[test]
    fn preserves_persian_text() {
        let xml = r#"<w:document xmlns:w="x"><w:body><w:p><w:r><w:t>فصل ۱</w:t></w:r></w:p><w:p><w:r><w:t>سلام دنیا</w:t></w:r></w:p></w:body></w:document>"#;
        let text = extract_text_from_document_xml(xml).unwrap();
        assert_eq!(text, "فصل ۱\nسلام دنیا");
    }
}
