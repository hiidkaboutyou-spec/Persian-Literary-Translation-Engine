use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ReviewStatus { Draft, PendingReview, Approved, Rejected, NeedsRevision }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ReviewCategory { Translation, Voice, Continuity, Terminology, Formatting, Other }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ReviewDecisionType { Approve, Reject, RequestRevision }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ManuscriptStatus { Draft, Review, Approved, Published }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReviewItem {
    pub id: Uuid,
    pub project_id: Uuid,
    pub chapter_id: Uuid,
    pub source_text: String,
    pub translated_text: String,
    pub quality_report_reference: Option<Uuid>,
    pub status: ReviewStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReviewComment {
    pub id: Uuid,
    pub review_item_id: Uuid,
    pub author: String,
    pub comment: String,
    pub category: ReviewCategory,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReviewDecision {
    pub id: Uuid,
    pub review_item_id: Uuid,
    pub decision: ReviewDecisionType,
    pub reviewer: String,
    pub reason: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RevisionHistory {
    pub id: Uuid,
    pub document_id: Uuid,
    pub version: u32,
    pub previous_version: Option<u32>,
    pub change_summary: String,
    pub changed_by: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ManuscriptVersion {
    pub id: Uuid,
    pub project_id: Uuid,
    pub version_number: u32,
    pub status: ManuscriptStatus,
    pub chapters: Vec<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PublishingPackage {
    pub id: Uuid,
    pub manuscript_version_id: Uuid,
    pub title: String,
    pub author: String,
    pub language: String,
    pub chapter_count: u32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkflowAuditEvent {
    pub id: Uuid,
    pub entity_id: Uuid,
    pub event_type: String,
    pub actor: String,
    pub timestamp: DateTime<Utc>,
    pub details: String,
}

pub fn can_update_canon(decision: &ReviewDecision) -> bool {
    matches!(decision.decision, ReviewDecisionType::Approve)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn approval_allows_canon_update() {
        let decision = ReviewDecision { id: Uuid::new_v4(), review_item_id: Uuid::new_v4(), decision: ReviewDecisionType::Approve, reviewer: "editor".into(), reason: "ok".into(), created_at: Utc::now() };
        assert!(can_update_canon(&decision));
    }

    #[test]
    fn rejection_blocks_canon_update() {
        let decision = ReviewDecision { id: Uuid::new_v4(), review_item_id: Uuid::new_v4(), decision: ReviewDecisionType::Reject, reviewer: "editor".into(), reason: "fix".into(), created_at: Utc::now() };
        assert!(!can_update_canon(&decision));
    }
}
