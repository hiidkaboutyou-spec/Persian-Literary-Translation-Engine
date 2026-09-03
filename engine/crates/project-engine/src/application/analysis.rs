//! Analysis orchestration through the application boundary.
//!
//! `analyze_book` runs Phase 13 deterministic analysis, persists the
//! `ManuscriptIntelligence` artifact, and reconciles Phase 14 proposals into
//! the review ledger. `run_advanced_analysis` runs the Phase 15 provider layer
//! with bounded units, validates findings, and reconciles literary proposals —
//! all in one application call.

use super::error::ApplicationError;
use super::models::{
    content_fingerprint, AdvancedRecord, AnalysisRecord, ProjectEvent, ProjectEventSink,
};
use super::project::{atomic_write_bytes, emit_and_history, ProjectLayout};
use super::review::{load_canon, load_ledger, new_ledger, save_ledger};
use advanced_literary_analysis::{
    AdvancedAnalysisConfig, AdvancedAnalysisResult, CanonContext, MockAnalysisProvider,
    OpenAIAnalysisProvider, ADVANCED_ANALYSIS_SCHEMA_VERSION,
};
use chrono::Utc;
use document_engine::ingest_file;
use human_review_workflow::{LiteraryFindingProposal, ReviewProposal};
use literary_intelligence_engine::{
    AnalysisCanon, Confidence, ConfidenceLevel, DeterministicManuscriptAnalyzer, EvidenceRef,
    ManuscriptAnalyzer, ManuscriptIntelligence,
};

pub const DETERMINISTIC_ARTIFACT: &str = "deterministic.json";
pub const ADVANCED_ARTIFACT: &str = "advanced.json";

/// Application-facing advanced-analysis configuration (bounded knobs only).
#[derive(Debug, Clone)]
pub struct AdvancedAnalysisSettings {
    /// `mock` (default, offline) or `openai`.
    pub provider: String,
    pub model: Option<String>,
    pub max_units: Option<usize>,
    /// Persist the fingerprint-keyed unit cache in the project.
    pub cache: bool,
}

impl Default for AdvancedAnalysisSettings {
    fn default() -> Self {
        Self {
            provider: "mock".to_string(),
            model: None,
            max_units: None,
            cache: true,
        }
    }
}

/// Load the source manuscript from the project's immutable source copy.
pub fn load_manuscript(
    layout: &ProjectLayout,
) -> Result<document_engine::Manuscript, ApplicationError> {
    let manifest = super::project::load_manifest(layout)?;
    let record = manifest.source.as_ref().ok_or_else(|| {
        ApplicationError::InvalidProject("project has no imported source".to_string())
    })?;
    let source_path = layout
        .source_dir
        .join(record.stored_relative_path.trim_start_matches("source/"));
    ingest_file(&source_path)
        .map_err(|error| ApplicationError::ImportFailed(source_path, error.to_string()))
}

/// Deterministic Phase 13 analysis + Phase 14 reconcile in one application op.
pub fn analyze_book(
    layout: &ProjectLayout,
    sink: &mut dyn ProjectEventSink,
) -> Result<(ManuscriptIntelligence, AnalysisRecord), ApplicationError> {
    let manifest = super::project::load_manifest(layout)?;
    let project_id = manifest.project_id.clone();
    let source_record = manifest.source.as_ref().ok_or_else(|| {
        ApplicationError::InvalidProject("project has no imported source".to_string())
    })?;
    let manuscript = load_manuscript(layout)?;
    let (characters, glossary) = load_canon(layout)?;

    emit_and_history(
        layout,
        &project_id,
        sink,
        None,
        ProjectEvent::AnalysisStarted {
            project_id: project_id.clone(),
        },
    );

    let intelligence = DeterministicManuscriptAnalyzer::default()
        .analyze(
            &manuscript,
            AnalysisCanon {
                characters: &characters,
                glossary: &glossary,
            },
        )
        .map_err(|error| ApplicationError::AnalysisFailed(error.to_string()))?;

    // Persist the deterministic artifact.
    let bytes = serde_json::to_vec_pretty(&intelligence).map_err(|error| {
        ApplicationError::PersistenceFailure(format!(
            "failed to serialize deterministic analysis: {error}"
        ))
    })?;
    atomic_write_bytes(
        &layout.intelligence_dir.join(DETERMINISTIC_ARTIFACT),
        &bytes,
    )?;

    // Reconcile into the review ledger (create on first run).
    let mut ledger = if layout.review_file.exists() {
        load_ledger(layout)?
    } else {
        new_ledger(&intelligence)
    };
    let reconciliation = ledger
        .reconcile(&intelligence, Utc::now())
        .map_err(|error| ApplicationError::AnalysisFailed(error.to_string()))?;
    save_ledger(layout, &ledger)?;

    let record = AnalysisRecord {
        source_fingerprint: source_record.fingerprint.clone(),
        schema_version: intelligence.schema_version,
        analyzer: intelligence.analysis.analyzer.clone(),
        analyzed_at: Utc::now(),
        character_seeds: intelligence.character_seeds.len(),
        relationship_seeds: intelligence.relationship_seeds.len(),
        terminology_seeds: intelligence.terminology_seeds.len(),
        review_items_added: reconciliation.added.len(),
        review_items_unchanged: reconciliation.unchanged.len(),
    };

    let mut manifest = super::project::load_manifest(layout)?;
    manifest.analysis = Some(record.clone());
    manifest.updated_at = Utc::now();
    super::project::save_manifest(layout, &manifest)?;

    emit_and_history(
        layout,
        &project_id,
        sink,
        None,
        ProjectEvent::AnalysisCompleted {
            project_id: project_id.clone(),
            review_items_added: record.review_items_added,
        },
    );
    Ok((intelligence, record))
}

