use chrono::Utc;
use literary_intelligence_engine::models::{
    character::CharacterProfile, context_snapshot::ContextSnapshot, literary_rule::LiteraryRule,
    novel_context::NovelContext, relationship::RelationshipState, scene::SceneContext,
    translation_decision::TranslationDecision,
};
use serde::{de::DeserializeOwned, Serialize};
use uuid::Uuid;

fn round_trip<T>(value: T)
where
    T: Serialize + DeserializeOwned + PartialEq + std::fmt::Debug,
{
    let json = serde_json::to_string(&value).unwrap();
    let restored: T = serde_json::from_str(&json).unwrap();
    assert_eq!(value, restored);
}

#[test]
fn all_models_round_trip_with_literary_fields() {
    let id = Uuid::new_v4();
    round_trip(CharacterProfile {
        id,
        novel_context_id: id,
        name: "A".into(),
        aliases: vec![],
        traits: vec!["kind".into()],
        motivations: vec!["goal".into()],
        fears: vec![],
        desires: vec!["peace".into()],
        voice_profile: None,
        speech_style: Some("quiet".into()),
        emotional_patterns: vec!["withdraws".into()],
        first_seen_chapter: Some(1),
        importance_score: 1.0,
    });
    round_trip(SceneContext {
        id,
        chapter_id: id,
        purpose: "turn".into(),
        emotional_tone: None,
        character_ids: vec![id],
        importance_score: 0.5,
        symbolic_elements: vec![],
        conflict_level: 0.7,
        character_goals: vec!["resolve".into()],
        narrative_events: vec!["reveal".into()],
    });
    round_trip(RelationshipState {
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
    });
    round_trip(TranslationDecision {
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
    });
    round_trip(NovelContext {
        id,
        project_id: id,
        title: "Novel".into(),
        genre: None,
        tone: None,
        global_rules: vec![],
        created_at: Utc::now(),
    });
    round_trip(ContextSnapshot {
        id,
        novel_context_id: id,
        chapter_id: id,
        active_character_ids: vec![id],
        active_relationship_ids: vec![],
        relevant_rules: vec![],
    });
    round_trip(LiteraryRule {
        id,
        novel_context_id: id,
        rule_text: "rule".into(),
        scope: "book".into(),
        active: true,
    });
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
        importance_score: 0.5
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
        narrative_events: vec![]
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
        conflict_history: vec![]
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
        confidence: 0.8
    }
    .validate()
    .is_err());
}
