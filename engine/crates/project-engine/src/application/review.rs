//! Phase 14 review + canon integration through the application boundary.
//!
//! These wrappers reuse `human-review-workflow` lifecycle and `review_store`
//! atomic persistence exactly. The application layer never re-implements
//! review validation and never creates a shortcut that bypasses
//! preview/fingerprint checks.

use super::error::ApplicationError;
use super::models::ReviewItemSummary;
use super::project::ProjectLayout;
pub(crate) use crate::review_store as store;
use chrono::{DateTime, Utc};
use human_review_workflow::{
    build_promotion_plan, CanonPromotionPlan, ConflictResolution, DecisionAction, ReviewLedger,
    ReviewProposal, ReviewedValue,
};
use literary_intelligence_engine::ManuscriptIntelligence;

/// Load the review ledger, creating a placeholder-free error if analysis has
/// not run yet (the ledger is bound to a manuscript ID).
pub fn load_ledger(layout: &ProjectLayout) -> Result<ReviewLedger, ApplicationError> {
    if !layout.review_file.exists() {
        return Err(ApplicationError::AnalysisFailed(
            "no review ledger exists yet; run analysis first".to_string(),
        ));
    }
    store::load_review_ledger(&layout.review_file)
        .map_err(|error| ApplicationError::PersistenceFailure(error.to_string()))
}

pub fn save_ledger(layout: &ProjectLayout, ledger: &ReviewLedger) -> Result<(), ApplicationError> {
    store::save_review_ledger(&layout.review_file, ledger)
        .map_err(|error| ApplicationError::PersistenceFailure(error.to_string()))
}

fn proposal_subject(proposal: &ReviewProposal) -> String {
    match proposal {
        ReviewProposal::Character(value) => value.canonical_name_candidate.clone(),
        ReviewProposal::Relationship(value) => {
            format!("{} / {}", value.character_a, value.character_b)
        }
        ReviewProposal::Terminology(value) => value.source_expression.clone(),
        ReviewProposal::Literary(value) => value.subject.clone(),
    }
}

pub fn to_summary(ledger: &ReviewLedger, id: &str) -> Result<ReviewItemSummary, ApplicationError> {
    let item = ledger
        .item(id)
        .map_err(|error| ApplicationError::ReviewConflict(error.to_string()))?;
    let proposal = &item.latest_proposal;
    Ok(ReviewItemSummary {
        id: item.id.clone(),
        kind: format!("{:?}", item.kind).to_ascii_lowercase(),
        status: format!("{:?}", item.status).to_ascii_lowercase(),
        availability: format!("{:?}", item.availability).to_ascii_lowercase(),
        revision: item.revision,
        subject: proposal_subject(proposal),
        confidence: format!("{:?}", proposal.confidence().level).to_ascii_lowercase(),
        evidence_count: proposal.evidence().len(),
    })
}

/// List review items as UI-facing summaries.
pub fn list_review_items(
    layout: &ProjectLayout,
    kind_filter: Option<&str>,
    status_filter: Option<&str>,
) -> Result<Vec<ReviewItemSummary>, ApplicationError> {
    let ledger = load_ledger(layout)?;
    let mut items = ledger
        .items
        .iter()
        .filter(|item| {
            kind_filter
                .is_none_or(|filter| format!("{:?}", item.kind).to_ascii_lowercase() == filter)
        })
        .filter(|item| {
            status_filter
                .is_none_or(|filter| format!("{:?}", item.status).to_ascii_lowercase() == filter)
        })
        .filter_map(|item| to_summary(&ledger, &item.id).ok())
        .collect::<Vec<_>>();
    items.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(items)
}

pub fn get_review_item(
    layout: &ProjectLayout,
    id: &str,
) -> Result<ReviewItemSummary, ApplicationError> {
    let ledger = load_ledger(layout)?;
    to_summary(&ledger, id)
}

/// Apply a review decision through the real Phase 14 lifecycle.
#[allow(clippy::too_many_arguments)]
pub fn decide_review_item(
    layout: &ProjectLayout,
    id: &str,
    action: DecisionAction,
    replacement: Option<ReviewedValue>,
    reviewer: &str,
    reason: &str,
    applied_at: DateTime<Utc>,
) -> Result<ReviewItemSummary, ApplicationError> {
    let mut ledger = load_ledger(layout)?;
    ledger
        .decide(
            id,
            action,
            replacement,
            reviewer.to_string(),
            reason.to_string(),
            applied_at,
        )
        .map_err(|error| ApplicationError::ReviewConflict(error.to_string()))?;
    save_ledger(layout, &ledger)?;
    get_review_item(layout, id)
}

