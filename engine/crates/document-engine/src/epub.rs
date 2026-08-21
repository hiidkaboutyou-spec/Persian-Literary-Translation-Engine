use std::fs::File;
use std::io::Read;
use std::path::Path;
use zip::ZipArchive;

use crate::{load_document, Document, DocumentError};

pub fn load_epub_file(path: impl AsRef<Path>) -> Result<Document, DocumentError> {
    let path = path.as_ref();
    let file = File::open(path)?;
    let mut archive = ZipArchive::new(file).map_err(DocumentError::Zip)?;

    let container_xml = read_zip_entry(&mut archive, "META-INF/container.xml")?;
    let opf_path = extract_rootfile_path(&container_xml).ok_or_else(|| {
        DocumentError::InvalidDocument("EPUB container has no rootfile".to_string())
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
        return Err(DocumentError::InvalidDocument(
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
        return Err(DocumentError::InvalidDocument(
            "EPUB contains no readable spine text".to_string(),
        ));
    }

    Ok(load_document(title, text))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ManifestItem {
    id: String,
    href: String,
}

fn read_zip_entry(archive: &mut ZipArchive<File>, name: &str) -> Result<String, DocumentError> {
    let mut entry = archive.by_name(name).map_err(DocumentError::Zip)?;
    let mut content = String::new();
    entry.read_to_string(&mut content)?;
    Ok(content)
}

fn extract_rootfile_path(xml: &str) -> Option<String> {
    find_attribute_on_tag(xml, "rootfile", "full-path")
}

fn extract_metadata_title(xml: &str) -> Option<String> {
    extract_element_text(xml, "dc:title").or_else(|| extract_element_text(xml, "title"))
}

fn extract_manifest_items(xml: &str) -> Vec<ManifestItem> {
    collect_start_tags(xml, "item")
        .into_iter()
        .filter_map(|tag| {
            let id = extract_attribute(tag, "id")?;
            let href = extract_attribute(tag, "href")?;
            Some(ManifestItem { id, href })
        })
        .collect()
}

fn extract_spine_ids(xml: &str) -> Vec<String> {
    collect_start_tags(xml, "itemref")
        .into_iter()
        .filter_map(|tag| extract_attribute(tag, "idref"))
        .collect()
}

fn find_attribute_on_tag(xml: &str, tag_name: &str, attribute: &str) -> Option<String> {
    collect_start_tags(xml, tag_name)
        .into_iter()
        .find_map(|tag| extract_attribute(tag, attribute))
}

fn collect_start_tags<'a>(xml: &'a str, tag_name: &str) -> Vec<&'a str> {
    let mut tags = Vec::new();
    let mut cursor = 0;
    while let Some(relative_start) = xml[cursor..].find('<') {
        let start = cursor + relative_start;
        let Some(relative_end) = xml[start..].find('>') else {
            break;
        };
        let end = start + relative_end;
        let tag = &xml[start + 1..end];
        let normalized = tag.trim_start();
        let local = normalized
            .split_whitespace()
            .next()
            .unwrap_or("")
            .trim_end_matches('/');
        if local == tag_name || local.rsplit(':').next() == Some(tag_name) {
            tags.push(tag);
        }
        cursor = end + 1;
    }
    tags
}

fn extract_attribute(tag: &str, name: &str) -> Option<String> {
    let pattern = format!("{name}=");
    let start = tag.find(&pattern)? + pattern.len();
    let quote = tag.as_bytes().get(start).copied()? as char;
    if quote != '"' && quote != '\'' {
        return None;
    }
    let value_start = start + 1;
    let value_end = tag[value_start..].find(quote)? + value_start;
    Some(decode_entities(&tag[value_start..value_end]))
}

fn extract_element_text(xml: &str, name: &str) -> Option<String> {
    let open = format!("<{name}");
    let start = xml.find(&open)?;
    let body_start = xml[start..].find('>')? + start + 1;
    let close = format!("</{name}>");
    let body_end = xml[body_start..].find(&close)? + body_start;
    Some(decode_entities(xml[body_start..body_end].trim()))
}

fn normalize_archive_path(base: &Path, href: &str) -> String {
    let href = href.split('#').next().unwrap_or(href);
    let mut parts: Vec<String> = Vec::new();
    for component in base.join(href).components() {
        match component {
            std::path::Component::ParentDir => {
                parts.pop();
            }
            std::path::Component::Normal(part) => {
                parts.push(part.to_string_lossy().into_owned());
            }
            _ => {}
        }
    }
    parts.join("/")
}

pub(crate) fn extract_html_text(html: &str) -> String {
    let mut output = String::new();
    let mut in_tag = false;
    let mut tag = String::new();
    let mut text = String::new();

    for ch in html.chars() {
        if in_tag {
            if ch == '>' {
                let normalized = tag.trim().trim_start_matches('/').to_ascii_lowercase();
                let name = normalized
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
                    .trim_end_matches('/');
                if matches!(
                    name,
                    "p" | "div" | "br" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "li"
                ) {
                    flush_text(&mut output, &mut text);
                    if !output.ends_with('\n') {
                        output.push('\n');
                    }
                }
                tag.clear();
                in_tag = false;
            } else {
                tag.push(ch);
            }
        } else if ch == '<' {
            flush_text(&mut output, &mut text);
            in_tag = true;
        } else {
            text.push(ch);
        }
    }
    flush_text(&mut output, &mut text);

    let mut cleaned = String::new();
    let mut previous_blank = false;
    for line in output.lines() {
        let line = collapse_whitespace(&decode_entities(line))
            .trim()
            .to_string();
        if line.is_empty() {
            if !previous_blank && !cleaned.is_empty() {
                cleaned.push('\n');
                previous_blank = true;
            }
        } else {
            if !cleaned.is_empty() {
                cleaned.push('\n');
            }
            cleaned.push_str(&line);
            previous_blank = false;
        }
    }
    cleaned.trim().to_string()
}

fn flush_text(output: &mut String, text: &mut String) {
    if !text.is_empty() {
        output.push_str(text);
        text.clear();
    }
}

fn collapse_whitespace(input: &str) -> String {
    input.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn decode_entities(input: &str) -> String {
    input
        .replace("&nbsp;", " ")
        .replace("&#160;", " ")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}

fn file_stem_title(path: &Path) -> String {
    path.file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("Untitled")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_rootfile_path() {
        let xml = r#"<container><rootfiles><rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/></rootfiles></container>"#;
        assert_eq!(
            extract_rootfile_path(xml).as_deref(),
            Some("OEBPS/content.opf")
        );
    }

    #[test]
    fn extracts_manifest_and_spine() {
        let xml = r#"<package><manifest><item id="c1" href="chapter1.xhtml" media-type="application/xhtml+xml"/></manifest><spine><itemref idref="c1"/></spine></package>"#;
        assert_eq!(
            extract_manifest_items(xml),
            vec![ManifestItem {
                id: "c1".into(),
                href: "chapter1.xhtml".into(),
            }]
        );
        assert_eq!(extract_spine_ids(xml), vec!["c1"]);
    }

    #[test]
    fn extracts_html_as_readable_text() {
        let html = r#"<html><body><h1>Chapter 1</h1><p>Hello <em>world</em> &amp; friends.</p><p>Second paragraph.</p></body></html>"#;
        assert_eq!(
            extract_html_text(html),
            "Chapter 1\nHello world & friends.\nSecond paragraph."
        );
    }

    #[test]
    fn normalizes_relative_archive_paths() {
        assert_eq!(
            normalize_archive_path(Path::new("OEBPS/text"), "../chapter1.xhtml"),
            "OEBPS/chapter1.xhtml"
        );
    }
}
