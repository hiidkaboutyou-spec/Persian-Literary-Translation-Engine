#![cfg(feature = "bookforge-epub")]

use document_engine::{export_translated_epub, ingest_file, EpubBlockTranslation};
use rbook::Epub;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use tempfile::tempdir;
use zip::{write::SimpleFileOptions, CompressionMethod, ZipWriter};

fn write_rights_safe_fixture(path: &Path) {
    let file = File::create(path).expect("fixture file");
    let mut zip = ZipWriter::new(file);
    let stored = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
    let deflated = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

    zip.start_file("mimetype", stored).unwrap();
    zip.write_all(b"application/epub+zip").unwrap();

    for (name, body) in [
        (
            "META-INF/container.xml",
            r#"<?xml version="1.0" encoding="UTF-8"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles><rootfile full-path="EPUB/package.opf" media-type="application/oebps-package+xml"/></rootfiles>
</container>"#,
        ),
        (
            "EPUB/package.opf",
            r#"<?xml version="1.0" encoding="UTF-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="book-id" xml:lang="en">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:identifier id="book-id">urn:uuid:00000000-0000-4000-8000-000000000024</dc:identifier>
    <dc:title>Phase 24 Differential Fixture</dc:title>
    <dc:language>en</dc:language>
    <meta property="dcterms:modified">2026-09-18T00:00:00Z</meta>
  </metadata>
  <manifest>
    <item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/>
    <item id="chapter" href="chapter.xhtml" media-type="application/xhtml+xml"/>
    <item id="css" href="style.css" media-type="text/css"/>
  </manifest>
  <spine><itemref idref="chapter"/></spine>
</package>"#,
        ),
        (
            "EPUB/nav.xhtml",
            r#"<?xml version="1.0" encoding="UTF-8"?>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops" lang="en" xml:lang="en">
  <head><title>Contents</title></head>
  <body><nav epub:type="toc"><ol><li><a href="chapter.xhtml">Chapter One</a></li></ol></nav></body>
</html>"#,
        ),
        (
            "EPUB/chapter.xhtml",
            r#"<?xml version="1.0" encoding="UTF-8"?>
<html xmlns="http://www.w3.org/1999/xhtml" lang="en" xml:lang="en">
  <head><title>Chapter One</title><link rel="stylesheet" type="text/css" href="style.css"/></head>
  <body><section><h1>Chapter One</h1><p>The first promise remained.</p><p>The second promise changed the scene.</p></section></body>
</html>"#,
        ),
        ("EPUB/style.css", "body { margin: 1em; }"),
    ] {
        zip.start_file(name, deflated).unwrap();
        zip.write_all(body.as_bytes()).unwrap();
    }
    zip.finish().unwrap();
}

#[test]
fn exported_persian_epub_reopens_in_independent_rbook_parser() {
    let root = tempdir().unwrap();
    let source = root.path().join("source.epub");
    let output = root.path().join("translated.epub");
    write_rights_safe_fixture(&source);

    let manuscript = ingest_file(&source).expect("BookForge-backed source ingestion");
    let translations = manuscript
        .translation_units()
        .filter_map(|paragraph| {
            paragraph.source.block_id.as_ref().map(|block_id| {
                EpubBlockTranslation::new(
                    block_id.clone(),
                    format!("ترجمهٔ آزمایشی بند {}", paragraph.position),
                )
            })
        })
        .collect::<Vec<_>>();
    assert!(
        !translations.is_empty(),
        "fixture must expose stable EPUB block provenance"
    );

    export_translated_epub(&source, &output, "fa", &translations)
        .expect("source-aware Persian EPUB export");

    // rbook is deliberately independent from BookForge and is a dev-only
    // dependency. A successful strict reopen catches a class of package/spine/
    // resource defects that a writer validating its own output may share.
    let epub = Epub::options()
        .strict(true)
        .open(&output)
        .expect("rbook strict parser should independently reopen exported EPUB");

    let mut reader = epub.reader();
    let mut readable = Vec::new();
    while let Some(item) = reader.read_next() {
        readable.push(item.expect("rbook should read every spine item").content().to_owned());
    }
    assert!(!readable.is_empty(), "exported EPUB must expose readable spine content");

    let joined = readable.join("\n");
    assert!(joined.contains("dir=\"rtl\""));
    assert!(joined.contains("lang=\"fa\""));
    assert!(joined.contains("ترجمهٔ آزمایشی"));
}