/// Build a promotion plan (dry run, no mutation). Reuses Phase 14 conflict
/// detection and fingerprint binding exactly.
pub fn preview_promotion(
    layout: &ProjectLayout,
    selected_ids: &[String],
    resolutions: &[ConflictResolution],
) -> Result<CanonPromotionPlan, ApplicationError> {
    let ledger = load_ledger(layout)?;
    let (characters, glossary) = load_canon(layout)?;
    build_promotion_plan(&ledger, &characters, &glossary, selected_ids, resolutions)
        .map_err(|error| ApplicationError::PromotionBlocked(error.to_string()))
}

/// Apply a previously previewed promotion plan through the atomic, journaled
/// Phase 14 promotion path.
pub fn apply_promotion(
    layout: &ProjectLayout,
    plan: &CanonPromotionPlan,
    reviewer: &str,
    reason: &str,
    applied_at: DateTime<Utc>,
) -> Result<store::PromotionApplyResult, ApplicationError> {
    let ledger = load_ledger(layout)?;
    let paths = store::ReviewStorePaths {
        review_file: layout.review_file.clone(),
        character_bible_file: layout.characters_file.clone(),
        glossary_file: layout.glossary_file.clone(),
    };
    store::apply_promotion(&paths, &ledger, plan, reviewer, reason, applied_at)
        .map_err(|error| ApplicationError::PromotionBlocked(error.to_string()))
}

// ---------------------------------------------------------------------------
// Character Bible + Glossary read/edit APIs (safe canonical owner APIs only)
// ---------------------------------------------------------------------------

pub fn load_canon(
    layout: &ProjectLayout,
) -> Result<
    (
        character_engine::CharacterBible,
        memory_engine::glossary::Glossary,
    ),
    ApplicationError,
> {
    let characters = match character_engine::CharacterBible::load_json(&layout.characters_file) {
        Ok(bible) => bible,
        Err(_) => character_engine::CharacterBible::new(),
    };
    let glossary = memory_engine::load_glossary(&layout.glossary_file).unwrap_or_default();
    Ok((characters, glossary))
}

pub fn list_characters(
    layout: &ProjectLayout,
) -> Result<Vec<character_engine::CharacterProfile>, ApplicationError> {
    let (bible, _) = load_canon(layout)?;
    Ok(bible.profiles().to_vec())
}

pub fn get_character(
    layout: &ProjectLayout,
    name: &str,
) -> Result<Option<character_engine::CharacterProfile>, ApplicationError> {
    let (bible, _) = load_canon(layout)?;
    Ok(bible.find_profile(name).cloned())
}

/// Upsert a character through `CharacterBible::upsert_profile` (normalized
/// duplicate/alias rules are enforced by the canonical owner).
pub fn upsert_character(
    layout: &ProjectLayout,
    profile: character_engine::CharacterProfile,
    replace_existing: bool,
) -> Result<(), ApplicationError> {
    let (mut bible, _) = load_canon(layout)?;
    bible
        .upsert_profile(profile, replace_existing)
        .map_err(|error| ApplicationError::ReviewConflict(error.to_string()))?;
    let bytes = serde_json::to_vec_pretty(&bible).map_err(|error| {
        ApplicationError::PersistenceFailure(format!(
            "failed to serialize Character Bible: {error}"
        ))
    })?;
    super::project::atomic_write_bytes(&layout.characters_file, &bytes)
}

pub fn list_glossary_entries(
    layout: &ProjectLayout,
) -> Result<Vec<memory_engine::glossary::GlossaryEntry>, ApplicationError> {
    let (_, glossary) = load_canon(layout)?;
    Ok(glossary.entries().to_vec())
}

pub fn get_glossary_entry(
    layout: &ProjectLayout,
    source_term: &str,
) -> Result<Option<memory_engine::glossary::GlossaryEntry>, ApplicationError> {
    let (_, glossary) = load_canon(layout)?;
    Ok(glossary.find_exact_term(source_term).cloned())
}

/// Upsert a glossary entry through the canonical owner's normalized,
/// conflict-checked upsert.
pub fn upsert_glossary_entry(
    layout: &ProjectLayout,
    entry: memory_engine::glossary::GlossaryEntry,
    replace_existing: bool,
) -> Result<(), ApplicationError> {
    let (_, mut glossary) = load_canon(layout)?;
    glossary
        .upsert(entry, replace_existing)
        .map_err(|error| ApplicationError::ReviewConflict(error.to_string()))?;
    memory_engine::save_glossary(&layout.glossary_file, &glossary).map_err(|error| {
        ApplicationError::PersistenceFailure(format!("failed to save Glossary: {error}"))
    })
}

/// Create a fresh ledger bound to the manuscript intelligence (used by the
/// analysis orchestration when no ledger exists yet).
pub fn new_ledger(intelligence: &ManuscriptIntelligence) -> ReviewLedger {
    ReviewLedger::new(intelligence)
}

pub(crate) fn _unused(_: Utc) {}
