use std::fs::File;
use std::io::Write;
use std::path::Path;

use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

use crate::{BookBlock, BookIr, Chapter, DocumentError, InlineRun, RunProtection, TextDirection};

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

    write_docx_package(path, title, &document_xml(title, chapters))
}

/// Export the canonical Book IR to a Persian DOCX without re-inferring inline direction.
///
/// Protected or explicitly LTR runs are represented with `w:rtl w:val="0"` inside
/// RTL paragraphs. Editable runs default to RTL unless their direction is explicitly
/// LTR. This keeps technical tokens stable while preserving the existing Persian
/// paragraph layout.
pub fn export_book_ir_persian_docx(
    path: impl AsRef<Path>,
    book: &BookIr,
) -> Result<(), DocumentError> {
    book.validate()
        .map_err(|error| DocumentError::InvalidStructure(format!("invalid Book IR: {error:?}")))?;
    if book.blocks.is_empty() {
        return Err(DocumentError::InvalidStructure(
            "cannot export a DOCX without Book IR blocks".to_string(),
        ));
    }

    write_docx_package(path, &book.title, &book_ir_document_xml(book))
}

fn write_docx_package(
    path: impl AsRef<Path>,
    title: &str,
    document: &str,
) -> Result<(), DocumentError> {
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
    write_part(&mut archive, "word/document.xml", document, deflated)?;

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

fn book_ir_document_xml(book: &BookIr) -> String {
    let mut body = String::new();
    body.push_str(&paragraph_xml(&book.title, "Title"));

    for block in &book.blocks {
        match block {
            BookBlock::ChapterHeading(heading) => {
                body.push_str(&inline_paragraph_xml(&heading.runs, "Heading1"));
            }
            BookBlock::Paragraph(paragraph) => {
                body.push_str(&inline_paragraph_xml(&paragraph.runs, "Normal"));
            }
            BookBlock::SceneBreak(scene_break) => {
                body.push_str(&paragraph_xml(
                    scene_break.marker.as_deref().unwrap_or("***"),
                    "Normal",
                ));
            }
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
    inline_paragraph_xml(&legacy_directional_runs(text), style)
}

/// Recover display direction at the legacy flattened-chapter boundary.
///
/// This runs only after translation, while serializing DOCX. It does not mark
/// text as protected and cannot influence translation or Book IR authority.
/// New code should carry explicit direction through Book IR instead.
fn legacy_directional_runs(text: &str) -> Vec<InlineRun> {
    let mut runs = Vec::new();
    let mut start = 0;
    let mut current = None;

    for (index, character) in text.char_indices() {
        let ascii_graphic = character.is_ascii_graphic();
        if let Some(previous) = current {
            if previous != ascii_graphic {
                runs.push(legacy_directional_run(&text[start..index], previous));
                start = index;
            }
        }
        current = Some(ascii_graphic);
    }
    if let Some(ascii_graphic) = current {
        runs.push(legacy_directional_run(&text[start..], ascii_graphic));
    }
    runs
}

fn legacy_directional_run(text: &str, ascii_graphic: bool) -> InlineRun {
    let ltr = ascii_graphic && text.bytes().any(|byte| byte.is_ascii_alphanumeric());
    InlineRun {
        text: text.to_string(),
        direction: if ltr {
            TextDirection::Ltr
        } else {
            TextDirection::Rtl
        },
        protection: RunProtection::Editable,
        lang: Some(if ltr { "en-US" } else { "fa-IR" }.to_string()),
    }
}

fn inline_paragraph_xml(runs: &[InlineRun], style: &str) -> String {
    let mut xml = format!(
        "<w:p><w:pPr><w:pStyle w:val=\"{}\"/><w:bidi/></w:pPr>",
        escape_xml(style)
    );
    for run in runs {
        let ltr = run.direction == TextDirection::Ltr
            || matches!(&run.protection, RunProtection::Protected(_));
        let language = run
            .lang
            .as_deref()
            .unwrap_or(if ltr { "en-US" } else { "fa-IR" });
        let properties = if ltr {
            format!(
                "<w:rtl w:val=\"0\"/><w:lang w:val=\"{}\"/>",
                escape_xml(language)
            )
        } else {
            format!(
                "<w:rtl/><w:lang w:val=\"{}\" w:bidi=\"fa-IR\"/>",
                escape_xml(language)
            )
        };
        xml.push_str(&format!(
            "<w:r><w:rPr>{properties}</w:rPr><w:t xml:space=\"preserve\">{}</w:t></w:r>",
            escape_xml(&run.text)
        ));
    }
    xml.push_str("</w:p>");
    xml
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
    use crate::{HeadingBlock, ParagraphBlock, ProtectedKind, SceneBreakBlock};
    use std::fs;
    use std::io::Read;
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
    fn legacy_export_preserves_mixed_script_direction_and_exact_text() {
        let path = temp_docx();
        let original =
            "به OpenAI.com و user@example.org نگاه کن؛ ISBN 978-1-4028-9462-6. می\u{200c}رود.";
        let chapters = vec![Chapter::translated(0, "فصل ۱", original)];

        export_persian_docx(&path, "نمونه", &chapters).unwrap();

        let file = File::open(&path).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();
        let mut document = String::new();
        archive
            .by_name("word/document.xml")
            .unwrap()
            .read_to_string(&mut document)
            .unwrap();
        drop(archive);
        let loaded = load_docx_file(&path).unwrap();
        fs::remove_file(path).ok();

        assert!(document.contains(
            "<w:rtl w:val=\"0\"/><w:lang w:val=\"en-US\"/></w:rPr><w:t xml:space=\"preserve\">OpenAI.com"
        ));
        assert!(document.contains(
            "<w:rtl w:val=\"0\"/><w:lang w:val=\"en-US\"/></w:rPr><w:t xml:space=\"preserve\">user@example.org"
        ));
        assert!(document.contains("می\u{200c}رود"));
        assert!(loaded.text.contains(original));
    }

    #[test]
    fn legacy_direction_inference_is_display_only_and_byte_preserving() {
        let original = "«سلام» A&B <tag> کتاب\u{200c}ها ۱۲۳";
        let runs = legacy_directional_runs(original);

        assert_eq!(
            runs.iter().map(|run| run.text.as_str()).collect::<String>(),
            original
        );
        assert!(runs
            .iter()
            .all(|run| run.protection == RunProtection::Editable));
        assert!(runs
            .iter()
            .any(|run| run.text == "A&B" && run.direction == TextDirection::Ltr));
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

    #[test]
    fn book_ir_export_preserves_protected_ltr_runs_inside_rtl_paragraphs() {
        let path = temp_docx();
        let mut book = BookIr::new("book-1", "رمان آزمایشی");
        book.blocks.push(BookBlock::ChapterHeading(HeadingBlock {
            id: "heading-1".to_string(),
            level: 1,
            runs: vec![InlineRun::persian("فصل یک")],
        }));
        book.blocks.push(BookBlock::Paragraph(ParagraphBlock {
            id: "paragraph-1".to_string(),
            runs: vec![
                InlineRun::persian("می\u{200c}روم به "),
                InlineRun::protected_ltr("https://example.com", ProtectedKind::Url),
                InlineRun::persian("؛ تمام."),
            ],
        }));
        book.blocks.push(BookBlock::SceneBreak(SceneBreakBlock {
            id: "break-1".to_string(),
            marker: None,
        }));

        export_book_ir_persian_docx(&path, &book).unwrap();

        let file = File::open(&path).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();
        let mut document = String::new();
        archive
            .by_name("word/document.xml")
            .unwrap()
            .read_to_string(&mut document)
            .unwrap();
        drop(archive);
        let loaded = load_docx_file(&path).unwrap();
        fs::remove_file(path).ok();

        assert!(document.contains("<w:pPr><w:pStyle w:val=\"Normal\"/><w:bidi/></w:pPr>"));
        assert!(document.contains(
            "<w:rtl w:val=\"0\"/><w:lang w:val=\"en-US\"/></w:rPr><w:t xml:space=\"preserve\">https://example.com"
        ));
        assert!(document.contains("می\u{200c}روم به "));
        assert!(loaded
            .text
            .contains("می\u{200c}روم به https://example.com؛ تمام."));
        assert!(loaded.text.contains("***"));
    }

    #[test]
    fn book_ir_export_rejects_invalid_ir_before_writing() {
        let path = temp_docx();
        let mut book = BookIr::new("book-1", "نمونه");
        for _ in 0..2 {
            book.blocks.push(BookBlock::Paragraph(ParagraphBlock {
                id: "duplicate".to_string(),
                runs: vec![InlineRun::persian("متن")],
            }));
        }

        let error = export_book_ir_persian_docx(&path, &book).unwrap_err();

        assert!(error.to_string().contains("DuplicateId"));
        assert!(!path.exists());
    }
}
