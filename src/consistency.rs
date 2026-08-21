use std::collections::HashMap;

use crate::models::{GlossaryEntry, TranslationMemoryEntry};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConsistencyIssueKind {
    MissingPreferredTerm,
    ConflictingMemory,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsistencyIssue {
    pub kind: ConsistencyIssueKind,
    pub message: String,
}

#[derive(Debug, Default, Clone)]
pub struct ConsistencyReport {
    pub issues: Vec<ConsistencyIssue>,
}

impl ConsistencyReport {
    pub fn is_clean(&self) -> bool { self.issues.is_empty() }

    pub fn score(&self) -> f32 {
        (1.0 - self.issues.len() as f32 * 0.1).max(0.0)
    }
}

pub fn check_glossary_usage(
    source_text: &str,
    translated_text: &str,
    glossary: &[GlossaryEntry],
) -> ConsistencyReport {
    let source = normalize(source_text);
    let translated = normalize(translated_text);
    let mut report = ConsistencyReport::default();

    for entry in glossary {
        let source_term = normalize(&entry.source);
        let preferred = normalize(&entry.preferred_translation);
        if !source_term.is_empty()
            && source.contains(&source_term)
            && !preferred.is_empty()
            && !translated.contains(&preferred)
        {
            report.issues.push(ConsistencyIssue {
                kind: ConsistencyIssueKind::MissingPreferredTerm,
                message: format!(
                    "Preferred glossary translation '{}' is missing for source term '{}'.",
                    entry.preferred_translation, entry.source
                ),
            });
        }
    }

    report
}

pub fn find_translation_memory_conflicts(
    entries: &[TranslationMemoryEntry],
) -> ConsistencyReport {
    let mut report = ConsistencyReport::default();
    let mut seen: HashMap<String, String> = HashMap::new();

    for entry in entries {
        let source = normalize(&entry.source);
        let translation = normalize(&entry.translation);
        if source.is_empty() || translation.is_empty() { continue; }

        match seen.get(&source) {
            Some(existing) if existing != &translation => {
                report.issues.push(ConsistencyIssue {
                    kind: ConsistencyIssueKind::ConflictingMemory,
                    message: format!(
                        "Translation memory contains conflicting translations for '{}'.",
                        entry.source
                    ),
                });
            }
            None => { seen.insert(source, translation); }
            _ => {}
        }
    }

    report
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flags_missing_preferred_glossary_term() {
        let glossary = vec![GlossaryEntry {
            source: "Institute".into(),
            preferred_translation: "مؤسسه".into(),
            notes: String::new(),
        }];
        let report = check_glossary_usage("At the Institute", "در ساختمان", &glossary);
        assert_eq!(report.issues.len(), 1);
        assert!(!report.is_clean());
    }

    #[test]
    fn flags_conflicting_memory_entries() {
        let entries = vec![
            TranslationMemoryEntry { source: "quiet smile".into(), translation: "لبخند آرام".into(), context: String::new() },
            TranslationMemoryEntry { source: "quiet smile".into(), translation: "لبخند ساکت".into(), context: String::new() },
        ];
        let report = find_translation_memory_conflicts(&entries);
        assert_eq!(report.issues.len(), 1);
    }
}
