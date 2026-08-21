use chrono::Utc;
use literary_intelligence_engine::models::{
    character::CharacterProfile, context_snapshot::ContextSnapshot, literary_rule::LiteraryRule,
    novel_context::NovelContext, relationship::RelationshipState, scene::SceneContext,
    translation_decision::TranslationDecision, understanding::LiteraryUnderstanding,
};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use uuid::Uuid;

fn round_trip<T>(value: T)
where
    T: Serialize + DeserializeOwned + PartialEq + std::fmt::Debug,
{
    let json = serde_json::to_string(&value).unwrap();
    let restored: T = serde_json::from_str(&json).unwrap();
    assert_eq!(value, restored);
}

fn valid_understanding() -> LiteraryUnderstanding {
    let id = Uuid::new_v4();
    LiteraryUnderstanding {
        novel_context: NovelContext {
            id,
            project_id: id,
            title: "Novel".into(),
            genre: Some("dark romance".into()),
            tone: Some("intimate".into()),
            global_rules: vec!["preserve voice".into()],
            created_at: Utc::now(),
        },
        characters: vec![CharacterProfile {
            id,
            novel_context_id: id,
            name: "A".into(),
            aliases: vec!["Alias".into()],
            traits: vec!["kind".into()],
            motivations: vec!["goal".into()],
            fears: vec!["loss".into()],
            desires: vec!["peace".into()],
            voice_profile: Some("restrained".into()),
            speech_style: Some("quiet".into()),
            emotional_patterns: vec!["withdraws".into()],
            first_seen_chapter: Some(1),
            importance_score: 1.0,
        }],
        scenes: vec![SceneContext {
            id,
            chapter_id: id,
            purpose: "turn".into(),
            emotional_tone: Some("tense".into()),
            character_ids: vec![id],
            importance_score: 0.5,
            symbolic_elements: vec!["rain".into()],
            conflict_level: 0.7,
            character_goals: vec!["resolve".into()],
            narrative_events: vec!["reveal".into()],
        }],
        relationships: vec![RelationshipState {
            id,
            character_a: id,
            character_b: Uuid::new_v4(),
            relationship_type: "friend".into(),
            trust_level: 0.5,
            conflict_level: 0.2,
            emotional_distance: 0.3,
            emotional_direction: Some("improving".into()),
            trust_change: 0.1,
            conflict_history: vec!["argument".into()],
        }],
        translation_decisions: vec![TranslationDecision {
            id,
            source_text: "darling".into(),
            translated_text: "عزیزم".into(),
            reason: "intimacy".into(),
            original_expression: "darling".into(),
            chosen_expression: "عزیزم".into(),
            relationship_context: Some("partners".into()),
            literary_effect: Some("affection".into()),
            chapter_id: Some(id),
            decision_type: "dialogue".into(),
            confidence: 0.8,
        }],
        rules: vec![LiteraryRule {
            id,
            novel_context_id: id,
            rule_text: "preserve voice".into(),
            scope: "book".into(),
            active: true,
        }],
        created_at: Utc::now(),
    }
}

#[test]
fn all_models_round_trip_with_literary_fields() {
    let understanding = valid_understanding();
    let id = understanding.novel_context.id;

    round_trip(understanding.characters[0].clone());
    round_trip(understanding.scenes[0].clone());
    round_trip(understanding.relationships[0].clone());
    round_trip(understanding.translation_decisions[0].clone());
    round_trip(understanding.novel_context.clone());
    round_trip(ContextSnapshot {
        id,
        novel_context_id: id,
        chapter_id: id,
        active_character_ids: vec![id],
        active_relationship_ids: vec![],
        relevant_rules: vec![],
    });
    round_trip(understanding.rules[0].clone());
}