/// Load the persisted deterministic artifact (used to skip re-analysis during
/// translation when the source fingerprint still matches).
pub fn load_deterministic_artifact(
    layout: &ProjectLayout,
) -> Result<Option<ManuscriptIntelligence>, ApplicationError> {
    let path = layout.intelligence_dir.join(DETERMINISTIC_ARTIFACT);
    if !path.exists() {
        return Ok(None);
    }
    let bytes = std::fs::read(&path).map_err(|error| {
        ApplicationError::PersistenceFailure(format!("failed to read {path:?}: {error}"))
    })?;
    let intelligence: ManuscriptIntelligence = serde_json::from_slice(&bytes).map_err(|error| {
        ApplicationError::PersistenceFailure(format!("invalid deterministic artifact: {error}"))
    })?;
    Ok(Some(intelligence))
}

/// Phase 15 advanced analysis: bounded units, evidence validation, cache, and
/// literary review reconciliation.
pub fn run_advanced_analysis(
    layout: &ProjectLayout,
    settings: &AdvancedAnalysisSettings,
    sink: &mut dyn ProjectEventSink,
) -> Result<AdvancedAnalysisResult, ApplicationError> {
    let manifest = super::project::load_manifest(layout)?;
    let project_id = manifest.project_id.clone();
    let source_record = manifest.source.as_ref().ok_or_else(|| {
        ApplicationError::InvalidProject("project has no imported source".to_string())
    })?;
    let manuscript = load_manuscript(layout)?;
    let (characters, glossary) = load_canon(layout)?;

    // Deterministic analysis first — Phase 15 always enriches Phase 13, and
    // the review ledger must be bound to this manuscript before advanced
    // proposals can reconcile into it.
    let (intelligence, _) = analyze_book(layout, sink)?;

    emit_and_history(
        layout,
        &project_id,
        sink,
        None,
        ProjectEvent::AdvancedAnalysisStarted {
            project_id: project_id.clone(),
        },
    );

    let provider: Box<dyn advanced_literary_analysis::LiteraryAnalysisProvider> =
        match settings.provider.as_str() {
            "mock" => Box::new(MockAnalysisProvider::with_default_findings()),
            "openai" => Box::new(
                OpenAIAnalysisProvider::from_env()
                    .map_err(|error| ApplicationError::ProviderNotConfigured(error.to_string()))?,
            ),
            other => {
                return Err(ApplicationError::ProviderNotConfigured(format!(
                    "unsupported analysis provider '{other}'; expected mock or openai"
                )))
            }
        };

    let canon = CanonContext {
        character_names: characters
            .profiles()
            .iter()
            .map(|profile| profile.name.clone())
            .collect(),
        character_aliases: characters
            .aliases()
            .iter()
            .map(|alias| alias.alias.clone())
            .collect(),
        glossary_terms: glossary
            .entries()
            .iter()
            .map(|entry| entry.source_term.clone())
            .collect(),
        relationship_pairs: Vec::new(),
    };

    let mut config = AdvancedAnalysisConfig::default();
    if let Some(max_units) = settings.max_units {
        config.planner.max_units = max_units;
    }
    if settings.cache {
        config.cache = Some(layout.intelligence_dir.join("advanced-cache.json"));
    }

    let result = advanced_literary_analysis::run_advanced_analysis(
        &manuscript,
        &canon,
        provider.as_ref(),
        &config,
        Utc::now(),
    )
    .map_err(|error| ApplicationError::AdvancedAnalysisFailed(error.to_string()))?;

    // Persist the advanced artifact.
    let bytes = serde_json::to_vec_pretty(&result).map_err(|error| {
        ApplicationError::PersistenceFailure(format!(
            "failed to serialize advanced analysis: {error}"
        ))
    })?;
    atomic_write_bytes(&layout.intelligence_dir.join(ADVANCED_ARTIFACT), &bytes)?;

    // Map validated findings to review-only Literary proposals and reconcile.
    let proposals = result
        .findings
        .iter()
        .filter(|finding| finding.is_review_eligible)
        .map(literary_proposal_from_finding)
        .collect::<Vec<_>>();
    let mut ledger = if layout.review_file.exists() {
        load_ledger(layout)?
    } else {
        new_ledger(&intelligence)
    };
    let reconciliation = ledger
        .reconcile_advanced(
            proposals,
            ADVANCED_ANALYSIS_SCHEMA_VERSION,
            format!(
                "advanced:{}/{}",
                result.provider_metadata.provider_name, result.provider_metadata.model
            ),
            result.provider_metadata.prompt_version.clone(),
            Utc::now(),
        )
        .map_err(|error| ApplicationError::AdvancedAnalysisFailed(error.to_string()))?;
    save_ledger(layout, &ledger)?;

    let record = AdvancedRecord {
        source_fingerprint: source_record.fingerprint.clone(),
        provider: result.provider_metadata.provider_name.clone(),
        model: result.provider_metadata.model.clone(),
        prompt_version: result.provider_metadata.prompt_version.clone(),
        analyzed_at: Utc::now(),
        total_units: result.total_units,
        succeeded_units: result.succeeded_units,
        cached_units: result.cached_units,
        failed_units: result.failed_units.len(),
        findings: result.findings.len(),
        review_items_added: reconciliation.added.len(),
        review_items_unchanged: reconciliation.unchanged.len(),
    };

    let mut manifest = super::project::load_manifest(layout)?;
    manifest.advanced = Some(record);
    manifest.updated_at = Utc::now();
    super::project::save_manifest(layout, &manifest)?;

    emit_and_history(
        layout,
        &project_id,
        sink,
        None,
        ProjectEvent::AdvancedAnalysisCompleted {
            project_id: project_id.clone(),
            findings: result.findings.len(),
            cached_units: result.cached_units,
            failed_units: result.failed_units.len(),
        },
    );
    Ok(result)
}

