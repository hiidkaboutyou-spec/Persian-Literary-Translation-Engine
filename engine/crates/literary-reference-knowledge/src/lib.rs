use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ReferenceValidationError {
    #[error("title cannot be empty")]
    EmptyTitle,
    #[error("author cannot be empty")]
    EmptyAuthor,
    #[error("source relationship is missing")]
    MissingSource,
    #[error("principle cannot be empty")]
    EmptyPrinciple,
    #[error("guideline relationship is missing")]
    MissingGuideline,
    #[error("unsupported validation type")]
    UnsupportedValidationType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReferenceSource {
    pub id: Uuid,
    pub title: String,
    pub author: String,
    pub category: String,
    pub description: String,
    pub usage_scope: Vec<String>,
    pub created_at: DateTime<Utc>,
}

impl ReferenceSource {
    pub fn validate(&self) -> Result<(), ReferenceValidationError> {
        if self.title.trim().is_empty() {
            return Err(ReferenceValidationError::EmptyTitle);
        }
        if self.author.trim().is_empty() {
            return Err(ReferenceValidationError::EmptyAuthor);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LiteraryGuideline {
    pub id: Uuid,
    pub source_id: Uuid,
    pub category: String,
    pub principle: String,
    pub explanation: String,
}

impl LiteraryGuideline {
    pub fn validate(&self) -> Result<(), ReferenceValidationError> {
        if self.source_id.is_nil() {
            return Err(ReferenceValidationError::MissingSource);
        }
        if self.principle.trim().is_empty() {
            return Err(ReferenceValidationError::EmptyPrinciple);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ValidationType {
    VoicePreservation,
    ContinuityCheck,
    LiteraryConsistency,
    DecisionCompliance,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EditorialRule {
    pub id: Uuid,
    pub guideline_id: Uuid,
    pub validation_type: ValidationType,
    pub severity: Severity,
    pub description: String,
}

impl EditorialRule {
    pub fn validate(&self) -> Result<(), ReferenceValidationError> {
        if self.guideline_id.is_nil() {
            return Err(ReferenceValidationError::MissingGuideline);
        }
        if self.description.trim().is_empty() {
            return Err(ReferenceValidationError::UnsupportedValidationType);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source() -> ReferenceSource {
        ReferenceSource {
            id: Uuid::new_v4(),
            title: "Literary Source".into(),
            author: "Author".into(),
            category: "editing".into(),
            description: "test".into(),
            usage_scope: vec!["voice".into()],
            created_at: Utc::now(),
        }
    }

    #[test]
    fn validates_reference_source() {
        assert!(source().validate().is_ok());
    }

    #[test]
    fn rejects_empty_source_fields() {
        let mut value = source();
        value.title.clear();
        assert_eq!(value.validate(), Err(ReferenceValidationError::EmptyTitle));
    }

    #[test]
    fn validates_guideline_relationship() {
        let guideline = LiteraryGuideline {
            id: Uuid::new_v4(),
            source_id: Uuid::new_v4(),
            category: "dialogue".into(),
            principle: "Preserve voice".into(),
            explanation: "Identity remains distinct".into(),
        };
        assert!(guideline.validate().is_ok());
    }

    #[test]
    fn validates_editorial_rule() {
        let rule = EditorialRule {
            id: Uuid::new_v4(),
            guideline_id: Uuid::new_v4(),
            validation_type: ValidationType::VoicePreservation,
            severity: Severity::High,
            description: "Keep voice".into(),
        };
        assert!(rule.validate().is_ok());
    }

    #[test]
    fn json_roundtrip() {
        let value = source();
        let json = serde_json::to_string(&value).unwrap();
        let decoded: ReferenceSource = serde_json::from_str(&json).unwrap();
        assert_eq!(value, decoded);
    }
}