#[test]
fn invalid_values_fail_validation() {
    let id = Uuid::new_v4();
    assert!(CharacterProfile {
        id,
        novel_context_id: id,
        name: " ".into(),
        aliases: vec![],
        traits: vec![],
        motivations: vec![],
        fears: vec![],
        desires: vec![],
        voice_profile: None,
        speech_style: None,
        emotional_patterns: vec![],
        first_seen_chapter: None,
        importance_score: 0.5,
    }
    .validate()
    .is_err());
    assert!(SceneContext {
        id,
        chapter_id: id,
        purpose: "x".into(),
        emotional_tone: None,
        character_ids: vec![],
        importance_score: 0.5,
        symbolic_elements: vec![],
        conflict_level: 2.0,
        character_goals: vec![],
        narrative_events: vec![],
    }
    .validate()
    .is_err());
    assert!(RelationshipState {
        id,
        character_a: id,
        character_b: id,
        relationship_type: "x".into(),
        trust_level: 0.0,
        conflict_level: 0.0,
        emotional_distance: 0.0,
        emotional_direction: None,
        trust_change: 2.0,
        conflict_history: vec![],
    }
    .validate()
    .is_err());
    assert!(TranslationDecision {
        id,
        source_text: "x".into(),
        translated_text: "y".into(),
        reason: "".into(),
        original_expression: "x".into(),
        chosen_expression: "y".into(),
        relationship_context: None,
        literary_effect: None,
        chapter_id: None,
        decision_type: "d".into(),
        confidence: 0.8,
    }
    .validate()
    .is_err());
}

#[test]
fn literary_understanding_creation_populates_all_fields() {
    let understanding = valid_understanding();
    assert!(!understanding.novel_context.title.is_empty());
    assert!(!understanding.characters.is_empty());
    assert!(!understanding.scenes.is_empty());
    assert!(!understanding.relationships.is_empty());
    assert!(!understanding.translation_decisions.is_empty());
    assert!(!understanding.rules.is_empty());
    assert!(understanding.validate().is_ok());
}

#[test]
fn literary_understanding_serializes_expected_structure() {
    let understanding = valid_understanding();
    let json = serde_json::to_value(&understanding).unwrap();
    let object = json.as_object().unwrap();

    for field in [
        "novel_context",
        "characters",
        "scenes",
        "relationships",
        "translation_decisions",
        "rules",
        "created_at",
    ] {
        assert!(object.contains_key(field), "missing field: {field}");
    }
}

#[test]
fn literary_understanding_deserializes_from_json() {
    let original = valid_understanding();
    let json = serde_json::to_string(&original).unwrap();
    let restored: LiteraryUnderstanding = serde_json::from_str(&json).unwrap();

    assert_eq!(restored.novel_context.title, "Novel");
    assert_eq!(restored.characters.len(), 1);
    assert_eq!(restored.scenes.len(), 1);
    assert_eq!(restored.relationships.len(), 1);
    assert_eq!(restored.translation_decisions.len(), 1);
    assert_eq!(restored.rules.len(), 1);
}

#[test]
fn literary_understanding_equality_round_trip() {
    round_trip(valid_understanding());
}

#[test]
fn literary_understanding_rejects_invalid_nested_models() {
    let mut invalid_novel = valid_understanding();
    invalid_novel.novel_context.title = " ".into();
    assert!(invalid_novel.validate().is_err());

    let mut invalid_character = valid_understanding();
    invalid_character.characters[0].importance_score = 1.1;
    assert!(invalid_character.validate().is_err());

    let mut invalid_scene_importance = valid_understanding();
    invalid_scene_importance.scenes[0].importance_score = -0.1;
    assert!(invalid_scene_importance.validate().is_err());

    let mut invalid_scene_conflict = valid_understanding();
    invalid_scene_conflict.scenes[0].conflict_level = 1.1;
    assert!(invalid_scene_conflict.validate().is_err());

    let mut invalid_relationship_trust = valid_understanding();
    invalid_relationship_trust.relationships[0].trust_level = 1.1;
    assert!(invalid_relationship_trust.validate().is_err());

    let mut invalid_relationship_change = valid_understanding();
    invalid_relationship_change.relationships[0].trust_change = -1.1;
    assert!(invalid_relationship_change.validate().is_err());

    let mut invalid_decision_confidence = valid_understanding();
    invalid_decision_confidence.translation_decisions[0].confidence = 1.1;
    assert!(invalid_decision_confidence.validate().is_err());

    let mut invalid_decision_reason = valid_understanding();
    invalid_decision_reason.translation_decisions[0].reason.clear();
    assert!(invalid_decision_reason.validate().is_err());
}

#[test]
fn literary_understanding_json_is_an_object() {
    let json = serde_json::to_value(valid_understanding()).unwrap();
    assert!(matches!(json, Value::Object(_)));
}
