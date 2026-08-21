use literary_intelligence_engine::models::{
    character::CharacterProfile,
    relationship::RelationshipState,
    scene::SceneContext,
};
use uuid::Uuid;

#[test]
fn character_voice_profile_survives_serialization() {
    let id = Uuid::new_v4();
    let character = CharacterProfile {
        id,
        novel_context_id: id,
        name: "Mina".into(),
        aliases: vec!["M".into()],
        traits: vec!["guarded".into()],
        motivations: vec!["protect someone".into()],
        fears: vec!["betrayal".into()],
        voice_profile: Some("quiet but sarcastic".into()),
        dialogue_register: Some("informal".into()),
        speech_patterns: vec!["short replies".into()],
        recurring_imagery: vec!["rain".into()],
        relationship_notes: vec!["trust grows slowly".into()],
        first_seen_chapter: Some(1),
        importance_score: 0.9,
    };

    character.validate().unwrap();
    let json = serde_json::to_string(&character).unwrap();
    let restored: CharacterProfile = serde_json::from_str(&json).unwrap();

    assert_eq!(restored.voice_profile.as_deref(), Some("quiet but sarcastic"));
    assert_eq!(restored.speech_patterns, vec!["short replies"]);
}

#[test]
fn relationship_evolution_metadata_is_preserved() {
    let id = Uuid::new_v4();
    let relationship = RelationshipState {
        id,
        character_a: id,
        character_b: Uuid::new_v4(),
        relationship_type: "rivals_to_allies".into(),
        trust_level: 0.7,
        conflict_level: 0.4,
        emotional_distance: 0.2,
        dynamic_notes: vec!["shared secret".into()],
        unresolved_tensions: vec!["old betrayal".into()],
        significant_turning_points: vec!["saved each other".into()],
    };

    relationship.validate().unwrap();
    assert_eq!(relationship.significant_turning_points.len(), 1);
}

#[test]
fn scene_subtext_and_turning_points_are_not_lost() {
    let id = Uuid::new_v4();
    let scene = SceneContext {
        id,
        chapter_id: id,
        purpose: "reveal hidden conflict".into(),
        emotional_tone: Some("tense".into()),
        conflict_level: Some("high".into()),
        character_ids: vec![id],
        importance_score: 1.0,
        symbolic_elements: vec!["mirror".into()],
        narrative_turning_points: vec!["truth revealed".into()],
        subtext_notes: vec!["dialogue hides regret".into()],
    };

    scene.validate().unwrap();
    assert_eq!(scene.subtext_notes[0], "dialogue hides regret");
}
