use std::fs::File;
use std::io::Write;
use std::path::Path;

use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

use crate::{Chapter, DocumentError};

pub fn export_persian_docx(
    path: impl AsRef<Path>,
    title: &str,
    chapters: &[Chapter],
) -> Result<(), DocumentError> {
    if chapters.is_empty() {
        return Err(DocumentError::InvalidStructure(
            "cannot export a DOCX without chapters".to_string(),
        ));
    }

    let file = File::create(path)?;
    let mut archive = ZipWriter::new(file);
    let stored = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
    let deflated = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

    write_part(&mut archive, "[Content_Types].xml", content_types(), stored)?;
    write_part(
        &mut archive,
        "_rels/.rels",
        package_relationships(),
        deflated,
    )?;
    write_part(
        &mut archive,
        "docProps/core.xml",
        &core_properties(title),
        deflated,
    )?;
    write_part(&mut archive, "word/styles.xml", styles_xml(), deflated)?;
    write_part(
        &mut archive,
        "word/_rels/document.xml.rels",
        document_relationships(),
        deflated,
    )?;
    write_part(
        &mut archive,
        "word/document.xml",
        &document_xml(title, chapters),
        deflated,
    )?;

    archive.finish().map_err(DocumentError::Zip)?;
    Ok(())
}

fn write_part(
    archive: &mut ZipWriter<File>,
    name: &str,
    content: &str,
    options: SimpleFileOptions,
) -> Result<(), DocumentError> {
    archive
        .start_file(name, options)
        .map_err(DocumentError::Zip)?;
    archive.write_all(content.as_bytes())?;
    Ok(())
}

fn content_types() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
  <Override PartName="/word/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml"/>
  <Override PartName="/docProps/core.xml" ContentType="application/vnd.openxmlformats-package.core-properties+xml"/>
</Types>"#
}

fn package_relationships() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
  <Relationship Id="rId2" Type="http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties" Target="docProps/core.xml"/>
</Relationships>"#
}

fn document_relationships() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>
</Relationships>"#
}

fn core_properties(title: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:dcterms="http://purl.org/dc/terms/" xmlns:dcmitype="http://purl.org/dc/dcmitype/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
  <dc:title>{}</dc:title>
  <dc:creator>Persian Literary Translation Engine</dc:creator>
  <dc:language>fa-IR</dc:language>
</cp:coreProperties>"#,
        escape_xml(title)
    )
}

fn styles_xml() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:docDefaults>
    <w:rPrDefault><w:rPr><w:rFonts w:ascii="Tahoma" w:hAnsi="Tahoma" w:cs="Tahoma"/><w:lang w:val="fa-IR" w:bidi="fa-IR"/><w:sz w:val="24"/><w:szCs w:val="24"/><w:rtl/></w:rPr></w:rPrDefault>
    <w:pPrDefault><w:pPr><w:bidi/><w:jc w:val="both"/><w:spacing w:after="160" w:line="360" w:lineRule="auto"/><w:ind w:firstLine="567"/></w:pPr></w:pPrDefault>
  </w:docDefaults>
  <w:style w:type="paragraph" w:default="1" w:styleId="Normal">
    <w:name w:val="Normal"/><w:qFormat/>
    <w:pPr><w:bidi/><w:jc w:val="both"/><w:spacing w:after="160" w:line="360" w:lineRule="auto"/><w:ind w:firstLine="567"/></w:pPr>
    <w:rPr><w:rFonts w:ascii="Tahoma" w:hAnsi="Tahoma" w:cs="Tahoma"/><w:lang w:val="fa-IR" w:bidi="fa-IR"/><w:sz w:val="24"/><w:szCs w:val="24"/><w:rtl/></w:rPr>
  </w:style>
  <w:style w:type="paragraph" w:styleId="Title">
    <w:name w:val="Title"/><w:basedOn w:val="Normal"/><w:qFormat/>
    <w:pPr><w:bidi/><w:jc w:val="center"/><w:spacing w:before="1200" w:after="480"/><w:ind w:firstLine="0"/></w:pPr>
    <w:rPr><w:b/><w:bCs/><w:sz w:val="44"/><w:szCs w:val="44"/><w:rtl/></w:rPr>
  </w:style>
  <w:style w:type="paragraph" w:styleId="Heading1">
    <w:name w:val="heading 1"/><w:basedOn w:val="Normal"/><w:qFormat/><w:uiPriority w:val="9"/>
    <w:pPr><w:bidi/><w:jc w:val="right"/><w:pageBreakBefore/><w:keepNext/><w:spacing w:before="360" w:after="360"/><w:ind w:firstLine="0"/></w:pPr>
    <w:rPr><w:b/><w:bCs/><w:sz w:val="36"/><w:szCs w:val="36"/><w:rtl/></w:rPr>
  </w:style>
