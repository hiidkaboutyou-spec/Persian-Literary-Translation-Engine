use std::cmp::Ordering;
use std::collections::HashSet;

use crate::models::{Character, GlossaryEntry, RelationshipMemory, TranslationMemoryEntry};

#[derive(Debug, Clone)]
pub struct ScoredMemory<'a> {
    pub entry: &'a TranslationMemoryEntry,
    pub score: f32,
}

#[derive(Debug, Default, Clone)]
pub struct LiteraryMemory {
    translation_memory: Vec<TranslationMemoryEntry>,
    glossary: Vec<GlossaryEntry>,
    characters: Vec<Character>,
    relationships: Vec<RelationshipMemory>,
}

impl LiteraryMemory {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn add_translation(&mut self, entry: TranslationMemoryEntry) {
        self.translation_memory.push(entry);
    }
    pub fn add_glossary_entry(&mut self, entry: GlossaryEntry) {
        self.glossary.push(entry);
    }
    pub fn add_character(&mut self, character: Character) {
        self.characters.push(character);
    }
    pub fn add_relationship(&mut self, relationship: RelationshipMemory) {
        self.relationships.push(relationship);
    }

    pub fn search_translation_memory(&self, query: &str, limit: usize) -> Vec<ScoredMemory<'_>> {
        if query.trim().is_empty() || limit == 0 {
            return Vec::new();
        }
        let mut hits: Vec<_> = self
            .translation_memory
            .iter()
            .filter_map(|entry| {
                let source_score = text_similarity(query, &entry.source);
                let context_score = text_similarity(query, &entry.context) * 0.35;
                let score = (source_score + context_score).min(1.0);
                (score > 0.0).then_some(ScoredMemory { entry, score })
            })
            .collect();
        hits.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(Ordering::Equal));
        hits.truncate(limit);
        hits
    }

    pub fn glossary_for_text(&self, source_text: &str) -> Vec<&GlossaryEntry> {
        let normalized = normalize(source_text);
        self.glossary
            .iter()
            .filter(|entry| contains_phrase(&normalized, &normalize(&entry.source)))
            .collect()
    }

    pub fn characters_for_text(&self, source_text: &str) -> Vec<&Character> {
        let normalized = normalize(source_text);
        self.characters
            .iter()
            .filter(|character| {
                contains_phrase(&normalized, &normalize(&character.name))
                    || character
                        .aliases
                        .iter()
                        .any(|alias| contains_phrase(&normalized, &normalize(alias)))
            })
            .collect()
    }

    pub fn relationships_for_characters<'a>(
        &'a self,
        character_names: &[&str],
    ) -> Vec<&'a RelationshipMemory> {
        let names: HashSet<String> = character_names.iter().map(|name| normalize(name)).collect();
        self.relationships
            .iter()
            .filter(|relationship| {
                names.contains(&normalize(&relationship.character_a))
                    && names.contains(&normalize(&relationship.character_b))
            })
            .collect()
    }

    pub fn build_context(&self, source_text: &str, memory_limit: usize) -> RetrievalContext<'_> {
        let characters = self.characters_for_text(source_text);
        let character_names: Vec<&str> = characters.iter().map(|c| c.name.as_str()).collect();
        RetrievalContext {
            translation_memory: self.search_translation_memory(source_text, memory_limit),
            glossary: self.glossary_for_text(source_text),
            relationships: self.relationships_for_characters(&character_names),
            characters,
        }
    }
}

#[derive(Debug)]
pub struct RetrievalContext<'a> {
    pub translation_memory: Vec<ScoredMemory<'a>>,
    pub glossary: Vec<&'a GlossaryEntry>,
    pub characters: Vec<&'a Character>,
    pub relationships: Vec<&'a RelationshipMemory>,
}

fn normalize(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|ch| if ch.is_alphanumeric() { ch } else { ' ' })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn contains_phrase(haystack: &str, needle: &str) -> bool {
    !needle.is_empty() && haystack.contains(needle)
}

fn text_similarity(a: &str, b: &str) -> f32 {
    let a_norm = normalize(a);
    let b_norm = normalize(b);
    if a_norm.is_empty() || b_norm.is_empty() {
        return 0.0;
    }
    if a_norm == b_norm {
        return 1.0;
    }
    let a_tokens: HashSet<&str> = a_norm.split_whitespace().collect();
    let b_tokens: HashSet<&str> = b_norm.split_whitespace().collect();
    let intersection = a_tokens.intersection(&b_tokens).count() as f32;
    let union = a_tokens.union(&b_tokens).count() as f32;
    let jaccard = if union == 0.0 {
        0.0
    } else {
        intersection / union
    };
    let substring_bonus = if a_norm.contains(&b_norm) || b_norm.contains(&a_norm) {
        0.25
    } else {
        0.0
    };
    (jaccard + substring_bonus).min(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retrieves_similar_translation_memory() {
        let mut memory = LiteraryMemory::new();
        memory.add_translation(TranslationMemoryEntry {
            source: "a quiet smile".into(),
            translation: "لبخندی آرام".into(),
            context: "gentle dialogue".into(),
        });
        let hits = memory.search_translation_memory("quiet smile", 3);
        assert_eq!(hits.len(), 1);
        assert!(hits[0].score > 0.0);
    }
}
