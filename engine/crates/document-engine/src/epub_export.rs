#[cfg(feature = "bookforge-epub")]
use bookforge_core::{
    config::SegmentationConfig,
    segment::{build_segments, BlockTranslation},
};
#[cfg(feature = "bookforge-epub")]
use bookforge_epub::{
    read_epub, rebuild_epub_with_options, validate_translated_epub, RebuildOptions,
    ValidationSeverity,
};
#[cfg(feature = "bookforge-epub")]
use quick_xml::{
    events::{BytesStart, Event},
    Reader, Writer,
};
#[cfg(feature = "bookforge-epub")]
use std::collections::{BTreeMap, BTreeSet};
#[cfg(feature = "bookforge-epub")]
use std::fs::{self, File};
#[cfg(feature = "bookforge-epub")]
use std::io::{Read, Write};
use std::path::Path;
#[cfg(feature = "bookforge-epub")]
use std::time::{SystemTime, UNIX_EPOCH};
#[cfg(feature = "bookforge-epub")]
use zip::{write::SimpleFileOptions, CompressionMethod, DateTime, ZipArchive, ZipWriter};

use crate::DocumentError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpubBlockTranslation {
    pub block_id: String,
    pub text: String,
}

impl EpubBlockTranslation {
    pub fn new(block_id: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            block_id: block_id.into(),
            text: text.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpubExportReport {
    pub translated_blocks: usize,
    pub files_checked: usize,
    pub warnings: Vec<String>,
    pub rtl_metadata_applied: bool,
}

#[cfg(feature = "bookforge-epub")]
pub fn export_translated_epub(
    source: impl AsRef<Path>,
    output: impl AsRef<Path>,
    target_language: &str,
    translations: &[EpubBlockTranslation],
) -> Result<EpubExportReport, DocumentError> {
    let source = source.as_ref();
    let output = output.as_ref();
    if target_language.trim().is_empty() {
        return Err(DocumentError::InvalidStructure(
            "EPUB target language must not be empty".to_string(),
        ));
    }

    let book = read_epub(source).map_err(bookforge_error)?;
    let segments = build_segments(
        &book,
        &SegmentationConfig {
            max_segment_tokens: 1200,
            context_tokens: 0,
        },
    )
    .map_err(bookforge_error)?;

    // The application layer owns completeness against the native manuscript
    // translation units (chapter heading + every paragraph provenance block).
    // BookForge intentionally also models package/nav/page-furniture text, so
    // requiring every BookForge block here would force non-literary metadata
    // through the literary translator. The writer and validator explicitly
    // support partial maps: missing BookForge-only blocks fall back to source.
    // This boundary therefore rejects invented/duplicate/empty mappings while
    // validating markers for every explicitly translated block.
    let known_blocks = book
        .blocks
        .iter()
        .map(|block| block.id.0.clone())
        .collect::<BTreeSet<_>>();
    let provided = validate_translation_mapping(&known_blocks, translations)?;
    let block_translations = provided
        .into_iter()
        .map(|(block_id, text)| BlockTranslation {
            block_id: bookforge_core::ir::BlockId(block_id),
            text,
        })
        .collect::<Vec<_>>();

    let preflight_issues =
        bookforge_epub::validate_block_translations(&segments, &block_translations);
    let blocking_preflight = preflight_issues
        .iter()
        .filter(|issue| issue.severity == ValidationSeverity::Error)
        .map(|issue| issue.message.clone())
        .collect::<Vec<_>>();
    if !blocking_preflight.is_empty() {
        return Err(DocumentError::InvalidStructure(format!(
            "EPUB translation mapping failed structural marker validation: {}",
            blocking_preflight.join("; ")
        )));
    }

    let rebuild_stage = sibling_stage_path(output, "rebuild");
    let rtl_stage = sibling_stage_path(output, "rtl");
    let _ = fs::remove_file(&rebuild_stage);
    let _ = fs::remove_file(&rtl_stage);

    let options = RebuildOptions::replace_with_target_language(Some(target_language));
    if let Err(error) =
        rebuild_epub_with_options(&book, &block_translations, &rebuild_stage, &options)
    {
        let _ = fs::remove_file(&rebuild_stage);
        return Err(bookforge_error(error));
    }

    let rtl = is_rtl_language(target_language);
    let candidate = if rtl {
        if let Err(error) = apply_rtl_directionality(&rebuild_stage, &rtl_stage, &book.id.0) {
            let _ = fs::remove_file(&rebuild_stage);
            let _ = fs::remove_file(&rtl_stage);
            return Err(error);
        }
        &rtl_stage
    } else {
        &rebuild_stage
    };

    let validation = validate_translated_epub(candidate, &segments, &block_translations);
    let blocking = validation
        .issues
        .iter()
        .filter(|issue| issue.severity == ValidationSeverity::Error)
        .map(|issue| issue.message.clone())
        .collect::<Vec<_>>();
    if !validation.xml_valid || !blocking.is_empty() {
        let _ = fs::remove_file(&rebuild_stage);
        let _ = fs::remove_file(&rtl_stage);
        return Err(DocumentError::InvalidStructure(format!(
            "rebuilt EPUB failed validation: {}",
            if blocking.is_empty() {
                "invalid XML or package structure".to_string()
            } else {
                blocking.join("; ")
            }
        )));
    }

    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }
    if output.exists() {
        fs::remove_file(output)?;
    }
    fs::rename(candidate, output)?;
    if candidate != &rebuild_stage {
        let _ = fs::remove_file(&rebuild_stage);
    }
    let _ = fs::remove_file(&rtl_stage);

    let warnings = preflight_issues
        .iter()
        .chain(validation.issues.iter())
        .filter(|issue| issue.severity == ValidationSeverity::Warning)
        .map(|issue| issue.message.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();

    Ok(EpubExportReport {
        translated_blocks: block_translations.len(),
        files_checked: validation.files_checked,
        warnings,
        rtl_metadata_applied: rtl,
    })
}

#[cfg(not(feature = "bookforge-epub"))]
pub fn export_translated_epub(
    _source: impl AsRef<Path>,
    _output: impl AsRef<Path>,
    _target_language: &str,
    _translations: &[EpubBlockTranslation],
) -> Result<EpubExportReport, DocumentError> {
    Err(DocumentError::InvalidStructure(
        "EPUB round-trip export requires the document-engine 'bookforge-epub' feature".to_string(),
    ))
}

#[cfg(feature = "bookforge-epub")]
fn validate_translation_mapping(
    known_blocks: &BTreeSet<String>,
    translations: &[EpubBlockTranslation],
) -> Result<BTreeMap<String, String>, DocumentError> {
    if translations.is_empty() {
        return Err(DocumentError::InvalidStructure(
            "EPUB export has no explicit translated block mappings".to_string(),
        ));
    }

    let mut provided = BTreeMap::new();
    for translation in translations {
        if !known_blocks.contains(&translation.block_id) {
            return Err(DocumentError::InvalidStructure(format!(
                "EPUB translation contains unknown block id '{}'",
                translation.block_id
            )));
        }
        if translation.text.trim().is_empty() {
            return Err(DocumentError::InvalidStructure(format!(
                "EPUB translation for block '{}' is empty",
                translation.block_id
            )));
        }
        if provided
            .insert(translation.block_id.clone(), translation.text.clone())
            .is_some()
        {
            return Err(DocumentError::InvalidStructure(format!(
                "EPUB translation contains duplicate block id '{}'",
                translation.block_id
            )));
        }
    }
    Ok(provided)
}

#[cfg(feature = "bookforge-epub")]
fn sibling_stage_path(output: &Path, label: &str) -> std::path::PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    let name = output
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("output.epub");
    output.with_file_name(format!(
        ".{name}.{}-{nonce}.{label}.epub",
        std::process::id()
    ))
}

#[cfg(feature = "bookforge-epub")]
fn is_rtl_language(language: &str) -> bool {
    let primary = language
        .trim()
        .to_ascii_lowercase()
        .split(['-', '_'])
        .next()
        .unwrap_or_default()
        .to_string();
    matches!(primary.as_str(), "fa" | "ar" | "he" | "ur" | "ps" | "ckb")
}

#[cfg(feature = "bookforge-epub")]
fn apply_rtl_directionality(
    input: &Path,
    output: &Path,
    package_path: &str,
) -> Result<(), DocumentError> {
    let source = File::open(input)?;
    let mut archive = ZipArchive::new(source).map_err(DocumentError::Zip)?;
    let destination = File::create(output)?;
    let mut writer = ZipWriter::new(destination);
    let fixed_time = DateTime::default();

    let mut mimetype = archive.by_name("mimetype").map_err(DocumentError::Zip)?;
    let mut mimetype_bytes = Vec::new();
    mimetype.read_to_end(&mut mimetype_bytes)?;
    drop(mimetype);
    writer
        .start_file(
            "mimetype",
            SimpleFileOptions::default()
                .compression_method(CompressionMethod::Stored)
                .last_modified_time(fixed_time),
        )
        .map_err(DocumentError::Zip)?;
    writer.write_all(&mimetype_bytes)?;

    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).map_err(DocumentError::Zip)?;
        let name = entry.name().to_string();
        if name == "mimetype" {
            continue;
        }
        let compression = if entry.compression() == CompressionMethod::Stored {
            CompressionMethod::Stored
        } else {
            CompressionMethod::Deflated
        };
        if entry.is_dir() {
            writer
                .add_directory(
                    name,
                    SimpleFileOptions::default()
                        .compression_method(compression)
                        .last_modified_time(fixed_time),
                )
                .map_err(DocumentError::Zip)?;
            continue;
        }
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes)?;
        let lower = name.to_ascii_lowercase();
        if lower.ends_with(".xhtml") || lower.ends_with(".html") {
            let text = String::from_utf8(bytes).map_err(|error| {
                DocumentError::ParsingFailure(format!(
                    "rebuilt XHTML '{name}' is not UTF-8: {error}"
                ))
            })?;
            bytes = patch_xml_attribute(&text, b"html", "dir", "rtl")?.into_bytes();
        } else if name == package_path {
            let text = String::from_utf8(bytes).map_err(|error| {
                DocumentError::ParsingFailure(format!("rebuilt OPF '{name}' is not UTF-8: {error}"))
            })?;
            bytes = patch_xml_attribute(&text, b"spine", "page-progression-direction", "rtl")?
                .into_bytes();
        }
        writer
            .start_file(
                name,
                SimpleFileOptions::default()
                    .compression_method(compression)
                    .last_modified_time(fixed_time),
            )
            .map_err(DocumentError::Zip)?;
        writer.write_all(&bytes)?;
    }
    writer.finish().map_err(DocumentError::Zip)?;
    Ok(())
}

