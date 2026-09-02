use character_engine::CharacterBible;
use chrono::{DateTime, Utc};
use human_review_workflow::{apply_plan_to_canon, CanonPromotionPlan, ReviewError, ReviewLedger};
use memory_engine::{glossary::Glossary, load_glossary};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;

const JOURNAL_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewStorePaths {
    pub review_file: PathBuf,
    pub character_bible_file: PathBuf,
    pub glossary_file: PathBuf,
}

impl ReviewStorePaths {
    pub fn validate(&self) -> Result<(), ProjectReviewError> {
        let paths = [
            &self.review_file,
            &self.character_bible_file,
            &self.glossary_file,
        ];
        for (index, left) in paths.iter().enumerate() {
            if left.as_os_str().is_empty() {
                return Err(ProjectReviewError::Validation(
                    "review and canonical paths cannot be empty".into(),
                ));
            }
            if paths[index + 1..].contains(left) {
                return Err(ProjectReviewError::Validation(
                    "review, Character Bible, and Glossary paths must be distinct".into(),
                ));
            }
        }
        Ok(())
    }

    fn journal_file(&self) -> PathBuf {
        append_suffix(&self.review_file, ".promotion-journal.json")
    }
}

#[derive(Debug, Error)]
pub enum ProjectReviewError {
    #[error("invalid project review storage: {0}")]
    Validation(String),
    #[error("project review I/O error for {path}: {source}")]
    Io { path: PathBuf, source: io::Error },
    #[error("project review JSON error for {path}: {source}")]
    Json {
        path: PathBuf,
        source: serde_json::Error,
    },
    #[error(transparent)]
    Review(#[from] ReviewError),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromotionApplyResult {
    pub schema_version: u32,
    pub plan_id: String,
    pub status: PromotionApplyStatus,
    pub applied_item_ids: Vec<String>,
    pub operation_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PromotionApplyStatus {
    Applied,
    AlreadyApplied,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PromotionJournal {
    schema_version: u32,
    plan_id: String,
    files: Vec<JournalFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct JournalFile {
    path: PathBuf,
    staged: PathBuf,
    backup: PathBuf,
    existed_before: bool,
    after_hash: String,
}

pub fn load_review_ledger(path: impl AsRef<Path>) -> Result<ReviewLedger, ProjectReviewError> {
    let path = path.as_ref();
    recover_journal_at(&append_suffix(path, ".promotion-journal.json"))?;
    let bytes = fs::read(path).map_err(|source| io_error(path, source))?;
    let ledger: ReviewLedger =
        serde_json::from_slice(&bytes).map_err(|source| ProjectReviewError::Json {
            path: path.to_path_buf(),
            source,
        })?;
    ledger.validate()?;
    Ok(ledger)
}

pub fn save_review_ledger(
    path: impl AsRef<Path>,
    ledger: &ReviewLedger,
) -> Result<(), ProjectReviewError> {
    ledger.validate()?;
    recover_journal_at(&append_suffix(path.as_ref(), ".promotion-journal.json"))?;
    let bytes = serde_json::to_vec_pretty(ledger).map_err(|source| ProjectReviewError::Json {
        path: path.as_ref().to_path_buf(),
        source,
    })?;
    atomic_write(path.as_ref(), &bytes)
}

pub fn load_character_bible_or_default(
    path: impl AsRef<Path>,
) -> Result<CharacterBible, ProjectReviewError> {
    let path = path.as_ref();
    if !path.exists() {
        return Ok(CharacterBible::new());
    }
    CharacterBible::load_json(path).map_err(|source| io_error(path, source))
}

pub fn load_glossary_or_default(path: impl AsRef<Path>) -> Result<Glossary, ProjectReviewError> {
    let path = path.as_ref();
    if !path.exists() {
        return Ok(Glossary::default());
    }
    load_glossary(path).map_err(|error| {
        ProjectReviewError::Validation(format!("failed to load {}: {error}", path.display()))
    })
}

pub fn recover_pending_promotion(paths: &ReviewStorePaths) -> Result<bool, ProjectReviewError> {
    paths.validate()?;
    let journal_path = paths.journal_file();
    recover_journal_at(&journal_path)
}

fn recover_journal_at(journal_path: &Path) -> Result<bool, ProjectReviewError> {
    if !journal_path.exists() {
        return Ok(false);
    }
    let bytes = fs::read(journal_path).map_err(|source| io_error(journal_path, source))?;
    let journal: PromotionJournal =
        serde_json::from_slice(&bytes).map_err(|source| ProjectReviewError::Json {
            path: journal_path.to_path_buf(),
            source,
        })?;
    if journal.schema_version != JOURNAL_SCHEMA_VERSION {
        return Err(ProjectReviewError::Validation(format!(
            "unsupported promotion journal schema {}",
            journal.schema_version
        )));
    }
    let fully_applied = journal.files.iter().all(|entry| {
        fs::read(&entry.path)
            .ok()
            .is_some_and(|bytes| hash_bytes(&bytes) == entry.after_hash)
    });
    if fully_applied {
        cleanup_transaction(&journal, journal_path)?;
        return Ok(true);
    }
    rollback_transaction(&journal, journal_path)?;
    Ok(true)
}

pub fn apply_promotion(
    paths: &ReviewStorePaths,
    ledger: &ReviewLedger,
    plan: &CanonPromotionPlan,
    reviewer: &str,
    reason: &str,
    applied_at: DateTime<Utc>,
) -> Result<PromotionApplyResult, ProjectReviewError> {
    apply_promotion_inner(paths, ledger, plan, reviewer, reason, applied_at, None)
}

fn apply_promotion_inner(
    paths: &ReviewStorePaths,
    ledger: &ReviewLedger,
    plan: &CanonPromotionPlan,
    reviewer: &str,
    reason: &str,
    applied_at: DateTime<Utc>,
    fail_after_replacements: Option<usize>,
) -> Result<PromotionApplyResult, ProjectReviewError> {
    paths.validate()?;
    recover_pending_promotion(paths)?;
    if ledger.already_applied(&plan.plan_id) {
        return Ok(PromotionApplyResult {
            schema_version: 1,
            plan_id: plan.plan_id.clone(),
            status: PromotionApplyStatus::AlreadyApplied,
            applied_item_ids: plan.promoted_item_ids.clone(),
            operation_count: 0,
        });
    }
    let characters = load_character_bible_or_default(&paths.character_bible_file)?;
    let glossary = load_glossary_or_default(&paths.glossary_file)?;
    let (next_characters, next_glossary) = apply_plan_to_canon(plan, &characters, &glossary)?;
    let mut next_ledger = ledger.clone();
    next_ledger.finalize_promotion(plan, reviewer, reason, applied_at)?;

    let files = vec![
        (
            paths.character_bible_file.clone(),
            serde_json::to_vec_pretty(&next_characters).map_err(|source| {
                ProjectReviewError::Json {
                    path: paths.character_bible_file.clone(),
                    source,
                }
            })?,
        ),
        (
            paths.glossary_file.clone(),
            serde_json::to_vec_pretty(next_glossary.entries()).map_err(|source| {
                ProjectReviewError::Json {
                    path: paths.glossary_file.clone(),
                    source,
                }
            })?,
        ),
        (
            paths.review_file.clone(),
            serde_json::to_vec_pretty(&next_ledger).map_err(|source| ProjectReviewError::Json {
                path: paths.review_file.clone(),
                source,
            })?,
        ),
    ];
    commit_files(paths, &plan.plan_id, files, fail_after_replacements)?;
    Ok(PromotionApplyResult {
        schema_version: 1,
        plan_id: plan.plan_id.clone(),
        status: PromotionApplyStatus::Applied,
        applied_item_ids: plan.promoted_item_ids.clone(),
        operation_count: plan.operations.len(),
    })
}

fn commit_files(
    paths: &ReviewStorePaths,
    plan_id: &str,
    files: Vec<(PathBuf, Vec<u8>)>,
    fail_after_replacements: Option<usize>,
) -> Result<(), ProjectReviewError> {
    let mut entries: Vec<JournalFile> = Vec::with_capacity(files.len());
    for (index, (path, bytes)) in files.iter().enumerate() {
        ensure_parent(path)?;
        let suffix = format!(".phase14-{}-{index}", short_plan_id(plan_id));
        let staged = append_suffix(path, &format!("{suffix}.tmp"));
        let backup = append_suffix(path, &format!("{suffix}.bak"));
        if let Err(error) = write_synced(&staged, bytes) {
            for prior in &entries {
                remove_if_exists(&prior.staged)?;
            }
            return Err(error);
        }
        entries.push(JournalFile {
            path: path.clone(),
            staged,
            backup,
            existed_before: path.exists(),
            after_hash: hash_bytes(bytes),
        });
    }
    let journal = PromotionJournal {
        schema_version: JOURNAL_SCHEMA_VERSION,
        plan_id: plan_id.into(),
        files: entries,
    };
    let journal_path = paths.journal_file();
    let journal_bytes =
        serde_json::to_vec_pretty(&journal).map_err(|source| ProjectReviewError::Json {
            path: journal_path.clone(),
            source,
        })?;
    atomic_write(&journal_path, &journal_bytes)?;

    let backup_result = (|| {
        for entry in &journal.files {
            if entry.existed_before {
                fs::copy(&entry.path, &entry.backup)
                    .map_err(|source| io_error(&entry.backup, source))?;
                sync_path(&entry.backup)?;
            }
        }
        Ok(())
    })();
    if let Err(error) = backup_result {
        rollback_transaction(&journal, &journal_path)?;
        return Err(error);
    }

    let result = (|| {
        for (index, entry) in journal.files.iter().enumerate() {
            if fail_after_replacements.is_some_and(|limit| index >= limit) {
                return Err(ProjectReviewError::Validation(
                    "injected promotion replacement failure".into(),
                ));
            }
            replace_file(&entry.staged, &entry.path)?;
        }
        Ok(())
    })();
    if let Err(error) = result {
        rollback_transaction(&journal, &journal_path)?;
        return Err(error);
    }
    cleanup_transaction(&journal, &journal_path)
}

fn rollback_transaction(
    journal: &PromotionJournal,
    journal_path: &Path,
) -> Result<(), ProjectReviewError> {
    for entry in journal.files.iter().rev() {
        if entry.existed_before && entry.backup.exists() {
            replace_file(&entry.backup, &entry.path)?;
        } else if !entry.existed_before && entry.path.exists() {
            fs::remove_file(&entry.path).map_err(|source| io_error(&entry.path, source))?;
        }
        remove_if_exists(&entry.staged)?;
        remove_if_exists(&entry.backup)?;
    }
    remove_if_exists(journal_path)
}

fn cleanup_transaction(
    journal: &PromotionJournal,
    journal_path: &Path,
) -> Result<(), ProjectReviewError> {
    for entry in &journal.files {
        remove_if_exists(&entry.staged)?;
        remove_if_exists(&entry.backup)?;
    }
    remove_if_exists(journal_path)
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), ProjectReviewError> {
    ensure_parent(path)?;
    let staged = append_suffix(path, &format!(".atomic-{}.tmp", std::process::id()));
    write_synced(&staged, bytes)?;
    replace_file(&staged, path)
}

fn replace_file(from: &Path, to: &Path) -> Result<(), ProjectReviewError> {
    #[cfg(not(unix))]
    if to.exists() {
        fs::remove_file(to).map_err(|source| io_error(to, source))?;
    }
    fs::rename(from, to).map_err(|source| io_error(to, source))?;
    sync_parent(to)
}

fn write_synced(path: &Path, bytes: &[u8]) -> Result<(), ProjectReviewError> {
    let mut file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(path)
        .map_err(|source| io_error(path, source))?;
    file.write_all(bytes)
        .map_err(|source| io_error(path, source))?;
    file.sync_all().map_err(|source| io_error(path, source))
}

fn sync_path(path: &Path) -> Result<(), ProjectReviewError> {
    File::open(path)
        .and_then(|file| file.sync_all())
        .map_err(|source| io_error(path, source))
}

fn sync_parent(path: &Path) -> Result<(), ProjectReviewError> {
    #[cfg(unix)]
    if let Some(parent) = path.parent() {
        File::open(parent)
            .and_then(|directory| directory.sync_all())
            .map_err(|source| io_error(parent, source))?;
    }
    Ok(())
}

fn ensure_parent(path: &Path) -> Result<(), ProjectReviewError> {
    if let Some(parent) = path.parent().filter(|value| !value.as_os_str().is_empty()) {
        fs::create_dir_all(parent).map_err(|source| io_error(parent, source))?;
    }
    Ok(())
}

fn remove_if_exists(path: &Path) -> Result<(), ProjectReviewError> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(source) => Err(io_error(path, source)),
    }
}

fn append_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut value = path.as_os_str().to_os_string();
    value.push(suffix);
    PathBuf::from(value)
}

fn short_plan_id(plan_id: &str) -> &str {
    plan_id.get(..plan_id.len().min(24)).unwrap_or(plan_id)
}

fn hash_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn io_error(path: &Path, source: io::Error) -> ProjectReviewError {
    ProjectReviewError::Io {
        path: path.to_path_buf(),
        source,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use human_review_workflow::{
        build_promotion_plan, DecisionAction, ReviewProposal, ReviewStatus, ReviewedCharacter,
        ReviewedValue,
    };
    use literary_intelligence_engine::{CharacterSeed, Confidence, EvidenceRef, SeedStatus};

    fn timestamp() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-09-02T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    fn ledger() -> ReviewLedger {
        let source = document_engine_placeholder();
        let proposal = ReviewProposal::Character(Box::new(CharacterSeed {
            id: "seed-character".into(),
            canonical_name_candidate: "Mina".into(),
            aliases: vec!["مینا".into()],
            first_appearance: source.clone(),
            appearance_count: 2,
            evidence: vec![source],
            likely_narrative_role: None,
            speech_register_observations: Vec::new(),
            recurring_lexical_patterns: Vec::new(),
            personality_observations: Vec::new(),
            relationship_refs: Vec::new(),
            confidence: Confidence::from_evidence(2),
            status: SeedStatus::Inferred,
            matches_approved_character: false,
        }));
        let semantic_fingerprint = {
            let bytes = serde_json::to_vec(&proposal).unwrap();
            hash_bytes(&bytes)
        };
        let mut ledger = ReviewLedger {
            schema_version: 1,
            manuscript_id: "book-1".into(),
            manuscript_title: "Synthetic".into(),
            analysis_schema_version: 1,
            items: vec![human_review_workflow::ReviewItem {
                id: "review-1".into(),
                proposal_id: "seed-character".into(),
                kind: human_review_workflow::ReviewKind::Character,
                subject_key: "seed-character".into(),
                revision: 1,
                semantic_fingerprint,
                original_proposal: proposal.clone(),
                latest_proposal: proposal,
                reviewed_value: None,
                status: ReviewStatus::Pending,
                availability: human_review_workflow::ProposalAvailability::Active,
                decisions: Vec::new(),
                archived_revisions: Vec::new(),
            }],
            reconciliations: Vec::new(),
            promotions: Vec::new(),
        };
        ledger
            .decide(
                "review-1",
                DecisionAction::Edit,
                Some(ReviewedValue::Character(ReviewedCharacter {
                    canonical_name: "Mina".into(),
                    aliases: vec!["مینا".into()],
                    voice_notes: "آرام".into(),
                    personality_notes: String::new(),
                })),
                "editor".into(),
                "approved spelling".into(),
                timestamp(),
            )
            .unwrap();
        ledger
    }

    fn document_engine_placeholder() -> EvidenceRef {
        serde_json::from_value(serde_json::json!({
            "chapter_id": "chapter-1",
            "scene_id": "scene-1",
            "paragraph_id": "paragraph-1",
            "source": {
                "path": "synthetic.txt",
                "format": "txt",
                "page": null,
                "resource": null,
                "chapter": 1,
                "scene": 1,
                "paragraph": 1
            }
        }))
        .unwrap()
    }

    #[test]
    fn promotion_is_atomic_and_idempotent() {
        let workspace = tempfile::tempdir().unwrap();
        let paths = ReviewStorePaths {
            review_file: workspace.path().join("review.json"),
            character_bible_file: workspace.path().join("characters.json"),
            glossary_file: workspace.path().join("glossary.json"),
        };
        let ledger = ledger();
        save_review_ledger(&paths.review_file, &ledger).unwrap();
        let plan = build_promotion_plan(
            &ledger,
            &CharacterBible::new(),
            &Glossary::default(),
            &[],
            &[],
        )
        .unwrap();
        let before_review = fs::read(&paths.review_file).unwrap();
        let failed = apply_promotion_inner(
            &paths,
            &ledger,
            &plan,
            "editor",
            "apply",
            timestamp(),
            Some(1),
        );
        assert!(failed.is_err());
        assert_eq!(fs::read(&paths.review_file).unwrap(), before_review);
        assert!(!paths.character_bible_file.exists());
        assert!(!paths.glossary_file.exists());

        let result =
            apply_promotion(&paths, &ledger, &plan, "editor", "apply", timestamp()).unwrap();
        assert_eq!(result.status, PromotionApplyStatus::Applied);
        let applied_ledger = load_review_ledger(&paths.review_file).unwrap();
        assert_eq!(applied_ledger.items[0].status, ReviewStatus::Applied);
        let already = apply_promotion(
            &paths,
            &applied_ledger,
            &plan,
            "editor",
            "apply",
            timestamp(),
        )
        .unwrap();
        assert_eq!(already.status, PromotionApplyStatus::AlreadyApplied);
        let bible = load_character_bible_or_default(&paths.character_bible_file).unwrap();
        assert_eq!(bible.profiles().len(), 1);
        assert_eq!(bible.aliases().len(), 1);
    }
}
