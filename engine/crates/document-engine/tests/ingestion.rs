#[macro_use]
extern crate lopdf;

use document_engine::{ingest_file, DocumentError, DocumentFormat};
use lopdf::content::{Content, Operation};
use lopdf::{Document, Object, Stream};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

fn zip_file(path: &Path, entries: &[(&str, &str)]) {
    let mut archive = ZipWriter::new(File::create(path).unwrap());
    for (name, content) in entries {
        archive
            .start_file(*name, SimpleFileOptions::default())
            .unwrap();
        archive.write_all(content.as_bytes()).unwrap();
    }
    archive.finish().unwrap();
}

#[test]
fn imports_txt_with_ordered_chapters_and_source_tracking() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("novel.txt");
    std::fs::write(
        &path,
        "Chapter 1\nFirst paragraph.\n\nSecond paragraph.\n\nChapter 2\nFinal paragraph.",
    )
    .unwrap();
    let manuscript = ingest_file(&path).unwrap();
    assert_eq!(manuscript.chapters.len(), 2);
    assert_eq!(
        manuscript
            .chapters
            .iter()
            .map(|chapter| chapter.order)
            .collect::<Vec<_>>(),
        vec![1, 2]
    );
    let paragraph = &manuscript.chapters[0].scenes[0].paragraphs[1];
    assert_eq!(paragraph.source.format, DocumentFormat::Txt);
    assert_eq!(paragraph.source.chapter, Some(1));
    assert_eq!(paragraph.source.paragraph, Some(2));
}

#[test]
fn imports_markdown_metadata_and_scenes() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("novel.md");
    std::fs::write(&path, "---\ntitle: Winter House\nauthor: Jane Example\nlanguage: en\n---\n\n## Chapter 1\n\nFirst.\n\n***\n\nSecond.").unwrap();
    let manuscript = ingest_file(&path).unwrap();
    assert_eq!(manuscript.book.title, "Winter House");
    assert_eq!(manuscript.book.author.as_deref(), Some("Jane Example"));
    assert_eq!(manuscript.book.language.as_deref(), Some("en"));
    assert_eq!(manuscript.chapters[0].scenes.len(), 2);
}

#[test]
fn imports_epub_spine_metadata_and_resource_locations() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("novel.epub");
    zip_file(
        &path,
        &[
            (
                "META-INF/container.xml",
                r#"<container><rootfiles><rootfile full-path="OEBPS/content.opf"/></rootfiles></container>"#,
            ),
            (
                "OEBPS/content.opf",
                r#"<package><metadata><dc:title>Moon Book</dc:title><dc:creator>M. Author</dc:creator><dc:language>en</dc:language></metadata><manifest><item id="c1" href="one.xhtml"/><item id="c2" href="two.xhtml"/></manifest><spine><itemref idref="c1"/><itemref idref="c2"/></spine></package>"#,
            ),
            (
                "OEBPS/one.xhtml",
                "<html><body><h1>Chapter 1</h1><p>Opening paragraph.</p></body></html>",
            ),
            (
                "OEBPS/two.xhtml",
                "<html><body><h1>Chapter 2</h1><p>Closing paragraph.</p></body></html>",
            ),
        ],
    );
    let manuscript = ingest_file(&path).unwrap();
    assert_eq!(manuscript.book.title, "Moon Book");
    assert_eq!(manuscript.book.author.as_deref(), Some("M. Author"));
    assert_eq!(manuscript.chapters.len(), 2);
    assert_eq!(
        manuscript.chapters[1].scenes[0].paragraphs[0]
            .source
            .resource
            .as_deref(),
        Some("OEBPS/two.xhtml")
    );
}