#[cfg(feature = "bookforge-epub")]
fn patch_xml_attribute(
    xml: &str,
    target_local_name: &[u8],
    attribute_name: &str,
    attribute_value: &str,
) -> Result<String, DocumentError> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);
    let mut writer = Writer::new(Vec::new());
    let mut patched = false;
    loop {
        let event = reader.read_event().map_err(xml_error)?;
        match event {
            Event::Start(element)
                if !patched && local_name(element.name().as_ref()) == target_local_name =>
            {
                let replacement = replace_attribute(&element, attribute_name, attribute_value)?;
                writer
                    .write_event(Event::Start(replacement))
                    .map_err(DocumentError::Io)?;
                patched = true;
            }
            Event::Empty(element)
                if !patched && local_name(element.name().as_ref()) == target_local_name =>
            {
                let replacement = replace_attribute(&element, attribute_name, attribute_value)?;
                writer
                    .write_event(Event::Empty(replacement))
                    .map_err(DocumentError::Io)?;
                patched = true;
            }
            Event::Eof => break,
            other => writer
                .write_event(other.borrow())
                .map_err(DocumentError::Io)?,
        }
    }
    if !patched {
        return Err(DocumentError::InvalidStructure(format!(
            "EPUB XML does not contain expected <{}> element",
            String::from_utf8_lossy(target_local_name)
        )));
    }
    String::from_utf8(writer.into_inner()).map_err(|error| {
        DocumentError::ParsingFailure(format!("patched EPUB XML is not UTF-8: {error}"))
    })
}

