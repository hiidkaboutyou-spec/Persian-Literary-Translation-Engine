#[derive(Debug, Clone)]
pub struct CharacterProfile {
    pub name: String,
    pub voice_notes: String,
    pub personality_notes: String,
}

pub fn build_character_context(profile: &CharacterProfile) -> String {
    let mut parts = vec![format!("Character: {}", profile.name)];
    if !profile.voice_notes.trim().is_empty() {
        parts.push(format!("Voice: {}", profile.voice_notes.trim()));
    }
    if !profile.personality_notes.trim().is_empty() {
        parts.push(format!("Personality: {}", profile.personality_notes.trim()));
    }
    parts.join("\n")
}

#[derive(Debug, Default, Clone)]
pub struct CharacterBible {
    profiles: Vec<CharacterProfile>,
}

impl CharacterBible {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, profile: CharacterProfile) {
        self.profiles.push(profile);
    }

    pub fn profiles(&self) -> &[CharacterProfile] {
        &self.profiles
    }

    /// Returns only characters explicitly present in the current passage.
    /// This keeps translation prompts focused and prevents voice notes from
    /// unrelated characters from contaminating dialogue decisions.
    pub fn relevant_to_text(&self, text: &str) -> Vec<&CharacterProfile> {
        let normalized_text = normalize_for_matching(text);
        self.profiles
            .iter()
            .filter(|profile| contains_name(&normalized_text, &profile.name))
            .collect()
    }

    /// Builds compact prompt context for characters found in a passage.
    pub fn context_for_text(&self, text: &str) -> String {
        self.relevant_to_text(text)
            .into_iter()
            .map(build_character_context)
            .collect::<Vec<_>>()
            .join("\n\n")
    }
}

fn contains_name(normalized_text: &str, name: &str) -> bool {
    let normalized_name = normalize_for_matching(name);
    if normalized_name.is_empty() {
        return false;
    }

    let padded_text = format!(" {} ", normalized_text);
    padded_text.contains(&format!(" {} ", normalized_name))
}

fn normalize_for_matching(text: &str) -> String {
    text.chars()
        .map(|ch| match ch {
            'ي' | 'ى' => 'ی',
            'ك' => 'ک',
            '\u{200c}' | '\u{200d}' | '\u{00a0}' => ' ',
            c if c.is_alphanumeric() => c,
            _ => ' ',
        })
        .collect::<String>()
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile(name: &str, voice: &str, personality: &str) -> CharacterProfile {
        CharacterProfile {
            name: name.into(),
            voice_notes: voice.into(),
            personality_notes: personality.into(),
        }
    }

    #[test]
    fn context_includes_voice_and_personality() {
        let context = build_character_context(&profile(
            "Magnus",
            "witty, theatrical, affectionate",
            "confident but emotionally guarded",
        ));

        assert!(context.contains("Voice: witty, theatrical, affectionate"));
        assert!(context.contains("Personality: confident but emotionally guarded"));
    }

    #[test]
    fn retrieves_only_characters_present_in_passage() {
        let mut bible = CharacterBible::new();
        bible.add(profile("Magnus", "witty", "warm"));
        bible.add(profile("Alec", "restrained", "loyal"));
        bible.add(profile("Jace", "sarcastic", "reckless"));

        let relevant = bible.relevant_to_text("Alec looked at Magnus and tried not to smile.");
        assert_eq!(relevant.len(), 2);
        assert_eq!(relevant[0].name, "Magnus");
        assert_eq!(relevant[1].name, "Alec");
    }

    #[test]
    fn name_matching_respects_word_boundaries() {
        let mut bible = CharacterBible::new();
        bible.add(profile("Al", "quiet", "observant"));

        assert!(bible.relevant_to_text("Alice crossed the room.").is_empty());
        assert_eq!(bible.relevant_to_text("Al crossed the room.").len(), 1);
    }
}
