//! Project lifecycle: one predictable root that knows where everything lives.
//!
//! ```text
//! <project-dir>/
//!   project.json                      application manifest (schema 1)
//!   source/<original>                 immutable imported source copy
//!   intelligence/deterministic.json   Phase 13 artifact
//!   intelligence/advanced.json        Phase 15 artifact
//!   review/review-ledger.json         Phase 14 ledger
//!   canon/characters.json             canonical Character Bible
//!   canon/glossary.json               canonical Glossary
//!   translation/progress.json         durable translation progress
//!   translation/chapters/             per-chapter artifacts + checkpoints
//!   history/history.json              bounded audit history
//!   exports/                          DOCX exports
//!   .lock                             project lock
//! ```

use super::error::ApplicationError;
use super::models::{
    canonical_root, project_id_for, HistoryEvent, ProjectEvent, ProjectEventSink, ProjectFile,
    SourceRecord, VecEventSink, PROJECT_SCHEMA_VERSION,
};
use chrono::{DateTime, Utc};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

pub const HISTORY_SCHEMA_VERSION: u32 = 1;
/// Bounded history length to keep memory/disk growth predictable.
const MAX_HISTORY_EVENTS: usize = 500;

/// Resolved, stable paths for one project. All paths derive from the root;
/// nothing is guessed by consumers.
#[derive(Debug, Clone)]
pub struct ProjectLayout {
    pub root: PathBuf,
    pub manifest: PathBuf,
    pub source_dir: PathBuf,
    pub intelligence_dir: PathBuf,
    pub review_file: PathBuf,
    pub characters_file: PathBuf,
    pub glossary_file: PathBuf,
    pub translation_dir: PathBuf,
    pub progress_file: PathBuf,
    pub chapters_dir: PathBuf,
    pub history_file: PathBuf,
    pub export_dir: PathBuf,
    pub lock_file: PathBuf,
}

impl ProjectLayout {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        let root = root.into();
        Self {
            manifest: root.join("project.json"),
            source_dir: root.join("source"),
            intelligence_dir: root.join("intelligence"),
            review_file: root.join("review").join("review-ledger.json"),
            characters_file: root.join("canon").join("characters.json"),
            glossary_file: root.join("canon").join("glossary.json"),
            translation_dir: root.join("translation"),
            progress_file: root.join("translation").join("progress.json"),
            chapters_dir: root.join("translation").join("chapters"),
            history_file: root.join("history").join("history.json"),
            export_dir: root.join("exports"),
            lock_file: root.join(".lock"),
            root,
        }
    }

    fn ensure_directories(&self) -> Result<(), ApplicationError> {
        let review_dir = self
            .review_file
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| self.root.clone());
        let history_dir = self
            .history_file
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| self.root.clone());
        let canon_dir = self.canon_file_parent();
        for directory in [
            &self.source_dir,
            &self.intelligence_dir,
            &review_dir,
            &canon_dir,
            &self.translation_dir,
            &self.chapters_dir,
            &history_dir,
            &self.export_dir,
        ] {
            fs::create_dir_all(directory).map_err(|error| {
                ApplicationError::PersistenceFailure(format!(
                    "failed to create {}: {error}",
                    directory.display()
                ))
            })?;
        }
        Ok(())
    }

    fn canon_file_parent(&self) -> PathBuf {
        self.characters_file
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| self.root.clone())
    }
}

// ---------------------------------------------------------------------------
// Project lock (local desktop use, no database)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct LockRecord {
    pid: u32,
    started_at: DateTime<Utc>,
    purpose: String,
}

/// Stale lock age: a lock older than this is treated as abandoned (crash).
const LOCK_STALE_AFTER: chrono::Duration = chrono::Duration::minutes(30);

/// Guard that holds the project lock; removed on drop.
pub struct ProjectLock {
    path: PathBuf,
}

impl ProjectLock {
    /// Try to acquire the exclusive project lock. Fails if another live lock
    /// exists; a stale lock (older than `LOCK_STALE_AFTER`) is removed and
    /// retried once so a crash never deadlocks the project.
    pub fn acquire(layout: &ProjectLayout, purpose: &str) -> Result<Self, ApplicationError> {
        // Return the guard from `try_create` directly: its `Drop` owns the
        // lock file, and dropping it here (e.g. via an `if let` scrutinee)
        // would delete the freshly-created lock and break mutual exclusion.
        match Self::try_create(layout, purpose) {
            Ok(guard) => Ok(guard),
            Err(ApplicationError::ProjectLocked) => {
                if Self::is_stale(&layout.lock_file) {
                    let _ = fs::remove_file(&layout.lock_file);
                    Self::try_create(layout, purpose)
                } else {
                    Err(ApplicationError::ProjectLocked)
                }
            }
            Err(other) => Err(other),
        }
    }