#[cfg(feature = "bookforge-epub")]
fn replace_attribute(
    source: &BytesStart<'_>,
    attribute_name: &str,
    attribute_value: &str,
) -> Result<BytesStart<'static>, DocumentError> {
    let source_name = source.name();
    let name = String::from_utf8_lossy(source_name.as_ref()).into_owned();
    let mut element = BytesStart::new(name);
    let mut attributes = Vec::new();
    for attribute in source.attributes() {
        let attribute = attribute.map_err(|error| {
            DocumentError::ParsingFailure(format!("invalid EPUB XML attribute: {error}"))
        })?;
        let key = String::from_utf8_lossy(attribute.key.as_ref()).into_owned();
        if key == attribute_name {
            continue;
        }
        let value = attribute
            .normalized_value(quick_xml::XmlVersion::Implicit1_0)
            .map_err(|error| {
                DocumentError::ParsingFailure(format!("invalid EPUB XML attribute value: {error}"))
            })?;
        attributes.push((key, value.into_owned()));
    }
    for (key, value) in &attributes {
        element.push_attribute((key.as_str(), value.as_str()));
    }
    element.push_attribute((attribute_name, attribute_value));
    Ok(element.into_owned())
}

#[cfg(feature = "bookforge-epub")]
fn local_name(name: &[u8]) -> &[u8] {
    name.rsplit(|byte| *byte == b':').next().unwrap_or(name)
}

