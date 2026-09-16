from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    p = Path(path)
    text = p.read_text()
    if old not in text:
        raise SystemExit(f"expected text not found in {path}: {old[:160]!r}")
    p.write_text(text.replace(old, new, 1))


# ---------------------------------------------------------------------------
# Stable application artifact schema: optional provenance keeps old JSON valid.
# ---------------------------------------------------------------------------
replace_once(
    "engine/crates/project-engine/src/application/models.rs",
    '''    #[serde(default = "default_translation_style_profile")]
    pub style_profile: String,
    pub paragraphs: Vec<TranslatedParagraph>,
''',
    '''    #[serde(default = "default_translation_style_profile")]
    pub style_profile: String,
    /// Real source heading block when the document format exposes one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title_source_block_id: Option<String>,
    /// Provider-reviewed translation of a real source heading. Synthetic
    /// chapter labels deliberately leave this empty.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub translated_title: Option<String>,
    pub paragraphs: Vec<TranslatedParagraph>,
''',
)
replace_once(
    "engine/crates/project-engine/src/application/models.rs",
    '''pub struct TranslatedParagraph {
    pub paragraph_id: String,
    pub source: String,
    pub translated: String,
''',
    '''pub struct TranslatedParagraph {
    pub paragraph_id: String,
    pub source: String,
    /// Stable source-format block identity. Required for fail-closed EPUB
    /// reconstruction; absent for legacy/non-structured artifacts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_block_id: Option<String>,
    pub translated: String,
''',
)
replace_once(
    "engine/crates/project-engine/src/application/models.rs",
    '            export_formats: vec!["docx".to_string()],\n',
    '            export_formats: vec!["docx".to_string(), "epub".to_string()],\n',
)

# ---------------------------------------------------------------------------
# Translation lifecycle: provenance, heading translation, safer neighbors,
# EPUB-aware resume, and format-selective export.
# ---------------------------------------------------------------------------
p = Path("engine/crates/project-engine/src/application/translation.rs")
text = p.read_text()
text = text.replace(
    'use document_engine::Chapter as ManuscriptChapter;\n',
    'use document_engine::{Chapter as ManuscriptChapter, DocumentFormat};\n',
    1,
)
text = text.replace('use std::fs;\n', 'use std::collections::BTreeMap;\nuse std::fs;\n', 1)

