use character_engine::CharacterProfile;
use memory_engine::glossary::GlossaryEntry;
use project_engine::application::{
    AdvancedAnalysisSettings, ApplicationCapabilities, ApplicationError, ApplicationErrorPayload,
    ApplicationService, ArtifactState, DecisionAction, HistoryEvent, LiteraryReviewArtifactView,
    LiteraryReviewRunSummary, LiteraryReviewSettings, Project, ProjectSnapshot, ReviewItemSummary,
    TranslatedChapter, TranslationConfig, TranslationProgress, TranslationRevision, VecEventSink,
};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use tauri::AppHandle;
use tauri_plugin_dialog::DialogExt;

type CommandResult<T> = Result<T, ApplicationErrorPayload>;

fn payload(error: ApplicationError) -> ApplicationErrorPayload {
    ApplicationService::error_payload(&error)
}

fn internal(message: impl Into<String>) -> ApplicationErrorPayload {
    payload(ApplicationError::Internal(message.into()))
}

fn load_project(root: impl Into<PathBuf>) -> Result<Project, ApplicationError> {
    ApplicationService::open_project(root)
}

async fn blocking<T, F>(operation: F) -> CommandResult<T>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, ApplicationError> + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(operation)
        .await
        .map_err(|error| internal(format!("desktop worker failed: {error}")))?
        .map_err(payload)
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranslationInput {
    pub provider: String,
    pub model: Option<String>,
    pub target_language: String,
    pub style_profile: String,
    #[serde(default)]
    pub adult_content_confirmed: bool,
    pub max_chapters: Option<usize>,
}