    fn try_create(layout: &ProjectLayout, purpose: &str) -> Result<Self, ApplicationError> {
        let record = LockRecord {
            pid: std::process::id(),
            started_at: Utc::now(),
            purpose: purpose.to_string(),
        };
        let bytes = serde_json::to_vec_pretty(&record).map_err(|error| {
            ApplicationError::PersistenceFailure(format!("failed to serialize lock: {error}"))
        })?;
        let mut options = fs::OpenOptions::new();
        options.write(true).create_new(true);
        match options.open(&layout.lock_file) {
            Ok(mut file) => {
                file.write_all(&bytes).map_err(|error| {
                    ApplicationError::PersistenceFailure(format!(
                        "failed to write lock {}: {error}",
                        layout.lock_file.display()
                    ))
                })?;
                Ok(Self {
                    path: layout.lock_file.clone(),
                })
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                Err(ApplicationError::ProjectLocked)
            }
            Err(error) => Err(ApplicationError::PersistenceFailure(format!(
                "failed to open lock {}: {error}",
                layout.lock_file.display()
            ))),
        }
    }

    fn is_stale(path: &Path) -> bool {
        let Ok(bytes) = fs::read(path) else {
            return false;
        };
        let Ok(record) = serde_json::from_slice::<LockRecord>(&bytes) else {
            return false;
        };
        Utc::now().signed_duration_since(record.started_at) > LOCK_STALE_AFTER
    }
}

impl Drop for ProjectLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

// ---------------------------------------------------------------------------
// History
// ---------------------------------------------------------------------------

pub fn append_history(
    layout: &ProjectLayout,
    project_id: &str,
    event: &str,
    detail: &str,
    timestamp: DateTime<Utc>,
) -> Result<(), ApplicationError> {
    let mut events = load_history(layout)?;
    events.push(HistoryEvent {
        history_schema_version: HISTORY_SCHEMA_VERSION,
        project_id: project_id.to_string(),
        event: event.to_string(),
        detail: detail.to_string(),
        timestamp,
    });
    if events.len() > MAX_HISTORY_EVENTS {
        events.drain(..events.len() - MAX_HISTORY_EVENTS);
    }
    atomic_write_json(&layout.history_file, &events)
}

pub fn load_history(layout: &ProjectLayout) -> Result<Vec<HistoryEvent>, ApplicationError> {
    if !layout.history_file.exists() {
        return Ok(Vec::new());
    }
    let bytes = fs::read(&layout.history_file).map_err(|error| {
        ApplicationError::PersistenceFailure(format!(
            "failed to read history {}: {error}",
            layout.history_file.display()
        ))
    })?;
    serde_json::from_slice(&bytes).map_err(|error| {
        ApplicationError::PersistenceFailure(format!(
            "invalid history {}: {error}",
            layout.history_file.display()
        ))
    })
}

// ---------------------------------------------------------------------------
// Manifest persistence
// ---------------------------------------------------------------------------

pub fn load_manifest(layout: &ProjectLayout) -> Result<ProjectFile, ApplicationError> {
    if !layout.manifest.exists() {
        return Err(ApplicationError::ProjectNotFound(layout.root.clone()));
    }
    let bytes = fs::read(&layout.manifest).map_err(|error| {
        ApplicationError::PersistenceFailure(format!(
            "failed to read {}: {error}",
            layout.manifest.display()
        ))
    })?;
    let manifest: ProjectFile = serde_json::from_slice(&bytes).map_err(|error| {
        ApplicationError::InvalidProject(format!(
            "invalid project manifest {}: {error}",
            layout.manifest.display()
        ))
    })?;
    if manifest.schema_version != PROJECT_SCHEMA_VERSION {
        return Err(ApplicationError::UnsupportedProjectVersion(
            manifest.schema_version,
            PROJECT_SCHEMA_VERSION,
        ));
    }
    Ok(manifest)
}

pub fn save_manifest(
    layout: &ProjectLayout,
    manifest: &ProjectFile,
) -> Result<(), ApplicationError> {
    atomic_write_json(&layout.manifest, manifest)
}

/// Atomic same-directory write (stage + rename) so a crash never leaves a
/// half-written critical file.
pub fn atomic_write_json<T: serde::Serialize>(
    path: &Path,
    value: &T,
) -> Result<(), ApplicationError> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|error| {
                ApplicationError::PersistenceFailure(format!(
                    "failed to create {}: {error}",
                    parent.display()
                ))
            })?;
        }
    }
    let staged = path.with_extension("json.tmp");
    let bytes = serde_json::to_vec_pretty(value).map_err(|error| {
        ApplicationError::PersistenceFailure(format!(
            "failed to serialize {}: {error}",
            path.display()
        ))
    })?;
    fs::write(&staged, bytes).map_err(|error| {
        ApplicationError::PersistenceFailure(format!(
            "failed to write {}: {error}",
            staged.display()
        ))
    })?;
    fs::rename(&staged, path).map_err(|error| {
        ApplicationError::PersistenceFailure(format!(
            "failed to finalize {}: {error}",
            path.display()
        ))
    })
}