old_align = '''fn align_paragraphs(chapter: &ManuscriptChapter, translated: &str) -> Vec<TranslatedParagraph> {
    let source_paragraphs = chapter
        .scenes
        .iter()
        .flat_map(|scene| scene.paragraphs.iter())
        .map(|paragraph| (paragraph.id.clone(), paragraph.original_text.clone()))
        .collect::<Vec<_>>();
    let translated_parts = translated
        .split("\\n\\n")
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(ToString::to_string)
        .collect::<Vec<_>>();

    if source_paragraphs.len() == translated_parts.len() && !source_paragraphs.is_empty() {
        source_paragraphs
            .iter()
            .zip(translated_parts.iter())
            .map(|((id, source), translated)| TranslatedParagraph {
                paragraph_id: id.clone(),
                source: source.clone(),
                translated: translated.clone(),
                origin: "provider".to_string(),
                revisions: Vec::new(),
            })
            .collect()
    } else {
        vec![TranslatedParagraph {
            paragraph_id: format!("chapter-{}", chapter.index + 1),
            source: source_paragraphs
                .iter()
                .map(|(_, text)| text.as_str())
                .collect::<Vec<_>>()
                .join("\\n\\n"),
            translated: translated.to_string(),
            origin: "provider".to_string(),
            revisions: Vec::new(),
        }]
    }
}
'''
new_align = '''fn align_paragraphs(chapter: &ManuscriptChapter, translated: &str) -> Vec<TranslatedParagraph> {
    let source_paragraphs = chapter
        .scenes
        .iter()
        .flat_map(|scene| scene.paragraphs.iter())
        .map(|paragraph| {
            (
                paragraph.id.clone(),
                paragraph.original_text.clone(),
                paragraph.source.block_id.clone(),
            )
        })
        .collect::<Vec<_>>();
    let translated_parts = translated
        .split("\\n\\n")
        .map(str::trim)
        .filter(|part| !part.is_empty())
        // Scene-break sentinels are structural separators, not translation
        // paragraphs. Ignoring them here preserves one-to-one block provenance.
        .filter(|part| !matches!(*part, "***" | "---" | "* * *"))
        .map(ToString::to_string)
        .collect::<Vec<_>>();

    if source_paragraphs.len() == translated_parts.len() && !source_paragraphs.is_empty() {
        source_paragraphs
            .iter()
            .zip(translated_parts.iter())
            .map(|((id, source, block_id), translated)| TranslatedParagraph {
                paragraph_id: id.clone(),
                source: source.clone(),
                source_block_id: block_id.clone(),
                translated: translated.clone(),
                origin: "provider".to_string(),
                revisions: Vec::new(),
            })
            .collect()
    } else {
        vec![TranslatedParagraph {
            paragraph_id: format!("chapter-{}", chapter.index + 1),
            source: source_paragraphs
                .iter()
                .map(|(_, text, _)| text.as_str())
                .collect::<Vec<_>>()
                .join("\\n\\n"),
            source_block_id: None,
            translated: translated.to_string(),
            origin: "provider".to_string(),
            revisions: Vec::new(),
        }]
    }
}

fn epub_artifact_has_provenance(
    chapter: &ManuscriptChapter,
    artifact: &TranslatedChapter,
) -> bool {
    if chapter.source.format != DocumentFormat::Epub {
        return true;
    }
    if let Some(title_block_id) = chapter.source.block_id.as_deref() {
        if artifact.title_source_block_id.as_deref() != Some(title_block_id)
            || artifact
                .translated_title
                .as_deref()
                .is_none_or(|title| title.trim().is_empty())
        {
            return false;
        }
    }
    let source_paragraphs = chapter
        .scenes
        .iter()
        .flat_map(|scene| scene.paragraphs.iter())
        .collect::<Vec<_>>();
    if source_paragraphs.len() != artifact.paragraphs.len() {
        return false;
    }
    source_paragraphs.iter().all(|source| {
        let Some(block_id) = source.source.block_id.as_deref() else {
            return false;
        };
        artifact.paragraphs.iter().any(|translated| {
            translated.paragraph_id == source.id
                && translated.source_block_id.as_deref() == Some(block_id)
        })
    })
}
'''
if old_align not in text:
    raise SystemExit("align_paragraphs block not found")
text = text.replace(old_align, new_align, 1)

text = text.replace(
    '    for chapter in &manuscript.chapters {\n',
    '    for (chapter_position, chapter) in manuscript.chapters.iter().enumerate() {\n',
    1,
)
old_neighbors = '''        let previous = chapter.index.checked_sub(1).and_then(|index| {
            manuscript
                .chapters
                .get(index)
                .map(|neighbor| NeighborContext {
                    id: &neighbor.id,
                    title: &neighbor.title,
                    text: &neighbor.content,
                })
        });
        let next = manuscript
            .chapters
            .get(chapter.index.saturating_add(1))
'''
new_neighbors = '''        let previous = chapter_position.checked_sub(1).and_then(|index| {
            manuscript
                .chapters
                .get(index)
                .map(|neighbor| NeighborContext {
                    id: &neighbor.id,
                    title: &neighbor.title,
                    text: &neighbor.content,
                })
        });
        let next = manuscript
            .chapters
            .get(chapter_position + 1)
'''
if old_neighbors not in text:
    raise SystemExit("neighbor block not found")
text = text.replace(old_neighbors, new_neighbors, 1)

