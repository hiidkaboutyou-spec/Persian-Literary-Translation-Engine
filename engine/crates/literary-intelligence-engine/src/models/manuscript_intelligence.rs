use crate::errors::LiteraryIntelligenceError;
use document_engine::SourceLocation;
use serde::{Deserialize, Serialize};

pub const MANUSCRIPT_INTELLIGENCE_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SeedStatus {
    Inferred,
    Approved,
    Rejected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfidenceLevel {
    Low,
    Moderate,
    High,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Confidence {
    pub level: ConfidenceLevel,
    pub supporting_evidence: usize,
}

impl Confidence {
    pub fn from_evidence(supporting_evidence: usize) -> Self {
        let level = match supporting_evidence {
            0 | 1 => ConfidenceLevel::Low,
            2..=4 => ConfidenceLevel::Moderate,
            _ => ConfidenceLevel::High,
        };
        Self {
            level,
            supporting_evidence,
        }
    }

    pub fn validate(&self) -> Result<(), LiteraryIntelligenceError> {
        if self.supporting_evidence == 0 && self.level != ConfidenceLevel::Low {
            return Err(LiteraryIntelligenceError::Validation(
                "confidence without evidence must be low".into(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceRef {
    pub chapter_id: String,
    pub scene_id: String,
    pub paragraph_id: String,
    pub source: SourceLocation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CharacterSeed {
    pub id: String,
    pub canonical_name_candidate: String,
    pub aliases: Vec<String>,
    pub first_appearance: EvidenceRef,
    pub appearance_count: usize,
    pub evidence: Vec<EvidenceRef>,
    pub likely_narrative_role: Option<String>,
    pub speech_register_observations: Vec<String>,
    pub recurring_lexical_patterns: Vec<String>,
    pub personality_observations: Vec<String>,
    pub relationship_refs: Vec<String>,
    pub confidence: Confidence,
    pub status: SeedStatus,
    pub matches_approved_character: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationshipSeed {
    pub id: String,
    pub character_a_id: String,
    pub character_b_id: String,
    pub character_a: String,
    pub character_b: String,
    pub relationship_label_candidate: Option<String>,
    pub forms_of_address: Vec<String>,
    pub interaction_count: usize,
    pub evidence: Vec<EvidenceRef>,
    pub confidence: Confidence,
    pub status: SeedStatus,
    pub matches_approved_relationship: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TerminologyCategory {
    CharacterName,
    ProperName,
    Honorific,
    RecurringExpression,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerminologySeed {
    pub id: String,
    pub source_expression: String,
    pub occurrence_count: usize,
    pub category: TerminologyCategory,
    pub evidence: Vec<EvidenceRef>,
    pub confidence: Confidence,
    pub status: SeedStatus,
    pub approved_translation: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChapterMap {
    pub chapter_id: String,
    pub source_chapter_index: usize,
    pub title: String,
    pub source: SourceLocation,
    pub scene_ids: Vec<String>,
    pub character_seed_ids: Vec<String>,
    pub important_named_entities: Vec<String>,
    pub recurring_terms: Vec<String>,
    pub relationship_seed_ids: Vec<String>,
    pub dialogue_density: f32,
    pub narrative_density: f32,
    pub unresolved_references: Vec<String>,
    pub continuity_hooks: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LiteraryProfile {
    pub observed: ObservedLiteraryProfile,
    pub inferred: InferredLiteraryProfile,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObservedLiteraryProfile {
    pub paragraph_count: usize,
    pub sentence_count: usize,
    pub average_sentence_words: f32,
    pub dialogue_density: f32,
    pub prose_density: f32,
    pub dialogue_conventions: Vec<String>,
    pub code_switching_detected: bool,
    pub chapter_count: usize,
    pub average_scenes_per_chapter: f32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct InferredLiteraryProfile {
    pub narrative_pov: Option<String>,
    pub narrative_tense: Option<String>,
    pub narrator_register: Option<String>,
    pub dialogue_register: Option<String>,
    pub recurring_imagery: Vec<String>,
    pub recurring_motifs: Vec<String>,
    pub humor_signals: Vec<String>,
    pub sarcasm_signals: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictKind {
    CharacterName,
    Terminology,
    Relationship,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeedConflict {
    pub kind: ConflictKind,
    pub seed_id: String,
    pub inferred_value: String,
    pub approved_value: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChapterInitializationProposal {
    pub chapter_id: String,
    pub character_seed_ids: Vec<String>,
    pub relationship_seed_ids: Vec<String>,
    pub terminology_seed_ids: Vec<String>,
    pub context: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InitializationProposal {
    pub mutates_canon: bool,
    pub chapters: Vec<ChapterInitializationProposal>,
    pub conflicts: Vec<SeedConflict>,
}

impl InitializationProposal {
    pub fn context_for_chapter(&self, chapter_id: &str) -> Option<&str> {
        self.chapters
            .iter()
            .find(|chapter| chapter.chapter_id == chapter_id)
            .map(|chapter| chapter.context.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnalysisMetadata {
    pub analyzer: String,
    pub analyzer_version: String,
    pub deterministic: bool,
    pub retained_evidence_limit: usize,
    pub character_seed_limit: usize,
    pub terminology_limit: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ManuscriptIntelligence {
    pub schema_version: u32,
    pub manuscript_id: String,
    pub manuscript_title: String,
    pub literary_profile: LiteraryProfile,
    pub character_seeds: Vec<CharacterSeed>,
    pub relationship_seeds: Vec<RelationshipSeed>,
    pub chapter_maps: Vec<ChapterMap>,
    pub terminology_seeds: Vec<TerminologySeed>,
    pub analysis: AnalysisMetadata,
    pub source: SourceLocation,
    pub initialization: InitializationProposal,
}

impl ManuscriptIntelligence {
    pub fn validate(&self) -> Result<(), LiteraryIntelligenceError> {
        if self.schema_version != MANUSCRIPT_INTELLIGENCE_SCHEMA_VERSION {
            return Err(LiteraryIntelligenceError::Validation(format!(
                "unsupported manuscript intelligence schema version {}",
                self.schema_version
            )));
        }
        if self.manuscript_id.trim().is_empty() {
            return Err(LiteraryIntelligenceError::Validation(
                "manuscript id cannot be empty".into(),
            ));
        }
        for confidence in self
            .character_seeds
            .iter()
            .map(|seed| &seed.confidence)
            .chain(self.relationship_seeds.iter().map(|seed| &seed.confidence))
            .chain(self.terminology_seeds.iter().map(|seed| &seed.confidence))
        {
            confidence.validate()?;
        }
        if self.initialization.mutates_canon {
            return Err(LiteraryIntelligenceError::Validation(
                "initialization proposals must not mutate canon".into(),
            ));
        }
        Ok(())
    }
}
