use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fmt;

pub mod sacrebleu;

pub const CORPUS_SCHEMA_VERSION: u32 = 1;
pub const SUBMISSION_SCHEMA_VERSION: u32 = 1;
pub const SCORECARD_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvaluationDimension {
    SemanticFidelity,
    PersianNaturalness,
    CharacterVoice,
    RelationshipRegister,
    DialogueSubtext,
    TerminologyContinuity,
    LongContextContinuity,
    Readability,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnchorMode {
    AllPresent,
    AnyPresent,
    AllAbsent,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnchorExpectation {
    pub id: String,
    pub dimension: EvaluationDimension,
    pub mode: AnchorMode,
    pub values: Vec<String>,
    #[serde(default)]
    pub rationale: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContrastiveVariant {
    pub id: String,
    pub translation: String,
    pub degradation: EvaluationDimension,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EvaluationCase {
    pub id: String,
    pub title: String,
    pub source: String,
    pub reference: String,
    #[serde(default)]
    pub context_before: Option<String>,
    #[serde(default)]
    pub context_after: Option<String>,
    pub dimensions: BTreeSet<EvaluationDimension>,
    #[serde(default)]
    pub anchors: Vec<AnchorExpectation>,
    #[serde(default)]
    pub contrastive_variants: Vec<ContrastiveVariant>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CorpusProvenance {
    pub owner: String,
    pub license: String,
    pub rights_safe: bool,
    pub creation_note: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LiteraryEvaluationCorpus {
    pub schema_version: u32,
    pub corpus_id: String,
    pub title: String,
    pub source_language: String,
    pub target_language: String,
    pub provenance: CorpusProvenance,
    pub cases: Vec<EvaluationCase>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CandidateOutput {
    pub case_id: String,
    pub translation: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CandidateSubmission {
    pub schema_version: u32,
    pub corpus_id: String,
    pub system_id: String,
    pub outputs: Vec<CandidateOutput>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnchorOutcome {
    pub anchor_id: String,
    pub dimension: EvaluationDimension,
    pub passed: bool,
    pub rationale: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CaseEvaluation {
    pub case_id: String,
    pub anchor_passed: usize,
    pub anchor_total: usize,
    pub outcomes: Vec<AnchorOutcome>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DimensionCoverage {
    pub passed: usize,
    pub total: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BenchmarkReport {
    pub corpus_id: String,
    pub system_id: String,
    pub cases: usize,
    pub anchor_passed: usize,
    pub anchor_total: usize,
    pub anchor_pass_rate: f32,
    pub by_dimension: BTreeMap<EvaluationDimension, DimensionCoverage>,
    pub case_results: Vec<CaseEvaluation>,
    pub advisory_notes: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewerExpertise {
    ProfessionalTranslator,
    NativeTargetReader,
    TrainedReviewer,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HumanDimensionRating {
    pub dimension: EvaluationDimension,
    pub score: u8,
    #[serde(default)]
    pub note: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PairwisePreference {
    CandidateA,
    CandidateB,
    Tie,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HumanScorecard {
    pub schema_version: u32,
    pub case_id: String,
    pub reviewer_id: String,
    pub expertise: ReviewerExpertise,
    pub ratings: Vec<HumanDimensionRating>,
    pub overall_literary_quality: u8,
    #[serde(default)]
    pub pairwise_preference: Option<PairwisePreference>,
    #[serde(default)]
    pub notes: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvaluationError(pub String);

impl fmt::Display for EvaluationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

impl std::error::Error for EvaluationError {}

impl LiteraryEvaluationCorpus {
    pub fn from_json(input: &str) -> Result<Self, EvaluationError> {
        let corpus: Self =
            serde_json::from_str(input).map_err(|error| EvaluationError(error.to_string()))?;
        corpus.validate()?;
        Ok(corpus)
    }

    pub fn validate(&self) -> Result<(), EvaluationError> {
        if self.schema_version != CORPUS_SCHEMA_VERSION {
            return Err(EvaluationError(format!(
                "unsupported corpus schema {}; expected {}",
                self.schema_version, CORPUS_SCHEMA_VERSION
            )));
        }
        if self.corpus_id.trim().is_empty() || self.title.trim().is_empty() {
            return Err(EvaluationError(
                "corpus_id and title must not be empty".to_string(),
            ));
        }
        if self.source_language.trim().is_empty() || self.target_language.trim().is_empty() {
            return Err(EvaluationError(
                "source_language and target_language must not be empty".to_string(),
            ));
        }
        if !self.provenance.rights_safe {
            return Err(EvaluationError(
                "Phase 21 committed corpora must be explicitly rights-safe".to_string(),
            ));
        }
        if self.provenance.owner.trim().is_empty()
            || self.provenance.license.trim().is_empty()
            || self.provenance.creation_note.trim().is_empty()
        {
            return Err(EvaluationError(
                "corpus provenance owner/license/creation_note are required".to_string(),
            ));
        }
        if self.cases.is_empty() {
            return Err(EvaluationError(
                "evaluation corpus must contain at least one case".to_string(),
            ));
        }

        let mut case_ids = HashSet::new();
        for case in &self.cases {
            if !case_ids.insert(case.id.as_str()) {
                return Err(EvaluationError(format!(
                    "duplicate evaluation case id '{}'",
                    case.id
                )));
            }
            validate_case(case)?;
        }
        Ok(())
    }
}

impl CandidateSubmission {
    pub fn from_json(input: &str) -> Result<Self, EvaluationError> {
        let submission: Self =
            serde_json::from_str(input).map_err(|error| EvaluationError(error.to_string()))?;
        if submission.schema_version != SUBMISSION_SCHEMA_VERSION {
            return Err(EvaluationError(format!(
                "unsupported submission schema {}; expected {}",
                submission.schema_version, SUBMISSION_SCHEMA_VERSION
            )));
        }
        if submission.corpus_id.trim().is_empty() || submission.system_id.trim().is_empty() {
            return Err(EvaluationError(
                "submission corpus_id and system_id must not be empty".to_string(),
            ));
        }
        let mut seen = HashSet::new();
        for output in &submission.outputs {
            if output.case_id.trim().is_empty() || output.translation.trim().is_empty() {
                return Err(EvaluationError(
                    "submission outputs require non-empty case_id and translation".to_string(),
                ));
            }
            if !seen.insert(output.case_id.as_str()) {
                return Err(EvaluationError(format!(
                    "duplicate candidate output for case '{}'",
                    output.case_id
                )));
            }
        }
        Ok(submission)
    }
}

impl HumanScorecard {
    pub fn validate(&self, case: &EvaluationCase) -> Result<(), EvaluationError> {
        if self.schema_version != SCORECARD_SCHEMA_VERSION {
            return Err(EvaluationError(format!(
                "unsupported scorecard schema {}; expected {}",
                self.schema_version, SCORECARD_SCHEMA_VERSION
            )));
        }
        if self.case_id != case.id {
            return Err(EvaluationError(format!(
                "scorecard case '{}' does not match evaluation case '{}'",
                self.case_id, case.id
            )));
        }
        if self.reviewer_id.trim().is_empty() {
            return Err(EvaluationError(
                "scorecard reviewer_id must not be empty".to_string(),
            ));
        }
        if !(1..=5).contains(&self.overall_literary_quality) {
            return Err(EvaluationError(
                "overall_literary_quality must be in 1..=5".to_string(),
            ));
        }
        if self.ratings.is_empty() || self.ratings.len() > 4 {
            return Err(EvaluationError(
                "a human scoring pass must rate between 1 and 4 focused dimensions".to_string(),
            ));
        }
        let mut dimensions = BTreeSet::new();
        for rating in &self.ratings {
            if !(1..=5).contains(&rating.score) {
                return Err(EvaluationError(format!(
                    "rating for {:?} must be in 1..=5",
                    rating.dimension
                )));
            }
            if !case.dimensions.contains(&rating.dimension) {
                return Err(EvaluationError(format!(
                    "scorecard rates {:?}, which is not requested for case '{}'",
                    rating.dimension, case.id
                )));
            }
            if !dimensions.insert(rating.dimension) {
                return Err(EvaluationError(format!(
                    "duplicate human rating for {:?}",
                    rating.dimension
                )));
            }
        }
        Ok(())
    }
}

fn validate_case(case: &EvaluationCase) -> Result<(), EvaluationError> {
    if case.id.trim().is_empty()
        || case.title.trim().is_empty()
        || case.source.trim().is_empty()
        || case.reference.trim().is_empty()
    {
        return Err(EvaluationError(
            "case id/title/source/reference must not be empty".to_string(),
        ));
    }
    if case.dimensions.is_empty() || case.dimensions.len() > 4 {
        return Err(EvaluationError(format!(
            "case '{}' must focus on 1..=4 dimensions to keep human review cognitively bounded",
            case.id
        )));
    }

    let mut anchor_ids = HashSet::new();
    for anchor in &case.anchors {
        if anchor.id.trim().is_empty() || anchor.values.is_empty() {
            return Err(EvaluationError(format!(
                "case '{}' contains an invalid empty anchor",
                case.id
            )));
        }
        if !case.dimensions.contains(&anchor.dimension) {
            return Err(EvaluationError(format!(
                "anchor '{}' uses a dimension not requested by case '{}'",
                anchor.id, case.id
            )));
        }
        if anchor.values.iter().any(|value| value.trim().is_empty()) {
            return Err(EvaluationError(format!(
                "anchor '{}' contains an empty value",
                anchor.id
            )));
        }
        if !anchor_ids.insert(anchor.id.as_str()) {
            return Err(EvaluationError(format!(
                "case '{}' contains duplicate anchor id '{}'",
                case.id, anchor.id
            )));
        }
    }

    let mut variant_ids = HashSet::new();
    for variant in &case.contrastive_variants {
        if variant.id.trim().is_empty()
            || variant.translation.trim().is_empty()
            || variant.description.trim().is_empty()
        {
            return Err(EvaluationError(format!(
                "case '{}' contains an incomplete contrastive variant",
                case.id
            )));
        }
        if !case.dimensions.contains(&variant.degradation) {
            return Err(EvaluationError(format!(
                "contrastive variant '{}' declares a dimension not requested by case '{}'",
                variant.id, case.id
            )));
        }
        if !variant_ids.insert(variant.id.as_str()) {
            return Err(EvaluationError(format!(
                "case '{}' contains duplicate contrastive variant id '{}'",
                case.id, variant.id
            )));
        }
    }
    Ok(())
}

pub fn evaluate_submission(
    corpus: &LiteraryEvaluationCorpus,
    submission: &CandidateSubmission,
) -> Result<BenchmarkReport, EvaluationError> {
    corpus.validate()?;
    if submission.corpus_id != corpus.corpus_id {
        return Err(EvaluationError(format!(
            "submission targets corpus '{}' but benchmark corpus is '{}'",
            submission.corpus_id, corpus.corpus_id
        )));
    }

    let outputs = submission
        .outputs
        .iter()
        .map(|output| (output.case_id.as_str(), output.translation.as_str()))
        .collect::<BTreeMap<_, _>>();

    let corpus_ids = corpus
        .cases
        .iter()
        .map(|case| case.id.as_str())
        .collect::<BTreeSet<_>>();
    for case_id in outputs.keys() {
        if !corpus_ids.contains(case_id) {
            return Err(EvaluationError(format!(
                "submission contains unknown case '{}'",
                case_id
            )));
        }
    }

    let mut anchor_passed = 0usize;
    let mut anchor_total = 0usize;
    let mut by_dimension = BTreeMap::<EvaluationDimension, DimensionCoverage>::new();
    let mut case_results = Vec::with_capacity(corpus.cases.len());

    for case in &corpus.cases {
        let translation = outputs.get(case.id.as_str()).ok_or_else(|| {
            EvaluationError(format!("submission is missing case '{}'", case.id))
        })?;
        let result = evaluate_case(case, translation);
        anchor_passed += result.anchor_passed;
        anchor_total += result.anchor_total;
        for outcome in &result.outcomes {
            let coverage = by_dimension
                .entry(outcome.dimension)
                .or_insert(DimensionCoverage { passed: 0, total: 0 });
            coverage.total += 1;
            if outcome.passed {
                coverage.passed += 1;
            }
        }
        case_results.push(result);
    }

    let anchor_pass_rate = if anchor_total == 0 {
        1.0
    } else {
        anchor_passed as f32 / anchor_total as f32
    };

    Ok(BenchmarkReport {
        corpus_id: corpus.corpus_id.clone(),
        system_id: submission.system_id.clone(),
        cases: corpus.cases.len(),
        anchor_passed,
        anchor_total,
        anchor_pass_rate,
        by_dimension,
        case_results,
        advisory_notes: vec![
            "Anchor checks are deterministic challenge evidence, not a complete literary-quality score.".into(),
            "Reference metrics such as chrF++/COMET and human scorecards must remain separate evidence channels.".into(),
            "Human review remains the authority for naturalness, voice, subtext, and overall literary quality.".into(),
        ],
    })
}

pub fn evaluate_case(case: &EvaluationCase, translation: &str) -> CaseEvaluation {
    let outcomes = case
        .anchors
        .iter()
        .map(|anchor| {
            let passed = anchor_matches(anchor, translation);
            AnchorOutcome {
                anchor_id: anchor.id.clone(),
                dimension: anchor.dimension,
                passed,
                rationale: anchor.rationale.clone(),
            }
        })
        .collect::<Vec<_>>();
    let anchor_passed = outcomes.iter().filter(|outcome| outcome.passed).count();
    CaseEvaluation {
        case_id: case.id.clone(),
        anchor_passed,
        anchor_total: outcomes.len(),
        outcomes,
    }
}

pub fn contrastive_sanity_check(
    corpus: &LiteraryEvaluationCorpus,
) -> Result<(), EvaluationError> {
    corpus.validate()?;
    for case in &corpus.cases {
        let reference = evaluate_case(case, &case.reference);
        if reference.anchor_passed != reference.anchor_total {
            return Err(EvaluationError(format!(
                "reference for case '{}' does not satisfy its own deterministic anchors",
                case.id
            )));
        }
        for variant in &case.contrastive_variants {
            let result = evaluate_case(case, &variant.translation);
            let target_outcomes = result
                .outcomes
                .iter()
                .filter(|outcome| outcome.dimension == variant.degradation)
                .collect::<Vec<_>>();
            if target_outcomes.is_empty() || target_outcomes.iter().all(|outcome| outcome.passed) {
                return Err(EvaluationError(format!(
                    "contrastive variant '{}' in case '{}' does not trigger its declared {:?} degradation",
                    variant.id, case.id, variant.degradation
                )));
            }
        }
    }
    Ok(())
}

fn anchor_matches(anchor: &AnchorExpectation, translation: &str) -> bool {
    let normalized = text_normalization::normalize_case_insensitive(translation);
    let values = anchor
        .values
        .iter()
        .map(|value| text_normalization::normalize_case_insensitive(value))
        .collect::<Vec<_>>();
    match anchor.mode {
        AnchorMode::AllPresent => values.iter().all(|value| normalized.contains(value)),
        AnchorMode::AnyPresent => values.iter().any(|value| normalized.contains(value)),
        AnchorMode::AllAbsent => values.iter().all(|value| !normalized.contains(value)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn small_case() -> EvaluationCase {
        EvaluationCase {
            id: "negation".into(),
            title: "Negation".into(),
            source: "I do not trust him.".into(),
            reference: "بهش اعتماد ندارم.".into(),
            context_before: None,
            context_after: None,
            dimensions: BTreeSet::from([EvaluationDimension::SemanticFidelity]),
            anchors: vec![AnchorExpectation {
                id: "keep-negation".into(),
                dimension: EvaluationDimension::SemanticFidelity,
                mode: AnchorMode::AnyPresent,
                values: vec!["اعتماد ندارم".into(), "بهش اعتماد ندارم".into()],
                rationale: "Polarity must not flip.".into(),
            }],
            contrastive_variants: vec![ContrastiveVariant {
                id: "polarity-flip".into(),
                translation: "بهش اعتماد دارم.".into(),
                degradation: EvaluationDimension::SemanticFidelity,
                description: "Negation is removed.".into(),
            }],
        }
    }

    #[test]
    fn reference_passes_and_declared_degradation_fails() {
        let corpus = LiteraryEvaluationCorpus {
            schema_version: CORPUS_SCHEMA_VERSION,
            corpus_id: "test".into(),
            title: "Test".into(),
            source_language: "en".into(),
            target_language: "fa".into(),
            provenance: CorpusProvenance {
                owner: "project".into(),
                license: "CC0-1.0".into(),
                rights_safe: true,
                creation_note: "synthetic".into(),
            },
            cases: vec![small_case()],
        };
        contrastive_sanity_check(&corpus).unwrap();
    }

    #[test]
    fn scorecard_limits_cognitive_load() {
        let case = small_case();
        let scorecard = HumanScorecard {
            schema_version: SCORECARD_SCHEMA_VERSION,
            case_id: case.id.clone(),
            reviewer_id: "reviewer".into(),
            expertise: ReviewerExpertise::ProfessionalTranslator,
            ratings: vec![HumanDimensionRating {
                dimension: EvaluationDimension::SemanticFidelity,
                score: 5,
                note: String::new(),
            }],
            overall_literary_quality: 5,
            pairwise_preference: None,
            notes: String::new(),
        };
        scorecard.validate(&case).unwrap();
    }

    #[test]
    fn unsafe_corpus_is_rejected() {
        let corpus = LiteraryEvaluationCorpus {
            schema_version: CORPUS_SCHEMA_VERSION,
            corpus_id: "unsafe".into(),
            title: "Unsafe".into(),
            source_language: "en".into(),
            target_language: "fa".into(),
            provenance: CorpusProvenance {
                owner: "unknown".into(),
                license: "unknown".into(),
                rights_safe: false,
                creation_note: "copied".into(),
            },
            cases: vec![small_case()],
        };
        assert!(corpus.validate().is_err());
    }
}