old_resume = '''        if resume {
            if let Some(existing) =
                resumable_chapter(layout, &stem, &source_fingerprint, &context_fingerprint)?
            {
                progress.completed_chapters = progress.completed_chapters.max(chapter.index + 1);
                progress.completed_paragraphs += count_translated_paragraphs(layout, &stem);
                progress.percent = if progress.total_chapters == 0 {
                    1.0
                } else {
                    progress.completed_chapters as f32 / progress.total_chapters as f32
                };
                progress.last_checkpoint = Some(format!("{stem}.txt"));
                progress.updated_at = Utc::now();
                save_progress(layout, &progress)?;
                completed_this_run += 1;
                let _ = existing;
                continue;
            }
        }
'''
new_resume = '''        if resume {
            if let Some(existing) =
                resumable_chapter(layout, &stem, &source_fingerprint, &context_fingerprint)?
            {
                let artifact = load_chapter_artifact(layout, &stem)?;
                let structured_reuse_ok = artifact
                    .as_ref()
                    .is_some_and(|artifact| epub_artifact_has_provenance(chapter, artifact));
                if chapter.source.format != DocumentFormat::Epub || structured_reuse_ok {
                    progress.completed_chapters = progress.completed_chapters.max(chapter.index + 1);
                    progress.completed_paragraphs += count_translated_paragraphs(layout, &stem);
                    progress.percent = if progress.total_chapters == 0 {
                        1.0
                    } else {
                        progress.completed_chapters as f32 / progress.total_chapters as f32
                    };
                    progress.last_checkpoint = Some(format!("{stem}.txt"));
                    progress.updated_at = Utc::now();
                    save_progress(layout, &progress)?;
                    completed_this_run += 1;
                    let _ = existing;
                    continue;
                }
                let warning = format!(
                    "EPUB checkpoint for {} predates block provenance or is structurally ambiguous; translating again",
                    chapter.title
                );
                if !progress.warnings.contains(&warning) {
                    progress.warnings.push(warning);
                }
            }
        }
'''
if old_resume not in text:
    raise SystemExit("resume block not found")
text = text.replace(old_resume, new_resume, 1)

old_pipeline = '''        let output = pipeline
            .execute(
                provider.as_ref(),
                PipelineInput {
                    source_text: source_text.clone(),
                    target_language: config.target_language.clone(),
                    context,
                },
            )
            .map_err(|error| {
                ApplicationError::Internal(format!(
                    "pipeline failed for {}: {error}",
                    chapter.title
                ))
            })?;

        let rules = terminology_rules(&glossary, &source_text);
'''
new_pipeline = '''        let translated_title = if chapter.source.block_id.is_some() {
            let title_output = pipeline
                .execute(
                    provider.as_ref(),
                    PipelineInput {
                        source_text: chapter.title.clone(),
                        target_language: config.target_language.clone(),
                        context: context.clone(),
                    },
                )
                .map_err(|error| {
                    ApplicationError::Internal(format!(
                        "pipeline failed for heading {}: {error}",
                        chapter.title
                    ))
                })?;
            let title_rules = terminology_rules(&glossary, &chapter.title);
            let title_quality =
                evaluate_translation(&chapter.title, &title_output.quality_review, &title_rules);
            if !title_quality.passes() {
                return Err(ApplicationError::Internal(format!(
                    "quality gate blocked heading {}: {}",
                    chapter.title,
                    title_quality.blocking_errors.join("; ")
                )));
            }
            Some(title_output.quality_review)
        } else {
            None
        };

        let output = pipeline
            .execute(
                provider.as_ref(),
                PipelineInput {
                    source_text: source_text.clone(),
                    target_language: config.target_language.clone(),
                    context,
                },
            )
            .map_err(|error| {
                ApplicationError::Internal(format!(
                    "pipeline failed for {}: {error}",
                    chapter.title
                ))
            })?;

        let rules = terminology_rules(&glossary, &source_text);
'''
if old_pipeline not in text:
    raise SystemExit("pipeline block not found")
text = text.replace(old_pipeline, new_pipeline, 1)

text = text.replace(
    '''            style_profile: style_profile.id().to_string(),
            paragraphs: align_paragraphs(chapter, &output.quality_review),
''',
    '''            style_profile: style_profile.id().to_string(),
            title_source_block_id: chapter.source.block_id.clone(),
            translated_title,
            paragraphs: align_paragraphs(chapter, &output.quality_review),
''',
    1,
)

