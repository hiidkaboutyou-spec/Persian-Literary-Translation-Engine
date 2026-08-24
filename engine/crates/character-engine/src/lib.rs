use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationshipProfile {
    pub character_a: String,
    pub character_b: String,
    pub dynamic_notes: String,
    pub address_notes: String,
    pub boundaries_notes: String,
}

impl RelationshipProfile {
    pub fn new(character_a: impl Into<String>, character_b: impl Into<String>) -> Self {
        Self {
            character_a: character_a.into(),
            character_b: character_b.into(),
            dynamic_notes: String::new(),
            address_notes: String::new(),
            boundaries_notes: String::new(),
        }
    }

    fn context(&self) -> String {
        let mut parts = vec![format!(
            "Relationship: {} ↔ {}",
            self.character_a, self.character_b
        )];
        if !self.dynamic_notes.trim().is_empty() {
            parts.push(format!("Dynamic: {}", self.dynamic_notes.trim()));
        }
        if !self.address_notes.trim().is_empty() {
            parts.push(format!("Forms of address: {}", self.address_notes.trim()));
        }
        if !self.boundaries_notes.trim().is_empty() {
            parts.push(format!(
                "Continuity constraints: {}",
                self.boundaries_notes.trim()
            ));
        }
        parts.join("\n")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CharacterAlias {
    pub canonical_name: String,
    pub alias: String,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CharacterBible {
    profiles: Vec<CharacterProfile>,
    aliases: Vec<CharacterAlias>,
    relationships: Vec<RelationshipProfile>,
}

impl CharacterBible {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, profile: CharacterProfile) {
        self.profiles.push(profile);
    }

    pub fn add_alias(&mut self, canonical_name: impl Into<String>, alias: impl Into<String>) {
        let canonical_name = canonical_name.into();
        let alias = alias.into();
        if canonical_name.trim().is_empty() || alias.trim().is_empty() {
            return;
        }
        if self.aliases.iter().any(|existing| {
            normalize_for_matching(&existing.canonical_name)
                == normalize_for_matching(&canonical_name)
                && normalize_for_matching(&existing.alias) == normalize_for_matching(&alias)
        }) {
            return;
        }
        self.aliases.push(CharacterAlias {
            canonical_name,
            alias,
        });
    }

    pub fn add_relationship(&mut self, relationship: RelationshipProfile) {
        self.relationships.push(relationship);
    }

    pub fn profiles(&self) -> &[CharacterProfile] {
        &self.profiles
    }

    pub fn aliases(&self) -> &[CharacterAlias] {
        &self.aliases
    }

    pub fn relationships(&self) -> &[RelationshipProfile] {
        &self.relationships
    }

    pub fn save_json(&self, path: impl AsRef<Path>) -> io::Result<()> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_vec_pretty(self).map_err(io::Error::other)?;
        fs::write(path, json)
    }

    pub fn load_json(path: impl AsRef<Path>) -> io::Result<Self> {
        let bytes = fs::read(path)?;
        serde_json::from_slice(&bytes).map_err(io::Error::other)
    }

    pub fn relevant_to_text(&self, text: &str) -> Vec<&CharacterProfile> {
        let normalized_text = normalize_for_matching(text);
        self.profiles
            .iter()
            .filter(|profile| self.profile_is_present(&normalized_text, profile))
            .collect()
    }

    pub fn relevant_relationships(&self, text: &str) -> Vec<&RelationshipProfile> {
        let normalized_text = normalize_for_matching(text);
        self.relationships
            .iter()
            .filter(|relationship| {
                self.character_is_present(&normalized_text, &relationship.character_a)
                    && self.character_is_present(&normalized_text, &relationship.character_b)
            })
            .collect()
    }

    pub fn context_for_text(&self, text: &str) -> String {
        let mut sections = self
            .relevant_to_text(text)
            .into_iter()
            .map(build_character_context)
            .collect::<Vec<_>>();

        sections.extend(
            self.relevant_relationships(text)
                .into_iter()
                .map(RelationshipProfile::context),
        );

        sections.join("\n\n")
    }

    fn profile_is_present(&self, normalized_text: &str, profile: &CharacterProfile) -> bool {
        self.character_is_present(normalized_text, &profile.name)
    }

    fn character_is_present(&self, normalized_text: &str, canonical_name: &str) -> bool {
        if contains_name(normalized_text, canonical_name) {
            return true;
        }

        let normalized_canonical = normalize_for_matching(canonical_name);
        self.aliases.iter().any(|registered| {
            normalize_for_matching(&registered.canonical_name) == normalized_canonical
                && contains_name(normalized_text, &registered.alias)
        })
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
    text_normalization::normalize_case_insensitive(text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

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
    fn alias_matching_restores_character_voice_context() {
        let mut bible = CharacterBible::new();
        bible.add(profile("Alexander Lightwood", "restrained", "loyal"));
        bible.add_alias("Alexander Lightwood", "Alec");

        let relevant = bible.relevant_to_text("Alec lowered his eyes before answering.");
        assert_eq!(relevant.len(), 1);
        assert_eq!(relevant[0].name, "Alexander Lightwood");
    }

    #[test]
    fn alias_matching_respects_word_boundaries() {
        let mut bible = CharacterBible::new();
        bible.add(profile("Al", "quiet", "observant"));

        assert!(bible.relevant_to_text("Alice crossed the room.").is_empty());
        assert_eq!(bible.relevant_to_text("Al crossed the room.").len(), 1);
    }

    #[test]
    fn relationship_context_requires_both_characters() {
        let mut bible = CharacterBible::new();
        bible.add(profile("Magnus", "witty", "warm"));
        bible.add(profile("Alexander Lightwood", "restrained", "loyal"));
        bible.add_alias("Alexander Lightwood", "Alec");

        let mut relationship = RelationshipProfile::new("Magnus", "Alexander Lightwood");
        relationship.dynamic_notes = "playful teasing with underlying tenderness".into();
        relationship.address_notes = "Magnus often calls Alexander 'darling'".into();
        bible.add_relationship(relationship);

        assert!(bible
            .relevant_relationships("Magnus stood alone by the window.")
            .is_empty());

        let relationships = bible.relevant_relationships("Alec looked over at Magnus.");
        assert_eq!(relationships.len(), 1);
        assert!(relationships[0].dynamic_notes.contains("teasing"));
    }

    #[test]
    fn context_orders_character_voice_before_relationship_dynamics() {
        let mut bible = CharacterBible::new();
        bible.add(profile("Magnus", "witty", "warm"));
        bible.add(profile("Alec", "restrained", "loyal"));

        let mut relationship = RelationshipProfile::new("Magnus", "Alec");
        relationship.dynamic_notes = "affectionate banter".into();
        bible.add_relationship(relationship);

        let context = bible.context_for_text("Magnus smiled at Alec.");
        let voice_position = context.find("Voice: witty").unwrap();
        let relationship_position = context.find("Relationship: Magnus ↔ Alec").unwrap();
        assert!(voice_position < relationship_position);
    }

    #[test]
    fn duplicate_aliases_are_not_added_twice() {
        let mut bible = CharacterBible::new();
        bible.add_alias("Alexander Lightwood", "Alec");
        bible.add_alias("Alexander Lightwood", "Alec");
        assert_eq!(bible.aliases().len(), 1);
    }

    #[test]
    fn character_bible_round_trips_as_json() {
        let mut bible = CharacterBible::new();
        bible.add(profile("Magnus", "witty", "warm"));
        bible.add_alias("Magnus", "Bane");
        let mut relationship = RelationshipProfile::new("Magnus", "Alec");
        relationship.dynamic_notes = "affectionate banter".into();
        bible.add_relationship(relationship);

        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("character-bible-{nonce}.json"));

        bible.save_json(&path).unwrap();
        let loaded = CharacterBible::load_json(&path).unwrap();
        let _ = fs::remove_file(&path);

        assert_eq!(loaded, bible);
        assert_eq!(
            loaded.context_for_text("Magnus smiled at Alec."),
            bible.context_for_text("Magnus smiled at Alec.")
        );
    }
}
