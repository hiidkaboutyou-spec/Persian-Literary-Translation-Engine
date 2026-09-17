use std::fs::File;
use std::io::Read;
use std::path::Path;
use zip::ZipArchive;

#[cfg(feature = "bookforge-epub")]
use std::collections::HashMap;

#[cfg(feature = "bookforge-epub")]
use bookforge_core::{
    ir::{Block as BookForgeBlock, BlockKind as BookForgeBlockKind},
    BookforgeError,
};

use crate::models::DocumentFormat;
use crate::parser::{
    base_source, file_stem_title as parser_file_stem_title, BlockKind, ParsedBlock, ParsedDocument,
};
use crate::{load_document, Document, DocumentError};

#[cfg(feature = "bookforge-epub")]
const BOOKFORGE_REVISION: &str = "23f8c9d3c97a06f48e13424698441bfb4b037844";

pub fn load_epub_file(path: impl AsRef<Path>) -> Result<Document, DocumentError> {
    let path = path.as_ref();
    let file = File::open(path)?;
    let mut archive = ZipArchive::new(file).map_err(DocumentError::Zip)?;

    let container_xml = read_zip_entry(&mut archive, "META-INF/container.xml")?;
    let opf_path = extract_rootfile_path(&container_xml).ok_or_else(|| {
        DocumentError::InvalidStructure("EPUB container has no rootfile".to_string())
    })?;
    let opf_xml = read_zip_entry(&mut archive, &opf_path)?;
    let base_dir = Path::new(&opf_path)
        .parent()
        .unwrap_or_else(|| Path::new(""))
        .to_path_buf();

    let title = extract_metadata_title(&opf_xml)
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| file_stem_title(path));

    let manifest = extract_manifest_items(&opf_xml);
    let spine = extract_spine_ids(&opf_xml);
    if spine.is_empty() {
        return Err(DocumentError::InvalidStructure(
            "EPUB package has an empty spine".to_string(),
        ));
    }

    let mut sections = Vec::new();
    for idref in spine {
        let Some(href) = manifest
            .iter()
            .find(|item| item.id == idref)
            .map(|item| item.href.as_str())
        else {
            continue;
        };

        let entry_path = normalize_archive_path(&base_dir, href);
        let html = read_zip_entry(&mut archive, &entry_path)?;
        let text = extract_html_text(&html);
        if !text.trim().is_empty() {
            sections.push(text);
        }
    }

    let text = sections.join("\n\n");
    if text.trim().is_empty() {
        return Err(DocumentError::EmptyDocument(path.to_path_buf()));
    }

    let mut document = load_document(&text, DocumentFormat::Epub, path)?;
    document.title = title;
    Ok(document)
}

#[derive(Debug, Clone)]
struct ManifestItem {
    id: String,
    href: String,
}

fn read_zip_entry<R: Read + std::io::Seek>(
    archive: &mut ZipArchive<R>,
    name: &str,
) -> Result<String, DocumentError> {
    let mut file = archive.by_name(name).map_err(DocumentError::Zip)?;
    let mut text = String::new();
    file.read_to_string(&mut text)?;
    Ok(text)
}

fn extract_rootfile_path(xml: &str) -> Option<String> {
    extract_attribute_from_tag(xml, "rootfile", "full-path")
}

fn extract_metadata_title(xml: &str) -> Option<String> {
    extract_tag_text(xml, "dc:title")
}

fn extract_manifest_items(xml: &str) -> Vec<ManifestItem> {
    extract_tags(xml, "item")
        .into_iter()
        .filter_map(|tag| {
            let id = extract_attribute(tag, "id")?;
            let href = extract_attribute(tag, "href")?;
            Some(ManifestItem { id, href })
        })
        .collect()
}

fn extract_spine_ids(xml: &str) -> Vec<String> {
    extract_tags(xml, "itemref")
        .into_iter()
        .filter_map(|tag| extract_attribute(tag, "idref"))
        .collect()
}