</w:styles>"#
}

fn document_xml(title: &str, chapters: &[Chapter]) -> String {
    let mut body = String::new();
    body.push_str(&paragraph_xml(title, "Title"));

    for chapter in chapters {
        body.push_str(&paragraph_xml(&chapter.title, "Heading1"));
        for paragraph in split_paragraphs(&chapter.content) {
            body.push_str(&paragraph_xml(paragraph, "Normal"));
        }
    }

    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>{body}<w:sectPr><w:pgSz w:w="11906" w:h="16838"/><w:pgMar w:top="1134" w:right="1276" w:bottom="1134" w:left="1276" w:header="708" w:footer="708" w:gutter="0"/><w:cols w:space="708"/><w:docGrid w:linePitch="360"/></w:sectPr></w:body>
</w:document>"#
    )
}

fn paragraph_xml(text: &str, style: &str) -> String {
    format!(
        "<w:p><w:pPr><w:pStyle w:val=\"{}\"/><w:bidi/></w:pPr><w:r><w:rPr><w:rtl/><w:lang w:val=\"fa-IR\" w:bidi=\"fa-IR\"/></w:rPr><w:t xml:space=\"preserve\">{}</w:t></w:r></w:p>",
        escape_xml(style),
        escape_xml(text)
    )
}

fn split_paragraphs(text: &str) -> Vec<&str> {
    let mut paragraphs = Vec::new();
    let mut start = 0usize;
    let bytes = text.as_bytes();
    let mut cursor = 0usize;

    while cursor + 1 < bytes.len() {
        if bytes[cursor] == b'\n' && bytes[cursor + 1] == b'\n' {
            let paragraph = text[start..cursor].trim();
            if !paragraph.is_empty() {
                paragraphs.push(paragraph);
            }
            cursor += 2;
            while cursor < bytes.len() && bytes[cursor] == b'\n' {
                cursor += 1;
            }
            start = cursor;
        } else {
            cursor += 1;
        }
    }

    let paragraph = text[start..].trim();
    if !paragraph.is_empty() {
        paragraphs.push(paragraph);
    }
    paragraphs
}

fn escape_xml(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::load_docx_file;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_docx() -> std::path::PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("persian-literary-export-{nonce}.docx"))
    }

    #[test]
    fn exported_docx_round_trips_persian_chapters() {
        let path = temp_docx();
        let chapters = vec![
            Chapter::translated(0, "فصل ۱", "این پاراگراف اول است.\n\nاین پاراگراف دوم است."),
            Chapter::translated(1, "فصل ۲", "پایان داستان."),
        ];

        export_persian_docx(&path, "رمان آزمایشی", &chapters).unwrap();
        let loaded = load_docx_file(&path).unwrap();
        fs::remove_file(path).ok();

        assert!(loaded.text.contains("رمان آزمایشی"));
        assert!(loaded.text.contains("فصل ۱"));
        assert!(loaded.text.contains("این پاراگراف دوم است"));
        assert!(loaded.text.contains("پایان داستان"));
    }

    #[test]
    fn escapes_xml_sensitive_characters() {
        assert_eq!(escape_xml("A & <B>"), "A &amp; &lt;B&gt;");
    }

    #[test]
    fn rejects_empty_manuscript() {
        let path = temp_docx();
        let error = export_persian_docx(&path, "خالی", &[]).unwrap_err();
        assert!(error.to_string().contains("without chapters"));
        fs::remove_file(path).ok();
    }
}