#[test]
fn imports_docx_core_metadata_heading_styles_and_paragraph_locations() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("novel.docx");
    zip_file(
        &path,
        &[
            (
                "docProps/core.xml",
                r#"<cp:coreProperties><dc:title>Glass City</dc:title><dc:creator>D. Writer</dc:creator><dc:language>en-US</dc:language></cp:coreProperties>"#,
            ),
            (
                "word/document.xml",
                r#"<w:document><w:body><w:p><w:pPr><w:pStyle w:val="Heading1"/></w:pPr><w:r><w:t>Chapter 1</w:t></w:r></w:p><w:p><w:r><w:t>DOCX paragraph.</w:t></w:r></w:p></w:body></w:document>"#,
            ),
        ],
    );
    let manuscript = ingest_file(&path).unwrap();
    assert_eq!(manuscript.book.title, "Glass City");
    assert_eq!(manuscript.book.author.as_deref(), Some("D. Writer"));
    let source = &manuscript.chapters[0].scenes[0].paragraphs[0].source;
    assert_eq!(source.resource.as_deref(), Some("word/document.xml"));
    assert_eq!(source.paragraph, Some(2));
}

fn create_pdf(path: &Path) {
    let mut doc = Document::with_version("1.5");
    let pages_id = doc.new_object_id();
    let font_id = doc.add_object(
        dictionary! { "Type" => "Font", "Subtype" => "Type1", "BaseFont" => "Courier" },
    );
    let resources_id = doc.add_object(dictionary! { "Font" => dictionary! { "F1" => font_id } });
    let content = Content {
        operations: vec![
            Operation::new("BT", vec![]),
            Operation::new("Tf", vec!["F1".into(), 14.into()]),
            Operation::new("Td", vec![50.into(), 700.into()]),
            Operation::new("Tj", vec![Object::string_literal("Chapter 1")]),
            Operation::new("Td", vec![0.into(), (-30).into()]),
            Operation::new("Tj", vec![Object::string_literal("A PDF paragraph.")]),
            Operation::new("ET", vec![]),
        ],
    };
    let content_id = doc.add_object(Stream::new(dictionary! {}, content.encode().unwrap()));
    let page_id = doc.add_object(
        dictionary! { "Type" => "Page", "Parent" => pages_id, "Contents" => content_id },
    );
    doc.objects.insert(pages_id, Object::Dictionary(dictionary! { "Type" => "Pages", "Kids" => vec![page_id.into()], "Count" => 1, "Resources" => resources_id, "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()] }));
    let catalog_id = doc.add_object(dictionary! { "Type" => "Catalog", "Pages" => pages_id });
    let info_id = doc.add_object(dictionary! { "Title" => Object::string_literal("PDF Novel"), "Author" => Object::string_literal("P. Author") });
    doc.trailer.set("Root", catalog_id);
    doc.trailer.set("Info", info_id);
    doc.save(path).unwrap();
}

#[test]
fn imports_pdf_text_metadata_and_page_locations() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("novel.pdf");
    create_pdf(&path);
    let manuscript = ingest_file(&path).unwrap();
    assert_eq!(manuscript.book.title, "PDF Novel");
    assert_eq!(manuscript.book.author.as_deref(), Some("P. Author"));
    assert_eq!(
        manuscript.translation_units().next().unwrap().source.page,
        Some(1)
    );
}

#[test]
fn rejects_empty_corrupted_and_unsupported_files_with_specific_errors() {
    let directory = tempfile::tempdir().unwrap();
    let empty = directory.path().join("empty.txt");
    std::fs::write(&empty, "  \n").unwrap();
    assert!(matches!(
        ingest_file(&empty),
        Err(DocumentError::EmptyDocument(_))
    ));
    let corrupted = directory.path().join("bad.epub");
    std::fs::write(&corrupted, "not a zip").unwrap();
    assert!(matches!(
        ingest_file(&corrupted),
        Err(DocumentError::CorruptedFile(_))
    ));
    let unsupported = directory.path().join("story.rtf");
    std::fs::write(&unsupported, "text").unwrap();
    assert!(matches!(
        ingest_file(&unsupported),
        Err(DocumentError::UnsupportedFormat(_))
    ));
}

#[test]
fn manuscript_json_round_trips_for_downstream_consumers() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("book.txt");
    std::fs::write(&path, "Chapter 1\nParagraph.").unwrap();
    let manuscript = ingest_file(&path).unwrap();
    let json = serde_json::to_string_pretty(&manuscript).unwrap();
    let restored: document_engine::Manuscript = serde_json::from_str(&json).unwrap();
    assert_eq!(restored, manuscript);
}