old_export = '''pub fn export_translation(
    layout: &ProjectLayout,
    sink: &mut dyn ProjectEventSink,
) -> Result<super::models::ExportRecord, ApplicationError> {
    let manifest = super::project::load_manifest(layout)?;
    let project_id = manifest.project_id.clone();
    let manuscript = load_manuscript(layout)?;
    if manuscript.chapters.is_empty() {
        return Err(ApplicationError::ExportUnavailable(
            "no chapters to export".to_string(),
        ));
    }

    emit_and_history(
        layout,
        &project_id,
        sink,
        None,
        ProjectEvent::ExportStarted {
            project_id: project_id.clone(),
        },
    );

    let mut translated = Vec::new();
    for chapter in &manuscript.chapters {
        let stem = chapter_stem(&chapter.title, chapter.index);
        let artifact = load_chapter_artifact(layout, &stem)?.ok_or_else(|| {
            ApplicationError::ExportUnavailable(format!(
                "chapter {} is not translated yet",
                chapter.index + 1
            ))
        })?;
        translated.push(document_engine::Chapter::translated(
            chapter.index,
            chapter.title.clone(),
            join_translated(&artifact.paragraphs),
        ));
    }

    let path = layout.export_dir.join("manuscript.docx");
    document_engine::export_persian_docx(&path, &manifest.name, &translated).map_err(|error| {
        ApplicationError::ExportUnavailable(format!("DOCX export failed: {error}"))
    })?;

    let record = super::models::ExportRecord {
        format: "docx".to_string(),
        relative_path: "manuscript.docx".to_string(),
        chapters: translated.len(),
        created_at: Utc::now(),
    };
    let mut manifest = super::project::load_manifest(layout)?;
    manifest.export = Some(record.clone());
    manifest.updated_at = Utc::now();
    super::project::save_manifest(layout, &manifest)?;

    emit_and_history(
        layout,
        &project_id,
        sink,
        None,
        ProjectEvent::ExportCompleted {
            project_id: project_id.clone(),
            format: "docx".to_string(),
        },
    );
    Ok(record)
}
'''
new_export = '''pub fn export_translation(
    layout: &ProjectLayout,
    sink: &mut dyn ProjectEventSink,
) -> Result<super::models::ExportRecord, ApplicationError> {
    export_translation_as(layout, "docx", sink)
}

pub fn export_translation_as(
    layout: &ProjectLayout,
    format: &str,
    sink: &mut dyn ProjectEventSink,
) -> Result<super::models::ExportRecord, ApplicationError> {
    let manifest = super::project::load_manifest(layout)?;
    let project_id = manifest.project_id.clone();
    let manuscript = load_manuscript(layout)?;
    if manuscript.chapters.is_empty() {
        return Err(ApplicationError::ExportUnavailable(
            "no chapters to export".to_string(),
        ));
    }

    emit_and_history(
        layout,
        &project_id,
        sink,
        None,
        ProjectEvent::ExportStarted {
            project_id: project_id.clone(),
        },
    );

    let normalized = format.trim().to_ascii_lowercase();
    let record = match normalized.as_str() {
        "docx" => export_docx(layout, &manifest, &manuscript)?,
        "epub" => export_epub(layout, &manifest, &manuscript)?,
        other => {
            return Err(ApplicationError::ExportUnavailable(format!(
                "unsupported export format '{other}'; expected docx or epub"
            )))
        }
    };

    let mut manifest = super::project::load_manifest(layout)?;
    manifest.export = Some(record.clone());
    manifest.updated_at = Utc::now();
    super::project::save_manifest(layout, &manifest)?;

    emit_and_history(
        layout,
        &project_id,
        sink,
        None,
        ProjectEvent::ExportCompleted {
            project_id: project_id.clone(),
            format: record.format.clone(),
        },
    );
    Ok(record)
}

fn export_docx(
    layout: &ProjectLayout,
    manifest: &super::models::ProjectFile,
    manuscript: &document_engine::Manuscript,
) -> Result<super::models::ExportRecord, ApplicationError> {
    let mut translated = Vec::new();
    for chapter in &manuscript.chapters {
        let stem = chapter_stem(&chapter.title, chapter.index);
        let artifact = load_chapter_artifact(layout, &stem)?.ok_or_else(|| {
            ApplicationError::ExportUnavailable(format!(
                "chapter {} is not translated yet",
                chapter.index + 1
            ))
        })?;
        translated.push(document_engine::Chapter::translated(
            chapter.index,
            artifact
                .translated_title
                .clone()
                .unwrap_or_else(|| chapter.title.clone()),
            join_translated(&artifact.paragraphs),
        ));
    }
    let path = layout.export_dir.join("manuscript.docx");
    document_engine::export_persian_docx(&path, &manifest.name, &translated).map_err(|error| {
        ApplicationError::ExportUnavailable(format!("DOCX export failed: {error}"))
    })?;
    Ok(super::models::ExportRecord {
        format: "docx".to_string(),
        relative_path: "manuscript.docx".to_string(),
        chapters: translated.len(),
        created_at: Utc::now(),
    })
}

fn export_epub(
    layout: &ProjectLayout,
    manifest: &super::models::ProjectFile,
    manuscript: &document_engine::Manuscript,
) -> Result<super::models::ExportRecord, ApplicationError> {
    let source = manifest.source.as_ref().ok_or_else(|| {
        ApplicationError::ExportUnavailable("project has no imported source".to_string())
    })?;
    if !source.format.eq_ignore_ascii_case("epub") {
        return Err(ApplicationError::ExportUnavailable(
            "EPUB export requires an EPUB source so original XHTML, navigation, CSS, fonts, links, and assets can be preserved"
                .to_string(),
        ));
    }

    let mut grouped: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for chapter in &manuscript.chapters {
        let stem = chapter_stem(&chapter.title, chapter.index);
        let artifact = load_chapter_artifact(layout, &stem)?.ok_or_else(|| {
            ApplicationError::ExportUnavailable(format!(
                "chapter {} is not translated yet",
                chapter.index + 1
            ))
        })?;
        if !epub_artifact_has_provenance(chapter, &artifact) {
            return Err(ApplicationError::ExportUnavailable(format!(
                "chapter {} lacks exact EPUB block provenance; resume/retranslate it before EPUB export",
                chapter.index + 1
            )));
        }
        if let Some(block_id) = chapter.source.block_id.as_ref() {
            let translated_title = artifact.translated_title.as_ref().ok_or_else(|| {
                ApplicationError::ExportUnavailable(format!(
                    "chapter {} has a source heading block but no translated heading",
                    chapter.index + 1
                ))
            })?;
            grouped
                .entry(block_id.clone())
                .or_default()
                .push(translated_title.clone());
        }
        for source_paragraph in chapter
            .scenes
            .iter()
            .flat_map(|scene| scene.paragraphs.iter())
        {
            let block_id = source_paragraph.source.block_id.as_ref().ok_or_else(|| {
                ApplicationError::ExportUnavailable(format!(
                    "paragraph '{}' has no EPUB block provenance",
                    source_paragraph.id
                ))
            })?;
            let translated = artifact
                .paragraphs
                .iter()
                .find(|paragraph| paragraph.paragraph_id == source_paragraph.id)
                .ok_or_else(|| {
                    ApplicationError::ExportUnavailable(format!(
                        "translated paragraph '{}' is missing",
                        source_paragraph.id
                    ))
                })?;
            if translated.source_block_id.as_ref() != Some(block_id) {
                return Err(ApplicationError::ExportUnavailable(format!(
                    "paragraph '{}' EPUB block provenance does not match its source",
                    source_paragraph.id
                )));
            }
            grouped
                .entry(block_id.clone())
                .or_default()
                .push(translated.translated.clone());
        }
    }

    let translations = grouped
        .into_iter()
        .map(|(block_id, parts)| {
            document_engine::EpubBlockTranslation::new(block_id, parts.join("\\n\\n"))
        })
        .collect::<Vec<_>>();
    let source_path = layout.root.join(&source.stored_relative_path);
    let output = layout.export_dir.join("manuscript.epub");
    document_engine::export_translated_epub(
        &source_path,
        &output,
        &manifest.target_language,
        &translations,
    )
    .map_err(|error| {
        ApplicationError::ExportUnavailable(format!("EPUB round-trip export failed: {error}"))
    })?;

    Ok(super::models::ExportRecord {
        format: "epub".to_string(),
        relative_path: "manuscript.epub".to_string(),
        chapters: manuscript.chapters.len(),
        created_at: Utc::now(),
    })
}
'''
if old_export not in text:
    raise SystemExit("export block not found")