fn extract_html_text(html: &str) -> String {
    let mut text = String::new();
    let mut in_tag = false;
    let mut tag_buf = String::new();
    let mut skip_depth = 0usize;

    for ch in html.chars() {
        if ch == '<' {
            in_tag = true;
            tag_buf.clear();
            tag_buf.push(ch);
            continue;
        }

        if in_tag {
            tag_buf.push(ch);
            if ch == '>' {
                in_tag = false;
                let tag = tag_buf.to_ascii_lowercase();
                if tag.starts_with("<script") || tag.starts_with("<style") {
                    skip_depth += 1;
                } else if tag.starts_with("</script") || tag.starts_with("</style") {
                    skip_depth = skip_depth.saturating_sub(1);
                } else if skip_depth == 0 && is_block_boundary(&tag) && !text.ends_with('\n') {
                    text.push('\n');
                }
            }
            continue;
        }

        if skip_depth == 0 {
            text.push(ch);
        }
    }

    decode_xml_entities(&text)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn is_block_boundary(tag: &str) -> bool {
    [
        "<p",
        "</p",
        "<div",
        "</div",
        "<h1",
        "</h1",
        "<h2",
        "</h2",
        "<h3",
        "</h3",
        "<li",
        "</li",
        "<br",
        "<blockquote",
        "</blockquote",
    ]
    .iter()
    .any(|prefix| tag.starts_with(prefix))
}

fn decode_xml_entities(text: &str) -> String {
    text.replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
}

fn extract_tag_text(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let start = xml.find(&open)? + open.len();
    let end = xml[start..].find(&close)? + start;
    Some(decode_xml_entities(xml[start..end].trim()))
}

fn extract_attribute_from_tag(xml: &str, tag: &str, attribute: &str) -> Option<String> {
    let start = xml.find(&format!("<{tag}"))?;
    let end = xml[start..].find('>')? + start + 1;
    extract_attribute(&xml[start..end], attribute)
}

fn extract_tags<'a>(xml: &'a str, tag: &str) -> Vec<&'a str> {
    let mut tags = Vec::new();
    let mut cursor = 0usize;
    let needle = format!("<{tag}");

    while let Some(relative) = xml[cursor..].find(&needle) {
        let start = cursor + relative;
        let Some(end_relative) = xml[start..].find('>') else {
            break;
        };
        let end = start + end_relative + 1;
        tags.push(&xml[start..end]);
        cursor = end;
    }

    tags
}

fn extract_attribute(tag: &str, attribute: &str) -> Option<String> {
    let marker = format!("{attribute}=\"");
    let start = tag.find(&marker)? + marker.len();
    let end = tag[start..].find('"')? + start;
    Some(decode_xml_entities(&tag[start..end]))
}

fn normalize_archive_path(base: &Path, href: &str) -> String {
    let mut parts = Vec::<String>::new();
    for component in base.join(href).components() {
        match component {
            std::path::Component::ParentDir => {
                parts.pop();
            }
            std::path::Component::Normal(value) => {
                parts.push(value.to_string_lossy().to_string());
            }
            _ => {}
        }
    }
    parts.join("/")
}

fn file_stem_title(path: &Path) -> String {
    path.file_stem()
        .and_then(|value| value.to_str())
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("Untitled")
        .to_string()
}

#[cfg(feature = "bookforge-epub")]
pub(crate) fn load_epub_structured(
    path: &Path,
    _text: &str,
) -> Result<ParsedDocument, DocumentError> {
    let book = bookforge_epub::EpubReader::new()
        .read(path)
        .map_err(|error| map_bookforge_error(path, error))?;

    let mut parsed = ParsedDocument::new(
        DocumentFormat::Epub,
        book.metadata
            .title
            .clone()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| parser_file_stem_title(path)),
        base_source(path, DocumentFormat::Epub),
    );
    parsed.author = book.metadata.creators.first().cloned();

    let resources = book
        .resource_files
        .iter()
        .map(|resource| (resource.id.clone(), resource.href.clone()))
        .collect::<HashMap<_, _>>();
    let spine_index = book
        .resource_files
        .iter()
        .filter_map(|resource| {
            resource
                .spine_index
                .map(|index| (resource.id.clone(), index))
        })
        .collect::<HashMap<_, _>>();

    for block in &book.blocks {
        if should_translate_bookforge_block(block.kind) {
            let source_path = resources.get(&block.resource_id).cloned();
            let chapter = spine_index
                .get(&block.resource_id)
                .map(|index| index.saturating_add(1));
            let line = u32::try_from(block.id.0.saturating_add(1)).ok();
            let page = chapter.and_then(|value| u32::try_from(value).ok());
            let mut source = base_source(path, DocumentFormat::Epub);
            source.resource = source_path;
            source.chapter = chapter;
            source.page = page;
            source.line = line;
            source.block_id = Some(block.id.to_string());

            parsed.blocks.push(ParsedBlock {
                kind: map_bookforge_block_kind(block.kind),
                text: block.plain_text.clone(),
                source,
            });
        }
    }

    if parsed.blocks.is_empty() {
        return Err(DocumentError::EmptyDocument(path.to_path_buf()));
    }

    Ok(parsed)
}