impl From<TranslationInput> for TranslationConfig {
    fn from(input: TranslationInput) -> Self {
        Self {
            provider: input.provider,
            model: input.model.filter(|value| !value.trim().is_empty()),
            target_language: input.target_language,
            style_profile: input.style_profile,
            adult_content_confirmed: input.adult_content_confirmed,
            max_chapters: input.max_chapters,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdvancedAnalysisInput {
    pub provider: String,
    pub model: Option<String>,
    pub max_units: Option<usize>,
    #[serde(default = "default_true")]
    pub cache: bool,
}

fn default_true() -> bool {
    true
}

impl From<AdvancedAnalysisInput> for AdvancedAnalysisSettings {
    fn from(input: AdvancedAnalysisInput) -> Self {
        Self {
            provider: input.provider,
            model: input.model.filter(|value| !value.trim().is_empty()),
            max_units: input.max_units,
            cache: input.cache,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiteraryReviewInput {
    pub provider: String,
    pub model: Option<String>,
    #[serde(default = "default_true")]
    pub semantic_alignment: bool,
    pub max_chapters: Option<usize>,
}

impl From<LiteraryReviewInput> for LiteraryReviewSettings {
    fn from(input: LiteraryReviewInput) -> Self {
        Self {
            provider: input.provider,
            model: input.model.filter(|value| !value.trim().is_empty()),
            semantic_alignment: input.semantic_alignment,
            max_chapters: input.max_chapters,
        }
    }
}

#[tauri::command]
pub fn app_capabilities() -> ApplicationCapabilities {
    ApplicationService::capabilities()
}

#[tauri::command]
pub fn pick_project_folder(app: AppHandle) -> CommandResult<Option<String>> {
    let selected = app
        .dialog()
        .file()
        .set_title("Choose translation project folder")
        .blocking_pick_folder();
    match selected {
        Some(path) => path
            .into_path()
            .map(|path| Some(path.display().to_string()))
            .map_err(|error| internal(format!("selected folder is not a local path: {error}"))),
        None => Ok(None),
    }
}

#[tauri::command]
pub fn pick_source_file(app: AppHandle) -> CommandResult<Option<String>> {
    let selected = app
        .dialog()
        .file()
        .set_title("Choose source book")
        .add_filter("Books", &["txt", "md", "docx", "epub", "pdf"])
        .blocking_pick_file();
    match selected {
        Some(path) => path
            .into_path()
            .map(|path| Some(path.display().to_string()))
            .map_err(|error| internal(format!("selected file is not a local path: {error}"))),
        None => Ok(None),
    }
}

#[tauri::command]
pub fn set_session_openai_key(api_key: String) -> CommandResult<bool> {
    let trimmed = api_key.trim();
    if trimmed.is_empty() {
        return Err(payload(ApplicationError::ProviderNotConfigured(
            "OpenAI API key cannot be empty".to_string(),
        )));
    }
    std::env::set_var("OPENAI_API_KEY", trimmed);
    Ok(true)
}

#[tauri::command]
pub fn clear_session_openai_key() -> bool {
    std::env::remove_var("OPENAI_API_KEY");
    true
}

#[tauri::command]
pub fn test_provider_configuration(
    translation_provider: String,
    analysis_provider: String,
) -> CommandResult<bool> {
    ApplicationService::test_provider_configuration(&translation_provider, &analysis_provider)
        .map(|_| true)
        .map_err(payload)
}

#[tauri::command]
pub fn create_project(
    project_root: String,
    name: String,
    source_path: Option<String>,
) -> CommandResult<ProjectSnapshot> {
    let source = source_path.as_deref().map(Path::new);
    let mut sink = VecEventSink::new();
    let project =
        ApplicationService::create_project(PathBuf::from(&project_root), name, source, &mut sink)
            .map_err(payload)?;
    ApplicationService.snapshot(&project).map_err(payload)
}

#[tauri::command]
pub fn open_project(project_root: String) -> CommandResult<ProjectSnapshot> {
    let project = load_project(project_root).map_err(payload)?;
    ApplicationService.snapshot(&project).map_err(payload)
}

#[tauri::command]
pub async fn import_book(
    project_root: String,
    source_path: String,
) -> CommandResult<ProjectSnapshot> {
    blocking(move || {
        let service = ApplicationService;
        let project = load_project(project_root)?;
        let mut sink = VecEventSink::new();
        service.import_book(&project, Path::new(&source_path), &mut sink)?;
        service.snapshot(&project)
    })
    .await
}

#[tauri::command]
pub fn verify_source(project_root: String) -> CommandResult<ArtifactState> {
    let project = load_project(project_root).map_err(payload)?;
    ApplicationService.verify_source(&project).map_err(payload)
}

#[tauri::command]
pub async fn analyze_project(project_root: String) -> CommandResult<ProjectSnapshot> {
    blocking(move || {
        let service = ApplicationService;
        let project = load_project(project_root)?;
        let mut sink = VecEventSink::new();
        service.analyze_book(&project, &mut sink)?;
        service.snapshot(&project)
    })
    .await
}

#[tauri::command]
pub async fn run_advanced_analysis(
    project_root: String,
    input: AdvancedAnalysisInput,
) -> CommandResult<ProjectSnapshot> {
    blocking(move || {
        let service = ApplicationService;
        let project = load_project(project_root)?;
        let mut sink = VecEventSink::new();
        let settings = AdvancedAnalysisSettings::from(input);
        service.run_advanced_analysis(&project, &settings, &mut sink)?;
        service.snapshot(&project)
    })
    .await
}

#[tauri::command]
pub fn list_review_items(
    project_root: String,
    kind_filter: Option<String>,
    status_filter: Option<String>,
) -> CommandResult<Vec<ReviewItemSummary>> {
    let project = load_project(project_root).map_err(payload)?;
    ApplicationService
        .list_review_items(&project, kind_filter.as_deref(), status_filter.as_deref())
        .map_err(payload)
}

fn parse_decision_action(action: &str) -> Result<DecisionAction, ApplicationError> {
    match action {
        "approve" => Ok(DecisionAction::Approve),
        "reject" => Ok(DecisionAction::Reject),
        "defer" => Ok(DecisionAction::Defer),
        "reopen" => Ok(DecisionAction::Reopen),
        other => Err(ApplicationError::InvalidProject(format!(
            "unsupported desktop review action '{other}'; expected approve, reject, defer, or reopen"
        ))),
    }
}

#[tauri::command]
pub fn decide_review_item(
    project_root: String,
    item_id: String,
    action: String,
    reviewer: String,
    reason: String,
) -> CommandResult<ReviewItemSummary> {
    let project = load_project(project_root).map_err(payload)?;
    let action = parse_decision_action(&action).map_err(payload)?;
    let mut sink = VecEventSink::new();
    ApplicationService
        .decide_review_item(
            &project, &item_id, action, None, &reviewer, &reason, &mut sink,
        )
        .map_err(payload)
}

#[tauri::command]
pub fn list_characters(project_root: String) -> CommandResult<Vec<CharacterProfile>> {
    let project = load_project(project_root).map_err(payload)?;
    ApplicationService
        .list_characters(&project)
        .map_err(payload)
}

#[tauri::command]
pub fn upsert_character(
    project_root: String,
    profile: CharacterProfile,
    replace_existing: bool,
) -> CommandResult<bool> {
    let project = load_project(project_root).map_err(payload)?;
    ApplicationService
        .upsert_character(&project, profile, replace_existing)
        .map(|_| true)
        .map_err(payload)
}

#[tauri::command]
pub fn list_glossary_entries(project_root: String) -> CommandResult<Vec<GlossaryEntry>> {
    let project = load_project(project_root).map_err(payload)?;
    ApplicationService
        .list_glossary_entries(&project)
        .map_err(payload)
}

#[tauri::command]
pub fn upsert_glossary_entry(
    project_root: String,
    entry: GlossaryEntry,
    replace_existing: bool,
) -> CommandResult<bool> {
    let project = load_project(project_root).map_err(payload)?;
    ApplicationService
        .upsert_glossary_entry(&project, entry, replace_existing)
        .map(|_| true)
        .map_err(payload)
}

#[tauri::command]
pub async fn start_translation(
    project_root: String,
    input: TranslationInput,
) -> CommandResult<TranslationProgress> {
    blocking(move || {
        let service = ApplicationService;
        let project = load_project(project_root)?;
        let config = TranslationConfig::from(input);
        let mut sink = VecEventSink::new();
        service.start_translation(&project, &config, &mut sink)
    })
    .await
}

#[tauri::command]
pub async fn resume_translation(
    project_root: String,
    input: TranslationInput,
) -> CommandResult<TranslationProgress> {
    blocking(move || {
        let service = ApplicationService;
        let project = load_project(project_root)?;
        let config = TranslationConfig::from(input);
        let mut sink = VecEventSink::new();
        service.resume_translation(&project, &config, &mut sink)
    })
    .await
}

#[tauri::command]
pub fn request_pause(project_root: String) -> CommandResult<bool> {
    let project = load_project(project_root).map_err(payload)?;
    ApplicationService
        .request_pause(&project)
        .map(|_| true)
        .map_err(payload)
}

#[tauri::command]
pub fn get_progress(project_root: String) -> CommandResult<TranslationProgress> {
    let project = load_project(project_root).map_err(payload)?;
    ApplicationService.get_progress(&project).map_err(payload)
}

#[tauri::command]
pub fn get_translated_chapter(
    project_root: String,
    chapter_index: usize,
) -> CommandResult<TranslatedChapter> {
    let project = load_project(project_root).map_err(payload)?;
    ApplicationService
        .get_translated_chapter(&project, chapter_index)
        .map_err(payload)
}

#[tauri::command]
pub fn apply_manual_edit(
    project_root: String,
    chapter_index: usize,
    paragraph_id: String,
    new_text: String,
    reviewer: Option<String>,
) -> CommandResult<TranslationRevision> {
    let project = load_project(project_root).map_err(payload)?;
    let mut sink = VecEventSink::new();
    ApplicationService
        .apply_manual_translation_edit(
            &project,
            chapter_index,
            &paragraph_id,
            &new_text,
            reviewer.as_deref(),
            &mut sink,
        )
        .map_err(payload)
}

#[tauri::command]
pub async fn run_literary_review(
    project_root: String,
    input: LiteraryReviewInput,
) -> CommandResult<LiteraryReviewRunSummary> {
    blocking(move || {
        let service = ApplicationService;
        let project = load_project(project_root)?;
        let settings = LiteraryReviewSettings::from(input);
        service.review_translation(&project, &settings)
    })
    .await
}

#[tauri::command]
pub fn get_literary_review(
    project_root: String,
    chapter_index: usize,
) -> CommandResult<LiteraryReviewArtifactView> {
    let project = load_project(project_root).map_err(payload)?;
    ApplicationService
        .get_literary_review(&project, chapter_index)
        .map_err(payload)
}

#[tauri::command]
pub fn accept_literary_review_revision(
    project_root: String,
    chapter_index: usize,
    finding_id: String,
    reviewer: String,
) -> CommandResult<TranslationRevision> {
    let project = load_project(project_root).map_err(payload)?;
    let mut sink = VecEventSink::new();
    ApplicationService
        .accept_literary_review_revision(&project, chapter_index, &finding_id, &reviewer, &mut sink)
        .map_err(payload)
}

#[tauri::command]
pub async fn export_project(
    project_root: String,
    format: String,
) -> CommandResult<project_engine::application::models::ExportRecord> {
    blocking(move || {
        let service = ApplicationService;
        let project = load_project(project_root)?;
        let mut sink = VecEventSink::new();
        service.export_project_as(&project, &format, &mut sink)
    })
    .await
}

#[tauri::command]
pub fn project_history(project_root: String) -> CommandResult<Vec<HistoryEvent>> {
    let project = load_project(project_root).map_err(payload)?;
    ApplicationService.history(&project).map_err(payload)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn desktop_translation_input_maps_all_bounded_application_knobs() {
        let config = TranslationConfig::from(TranslationInput {
            provider: "openai".into(),
            model: Some("gpt-test".into()),
            target_language: "fa".into(),
            style_profile: "literary".into(),
            adult_content_confirmed: false,
            max_chapters: Some(2),
        });
        assert_eq!(config.provider, "openai");
        assert_eq!(config.model.as_deref(), Some("gpt-test"));
        assert_eq!(config.target_language, "fa");
        assert_eq!(config.style_profile, "literary");
        assert_eq!(config.max_chapters, Some(2));
    }

    #[test]
    fn desktop_review_actions_never_invent_edit_values() {
        assert_eq!(
            parse_decision_action("approve").unwrap(),
            DecisionAction::Approve
        );
        assert_eq!(
            parse_decision_action("reject").unwrap(),
            DecisionAction::Reject
        );
        assert!(parse_decision_action("edit").is_err());
        assert!(parse_decision_action("apply").is_err());
    }
}