text = text.replace(old_export, new_export, 1)
p.write_text(text)

# ---------------------------------------------------------------------------
# Service boundary: preserve old default, add explicit format API.
# ---------------------------------------------------------------------------
replace_once(
    "engine/crates/project-engine/src/application/service.rs",
    '''    pub fn export_project(
        &self,
        project: &Project,
        sink: &mut dyn ProjectEventSink,
    ) -> Result<super::models::ExportRecord, ApplicationError> {
        let _lock = ProjectLock::acquire(&project.layout, "export")?;
        let manifest = load_manifest(&project.layout)?;
        let project_id = manifest.project_id.clone();
        let mut sink = HistoryForwardingSink {
            history: HistorySink::new(&project.layout, &project_id),
            inner: sink,
        };
        translation_ops::export_translation(&project.layout, &mut sink)
    }
''',
    '''    pub fn export_project(
        &self,
        project: &Project,
        sink: &mut dyn ProjectEventSink,
    ) -> Result<super::models::ExportRecord, ApplicationError> {
        self.export_project_as(project, "docx", sink)
    }

    pub fn export_project_as(
        &self,
        project: &Project,
        format: &str,
        sink: &mut dyn ProjectEventSink,
    ) -> Result<super::models::ExportRecord, ApplicationError> {
        let _lock = ProjectLock::acquire(&project.layout, "export")?;
        let manifest = load_manifest(&project.layout)?;
        let project_id = manifest.project_id.clone();
        let mut sink = HistoryForwardingSink {
            history: HistorySink::new(&project.layout, &project_id),
            inner: sink,
        };
        translation_ops::export_translation_as(&project.layout, format, &mut sink)
    }
''',
)