/// Write raw bytes atomically (used for source copies and chapter artifacts).
pub fn atomic_write_bytes(path: &Path, bytes: &[u8]) -> Result<(), ApplicationError> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|error| {
                ApplicationError::PersistenceFailure(format!(
                    "failed to create {}: {error}",
                    parent.display()
                ))
            })?;
        }
    }
    let staged = path.with_extension("tmp");
    fs::write(&staged, bytes).map_err(|error| {
        ApplicationError::PersistenceFailure(format!(
            "failed to write {}: {error}",
            staged.display()
        ))
    })?;
    fs::rename(&staged, path).map_err(|error| {
        ApplicationError::PersistenceFailure(format!(
            "failed to finalize {}: {error}",
            path.display()
        ))
    })
}

pub fn emit_and_history(
    layout: &ProjectLayout,
    project_id: &str,
    sink: &mut dyn ProjectEventSink,
    history: Option<(&str, &str)>,
    event: ProjectEvent,
) {
    if let Some((name, detail)) = history {
        let _ = append_history(layout, project_id, name, detail, Utc::now());
    }
    sink.emit(event);
}

/// Convenience sink that records history events automatically.
pub struct HistorySink<'a> {
    layout: &'a ProjectLayout,
    project_id: &'a str,
}

impl<'a> HistorySink<'a> {
    pub fn new(layout: &'a ProjectLayout, project_id: &'a str) -> Self {
        Self { layout, project_id }
    }
}

impl ProjectEventSink for HistorySink<'_> {
    fn emit(&mut self, event: ProjectEvent) {
        let detail = history_detail(&event);
        let _ = append_history(
            self.layout,
            self.project_id,
            &detail.0,
            &detail.1,
            Utc::now(),
        );
    }
}

fn history_detail(event: &ProjectEvent) -> (String, String) {
    match event {
        ProjectEvent::ImportStarted { .. } => {
            ("source_import_started".into(), "import started".into())
        }
        ProjectEvent::ImportCompleted { chapters, .. } => {
            ("source_imported".into(), format!("chapters={chapters}"))
        }
        ProjectEvent::AnalysisStarted { .. } => {
            ("analysis_started".into(), "analysis started".into())
        }
        ProjectEvent::AnalysisCompleted {
            review_items_added, ..
        } => (
            "analysis_completed".into(),
            format!("review_items_added={review_items_added}"),
        ),
        ProjectEvent::AdvancedAnalysisStarted { .. } => (
            "advanced_analysis_started".into(),
            "advanced analysis started".into(),
        ),
        ProjectEvent::AdvancedAnalysisCompleted {
            findings,
            cached_units,
            failed_units,
            ..
        } => (
            "advanced_analysis_completed".into(),
            format!("findings={findings},cached={cached_units},failed={failed_units}"),
        ),
        ProjectEvent::ReviewStateChanged { item_id, .. } => {
            ("review_decision".into(), format!("item={item_id}"))
        }
        ProjectEvent::CanonPromoted { item_count, .. } => {
            ("canon_promoted".into(), format!("items={item_count}"))
        }
        ProjectEvent::TranslationStarted { run_id, .. } => {
            ("translation_started".into(), format!("run={run_id}"))
        }
        ProjectEvent::ChapterStarted { chapter_index, .. } => {
            ("chapter_started".into(), format!("chapter={chapter_index}"))
        }
        ProjectEvent::ChapterCompleted {
            chapter_index,
            quality_score,
            ..
        } => (
            "chapter_completed".into(),
            format!("chapter={chapter_index},quality={quality_score:.2}"),
        ),
        ProjectEvent::CheckpointSaved { chapter_index, .. } => (
            "checkpoint_saved".into(),
            format!("chapter={chapter_index}"),
        ),
        ProjectEvent::TranslationPaused { run_id, .. } => {
            ("translation_paused".into(), format!("run={run_id}"))
        }
        ProjectEvent::TranslationCompleted {
            run_id, chapters, ..
        } => (
            "translation_completed".into(),
            format!("run={run_id},chapters={chapters}"),
        ),
        ProjectEvent::ManualEdit {
            chapter_index,
            revision_id,
            ..
        } => (
            "manual_translation_edit".into(),
            format!("chapter={chapter_index},revision={revision_id}"),
        ),
        ProjectEvent::ExportStarted { .. } => ("export_started".into(), "export started".into()),
        ProjectEvent::ExportCompleted { format, .. } => {
            ("export_completed".into(), format!("format={format}"))
        }
        ProjectEvent::Warning { message, .. } => ("warning".into(), message.clone()),
        ProjectEvent::Failure { message, .. } => ("failure".into(), message.clone()),
    }
}