#[cfg(not(feature = "bookforge-epub"))]
pub(crate) fn load_epub_structured(
    path: &Path,
    text: &str,
) -> Result<ParsedDocument, DocumentError> {
    load_document(text, DocumentFormat::Epub, path).map(|document| ParsedDocument::from(document))
}

#[cfg(feature = "bookforge-epub")]
fn map_bookforge_error(path: &Path, error: BookforgeError) -> DocumentError {
    let message = error.to_string();
    let lower = message.to_ascii_lowercase();
    if lower.contains("zip")
        || lower.contains("central directory")
        || lower.contains("invalid archive")
        || lower.contains("decompression")
    {
        return DocumentError::CorruptedFile(path.to_path_buf());
    }

    DocumentError::InvalidStructure(format!(
        "BookForge EPUB validation failed at revision {BOOKFORGE_REVISION}: {message}"
    ))
}

#[cfg(feature = "bookforge-epub")]
fn should_translate_bookforge_block(kind: BookForgeBlockKind) -> bool {
    matches!(
        kind,
        BookForgeBlockKind::Heading
            | BookForgeBlockKind::Paragraph
            | BookForgeBlockKind::ListItem
            | BookForgeBlockKind::BlockQuote
            | BookForgeBlockKind::Footnote
    )
}

#[cfg(feature = "bookforge-epub")]
fn map_bookforge_block_kind(kind: BookForgeBlockKind) -> BlockKind {
    match kind {
        BookForgeBlockKind::Heading => BlockKind::Heading,
        BookForgeBlockKind::ListItem => BlockKind::ListItem,
        BookForgeBlockKind::BlockQuote => BlockKind::Quote,
        BookForgeBlockKind::Footnote => BlockKind::Footnote,
        BookForgeBlockKind::Code => BlockKind::Code,
        BookForgeBlockKind::PageFurniture => BlockKind::PageBreak,
        BookForgeBlockKind::Paragraph => BlockKind::Paragraph,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        extract_html_text, extract_manifest_items, extract_rootfile_path, extract_spine_ids,
        normalize_archive_path,
    };
    use std::path::Path;

    #[test]
    fn extracts_rootfile_path() {
        let xml = r#"<rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/>"#;
        assert_eq!(
            extract_rootfile_path(xml).as_deref(),
            Some("OEBPS/content.opf")
        );
    }

    #[test]
    fn extracts_manifest_and_spine() {
        let opf = r#"
            <manifest>
              <item id="c1" href="Text/chapter1.xhtml" media-type="application/xhtml+xml"/>
              <item id="c2" href="Text/chapter2.xhtml" media-type="application/xhtml+xml"/>
            </manifest>
            <spine>
              <itemref idref="c2"/>
              <itemref idref="c1"/>
            </spine>
        "#;

        let manifest = extract_manifest_items(opf);
        assert_eq!(manifest.len(), 2);
        assert_eq!(manifest[0].id, "c1");
        assert_eq!(manifest[0].href, "Text/chapter1.xhtml");
        assert_eq!(extract_spine_ids(opf), vec!["c2", "c1"]);
    }

    #[test]
    fn extracts_html_as_readable_text() {
        let html = r#"
            <html><body>
              <h1>Chapter &amp; One</h1>
              <p>Hello <em>world</em>.</p>
              <script>ignore()</script>
              <p>Second&nbsp;line.</p>
            </body></html>
        "#;
        let text = extract_html_text(html);
        assert!(text.contains("Chapter & One"));
        assert!(text.contains("Hello world."));
        assert!(text.contains("Second line."));
        assert!(!text.contains("ignore"));
    }

    #[test]
    fn normalizes_relative_archive_paths() {
        let base = Path::new("OEBPS/Text");
        assert_eq!(
            normalize_archive_path(base, "../Styles/main.css"),
            "OEBPS/Styles/main.css"
        );
    }

    #[cfg(feature = "bookforge-epub")]
    #[test]
    fn bookforge_page_furniture_and_code_are_not_translation_units() {
        assert!(!super::should_translate_bookforge_block(
            bookforge_core::ir::BlockKind::PageFurniture
        ));
        assert!(!super::should_translate_bookforge_block(
            bookforge_core::ir::BlockKind::Code
        ));
        assert!(super::should_translate_bookforge_block(
            bookforge_core::ir::BlockKind::Footnote
        ));
    }
}