fn literary_proposal_from_finding(
    finding: &advanced_literary_analysis::AdvancedLiteraryFinding,
) -> ReviewProposal {
    let level = match finding.confidence.level {
        advanced_literary_analysis::ConfidenceLevel::Low => ConfidenceLevel::Low,
        advanced_literary_analysis::ConfidenceLevel::Moderate => ConfidenceLevel::Moderate,
        advanced_literary_analysis::ConfidenceLevel::High => ConfidenceLevel::High,
    };
    ReviewProposal::Literary(Box::new(LiteraryFindingProposal {
        id: finding.finding_id.clone(),
        finding_category: finding.category.to_string(),
        subject: finding.subject.clone(),
        claim: finding.claim.clone(),
        scope_label: finding.scope.label(),
        confidence: Confidence {
            level,
            supporting_evidence: finding.evidence.len(),
        },
        evidence: finding
            .evidence
            .iter()
            .map(|reference| EvidenceRef {
                chapter_id: reference.chapter_id.clone(),
                scene_id: reference.scene_id.clone(),
                paragraph_id: reference.paragraph_id.clone(),
                source: reference.source.clone(),
            })
            .collect(),
        analysis_unit_id: finding.analysis_unit_id.clone(),
        alternative_interpretations: finding.alternative_interpretations.clone(),
    }))
}

/// Recompute the source fingerprint from the current source copy.
pub fn source_fingerprint(layout: &ProjectLayout) -> Result<Option<String>, ApplicationError> {
    let manifest = super::project::load_manifest(layout)?;
    let Some(record) = manifest.source.as_ref() else {
        return Ok(None);
    };
    let source_path = layout
        .source_dir
        .join(record.stored_relative_path.trim_start_matches("source/"));
    let bytes = std::fs::read(&source_path).map_err(|error| {
        ApplicationError::PersistenceFailure(format!("failed to read source: {error}"))
    })?;
    Ok(Some(content_fingerprint(&bytes)))
}