pub(crate) fn import_source_record(
    layout: &ProjectLayout,
    source_path: &Path,
    manuscript: &document_engine::Manuscript,
    imported_at: DateTime<Utc>,
) -> Result<SourceRecord, ApplicationError> {
    let original_filename = source_path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| {
            ApplicationError::ImportFailed(
                source_path.to_path_buf(),
                "source file has no valid filename".to_string(),
            )
        })?
        .to_string();
    let format = source_path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("txt")
        .to_ascii_lowercase();
    let bytes = fs::read(source_path).map_err(|error| {
        ApplicationError::ImportFailed(source_path.to_path_buf(), error.to_string())
    })?;
    let fingerprint = super::models::content_fingerprint(&bytes);
    let stored = layout.source_dir.join(&original_filename);
    atomic_write_bytes(&stored, &bytes)?;

    let mut scenes = 0usize;
    let mut paragraphs = 0usize;
    for chapter in &manuscript.chapters {
        scenes += chapter.scenes.len();
        for scene in &chapter.scenes {
            paragraphs += scene.paragraphs.len();
        }
    }

    Ok(SourceRecord {
        original_filename,
        stored_relative_path: format!(
            "source/{}",
            stored
                .file_name()
                .and_then(|v| v.to_str())
                .unwrap_or("source")
        ),
        format,
        fingerprint,
        size_bytes: bytes.len() as u64,
        book_id: manuscript.book.id.clone(),
        title: manuscript.book.title.clone(),
        chapter_count: manuscript.chapters.len(),
        scene_count: scenes,
        paragraph_count: paragraphs,
        imported_at,
    })
}

/// Create the project root and a fresh manifest. Never overwrites an existing
/// project silently: if `project.json` already exists this fails.
pub fn create_project(
    root: impl Into<PathBuf>,
    name: impl Into<String>,
    source: Option<&Path>,
    sink: &mut dyn ProjectEventSink,
) -> Result<ProjectLayout, ApplicationError> {
    let root = root.into();
    let layout = ProjectLayout::new(&root);
    if layout.manifest.exists() {
        return Err(ApplicationError::InvalidProject(format!(
            "project already exists at {}",
            root.display()
        )));
    }
    layout.ensure_directories()?;
    let name = name.into();
    let project_id = project_id_for(&name, &canonical_root(&root));
    let now = Utc::now();
    let manifest = ProjectFile {
        schema_version: PROJECT_SCHEMA_VERSION,
        project_id: project_id.clone(),
        name: name.clone(),
        created_at: now,
        updated_at: now,
        target_language: "fa".to_string(),
        source: None,
        analysis: None,
        advanced: None,
        canon_fingerprint: String::new(),
        translation: None,
        export: None,
    };
    save_manifest(&layout, &manifest)?;
    append_history(
        &layout,
        &project_id,
        "project_created",
        &format!("name={name}"),
        now,
    )?;
    sink.emit(ProjectEvent::ImportStarted {
        project_id: project_id.clone(),
    });
    if let Some(source_path) = source {
        let manuscript = document_engine::ingest_file(source_path).map_err(|error| {
            ApplicationError::ImportFailed(
                source_path.to_path_buf(),
                format!("ingestion failed: {error}"),
            )
        })?;
        let record = import_source_record(&layout, source_path, &manuscript, Utc::now())?;
        let mut manifest = load_manifest(&layout)?;
        manifest.source = Some(record);
        save_manifest(&layout, &manifest)?;
        append_history(
            &layout,
            &project_id,
            "source_imported",
            &format!("chapters={}", manuscript.chapters.len()),
            Utc::now(),
        )?;
        sink.emit(ProjectEvent::ImportCompleted {
            project_id,
            chapters: manuscript.chapters.len(),
        });
    } else {
        sink.emit(ProjectEvent::ImportCompleted {
            project_id,
            chapters: 0,
        });
    }
    Ok(layout)
}

/// Validate a project root and return its resolved layout + manifest.
pub fn open_project(
    root: impl Into<PathBuf>,
) -> Result<(ProjectLayout, ProjectFile), ApplicationError> {
    let layout = ProjectLayout::new(root);
    let manifest = load_manifest(&layout)?;
    Ok((layout, manifest))
}

#[allow(dead_code)] // used by application tests
pub(crate) fn noop_sink() -> VecEventSink {
    VecEventSink::new()
}
