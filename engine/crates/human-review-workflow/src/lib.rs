use character_engine::{CharacterBible, CharacterProfile, RelationshipProfile};
use chrono::{DateTime, Utc};
use literary_intelligence_engine::{
    CharacterSeed, Confidence, ConfidenceLevel, EvidenceRef, ManuscriptIntelligence,
    RelationshipSeed, TerminologyCategory, TerminologySeed,
};
use memory_engine::glossary::{Glossary, GlossaryEntry};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

pub const REVIEW_LEDGER_SCHEMA_VERSION: u32 = 1;
pub const PROMOTION_PLAN_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Error)]
pub enum ReviewError {
    #[error("unsupported review ledger schema version {0}")]
    UnsupportedSchema(u32),
    #[error("review ledger belongs to manuscript '{expected}', not '{actual}'")]
    ManuscriptMismatch { expected: String, actual: String },
    #[error("review item '{0}' was not found")]
    ItemNotFound(String),
    #[error("invalid transition for review item '{item_id}': {from:?} -> {to:?}")]
    InvalidTransition {
        item_id: String,
        from: ReviewStatus,
        to: ReviewStatus,
    },
    #[error("review item '{0}' is stale or obsolete")]
    StaleProposal(String),
    #[error("invalid reviewed value: {0}")]
    Validation(String),
    #[error("review item kind does not match replacement value")]
    ReplacementKindMismatch,
    #[error("promotion plan is blocked by unresolved conflicts")]
    BlockingConflicts,
    #[error("promotion plan '{0}' is stale")]
    StalePlan(String),
    #[error("promotion plan '{0}' was already applied")]
    AlreadyApplied(String),
    #[error("unsupported promotion: {0}")]
    UnsupportedPromotion(String),
    #[error("canonical mutation failed: {0}")]
    Canon(String),
    #[error("serialization failed: {0}")]
    Serialization(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewKind {
    Character,
    Relationship,
    Terminology,
    /// Advanced model-assisted literary finding (voice, tone, POV, subtext, …).
    /// Reviewable through the standard lifecycle but never promotable to canon
    /// because no canonical owner exists for these dimensions.
    Literary,
}

impl ReviewKind {
    /// Kinds whose reviewed value can be promoted into a canonical owner.
    pub fn supports_canon_promotion(self) -> bool {
        matches!(
            self,
            Self::Character | Self::Relationship | Self::Terminology
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewStatus {
    Pending,
    Approved,
    Edited,
    Rejected,
    Deferred,
    Applied,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProposalAvailability {
    Active,
    Obsolete,
}

/// A reviewable advanced literary finding (produced by provider-assisted
/// analysis and validated before it reaches the queue). Literary proposals
/// carry no canonical owner: approving one records a human-reviewed literary
/// observation, not canon.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiteraryFindingProposal {
    /// Stable finding identity derived from analysis unit, category,
    /// normalized subject/claim, and scope — never from raw provider prose.
    pub id: String,
    /// Snake_case finding category (e.g. `character_voice`, `tone`).
    pub finding_category: String,
    /// Target of the observation (character, relationship pair, scene label).
    pub subject: String,
    /// Concise evidence-backed claim.
    pub claim: String,
    /// Human-readable narrative scope (chapter/scene/range/global).
    pub scope_label: String,
    pub confidence: Confidence,
    pub evidence: Vec<EvidenceRef>,
    pub analysis_unit_id: String,
    pub alternative_interpretations: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "proposal", rename_all = "snake_case")]
pub enum ReviewProposal {
    Character(Box<CharacterSeed>),
    Relationship(Box<RelationshipSeed>),
    Terminology(Box<TerminologySeed>),
    Literary(Box<LiteraryFindingProposal>),
}

impl ReviewProposal {
    pub fn kind(&self) -> ReviewKind {
        match self {
            Self::Character(_) => ReviewKind::Character,
            Self::Relationship(_) => ReviewKind::Relationship,
            Self::Terminology(_) => ReviewKind::Terminology,
            Self::Literary(_) => ReviewKind::Literary,
        }
    }

    pub fn proposal_id(&self) -> &str {
        match self {
            Self::Character(value) => &value.id,
            Self::Relationship(value) => &value.id,
            Self::Terminology(value) => &value.id,
            Self::Literary(value) => &value.id,
        }
    }

    pub fn confidence(&self) -> &Confidence {
        match self {
            Self::Character(value) => &value.confidence,
            Self::Relationship(value) => &value.confidence,
            Self::Terminology(value) => &value.confidence,
            Self::Literary(value) => &value.confidence,
        }
    }

    pub fn evidence(&self) -> &[EvidenceRef] {
        match self {
            Self::Character(value) => &value.evidence,
            Self::Relationship(value) => &value.evidence,
            Self::Terminology(value) => &value.evidence,
            Self::Literary(value) => &value.evidence,
        }
    }

    fn already_canonical(&self) -> bool {
        match self {
            Self::Character(value) => value.matches_approved_character,
            Self::Relationship(value) => value.matches_approved_relationship,
            Self::Terminology(value) => value.approved_translation.is_some(),
            // Literary findings are never canonicalized; they remain
            // human-reviewed observations only.
            Self::Literary(_) => false,
        }
    }

    fn semantic_fingerprint(&self) -> Result<String, ReviewError> {
        #[derive(Serialize)]
        struct CharacterSemantic<'a> {
            name: String,
            aliases: Vec<String>,
            narrative_role: &'a Option<String>,
            speech_register: &'a Vec<String>,
            lexical_patterns: &'a Vec<String>,
            personality: &'a Vec<String>,
        }
        #[derive(Serialize)]
        struct RelationshipSemantic<'a> {
            a: String,
            b: String,
            label: &'a Option<String>,
            forms_of_address: &'a Vec<String>,
        }
        #[derive(Serialize)]
        struct TerminologySemantic {
            source: String,
            category: TerminologyCategory,
        }
        let bytes = match self {
            Self::Character(value) => serde_json::to_vec(&CharacterSemantic {
                name: normalize(&value.canonical_name_candidate),
                aliases: normalized_sorted(&value.aliases),
                narrative_role: &value.likely_narrative_role,
                speech_register: &value.speech_register_observations,
                lexical_patterns: &value.recurring_lexical_patterns,
                personality: &value.personality_observations,
            })?,
            Self::Relationship(value) => {
                let (a, b) = normalized_pair(&value.character_a, &value.character_b);
                serde_json::to_vec(&RelationshipSemantic {
                    a,
                    b,
                    label: &value.relationship_label_candidate,
                    forms_of_address: &value.forms_of_address,
                })?
            }
            Self::Terminology(value) => serde_json::to_vec(&TerminologySemantic {
                source: normalize(&value.source_expression),
                category: value.category,
            })?,
            Self::Literary(value) => {
                #[derive(Serialize)]
                struct LiterarySemantic {
                    category: String,
                    subject: String,
                    claim: String,
                    scope: String,
                }
                serde_json::to_vec(&LiterarySemantic {
                    category: normalize(&value.finding_category),
                    subject: normalize(&value.subject),
                    claim: normalize(&value.claim),
                    scope: normalize(&value.scope_label),
                })?
            }
        };
        Ok(stable_hash(&bytes))
    }

    pub fn default_reviewed_value(&self) -> ReviewedValue {
        match self {
            Self::Character(value) => ReviewedValue::Character(ReviewedCharacter {
                canonical_name: value.canonical_name_candidate.clone(),
                aliases: value.aliases.clone(),
                voice_notes: value.speech_register_observations.join("; "),
                personality_notes: value.personality_observations.join("; "),
            }),
            Self::Relationship(value) => ReviewedValue::Relationship(ReviewedRelationship {
                character_a: value.character_a.clone(),
                character_b: value.character_b.clone(),
                dynamic_notes: value
                    .relationship_label_candidate
                    .clone()
                    .unwrap_or_default(),
                address_notes: value.forms_of_address.join("; "),
                boundaries_notes: String::new(),
            }),
            Self::Terminology(value) => ReviewedValue::Terminology(ReviewedTerminology {
                source_term: value.source_expression.clone(),
                preferred_translation: value.approved_translation.clone().unwrap_or_default(),
                context: format!("reviewed {:?} manuscript terminology", value.category),
            }),
            Self::Literary(value) => ReviewedValue::Literary(ReviewedLiteraryFinding {
                finding_category: value.finding_category.clone(),
                subject: value.subject.clone(),
                claim: value.claim.clone(),
                scope_label: value.scope_label.clone(),
                analysis_unit_id: value.analysis_unit_id.clone(),
                alternative_interpretations: value.alternative_interpretations.clone(),
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum ReviewedValue {
    Character(ReviewedCharacter),
    Relationship(ReviewedRelationship),
    Terminology(ReviewedTerminology),
    Literary(ReviewedLiteraryFinding),
}

impl ReviewedValue {
    pub fn kind(&self) -> ReviewKind {
        match self {
            Self::Character(_) => ReviewKind::Character,
            Self::Relationship(_) => ReviewKind::Relationship,
            Self::Terminology(_) => ReviewKind::Terminology,
            Self::Literary(_) => ReviewKind::Literary,
        }
    }

    pub fn validate(&self) -> Result<(), ReviewError> {
        match self {
            Self::Character(value) => {
                if normalize(&value.canonical_name).is_empty() {
                    return Err(ReviewError::Validation(
                        "canonical character name cannot be empty".into(),
                    ));
                }
                if value
                    .aliases
                    .iter()
                    .any(|alias| normalize(alias).is_empty())
                {
                    return Err(ReviewError::Validation(
                        "character aliases cannot be empty".into(),
                    ));
                }
            }
            Self::Relationship(value) => {
                let (a, b) = normalized_pair(&value.character_a, &value.character_b);
                if a.is_empty() || b.is_empty() || a == b {
                    return Err(ReviewError::Validation(
                        "relationship endpoints must be non-empty and distinct".into(),
                    ));
                }
                if value.dynamic_notes.trim().is_empty()
                    && value.address_notes.trim().is_empty()
                    && value.boundaries_notes.trim().is_empty()
                {
                    return Err(ReviewError::Validation(
                        "a relationship needs reviewed dynamics, address, or boundary notes".into(),
                    ));
                }
            }
            Self::Terminology(value) => {
                if normalize(&value.source_term).is_empty() {
                    return Err(ReviewError::Validation(
                        "terminology source expression cannot be empty".into(),
                    ));
                }
                if normalize(&value.preferred_translation).is_empty() {
                    return Err(ReviewError::Validation(
                        "terminology approval requires a preferred translation".into(),
                    ));
                }
            }
            Self::Literary(value) => {
                if normalize(&value.finding_category).is_empty() {
                    return Err(ReviewError::Validation(
                        "literary finding category cannot be empty".into(),
                    ));
                }
                if normalize(&value.subject).is_empty() {
                    return Err(ReviewError::Validation(
                        "literary finding subject cannot be empty".into(),
                    ));
                }
                if normalize(&value.claim).is_empty() {
                    return Err(ReviewError::Validation(
                        "literary finding claim cannot be empty".into(),
                    ));
                }
                if value.claim.chars().count() > 2_000 {
                    return Err(ReviewError::Validation(
                        "literary finding claim is unreasonably large".into(),
                    ));
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewedCharacter {
    pub canonical_name: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub voice_notes: String,
    #[serde(default)]
    pub personality_notes: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewedRelationship {
    pub character_a: String,
    pub character_b: String,
    pub dynamic_notes: String,
    #[serde(default)]
    pub address_notes: String,
    #[serde(default)]
    pub boundaries_notes: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewedTerminology {
    pub source_term: String,
    pub preferred_translation: String,
    #[serde(default)]
    pub context: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewedLiteraryFinding {
    pub finding_category: String,
    pub subject: String,
    /// The claim approved/edited by the human reviewer.
    pub claim: String,
    pub scope_label: String,
    pub analysis_unit_id: String,
    #[serde(default)]
    pub alternative_interpretations: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionAction {
    Approve,
    Edit,
    Reject,
    Defer,
    Reopen,
    Apply,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewDecision {
    pub action: DecisionAction,
    pub reviewer: String,
    pub reason: String,
    pub decided_at: DateTime<Utc>,
    pub proposal_revision: u32,
    pub reviewed_value: Option<ReviewedValue>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArchivedRevision {
    pub revision: u32,
    pub semantic_fingerprint: String,
    pub proposal: ReviewProposal,
    pub reviewed_value: Option<ReviewedValue>,
    pub status: ReviewStatus,
    pub decisions: Vec<ReviewDecision>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewItem {
    pub id: String,
    pub proposal_id: String,
    pub kind: ReviewKind,
    pub subject_key: String,
    pub revision: u32,
    pub semantic_fingerprint: String,
    pub original_proposal: ReviewProposal,
    pub latest_proposal: ReviewProposal,
    pub reviewed_value: Option<ReviewedValue>,
    pub status: ReviewStatus,
    pub availability: ProposalAvailability,
    pub decisions: Vec<ReviewDecision>,
    pub archived_revisions: Vec<ArchivedRevision>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReconciliationRecord {
    pub analyzed_at: DateTime<Utc>,
    pub analyzer: String,
    pub analyzer_version: String,
    pub added: Vec<String>,
    pub unchanged: Vec<String>,
    pub changed: Vec<String>,
    pub obsolete: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromotionRecord {
    pub plan_id: String,
    pub item_ids: Vec<String>,
    pub applied_at: DateTime<Utc>,
    pub plan: CanonPromotionPlan,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewLedger {
    pub schema_version: u32,
    pub manuscript_id: String,
    pub manuscript_title: String,
    pub analysis_schema_version: u32,
    pub items: Vec<ReviewItem>,
    pub reconciliations: Vec<ReconciliationRecord>,
    pub promotions: Vec<PromotionRecord>,
}

impl ReviewLedger {
    pub fn new(intelligence: &ManuscriptIntelligence) -> Self {
        Self {
            schema_version: REVIEW_LEDGER_SCHEMA_VERSION,
            manuscript_id: intelligence.manuscript_id.clone(),
            manuscript_title: intelligence.manuscript_title.clone(),
            analysis_schema_version: intelligence.schema_version,
            items: Vec::new(),
            reconciliations: Vec::new(),
            promotions: Vec::new(),
        }
    }

    pub fn validate(&self) -> Result<(), ReviewError> {
        if self.schema_version != REVIEW_LEDGER_SCHEMA_VERSION {
            return Err(ReviewError::UnsupportedSchema(self.schema_version));
        }
        if self.manuscript_id.trim().is_empty() {
            return Err(ReviewError::Validation(
                "review ledger manuscript ID cannot be empty".into(),
            ));
        }
        let mut ids = BTreeSet::new();
        for item in &self.items {
            if !ids.insert(&item.id) {
                return Err(ReviewError::Validation(format!(
                    "duplicate review item ID '{}'",
                    item.id
                )));
            }
            if item.kind != item.latest_proposal.kind()
                || item.kind != item.original_proposal.kind()
            {
                return Err(ReviewError::Validation(format!(
                    "review item '{}' has inconsistent proposal kind",
                    item.id
                )));
            }
            if item.proposal_id != item.original_proposal.proposal_id()
                || item.subject_key.trim().is_empty()
                || item.revision == 0
                || item.semantic_fingerprint.trim().is_empty()
            {
                return Err(ReviewError::Validation(format!(
                    "review item '{}' has invalid identity metadata",
                    item.id
                )));
            }
            if matches!(
                item.status,
                ReviewStatus::Approved | ReviewStatus::Edited | ReviewStatus::Applied
            ) {
                let reviewed = item.reviewed_value.as_ref().ok_or_else(|| {
                    ReviewError::Validation(format!(
                        "review item '{}' has status {:?} without a reviewed value",
                        item.id, item.status
                    ))
                })?;
                if reviewed.kind() != item.kind {
                    return Err(ReviewError::ReplacementKindMismatch);
                }
                reviewed.validate()?;
            }
        }
        Ok(())
    }

    pub fn reconcile(
        &mut self,
        intelligence: &ManuscriptIntelligence,
        analyzed_at: DateTime<Utc>,
    ) -> Result<ReconciliationRecord, ReviewError> {
        self.validate()?;
        if self.manuscript_id != intelligence.manuscript_id {
            return Err(ReviewError::ManuscriptMismatch {
                expected: self.manuscript_id.clone(),
                actual: intelligence.manuscript_id.clone(),
            });
        }
        let incoming = intelligence
            .character_seeds
            .iter()
            .cloned()
            .map(|value| ReviewProposal::Character(Box::new(value)))
            .chain(
                intelligence
                    .relationship_seeds
                    .iter()
                    .cloned()
                    .map(|value| ReviewProposal::Relationship(Box::new(value))),
            )
            .chain(
                intelligence
                    .terminology_seeds
                    .iter()
                    .cloned()
                    .map(|value| ReviewProposal::Terminology(Box::new(value))),
            )
            .collect::<Vec<_>>();
        let record = self.reconcile_incoming(
            incoming,
            intelligence.schema_version,
            analyzed_at,
            intelligence.analysis.analyzer.clone(),
            intelligence.analysis.analyzer_version.clone(),
            &[
                ReviewKind::Character,
                ReviewKind::Relationship,
                ReviewKind::Terminology,
            ],
        )?;
        self.analysis_schema_version = intelligence.schema_version;
        self.manuscript_title = intelligence.manuscript_title.clone();
        Ok(record)
    }

    /// Reconcile advanced literary proposals from the optional provider-assisted
    /// layer into the same review ledger. Unchanged proposals keep their human
    /// decisions; materially changed proposals are archived and reset to
    /// Pending; missing proposals become Obsolete — exactly the Phase 14
    /// semantics, applied to model-assisted findings.
    pub fn reconcile_advanced(
        &mut self,
        proposals: Vec<ReviewProposal>,
        analysis_schema_version: u32,
        analyzer: String,
        analyzer_version: String,
        analyzed_at: DateTime<Utc>,
    ) -> Result<ReconciliationRecord, ReviewError> {
        self.validate()?;
        self.reconcile_incoming(
            proposals,
            analysis_schema_version,
            analyzed_at,
            analyzer,
            analyzer_version,
            &[ReviewKind::Literary],
        )
    }

    fn reconcile_incoming(
        &mut self,
        incoming: Vec<ReviewProposal>,
        analysis_schema_version: u32,
        analyzed_at: DateTime<Utc>,
        analyzer: String,
        analyzer_version: String,
        scope_kinds: &[ReviewKind],
    ) -> Result<ReconciliationRecord, ReviewError> {
        let mut existing_by_key = self
            .items
            .iter()
            .enumerate()
            .map(|(index, item)| ((item.kind, item.subject_key.clone()), index))
            .collect::<BTreeMap<_, _>>();
        let mut seen = BTreeSet::new();
        let mut record = ReconciliationRecord {
            analyzed_at,
            analyzer,
            analyzer_version,
            added: Vec::new(),
            unchanged: Vec::new(),
            changed: Vec::new(),
            obsolete: Vec::new(),
        };

        for proposal in incoming {
            let key = (proposal.kind(), proposal.proposal_id().to_string());
            seen.insert(key.clone());
            let semantic_fingerprint = proposal.semantic_fingerprint()?;
            if let Some(index) = existing_by_key.get(&key).copied() {
                let item = &mut self.items[index];
                let was_active = item.availability == ProposalAvailability::Active;
                if item.semantic_fingerprint == semantic_fingerprint {
                    item.latest_proposal = proposal;
                    record.unchanged.push(item.id.clone());
                } else {
                    item.archived_revisions.push(ArchivedRevision {
                        revision: item.revision,
                        semantic_fingerprint: item.semantic_fingerprint.clone(),
                        proposal: item.latest_proposal.clone(),
                        reviewed_value: item.reviewed_value.take(),
                        status: item.status,
                        decisions: std::mem::take(&mut item.decisions),
                    });
                    item.revision += 1;
                    item.semantic_fingerprint = semantic_fingerprint;
                    item.latest_proposal = proposal;
                    item.status = ReviewStatus::Pending;
                    record.changed.push(item.id.clone());
                }
                if item.latest_proposal.already_canonical() {
                    item.availability = ProposalAvailability::Obsolete;
                    if was_active {
                        record.obsolete.push(item.id.clone());
                    }
                } else {
                    item.availability = ProposalAvailability::Active;
                }
                continue;
            }
            if proposal.already_canonical() {
                continue;
            }
            let id = review_item_id(
                analysis_schema_version,
                &self.manuscript_id,
                proposal.kind(),
                proposal.proposal_id(),
            );
            self.items.push(ReviewItem {
                id: id.clone(),
                proposal_id: proposal.proposal_id().to_string(),
                kind: proposal.kind(),
                subject_key: proposal.proposal_id().to_string(),
                revision: 1,
                semantic_fingerprint,
                original_proposal: proposal.clone(),
                latest_proposal: proposal,
                reviewed_value: None,
                status: ReviewStatus::Pending,
                availability: ProposalAvailability::Active,
                decisions: Vec::new(),
                archived_revisions: Vec::new(),
            });
            existing_by_key.insert(key, self.items.len() - 1);
            record.added.push(id);
        }

        // Obsolete sweep is scoped to the proposal kinds this reconcile pass
        // owns, so a deterministic sync never obsoletes advanced Literary items
        // (and an advanced reconcile never obsoletes deterministic ones).
        for item in &mut self.items {
            if !scope_kinds.contains(&item.kind) {
                continue;
            }
            let key = (item.kind, item.subject_key.clone());
            if !seen.contains(&key) && item.availability == ProposalAvailability::Active {
                item.availability = ProposalAvailability::Obsolete;
                record.obsolete.push(item.id.clone());
            }
        }
        self.items.sort_by(|left, right| left.id.cmp(&right.id));
        for values in [
            &mut record.added,
            &mut record.unchanged,
            &mut record.changed,
            &mut record.obsolete,
        ] {
            values.sort();
        }
        self.reconciliations.push(record.clone());
        Ok(record)
    }

    pub fn item(&self, id: &str) -> Result<&ReviewItem, ReviewError> {
        self.items
            .iter()
            .find(|item| item.id == id)
            .ok_or_else(|| ReviewError::ItemNotFound(id.into()))
    }

    pub fn item_mut(&mut self, id: &str) -> Result<&mut ReviewItem, ReviewError> {
        self.items
            .iter_mut()
            .find(|item| item.id == id)
            .ok_or_else(|| ReviewError::ItemNotFound(id.into()))
    }

    pub fn decide(
        &mut self,
        id: &str,
        action: DecisionAction,
        replacement: Option<ReviewedValue>,
        reviewer: String,
        reason: String,
        decided_at: DateTime<Utc>,
    ) -> Result<&ReviewItem, ReviewError> {
        if reviewer.trim().is_empty() || reason.trim().is_empty() {
            return Err(ReviewError::Validation(
                "reviewer and reason are required".into(),
            ));
        }
        let item = self.item_mut(id)?;
        if item.availability == ProposalAvailability::Obsolete {
            return Err(ReviewError::StaleProposal(id.into()));
        }
        let (target, reviewed_value) = match action {
            DecisionAction::Approve => {
                let value = item.latest_proposal.default_reviewed_value();
                value.validate()?;
                (ReviewStatus::Approved, Some(value))
            }
            DecisionAction::Edit => {
                let value = replacement.ok_or_else(|| {
                    ReviewError::Validation("edit requires a replacement value".into())
                })?;
                if value.kind() != item.kind {
                    return Err(ReviewError::ReplacementKindMismatch);
                }
                value.validate()?;
                (ReviewStatus::Edited, Some(value))
            }
            DecisionAction::Reject => (ReviewStatus::Rejected, None),
            DecisionAction::Defer => (ReviewStatus::Deferred, None),
            DecisionAction::Reopen => (ReviewStatus::Pending, None),
            DecisionAction::Apply => {
                return Err(ReviewError::Validation(
                    "apply is only valid through canon promotion".into(),
                ))
            }
        };
        if !valid_transition(item.status, target, action) {
            return Err(ReviewError::InvalidTransition {
                item_id: id.into(),
                from: item.status,
                to: target,
            });
        }
        item.status = target;
        item.reviewed_value = reviewed_value.clone();
        item.decisions.push(ReviewDecision {
            action,
            reviewer,
            reason,
            decided_at,
            proposal_revision: item.revision,
            reviewed_value,
        });
        Ok(item)
    }

    pub fn already_applied(&self, plan_id: &str) -> bool {
        self.promotions
            .iter()
            .any(|record| record.plan_id == plan_id)
    }

    pub fn finalize_promotion(
        &mut self,
        plan: &CanonPromotionPlan,
        reviewer: &str,
        reason: &str,
        applied_at: DateTime<Utc>,
    ) -> Result<(), ReviewError> {
        if reviewer.trim().is_empty() || reason.trim().is_empty() {
            return Err(ReviewError::Validation(
                "reviewer and reason are required for promotion".into(),
            ));
        }
        if plan.schema_version != PROMOTION_PLAN_SCHEMA_VERSION {
            return Err(ReviewError::Validation(format!(
                "unsupported promotion plan schema {}",
                plan.schema_version
            )));
        }
        if plan.manuscript_id != self.manuscript_id {
            return Err(ReviewError::ManuscriptMismatch {
                expected: self.manuscript_id.clone(),
                actual: plan.manuscript_id.clone(),
            });
        }
        if plan.blocked {
            return Err(ReviewError::BlockingConflicts);
        }
        if self.already_applied(&plan.plan_id) {
            return Err(ReviewError::AlreadyApplied(plan.plan_id.clone()));
        }
        for id in &plan.promoted_item_ids {
            let item = self.item(id)?;
            if !matches!(item.status, ReviewStatus::Approved | ReviewStatus::Edited) {
                return Err(ReviewError::InvalidTransition {
                    item_id: id.clone(),
                    from: item.status,
                    to: ReviewStatus::Applied,
                });
            }
        }
        for id in &plan.promoted_item_ids {
            let item = self.item_mut(id)?;
            item.status = ReviewStatus::Applied;
            item.decisions.push(ReviewDecision {
                action: DecisionAction::Apply,
                reviewer: reviewer.into(),
                reason: reason.into(),
                decided_at: applied_at,
                proposal_revision: item.revision,
                reviewed_value: item.reviewed_value.clone(),
            });
        }
        self.promotions.push(PromotionRecord {
            plan_id: plan.plan_id.clone(),
            item_ids: plan.promoted_item_ids.clone(),
            applied_at,
            plan: plan.clone(),
        });
        Ok(())
    }
}

fn valid_transition(from: ReviewStatus, to: ReviewStatus, action: DecisionAction) -> bool {
    matches!(
        (from, to, action),
        (
            ReviewStatus::Pending,
            ReviewStatus::Approved
                | ReviewStatus::Edited
                | ReviewStatus::Rejected
                | ReviewStatus::Deferred,
            _,
        ) | (
            ReviewStatus::Deferred,
            ReviewStatus::Approved | ReviewStatus::Edited | ReviewStatus::Rejected,
            _,
        ) | (
            ReviewStatus::Approved | ReviewStatus::Edited | ReviewStatus::Rejected,
            ReviewStatus::Pending,
            DecisionAction::Reopen,
        )
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictSeverity {
    Informational,
    Warning,
    Blocking,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CanonConflictKind {
    CharacterIdentity,
    CharacterAliasCollision,
    CharacterProfile,
    GlossaryTranslation,
    RelationshipContradiction,
    StaleProposal,
    UnsupportedPromotion,
    AlreadyApplied,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResolutionKind {
    KeepCanon,
    UseReviewedProposal,
    Merge,
    CancelPromotion,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConflictResolution {
    pub conflict_id: String,
    pub resolution: ResolutionKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonConflict {
    pub id: String,
    pub kind: CanonConflictKind,
    pub severity: ConflictSeverity,
    pub item_id: String,
    pub proposal_revision: u32,
    pub affected_target: String,
    pub existing_value: Option<CanonicalValue>,
    pub proposed_value: Option<CanonicalValue>,
    pub evidence: Vec<EvidenceRef>,
    pub allowed_resolutions: Vec<ResolutionKind>,
    pub message: String,
    pub resolution: Option<ResolutionKind>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum CanonicalValue {
    Character(ReviewedCharacter),
    Relationship(ReviewedRelationship),
    Terminology(ReviewedTerminology),
    Alias {
        canonical_name: String,
        alias: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PromotionAction {
    Insert,
    Replace,
    Merge,
    KeepCanon,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonOperation {
    pub item_id: String,
    pub action: PromotionAction,
    pub target: String,
    pub before: Option<CanonicalValue>,
    pub after: Option<CanonicalValue>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonPromotionPlan {
    pub schema_version: u32,
    pub plan_id: String,
    pub manuscript_id: String,
    pub selected_item_ids: Vec<String>,
    pub promoted_item_ids: Vec<String>,
    pub cancelled_item_ids: Vec<String>,
    pub character_canon_fingerprint: String,
    pub glossary_canon_fingerprint: String,
    pub operations: Vec<CanonOperation>,
    pub conflicts: Vec<CanonConflict>,
    pub blocked: bool,
}

pub fn build_promotion_plan(
    ledger: &ReviewLedger,
    characters: &CharacterBible,
    glossary: &Glossary,
    selected_ids: &[String],
    resolutions: &[ConflictResolution],
) -> Result<CanonPromotionPlan, ReviewError> {
    ledger.validate()?;
    let selected = if selected_ids.is_empty() {
        ledger
            .items
            .iter()
            // Literary findings are review-only: they have no canonical owner,
            // so default selection never tries to promote them.
            .filter(|item| {
                matches!(item.status, ReviewStatus::Approved | ReviewStatus::Edited)
                    && item.kind.supports_canon_promotion()
            })
            .map(|item| item.id.clone())
            .collect::<Vec<_>>()
    } else {
        let mut ids = selected_ids.to_vec();
        ids.sort();
        ids.dedup();
        ids
    };
    let resolution_map = resolutions
        .iter()
        .map(|value| (value.conflict_id.clone(), value.resolution))
        .collect::<BTreeMap<_, _>>();
    let mut working_characters = characters.clone();
    let mut working_glossary = glossary.clone();
    let mut operations = Vec::new();
    let mut conflicts = Vec::new();
    let mut promoted = Vec::new();
    let mut cancelled = Vec::new();

    for id in &selected {
        let item = ledger.item(id)?;
        if item.status == ReviewStatus::Applied {
            conflicts.push(conflict(
                item,
                CanonConflictKind::AlreadyApplied,
                ConflictSeverity::Informational,
                id,
                None,
                item.reviewed_value
                    .as_ref()
                    .and_then(canonical_from_reviewed),
                Vec::new(),
                "proposal revision was already applied",
                None,
            ));
            continue;
        }
        if item.availability == ProposalAvailability::Obsolete {
            conflicts.push(conflict(
                item,
                CanonConflictKind::StaleProposal,
                ConflictSeverity::Blocking,
                id,
                None,
                item.reviewed_value
                    .as_ref()
                    .and_then(canonical_from_reviewed),
                vec![ResolutionKind::CancelPromotion],
                "proposal is no longer present in the latest analysis",
                None,
            ));
            continue;
        }
        if !matches!(item.status, ReviewStatus::Approved | ReviewStatus::Edited) {
            return Err(ReviewError::InvalidTransition {
                item_id: id.clone(),
                from: item.status,
                to: ReviewStatus::Applied,
            });
        }
        let reviewed = item.reviewed_value.as_ref().ok_or_else(|| {
            ReviewError::Validation(format!("review item '{}' has no approved value", item.id))
        })?;
        reviewed.validate()?;
        let start_conflicts = conflicts.len();
        plan_item(
            item,
            reviewed,
            &mut working_characters,
            &mut working_glossary,
            &resolution_map,
            &mut operations,
            &mut conflicts,
        )?;
        let item_conflicts = &conflicts[start_conflicts..];
        if item_conflicts
            .iter()
            .any(|value| value.resolution == Some(ResolutionKind::CancelPromotion))
        {
            cancelled.push(id.clone());
        } else if !item_conflicts
            .iter()
            .any(|value| value.severity == ConflictSeverity::Blocking && value.resolution.is_none())
        {
            promoted.push(id.clone());
        }
    }
    operations.sort_by(|a, b| {
        a.item_id
            .cmp(&b.item_id)
            .then_with(|| a.target.cmp(&b.target))
    });
    conflicts.sort_by(|a, b| a.id.cmp(&b.id));
    let blocked = conflicts
        .iter()
        .any(|value| value.severity == ConflictSeverity::Blocking && value.resolution.is_none());
    let character_canon_fingerprint = character_bible_fingerprint(characters)?;
    let glossary_canon_fingerprint = glossary_fingerprint(glossary)?;
    #[derive(Serialize)]
    struct PlanIdentity<'a> {
        schema_version: u32,
        manuscript_id: &'a str,
        selected: &'a [String],
        promoted: &'a [String],
        cancelled: &'a [String],
        character: &'a str,
        glossary: &'a str,
        operations: &'a [CanonOperation],
        conflicts: &'a [CanonConflict],
    }
    let plan_id = stable_hash(&serde_json::to_vec(&PlanIdentity {
        schema_version: PROMOTION_PLAN_SCHEMA_VERSION,
        manuscript_id: &ledger.manuscript_id,
        selected: &selected,
        promoted: &promoted,
        cancelled: &cancelled,
        character: &character_canon_fingerprint,
        glossary: &glossary_canon_fingerprint,
        operations: &operations,
        conflicts: &conflicts,
    })?);
    Ok(CanonPromotionPlan {
        schema_version: PROMOTION_PLAN_SCHEMA_VERSION,
        plan_id: format!("promotion-{plan_id}"),
        manuscript_id: ledger.manuscript_id.clone(),
        selected_item_ids: selected,
        promoted_item_ids: promoted,
        cancelled_item_ids: cancelled,
        character_canon_fingerprint,
        glossary_canon_fingerprint,
        operations,
        conflicts,
        blocked,
    })
}

#[allow(clippy::too_many_arguments)]
fn plan_item(
    item: &ReviewItem,
    reviewed: &ReviewedValue,
    characters: &mut CharacterBible,
    glossary: &mut Glossary,
    resolutions: &BTreeMap<String, ResolutionKind>,
    operations: &mut Vec<CanonOperation>,
    conflicts: &mut Vec<CanonConflict>,
) -> Result<(), ReviewError> {
    match reviewed {
        ReviewedValue::Character(value) => {
            let characters_before_item = characters.clone();
            if let Some(owner) = characters.find_alias_owner(&value.canonical_name) {
                if normalize(owner) != normalize(&value.canonical_name) {
                    let mut identity_conflict = conflict(
                        item,
                        CanonConflictKind::CharacterIdentity,
                        ConflictSeverity::Blocking,
                        &value.canonical_name,
                        Some(CanonicalValue::Alias {
                            canonical_name: owner.into(),
                            alias: value.canonical_name.clone(),
                        }),
                        Some(CanonicalValue::Character(value.clone())),
                        vec![ResolutionKind::CancelPromotion],
                        "proposed canonical name is already an alias of another character; edit the proposal or cancel promotion",
                        None,
                    );
                    apply_resolution(&mut identity_conflict, resolutions)?;
                    conflicts.push(identity_conflict);
                    if item_is_cancelled_or_blocked(item, conflicts) {
                        return Ok(());
                    }
                }
            }
            let existing = characters.find_profile(&value.canonical_name).cloned();
            let before = existing
                .as_ref()
                .map(|profile| CanonicalValue::Character(character_value(profile, characters)));
            let proposed = CanonicalValue::Character(value.clone());
            let mut action = PromotionAction::Insert;
            let mut final_value = value.clone();
            if let Some(profile) = existing {
                let current = character_value(&profile, characters);
                if !characters_semantically_equal(&current, value) {
                    let mut current_conflict = conflict(
                        item,
                        CanonConflictKind::CharacterProfile,
                        ConflictSeverity::Blocking,
                        &value.canonical_name,
                        Some(CanonicalValue::Character(current.clone())),
                        Some(proposed.clone()),
                        vec![
                            ResolutionKind::KeepCanon,
                            ResolutionKind::UseReviewedProposal,
                            ResolutionKind::Merge,
                            ResolutionKind::CancelPromotion,
                        ],
                        "canonical character profile differs from the reviewed proposal",
                        None,
                    );
                    apply_resolution(&mut current_conflict, resolutions)?;
                    match current_conflict.resolution {
                        Some(ResolutionKind::KeepCanon) => {
                            final_value = current;
                            action = PromotionAction::KeepCanon;
                        }
                        Some(ResolutionKind::UseReviewedProposal) => {
                            action = PromotionAction::Replace;
                        }
                        Some(ResolutionKind::Merge) => {
                            final_value = merge_character(&current, value);
                            action = PromotionAction::Merge;
                        }
                        Some(ResolutionKind::CancelPromotion) | None => {}
                    }
                    conflicts.push(current_conflict);
                } else {
                    final_value = current;
                    action = PromotionAction::KeepCanon;
                }
            }
            let item_cancelled = conflicts.iter().any(|value| {
                value.item_id == item.id
                    && value.resolution == Some(ResolutionKind::CancelPromotion)
            });
            if item_cancelled {
                return Ok(());
            }
            if !conflicts.iter().any(|value| {
                value.item_id == item.id
                    && value.severity == ConflictSeverity::Blocking
                    && value.resolution.is_none()
            }) {
                characters
                    .upsert_profile(
                        CharacterProfile {
                            name: final_value.canonical_name.clone(),
                            voice_notes: final_value.voice_notes.clone(),
                            personality_notes: final_value.personality_notes.clone(),
                        },
                        true,
                    )
                    .map_err(|error| ReviewError::Canon(error.to_string()))?;
                let mut accepted_aliases = Vec::new();
                for alias in normalized_unique_preserving(&final_value.aliases) {
                    if normalize(&alias) == normalize(&final_value.canonical_name) {
                        continue;
                    }
                    if let Some(owner) = characters.find_alias_owner(&alias) {
                        if normalize(owner) != normalize(&final_value.canonical_name) {
                            let mut alias_conflict = conflict(
                                item,
                                CanonConflictKind::CharacterAliasCollision,
                                ConflictSeverity::Blocking,
                                &alias,
                                Some(CanonicalValue::Alias {
                                    canonical_name: owner.into(),
                                    alias: alias.clone(),
                                }),
                                Some(CanonicalValue::Alias {
                                    canonical_name: final_value.canonical_name.clone(),
                                    alias: alias.clone(),
                                }),
                                vec![ResolutionKind::KeepCanon, ResolutionKind::CancelPromotion],
                                "alias already belongs to another canonical character",
                                None,
                            );
                            apply_resolution(&mut alias_conflict, resolutions)?;
                            conflicts.push(alias_conflict);
                            continue;
                        }
                    }
                    accepted_aliases.push(alias);
                }
                if item_is_cancelled_or_blocked(item, conflicts) {
                    *characters = characters_before_item;
                    return Ok(());
                }
                characters
                    .replace_aliases_checked(
                        final_value.canonical_name.clone(),
                        accepted_aliases.clone(),
                    )
                    .map_err(|error| ReviewError::Canon(error.to_string()))?;
                final_value.aliases = accepted_aliases;
                operations.push(CanonOperation {
                    item_id: item.id.clone(),
                    action,
                    target: format!("character:{}", normalize(&final_value.canonical_name)),
                    before,
                    after: Some(CanonicalValue::Character(final_value)),
                });
            }
        }
        ReviewedValue::Relationship(value) => {
            let existing = characters
                .find_relationship(&value.character_a, &value.character_b)
                .cloned();
            let before = existing
                .as_ref()
                .map(|current| CanonicalValue::Relationship(relationship_value(current)));
            let mut final_value = value.clone();
            let mut action = PromotionAction::Insert;
            if let Some(current) = existing.as_ref().map(relationship_value) {
                if relationships_semantically_equal(&current, value) {
                    final_value = current;
                    action = PromotionAction::KeepCanon;
                } else {
                    let mut current_conflict = conflict(
                        item,
                        CanonConflictKind::RelationshipContradiction,
                        ConflictSeverity::Blocking,
                        &format!("{}↔{}", value.character_a, value.character_b),
                        Some(CanonicalValue::Relationship(current.clone())),
                        Some(CanonicalValue::Relationship(value.clone())),
                        vec![
                            ResolutionKind::KeepCanon,
                            ResolutionKind::UseReviewedProposal,
                            ResolutionKind::Merge,
                            ResolutionKind::CancelPromotion,
                        ],
                        "canonical relationship differs from the reviewed proposal",
                        None,
                    );
                    apply_resolution(&mut current_conflict, resolutions)?;
                    match current_conflict.resolution {
                        Some(ResolutionKind::KeepCanon) => {
                            final_value = current;
                            action = PromotionAction::KeepCanon;
                        }
                        Some(ResolutionKind::UseReviewedProposal) => {
                            action = PromotionAction::Replace
                        }
                        Some(ResolutionKind::Merge) => {
                            final_value = merge_relationship(&current, value);
                            action = PromotionAction::Merge;
                        }
                        Some(ResolutionKind::CancelPromotion) | None => {}
                    }
                    conflicts.push(current_conflict);
                }
            }
            if !item_is_cancelled_or_blocked(item, conflicts) {
                characters
                    .upsert_relationship(
                        RelationshipProfile {
                            character_a: final_value.character_a.clone(),
                            character_b: final_value.character_b.clone(),
                            dynamic_notes: final_value.dynamic_notes.clone(),
                            address_notes: final_value.address_notes.clone(),
                            boundaries_notes: final_value.boundaries_notes.clone(),
                        },
                        true,
                    )
                    .map_err(|error| ReviewError::Canon(error.to_string()))?;
                operations.push(CanonOperation {
                    item_id: item.id.clone(),
                    action,
                    target: format!(
                        "relationship:{}↔{}",
                        normalized_pair(&value.character_a, &value.character_b).0,
                        normalized_pair(&value.character_a, &value.character_b).1
                    ),
                    before,
                    after: Some(CanonicalValue::Relationship(final_value)),
                });
            }
        }
        ReviewedValue::Terminology(value) => {
            let matching_entries = glossary.entries_for_term(&value.source_term);
            let distinct_translations: BTreeSet<_> = matching_entries
                .iter()
                .map(|entry| normalize(&entry.preferred_translation))
                .collect();
            let canon_is_internally_conflicted = distinct_translations.len() > 1;
            let existing = matching_entries.first().map(|entry| (*entry).clone());
            let before = existing
                .as_ref()
                .map(|entry| CanonicalValue::Terminology(terminology_value(entry)));
            let mut final_value = value.clone();
            let mut action = PromotionAction::Insert;
            if let Some(current) = existing.as_ref().map(terminology_value) {
                if !canon_is_internally_conflicted
                    && normalize(&current.preferred_translation)
                        == normalize(&value.preferred_translation)
                    && current.context == value.context
                {
                    action = PromotionAction::KeepCanon;
                } else {
                    let allowed_resolutions = if canon_is_internally_conflicted {
                        vec![
                            ResolutionKind::UseReviewedProposal,
                            ResolutionKind::CancelPromotion,
                        ]
                    } else {
                        vec![
                            ResolutionKind::KeepCanon,
                            ResolutionKind::UseReviewedProposal,
                            ResolutionKind::CancelPromotion,
                        ]
                    };
                    let mut current_conflict = conflict(
                        item,
                        CanonConflictKind::GlossaryTranslation,
                        ConflictSeverity::Blocking,
                        &value.source_term,
                        Some(CanonicalValue::Terminology(current.clone())),
                        Some(CanonicalValue::Terminology(value.clone())),
                        allowed_resolutions,
                        if canon_is_internally_conflicted {
                            "approved glossary contains conflicting translations for this term"
                        } else {
                            "approved glossary translation differs from the reviewed proposal"
                        },
                        None,
                    );
                    apply_resolution(&mut current_conflict, resolutions)?;
                    match current_conflict.resolution {
                        Some(ResolutionKind::KeepCanon) => {
                            final_value = current;
                            action = PromotionAction::KeepCanon;
                        }
                        Some(ResolutionKind::UseReviewedProposal) => {
                            action = PromotionAction::Replace
                        }
                        Some(ResolutionKind::CancelPromotion) | None => {}
                        Some(ResolutionKind::Merge) => {
                            return Err(ReviewError::Validation(
                                "glossary conflicts do not support merge".into(),
                            ))
                        }
                    }
                    conflicts.push(current_conflict);
                }
            }
            if !item_is_cancelled_or_blocked(item, conflicts) {
                glossary
                    .upsert(
                        GlossaryEntry {
                            source_term: final_value.source_term.clone(),
                            preferred_translation: final_value.preferred_translation.clone(),
                            context: final_value.context.clone(),
                        },
                        true,
                    )
                    .map_err(|error| ReviewError::Canon(error.to_string()))?;
                operations.push(CanonOperation {
                    item_id: item.id.clone(),
                    action,
                    target: format!("glossary:{}", normalize(&value.source_term)),
                    before,
                    after: Some(CanonicalValue::Terminology(final_value)),
                });
            }
        }
        ReviewedValue::Literary(value) => {
            // Literary findings are review-only. There is no canonical owner
            // for voice/tone/POV/subtext, so promotion is unsupported and any
            // explicit selection is surfaced as a blocking, cancellable
            // conflict instead of being silently dropped or fake-canonized.
            conflicts.push(conflict(
                item,
                CanonConflictKind::UnsupportedPromotion,
                ConflictSeverity::Blocking,
                &format!("{}: {}", value.finding_category, value.subject),
                None,
                None,
                vec![ResolutionKind::CancelPromotion],
                "literary review findings have no canonical owner and cannot be promoted to canon",
                None,
            ));
        }
    }
    Ok(())
}

fn item_is_cancelled_or_blocked(item: &ReviewItem, conflicts: &[CanonConflict]) -> bool {
    conflicts.iter().any(|value| {
        value.item_id == item.id
            && (value.resolution == Some(ResolutionKind::CancelPromotion)
                || (value.severity == ConflictSeverity::Blocking && value.resolution.is_none()))
    })
}

fn apply_resolution(
    conflict: &mut CanonConflict,
    resolutions: &BTreeMap<String, ResolutionKind>,
) -> Result<(), ReviewError> {
    if let Some(resolution) = resolutions.get(&conflict.id).copied() {
        if !conflict.allowed_resolutions.contains(&resolution) {
            return Err(ReviewError::Validation(format!(
                "resolution {resolution:?} is not allowed for conflict '{}'",
                conflict.id
            )));
        }
        conflict.resolution = Some(resolution);
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn conflict(
    item: &ReviewItem,
    kind: CanonConflictKind,
    severity: ConflictSeverity,
    target: &str,
    existing: Option<CanonicalValue>,
    proposed: Option<CanonicalValue>,
    allowed: Vec<ResolutionKind>,
    message: &str,
    resolution: Option<ResolutionKind>,
) -> CanonConflict {
    let identity = format!("{}\0{}\0{:?}\0{}", item.id, item.revision, kind, target);
    CanonConflict {
        id: format!("conflict-{}", stable_hash(identity.as_bytes())),
        kind,
        severity,
        item_id: item.id.clone(),
        proposal_revision: item.revision,
        affected_target: target.into(),
        existing_value: existing,
        proposed_value: proposed,
        evidence: item.latest_proposal.evidence().to_vec(),
        allowed_resolutions: allowed,
        message: message.into(),
        resolution,
    }
}

pub fn apply_plan_to_canon(
    plan: &CanonPromotionPlan,
    characters: &CharacterBible,
    glossary: &Glossary,
) -> Result<(CharacterBible, Glossary), ReviewError> {
    if plan.blocked {
        return Err(ReviewError::BlockingConflicts);
    }
    if plan.character_canon_fingerprint != character_bible_fingerprint(characters)?
        || plan.glossary_canon_fingerprint != glossary_fingerprint(glossary)?
    {
        return Err(ReviewError::StalePlan(plan.plan_id.clone()));
    }
    let mut next_characters = characters.clone();
    let mut next_glossary = glossary.clone();
    for operation in &plan.operations {
        let Some(after) = &operation.after else {
            continue;
        };
        match after {
            CanonicalValue::Character(value) => {
                next_characters
                    .upsert_profile(
                        CharacterProfile {
                            name: value.canonical_name.clone(),
                            voice_notes: value.voice_notes.clone(),
                            personality_notes: value.personality_notes.clone(),
                        },
                        true,
                    )
                    .map_err(|error| ReviewError::Canon(error.to_string()))?;
                next_characters
                    .replace_aliases_checked(value.canonical_name.clone(), value.aliases.clone())
                    .map_err(|error| ReviewError::Canon(error.to_string()))?;
            }
            CanonicalValue::Relationship(value) => {
                next_characters
                    .upsert_relationship(
                        RelationshipProfile {
                            character_a: value.character_a.clone(),
                            character_b: value.character_b.clone(),
                            dynamic_notes: value.dynamic_notes.clone(),
                            address_notes: value.address_notes.clone(),
                            boundaries_notes: value.boundaries_notes.clone(),
                        },
                        true,
                    )
                    .map_err(|error| ReviewError::Canon(error.to_string()))?;
            }
            CanonicalValue::Terminology(value) => {
                next_glossary
                    .upsert(
                        GlossaryEntry {
                            source_term: value.source_term.clone(),
                            preferred_translation: value.preferred_translation.clone(),
                            context: value.context.clone(),
                        },
                        true,
                    )
                    .map_err(|error| ReviewError::Canon(error.to_string()))?;
            }
            CanonicalValue::Alias {
                canonical_name,
                alias,
            } => {
                next_characters
                    .add_alias_checked(canonical_name.clone(), alias.clone())
                    .map_err(|error| ReviewError::Canon(error.to_string()))?;
            }
        }
    }
    Ok((next_characters, next_glossary))
}

pub fn character_bible_fingerprint(value: &CharacterBible) -> Result<String, ReviewError> {
    Ok(stable_hash(&serde_json::to_vec(value)?))
}

pub fn glossary_fingerprint(value: &Glossary) -> Result<String, ReviewError> {
    Ok(stable_hash(&serde_json::to_vec(value.entries())?))
}

fn review_item_id(
    analysis_schema: u32,
    manuscript_id: &str,
    kind: ReviewKind,
    proposal_id: &str,
) -> String {
    let identity = format!(
        "review-v{REVIEW_LEDGER_SCHEMA_VERSION}\0analysis-v{analysis_schema}\0{manuscript_id}\0{kind:?}\0{proposal_id}"
    );
    format!("review-{}", stable_hash(identity.as_bytes()))
}

fn stable_hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn normalize(value: &str) -> String {
    text_normalization::normalize_case_insensitive(value)
}

fn normalized_pair(a: &str, b: &str) -> (String, String) {
    let a = normalize(a);
    let b = normalize(b);
    if a <= b {
        (a, b)
    } else {
        (b, a)
    }
}

fn normalized_sorted(values: &[String]) -> Vec<String> {
    values
        .iter()
        .map(|value| normalize(value))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn normalized_unique_preserving(values: &[String]) -> Vec<String> {
    let mut seen = BTreeSet::new();
    values
        .iter()
        .filter(|value| seen.insert(normalize(value)))
        .cloned()
        .collect()
}

fn character_value(profile: &CharacterProfile, bible: &CharacterBible) -> ReviewedCharacter {
    ReviewedCharacter {
        canonical_name: profile.name.clone(),
        aliases: bible
            .aliases()
            .iter()
            .filter(|alias| normalize(&alias.canonical_name) == normalize(&profile.name))
            .map(|alias| alias.alias.clone())
            .collect(),
        voice_notes: profile.voice_notes.clone(),
        personality_notes: profile.personality_notes.clone(),
    }
}

fn relationship_value(value: &RelationshipProfile) -> ReviewedRelationship {
    ReviewedRelationship {
        character_a: value.character_a.clone(),
        character_b: value.character_b.clone(),
        dynamic_notes: value.dynamic_notes.clone(),
        address_notes: value.address_notes.clone(),
        boundaries_notes: value.boundaries_notes.clone(),
    }
}

fn terminology_value(value: &GlossaryEntry) -> ReviewedTerminology {
    ReviewedTerminology {
        source_term: value.source_term.clone(),
        preferred_translation: value.preferred_translation.clone(),
        context: value.context.clone(),
    }
}

fn canonical_from_reviewed(value: &ReviewedValue) -> Option<CanonicalValue> {
    match value {
        ReviewedValue::Character(value) => Some(CanonicalValue::Character(value.clone())),
        ReviewedValue::Relationship(value) => Some(CanonicalValue::Relationship(value.clone())),
        ReviewedValue::Terminology(value) => Some(CanonicalValue::Terminology(value.clone())),
        // Literary findings never map onto a canonical owner.
        ReviewedValue::Literary(_) => None,
    }
}

fn characters_semantically_equal(left: &ReviewedCharacter, right: &ReviewedCharacter) -> bool {
    normalize(&left.canonical_name) == normalize(&right.canonical_name)
        && normalized_sorted(&left.aliases) == normalized_sorted(&right.aliases)
        && left.voice_notes == right.voice_notes
        && left.personality_notes == right.personality_notes
}

fn relationships_semantically_equal(
    left: &ReviewedRelationship,
    right: &ReviewedRelationship,
) -> bool {
    normalized_pair(&left.character_a, &left.character_b)
        == normalized_pair(&right.character_a, &right.character_b)
        && left.dynamic_notes == right.dynamic_notes
        && left.address_notes == right.address_notes
        && left.boundaries_notes == right.boundaries_notes
}

fn merge_character(canon: &ReviewedCharacter, proposed: &ReviewedCharacter) -> ReviewedCharacter {
    let aliases = canon
        .aliases
        .iter()
        .chain(proposed.aliases.iter())
        .cloned()
        .collect::<Vec<_>>();
    ReviewedCharacter {
        canonical_name: canon.canonical_name.clone(),
        aliases: normalized_unique_preserving(&aliases),
        voice_notes: if canon.voice_notes.trim().is_empty() {
            proposed.voice_notes.clone()
        } else {
            canon.voice_notes.clone()
        },
        personality_notes: if canon.personality_notes.trim().is_empty() {
            proposed.personality_notes.clone()
        } else {
            canon.personality_notes.clone()
        },
    }
}

fn merge_relationship(
    canon: &ReviewedRelationship,
    proposed: &ReviewedRelationship,
) -> ReviewedRelationship {
    ReviewedRelationship {
        character_a: canon.character_a.clone(),
        character_b: canon.character_b.clone(),
        dynamic_notes: merge_note(&canon.dynamic_notes, &proposed.dynamic_notes),
        address_notes: merge_note(&canon.address_notes, &proposed.address_notes),
        boundaries_notes: merge_note(&canon.boundaries_notes, &proposed.boundaries_notes),
    }
}

fn merge_note(canon: &str, proposed: &str) -> String {
    match (
        canon.trim().is_empty(),
        proposed.trim().is_empty(),
        canon == proposed,
    ) {
        (true, _, _) => proposed.to_string(),
        (_, true, _) | (_, _, true) => canon.to_string(),
        _ => format!("{}; {}", canon.trim(), proposed.trim()),
    }
}

pub fn confidence_rank(level: ConfidenceLevel) -> u8 {
    match level {
        ConfidenceLevel::High => 0,
        ConfidenceLevel::Moderate => 1,
        ConfidenceLevel::Low => 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use document_engine::{Book, DocumentFormat, Manuscript, SourceLocation};
    use literary_intelligence_engine::{
        AnalysisMetadata, InferredLiteraryProfile, InitializationProposal, LiteraryProfile,
        ObservedLiteraryProfile, SeedStatus, MANUSCRIPT_INTELLIGENCE_SCHEMA_VERSION,
    };

    fn evidence() -> EvidenceRef {
        EvidenceRef {
            chapter_id: "chapter-1".into(),
            scene_id: "scene-1".into(),
            paragraph_id: "paragraph-1".into(),
            source: SourceLocation::new("synthetic.txt", DocumentFormat::Txt),
        }
    }

    fn intelligence(alias: &str) -> ManuscriptIntelligence {
        ManuscriptIntelligence {
            schema_version: MANUSCRIPT_INTELLIGENCE_SCHEMA_VERSION,
            manuscript_id: "book-1".into(),
            manuscript_title: "Synthetic".into(),
            literary_profile: LiteraryProfile {
                observed: ObservedLiteraryProfile {
                    paragraph_count: 1,
                    sentence_count: 1,
                    average_sentence_words: 1.0,
                    dialogue_density: 0.0,
                    prose_density: 1.0,
                    dialogue_conventions: Vec::new(),
                    code_switching_detected: false,
                    chapter_count: 1,
                    average_scenes_per_chapter: 1.0,
                },
                inferred: InferredLiteraryProfile::default(),
            },
            character_seeds: vec![CharacterSeed {
                id: "seed-character".into(),
                canonical_name_candidate: "Elizabeth Bennet".into(),
                aliases: vec![alias.into()],
                first_appearance: evidence(),
                appearance_count: 3,
                evidence: vec![evidence()],
                likely_narrative_role: None,
                speech_register_observations: Vec::new(),
                recurring_lexical_patterns: Vec::new(),
                personality_observations: Vec::new(),
                relationship_refs: Vec::new(),
                confidence: Confidence::from_evidence(3),
                status: SeedStatus::Inferred,
                matches_approved_character: false,
            }],
            relationship_seeds: Vec::new(),
            chapter_maps: Vec::new(),
            terminology_seeds: Vec::new(),
            analysis: AnalysisMetadata {
                analyzer: "test".into(),
                analyzer_version: "1".into(),
                deterministic: true,
                retained_evidence_limit: 8,
                character_seed_limit: 8,
                terminology_limit: 8,
            },
            source: SourceLocation::new("synthetic.txt", DocumentFormat::Txt),
            initialization: InitializationProposal {
                mutates_canon: false,
                chapters: Vec::new(),
                conflicts: Vec::new(),
            },
        }
    }

    fn timestamp() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-09-02T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    fn ledger_for_proposal(proposal: ReviewProposal) -> ReviewLedger {
        ReviewLedger {
            schema_version: REVIEW_LEDGER_SCHEMA_VERSION,
            manuscript_id: "book-1".into(),
            manuscript_title: "Synthetic".into(),
            analysis_schema_version: 1,
            items: vec![ReviewItem {
                id: "review-item".into(),
                proposal_id: proposal.proposal_id().into(),
                kind: proposal.kind(),
                subject_key: proposal.proposal_id().into(),
                revision: 1,
                semantic_fingerprint: "semantic".into(),
                original_proposal: proposal.clone(),
                latest_proposal: proposal,
                reviewed_value: None,
                status: ReviewStatus::Pending,
                availability: ProposalAvailability::Active,
                decisions: Vec::new(),
                archived_revisions: Vec::new(),
            }],
            reconciliations: Vec::new(),
            promotions: Vec::new(),
        }
    }

    #[test]
    fn lifecycle_and_changed_proposal_are_audited() {
        let first = intelligence("Lizzy");
        let mut ledger = ReviewLedger::new(&first);
        ledger.reconcile(&first, timestamp()).unwrap();
        let id = ledger.items[0].id.clone();
        ledger
            .decide(
                &id,
                DecisionAction::Reject,
                None,
                "editor".into(),
                "wrong identity".into(),
                timestamp(),
            )
            .unwrap();
        ledger.reconcile(&first, timestamp()).unwrap();
        assert_eq!(ledger.item(&id).unwrap().status, ReviewStatus::Rejected);

        let changed = intelligence("Miss Bennet");
        let result = ledger.reconcile(&changed, timestamp()).unwrap();
        assert_eq!(result.changed, vec![id.clone()]);
        let item = ledger.item(&id).unwrap();
        assert_eq!(item.status, ReviewStatus::Pending);
        assert_eq!(item.archived_revisions[0].status, ReviewStatus::Rejected);
    }

    #[test]
    fn canon_satisfied_proposal_leaves_and_can_reenter_the_pending_queue() {
        let first = intelligence("Lizzy");
        let mut ledger = ReviewLedger::new(&first);
        ledger.reconcile(&first, timestamp()).unwrap();
        let id = ledger.items[0].id.clone();

        let mut canonical = first.clone();
        canonical.character_seeds[0].matches_approved_character = true;
        ledger.reconcile(&canonical, timestamp()).unwrap();
        assert_eq!(
            ledger.item(&id).unwrap().availability,
            ProposalAvailability::Obsolete
        );

        ledger.reconcile(&first, timestamp()).unwrap();
        let restored = ledger.item(&id).unwrap();
        assert_eq!(restored.availability, ProposalAvailability::Active);
        assert_eq!(restored.status, ReviewStatus::Pending);
    }

    #[test]
    fn invalid_transitions_are_rejected() {
        let intelligence = intelligence("Lizzy");
        let mut ledger = ReviewLedger::new(&intelligence);
        ledger.reconcile(&intelligence, timestamp()).unwrap();
        let id = ledger.items[0].id.clone();
        ledger
            .decide(
                &id,
                DecisionAction::Reject,
                None,
                "editor".into(),
                "no".into(),
                timestamp(),
            )
            .unwrap();
        assert!(matches!(
            ledger.decide(
                &id,
                DecisionAction::Approve,
                None,
                "editor".into(),
                "changed mind".into(),
                timestamp()
            ),
            Err(ReviewError::InvalidTransition { .. })
        ));
    }

    #[test]
    fn all_supported_lifecycle_transitions_are_explicit() {
        for action in [
            DecisionAction::Approve,
            DecisionAction::Edit,
            DecisionAction::Reject,
            DecisionAction::Defer,
        ] {
            let intelligence = intelligence("Lizzy");
            let mut ledger = ReviewLedger::new(&intelligence);
            ledger.reconcile(&intelligence, timestamp()).unwrap();
            let id = ledger.items[0].id.clone();
            let replacement = (action == DecisionAction::Edit).then(|| {
                ReviewedValue::Character(ReviewedCharacter {
                    canonical_name: "Elizabeth Bennet".into(),
                    aliases: vec!["Lizzy".into()],
                    voice_notes: String::new(),
                    personality_notes: String::new(),
                })
            });
            ledger
                .decide(
                    &id,
                    action,
                    replacement,
                    "editor".into(),
                    "reviewed".into(),
                    timestamp(),
                )
                .unwrap();
        }

        for action in [
            DecisionAction::Approve,
            DecisionAction::Edit,
            DecisionAction::Reject,
        ] {
            let intelligence = intelligence("Lizzy");
            let mut ledger = ReviewLedger::new(&intelligence);
            ledger.reconcile(&intelligence, timestamp()).unwrap();
            let id = ledger.items[0].id.clone();
            ledger
                .decide(
                    &id,
                    DecisionAction::Defer,
                    None,
                    "editor".into(),
                    "wait".into(),
                    timestamp(),
                )
                .unwrap();
            let replacement = (action == DecisionAction::Edit).then(|| {
                ReviewedValue::Character(ReviewedCharacter {
                    canonical_name: "Elizabeth Bennet".into(),
                    aliases: vec!["Lizzy".into()],
                    voice_notes: String::new(),
                    personality_notes: String::new(),
                })
            });
            ledger
                .decide(
                    &id,
                    action,
                    replacement,
                    "editor".into(),
                    "resolved".into(),
                    timestamp(),
                )
                .unwrap();
        }
    }

    #[test]
    fn approved_character_produces_a_deterministic_plan() {
        let intelligence = intelligence("Lizzy");
        let mut ledger = ReviewLedger::new(&intelligence);
        ledger.reconcile(&intelligence, timestamp()).unwrap();
        let id = ledger.items[0].id.clone();
        ledger
            .decide(
                &id,
                DecisionAction::Approve,
                None,
                "editor".into(),
                "correct".into(),
                timestamp(),
            )
            .unwrap();
        let first = build_promotion_plan(
            &ledger,
            &CharacterBible::new(),
            &Glossary::default(),
            &[],
            &[],
        )
        .unwrap();
        let second = build_promotion_plan(
            &ledger,
            &CharacterBible::new(),
            &Glossary::default(),
            &[],
            &[],
        )
        .unwrap();
        assert_eq!(first, second);
        assert!(!first.blocked);
        let (characters, _) =
            apply_plan_to_canon(&first, &CharacterBible::new(), &Glossary::default()).unwrap();
        assert!(characters.find_profile("Elizabeth Bennet").is_some());
        assert_eq!(
            characters.find_alias_owner("Lizzy"),
            Some("Elizabeth Bennet")
        );
    }

    #[test]
    fn alias_collision_is_blocking_and_typed() {
        let intelligence = intelligence("Lizzy");
        let mut ledger = ReviewLedger::new(&intelligence);
        ledger.reconcile(&intelligence, timestamp()).unwrap();
        let id = ledger.items[0].id.clone();
        ledger
            .decide(
                &id,
                DecisionAction::Approve,
                None,
                "editor".into(),
                "correct".into(),
                timestamp(),
            )
            .unwrap();
        let mut characters = CharacterBible::new();
        characters.add(CharacterProfile {
            name: "Lydia Bennet".into(),
            voice_notes: String::new(),
            personality_notes: String::new(),
        });
        characters.add_alias("Lydia Bennet", "Lizzy");
        let plan =
            build_promotion_plan(&ledger, &characters, &Glossary::default(), &[], &[]).unwrap();
        assert!(plan.blocked);
        assert!(plan.conflicts.iter().any(|value| {
            value.kind == CanonConflictKind::CharacterAliasCollision
                && value.severity == ConflictSeverity::Blocking
        }));
    }

    #[test]
    fn glossary_collision_requires_an_explicit_supported_resolution() {
        let proposal = ReviewProposal::Terminology(Box::new(TerminologySeed {
            id: "term-duke".into(),
            source_expression: "Duke".into(),
            occurrence_count: 4,
            category: TerminologyCategory::Honorific,
            evidence: vec![evidence()],
            confidence: Confidence::from_evidence(4),
            status: SeedStatus::Inferred,
            approved_translation: None,
        }));
        let mut ledger = ledger_for_proposal(proposal);
        ledger
            .decide(
                "review-item",
                DecisionAction::Edit,
                Some(ReviewedValue::Terminology(ReviewedTerminology {
                    source_term: "Duke".into(),
                    preferred_translation: "دوک".into(),
                    context: "لقب".into(),
                })),
                "ویراستار".into(),
                "انتخاب فارسی".into(),
                timestamp(),
            )
            .unwrap();
        let mut glossary = Glossary::default();
        glossary.add(GlossaryEntry {
            source_term: "duke".into(),
            preferred_translation: "دوک اعظم".into(),
            context: "approved".into(),
        });
        glossary.add(GlossaryEntry {
            source_term: "Duke".into(),
            preferred_translation: "لقب دوک".into(),
            context: "legacy duplicate".into(),
        });
        let blocked =
            build_promotion_plan(&ledger, &CharacterBible::new(), &glossary, &[], &[]).unwrap();
        assert!(blocked.blocked);
        let conflict = blocked
            .conflicts
            .iter()
            .find(|value| value.kind == CanonConflictKind::GlossaryTranslation)
            .unwrap();
        assert!(!conflict
            .allowed_resolutions
            .contains(&ResolutionKind::KeepCanon));
        let resolved = build_promotion_plan(
            &ledger,
            &CharacterBible::new(),
            &glossary,
            &[],
            &[ConflictResolution {
                conflict_id: conflict.id.clone(),
                resolution: ResolutionKind::UseReviewedProposal,
            }],
        )
        .unwrap();
        assert!(!resolved.blocked);
        let (_, changed) =
            apply_plan_to_canon(&resolved, &CharacterBible::new(), &glossary).unwrap();
        assert_eq!(
            changed
                .find_exact_term("DUKE")
                .unwrap()
                .preferred_translation,
            "دوک"
        );
        assert_eq!(changed.entries_for_term("duke").len(), 1);
    }

    #[test]
    fn reversed_relationship_contradiction_is_typed_and_mergeable() {
        let proposal = ReviewProposal::Relationship(Box::new(RelationshipSeed {
            id: "relationship-a-b".into(),
            character_a_id: "a".into(),
            character_b_id: "b".into(),
            character_a: "آرش".into(),
            character_b: "لیلا".into(),
            relationship_label_candidate: Some("siblings".into()),
            forms_of_address: vec!["خواهرم".into()],
            interaction_count: 3,
            evidence: vec![evidence()],
            confidence: Confidence::from_evidence(3),
            status: SeedStatus::Inferred,
            matches_approved_relationship: false,
        }));
        let mut ledger = ledger_for_proposal(proposal);
        ledger
            .decide(
                "review-item",
                DecisionAction::Approve,
                None,
                "editor".into(),
                "supported".into(),
                timestamp(),
            )
            .unwrap();
        let mut characters = CharacterBible::new();
        let mut existing = RelationshipProfile::new("لیلا", "آرش");
        existing.dynamic_notes = "spouses".into();
        characters.add_relationship(existing);
        let blocked =
            build_promotion_plan(&ledger, &characters, &Glossary::default(), &[], &[]).unwrap();
        let conflict = blocked
            .conflicts
            .iter()
            .find(|value| value.kind == CanonConflictKind::RelationshipContradiction)
            .unwrap();
        assert!(blocked.blocked);
        let resolved = build_promotion_plan(
            &ledger,
            &characters,
            &Glossary::default(),
            &[],
            &[ConflictResolution {
                conflict_id: conflict.id.clone(),
                resolution: ResolutionKind::Merge,
            }],
        )
        .unwrap();
        assert!(!resolved.blocked);
        let (changed, _) =
            apply_plan_to_canon(&resolved, &characters, &Glossary::default()).unwrap();
        let relationship = changed.find_relationship("آرش", "لیلا").unwrap();
        assert!(relationship.dynamic_notes.contains("spouses"));
        assert!(relationship.dynamic_notes.contains("siblings"));
        assert_eq!(changed.relationships().len(), 1);
    }

    #[test]
    fn reversed_relationship_with_the_same_canon_is_not_a_conflict() {
        let left = ReviewedRelationship {
            character_a: "آرش".into(),
            character_b: "لیلا".into(),
            dynamic_notes: "siblings".into(),
            address_notes: "خواهرم".into(),
            boundaries_notes: "—".into(),
        };
        let right = ReviewedRelationship {
            character_a: "لیلا".into(),
            character_b: "آرش".into(),
            dynamic_notes: "siblings".into(),
            address_notes: "خواهرم".into(),
            boundaries_notes: "—".into(),
        };

        assert!(relationships_semantically_equal(&left, &right));
    }

    #[test]
    fn stale_canon_invalidates_a_previously_generated_plan() {
        let intelligence = intelligence("Lizzy");
        let mut ledger = ReviewLedger::new(&intelligence);
        ledger.reconcile(&intelligence, timestamp()).unwrap();
        let id = ledger.items[0].id.clone();
        ledger
            .decide(
                &id,
                DecisionAction::Approve,
                None,
                "editor".into(),
                "correct".into(),
                timestamp(),
            )
            .unwrap();
        let plan = build_promotion_plan(
            &ledger,
            &CharacterBible::new(),
            &Glossary::default(),
            &[],
            &[],
        )
        .unwrap();
        let mut newer_canon = CharacterBible::new();
        newer_canon.add(CharacterProfile {
            name: "New Canon".into(),
            voice_notes: String::new(),
            personality_notes: String::new(),
        });
        assert!(matches!(
            apply_plan_to_canon(&plan, &newer_canon, &Glossary::default()),
            Err(ReviewError::StalePlan(_))
        ));
    }

    #[test]
    fn large_reconciliation_is_stable_and_deduplicated() {
        let mut analysis = intelligence("Lizzy");
        analysis.character_seeds = (0..300)
            .map(|index| {
                let mut seed = intelligence("Lizzy").character_seeds.remove(0);
                seed.id = format!("seed-{index:03}");
                seed.canonical_name_candidate = format!("Character {index:03}");
                seed
            })
            .collect();
        let mut ledger = ReviewLedger::new(&analysis);
        let first = ledger.reconcile(&analysis, timestamp()).unwrap();
        assert_eq!(first.added.len(), 300);
        let ids = ledger
            .items
            .iter()
            .map(|item| item.id.clone())
            .collect::<BTreeSet<_>>();
        assert_eq!(ids.len(), 300);
        let second = ledger.reconcile(&analysis, timestamp()).unwrap();
        assert_eq!(second.unchanged.len(), 300);
        assert_eq!(ledger.items.len(), 300);
        assert!(ledger.items.windows(2).all(|pair| pair[0].id <= pair[1].id));
        assert!(ledger
            .items
            .iter()
            .all(|item| { item.latest_proposal.evidence().len() <= 1 }));
    }

    fn literary_proposal() -> ReviewProposal {
        ReviewProposal::Literary(Box::new(LiteraryFindingProposal {
            id: "finding-tone-1".into(),
            finding_category: "tone".into(),
            subject: "scene".into(),
            claim: "warm affectionate tone".into(),
            scope_label: "chapter 1 [chapter-1]".into(),
            confidence: Confidence::from_evidence(2),
            evidence: vec![evidence()],
            analysis_unit_id: "unit-1".into(),
            alternative_interpretations: Vec::new(),
        }))
    }

    #[test]
    fn literary_proposals_reconcile_and_survive_deterministic_sync() {
        let intelligence = intelligence("Lizzy");
        let mut ledger = ReviewLedger::new(&intelligence);
        ledger.reconcile(&intelligence, timestamp()).unwrap();
        assert_eq!(ledger.items.len(), 1);

        let record = ledger
            .reconcile_advanced(
                vec![literary_proposal()],
                1,
                "advanced-test".into(),
                "1".into(),
                timestamp(),
            )
            .unwrap();
        assert_eq!(record.added.len(), 1);
        assert_eq!(ledger.items.len(), 2);
        let literary = ledger
            .items
            .iter()
            .find(|item| item.kind == ReviewKind::Literary)
            .expect("literary item present");
        assert_eq!(literary.availability, ProposalAvailability::Active);
        assert_eq!(literary.status, ReviewStatus::Pending);

        // A later deterministic-only sync must NOT obsolete the literary item.
        let second = ledger.reconcile(&intelligence, timestamp()).unwrap();
        assert!(second.obsolete.is_empty());
        let literary = ledger
            .items
            .iter()
            .find(|item| item.kind == ReviewKind::Literary)
            .unwrap();
        assert_eq!(literary.availability, ProposalAvailability::Active);
        assert_eq!(ledger.items.len(), 2);
    }

    #[test]
    fn approved_literary_finding_is_review_only_and_never_promoted() {
        let intelligence = intelligence("Lizzy");
        let mut ledger = ReviewLedger::new(&intelligence);
        ledger.reconcile(&intelligence, timestamp()).unwrap();
        ledger
            .reconcile_advanced(
                vec![literary_proposal()],
                1,
                "advanced-test".into(),
                "1".into(),
                timestamp(),
            )
            .unwrap();
        let id = ledger
            .items
            .iter()
            .find(|item| item.kind == ReviewKind::Literary)
            .unwrap()
            .id
            .clone();
        ledger
            .decide(
                &id,
                DecisionAction::Approve,
                None,
                "editor".into(),
                "matches reading".into(),
                timestamp(),
            )
            .unwrap();
        assert_eq!(ledger.item(&id).unwrap().status, ReviewStatus::Approved);
        assert!(matches!(
            ledger.item(&id).unwrap().reviewed_value,
            Some(ReviewedValue::Literary(_))
        ));

        // Default plan selection never includes review-only literary items.
        let default_plan = build_promotion_plan(
            &ledger,
            &CharacterBible::new(),
            &Glossary::default(),
            &[],
            &[],
        )
        .unwrap();
        assert!(default_plan
            .selected_item_ids
            .iter()
            .all(|selected| selected != &id));
        assert!(default_plan.promoted_item_ids.is_empty());

        // Even an explicit selection is surfaced as a blocking conflict,
        // never silently dropped and never fake-canonized.
        let explicit = build_promotion_plan(
            &ledger,
            &CharacterBible::new(),
            &Glossary::default(),
            std::slice::from_ref(&id),
            &[],
        )
        .unwrap();
        assert!(explicit.blocked);
        assert!(explicit.conflicts.iter().any(|conflict| {
            conflict.kind == CanonConflictKind::UnsupportedPromotion && conflict.item_id == id
        }));
    }

    #[test]
    fn advanced_reconcile_obsoletes_only_literary_items() {
        let intelligence = intelligence("Lizzy");
        let mut ledger = ReviewLedger::new(&intelligence);
        ledger.reconcile(&intelligence, timestamp()).unwrap();
        ledger
            .reconcile_advanced(
                vec![literary_proposal()],
                1,
                "advanced-test".into(),
                "1".into(),
                timestamp(),
            )
            .unwrap();

        // Finding disappears from a later advanced pass: only the literary
        // item becomes obsolete; the deterministic character item stays active.
        let record = ledger
            .reconcile_advanced(
                Vec::new(),
                1,
                "advanced-test".into(),
                "1".into(),
                timestamp(),
            )
            .unwrap();
        assert_eq!(record.obsolete.len(), 1);
        let character_item = ledger
            .items
            .iter()
            .find(|item| item.kind == ReviewKind::Character)
            .unwrap();
        assert_eq!(character_item.availability, ProposalAvailability::Active);
        let literary_item = ledger
            .items
            .iter()
            .find(|item| item.kind == ReviewKind::Literary)
            .unwrap();
        assert_eq!(literary_item.availability, ProposalAvailability::Obsolete);
    }

    #[test]
    fn edited_literary_value_replaces_the_claim_through_review() {
        let intelligence = intelligence("Lizzy");
        let mut ledger = ReviewLedger::new(&intelligence);
        ledger.reconcile(&intelligence, timestamp()).unwrap();
        ledger
            .reconcile_advanced(
                vec![literary_proposal()],
                1,
                "advanced-test".into(),
                "1".into(),
                timestamp(),
            )
            .unwrap();
        let id = ledger
            .items
            .iter()
            .find(|item| item.kind == ReviewKind::Literary)
            .unwrap()
            .id
            .clone();
        ledger
            .decide(
                &id,
                DecisionAction::Edit,
                Some(ReviewedValue::Literary(ReviewedLiteraryFinding {
                    finding_category: "tone".into(),
                    subject: "scene".into(),
                    claim: "reserved but warm tone, not affectionate".into(),
                    scope_label: "chapter 1 [chapter-1]".into(),
                    analysis_unit_id: "unit-1".into(),
                    alternative_interpretations: Vec::new(),
                })),
                "editor".into(),
                "overstated warmth".into(),
                timestamp(),
            )
            .unwrap();
        let item = ledger.item(&id).unwrap();
        assert_eq!(item.status, ReviewStatus::Edited);
        let ReviewedValue::Literary(reviewed) = item.reviewed_value.as_ref().unwrap() else {
            panic!("expected literary reviewed value");
        };
        assert_eq!(reviewed.claim, "reserved but warm tone, not affectionate");
        assert_eq!(item.decisions.last().unwrap().action, DecisionAction::Edit);
    }

    #[test]
    fn source_model_fixture_is_valid() {
        let _ = Manuscript {
            book: Book {
                id: "book-1".into(),
                title: "Synthetic".into(),
                author: None,
                language: None,
                metadata: BTreeMap::new(),
            },
            chapters: Vec::new(),
            source: SourceLocation::new("synthetic.txt", DocumentFormat::Txt),
        };
    }
}
