use crate::domain::CharacterProfile;

#[derive(Default)]
pub struct CharacterMemory {
    characters: Vec<CharacterProfile>,
}

impl CharacterMemory {
    pub fn new() -> Self {
        Self { characters: Vec::new() }
    }

    pub fn add(&mut self, character: CharacterProfile) {
        self.characters.push(character);
    }

    pub fn get(&self, name: &str) -> Option<&CharacterProfile> {
        self.characters.iter().find(|item| item.name == name)
    }
}