# ---------------------------------------------------------------------------
# CLI: explicit --export-format avoids collision with global --format text/json.
# ---------------------------------------------------------------------------
p = Path("engine/cli/src/project_cmd.rs")
text = p.read_text()
text = text.replace(
    '       export <dir>                                     export translated DOCX\\n\\\n',
    '       export <dir> [--export-format docx|epub]         export translated publication\\n\\\n',
    1,
)
old_cli_export = '''fn export(service: &ApplicationService, args: &[String]) -> Result<()> {
    let dir = project_path(args, true)?;
    let project = open(service, &dir)?;
    let mut sink = VecEventSink::new();
    let record = service
        .export_project(&project, &mut sink)
        .map_err(|error| format!("export failed: {error}"))?;
    println!(
        "exported {} ({} chapters) to {}",
        record.format,
        record.chapters,
        dir.join(&record.relative_path).display()
    );
    Ok(())
}
'''
new_cli_export = '''fn export(service: &ApplicationService, args: &[String]) -> Result<()> {
    let dir = project_path(args, true)?;
    let project = open(service, &dir)?;
    let export_format = flag_value(args, "--export-format").unwrap_or("docx");
    let mut sink = VecEventSink::new();
    let record = service
        .export_project_as(&project, export_format, &mut sink)
        .map_err(|error| format!("export failed: {error}"))?;
    println!(
        "exported {} ({} chapters) to {}",
        record.format,
        record.chapters,
        dir.join("export").join(&record.relative_path).display()
    );
    Ok(())
}
'''
if old_cli_export not in text:
    raise SystemExit("CLI export function not found")
text = text.replace(old_cli_export, new_cli_export, 1)
p.write_text(text)
