use literary_intelligence_engine::models::character::CharacterProfile;
use serde_json;
use uuid::Uuid;

#[test]
fn character_profile_serializes_and_deserializes() {
    let model = CharacterProfile {
        id: Uuid::new_v4(),
        novel_context_id: Uuid::new_v4(),
        name: "Example".to_string(),
        aliases: vec![],
        traits: vec!["quiet".to_string()],
        motivations: vec![],
        fears: vec![],
        voice_profile: None,
        first_seen_chapter: Some(1),
        importance_score: 1.0,
    };

    let json = serde_json::to_string(&model).unwrap();
    let decoded: CharacterProfile = serde_json::from_str(&json).unwrap();

    assert_eq!(decoded.name, "Example");
}
