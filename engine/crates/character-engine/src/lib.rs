#[derive(Debug, Clone)]
pub struct CharacterProfile {
    pub name: String,
    pub voice_notes: String,
    pub personality_notes: String,
}

pub fn build_character_context(profile: &CharacterProfile) -> String {
    format!("{}: {}", profile.name, profile.voice_notes)
}