#[cfg(feature = "bookforge-epub")]
fn xml_error(error: quick_xml::Error) -> DocumentError {
    DocumentError::ParsingFailure(format!("EPUB XML rewrite failed: {error}"))
}

#[cfg(feature = "bookforge-epub")]
fn bookforge_error(error: bookforge_core::BookforgeError) -> DocumentError {
    DocumentError::ParsingFailure(format!("BookForge EPUB operation failed: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "bookforge-epub")]
    #[test]
    fn explicit_partial_mapping_does_not_require_untranslated_bookforge_metadata() {
        let known = ["b_000001".to_string(), "b_000002".to_string()]
            .into_iter()
            .collect::<BTreeSet<_>>();
        let provided =
            validate_translation_mapping(&known, &[EpubBlockTranslation::new("b_000002", "ترجمه")])
                .unwrap();
        assert_eq!(provided.len(), 1);
        assert_eq!(provided.get("b_000002").map(String::as_str), Some("ترجمه"));
    }

    #[cfg(feature = "bookforge-epub")]
    #[test]
    fn explicit_mapping_rejects_unknown_duplicate_and_empty_blocks() {
        let known = ["b_000002".to_string()]
            .into_iter()
            .collect::<BTreeSet<_>>();
        assert!(validate_translation_mapping(
            &known,
            &[EpubBlockTranslation::new("b_unknown", "x")]
        )
        .is_err());
        assert!(validate_translation_mapping(
            &known,
            &[
                EpubBlockTranslation::new("b_000002", "x"),
                EpubBlockTranslation::new("b_000002", "y"),
            ]
        )
        .is_err());
        assert!(validate_translation_mapping(
            &known,
            &[EpubBlockTranslation::new("b_000002", "   ")]
        )
        .is_err());
    }

    #[cfg(feature = "bookforge-epub")]
    #[test]
    fn rtl_language_detection_is_conservative() {
        for language in ["fa", "fa-IR", "ar", "he", "ur", "ps", "ckb"] {
            assert!(is_rtl_language(language), "{language}");
        }
        for language in ["en", "fr", "ja", "zh"] {
            assert!(!is_rtl_language(language), "{language}");
        }
    }

    #[cfg(feature = "bookforge-epub")]
    #[test]
    fn xml_direction_patch_replaces_existing_attribute_without_duplicates() {
        let html = r#"<?xml version="1.0"?><html xmlns="http://www.w3.org/1999/xhtml" dir="ltr"><body><p>x</p></body></html>"#;
        let patched = patch_xml_attribute(html, b"html", "dir", "rtl").unwrap();
        assert!(patched.contains("dir=\"rtl\""));
        assert!(!patched.contains("dir=\"ltr\""));
        assert_eq!(patched.matches("dir=\"").count(), 1);
    }

    #[cfg(feature = "bookforge-epub")]
    #[test]
    fn xml_spine_patch_adds_rtl_progression() {
        let opf = r#"<?xml version="1.0"?><package xmlns="http://www.idpf.org/2007/opf"><spine toc="ncx"></spine></package>"#;
        let patched =
            patch_xml_attribute(opf, b"spine", "page-progression-direction", "rtl").unwrap();
        assert!(patched.contains("page-progression-direction=\"rtl\""));
    }
}
