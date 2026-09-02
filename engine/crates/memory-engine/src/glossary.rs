use crate::retrieval::normalize;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GlossaryEntry {
    pub source_term: String,
    pub preferred_translation: String,
    pub context: String,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Glossary {
    entries: Vec<GlossaryEntry>,
}

impl Glossary {
    pub fn add(&mut self, entry: GlossaryEntry) {
        self.entries.push(entry);
    }

    pub fn upsert(
        &mut self,
        entry: GlossaryEntry,
        replace_existing: bool,
    ) -> Result<GlossaryUpdate, GlossaryCanonError> {
        if normalize(&entry.source_term).is_empty() {
            return Err(GlossaryCanonError::EmptySourceTerm);
        }
        if normalize(&entry.preferred_translation).is_empty() {
            return Err(GlossaryCanonError::EmptyTranslation);
        }
        let wanted = normalize(&entry.source_term);
        let matching_indices: Vec<_> = self
            .entries
            .iter()
            .enumerate()
            .filter_map(|(index, existing)| {
                (normalize(&existing.source_term) == wanted).then_some(index)
            })
            .collect();
        if let Some(&first_index) = matching_indices.first() {
            let existing = &self.entries[first_index];
            if normalize(&existing.preferred_translation) == normalize(&entry.preferred_translation)
                && existing.context == entry.context
                && matching_indices.len() == 1
            {
                return Ok(GlossaryUpdate::Unchanged);
            }
            if !replace_existing {
                return Err(GlossaryCanonError::TranslationConflict {
                    source_term: existing.source_term.clone(),
                    existing_translation: existing.preferred_translation.clone(),
                    proposed_translation: entry.preferred_translation,
                });
            }
            for index in matching_indices.into_iter().rev() {
                self.entries.remove(index);
            }
            self.entries.insert(first_index, entry);
            return Ok(GlossaryUpdate::Replaced);
        }
        self.entries.push(entry);
        Ok(GlossaryUpdate::Inserted)
    }

    pub fn entries(&self) -> &[GlossaryEntry] {
        &self.entries
    }

    pub fn entries_for_term(&self, source_term: &str) -> Vec<&GlossaryEntry> {
        let wanted = normalize(source_term);
        self.entries
            .iter()
            .filter(|entry| normalize(&entry.source_term) == wanted)
            .collect()
    }

    /// Returns glossary entries whose source term actually appears in the
    /// current passage. Longest terms are returned first so multi-word terms
    /// win over shorter overlapping entries when prompt context is assembled.
    pub fn relevant_to_text(&self, text: &str) -> Vec<&GlossaryEntry> {
        let normalized_text = format!(" {} ", normalize(text));
        let mut matches: Vec<_> = self
            .entries
            .iter()
            .filter(|entry| {
                let term = normalize(&entry.source_term);
                !term.is_empty() && normalized_text.contains(&format!(" {} ", term))
            })
            .collect();

        matches.sort_by(|a, b| {
            normalize(&b.source_term)
                .split_whitespace()
                .count()
                .cmp(&normalize(&a.source_term).split_whitespace().count())
                .then_with(|| b.source_term.len().cmp(&a.source_term.len()))
        });
        matches
    }

    /// Finds a glossary entry using Persian/Arabic letter normalization and
    /// punctuation-insensitive matching.
    pub fn find_exact_term(&self, source_term: &str) -> Option<&GlossaryEntry> {
        let wanted = normalize(source_term);
        self.entries
            .iter()
            .find(|entry| normalize(&entry.source_term) == wanted)
    }

    /// Detects glossary terms that have been assigned more than one preferred
    /// translation. These conflicts are especially harmful in long fiction,
    /// where names, titles and recurring world-building terms must stay fixed.
    pub fn conflicting_terms(&self) -> Vec<GlossaryConflict<'_>> {
        let mut conflicts = Vec::new();

        for (index, entry) in self.entries.iter().enumerate() {
            let term = normalize(&entry.source_term);
            let translation = normalize(&entry.preferred_translation);

            if self.entries[..index].iter().any(|previous| {
                normalize(&previous.source_term) == term
                    && normalize(&previous.preferred_translation) == translation
            }) {
                continue;
            }

            let alternatives: Vec<_> = self
                .entries
                .iter()
                .filter(|candidate| {
                    normalize(&candidate.source_term) == term
                        && normalize(&candidate.preferred_translation) != translation
                })
                .collect();

            if !alternatives.is_empty() {
                conflicts.push(GlossaryConflict {
                    canonical: entry,
                    alternatives,
                });
            }
        }

        conflicts
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlossaryUpdate {
    Inserted,
    Replaced,
    Unchanged,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GlossaryCanonError {
    EmptySourceTerm,
    EmptyTranslation,
    TranslationConflict {
        source_term: String,
        existing_translation: String,
        proposed_translation: String,
    },
}

impl std::fmt::Display for GlossaryCanonError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptySourceTerm => write!(formatter, "glossary source term cannot be empty"),
            Self::EmptyTranslation => write!(formatter, "preferred translation cannot be empty"),
            Self::TranslationConflict {
                source_term,
                existing_translation,
                proposed_translation,
            } => write!(
                formatter,
                "glossary term '{source_term}' is already '{existing_translation}', not '{proposed_translation}'"
            ),
        }
    }
}

impl std::error::Error for GlossaryCanonError {}

#[derive(Debug)]
pub struct GlossaryConflict<'a> {
    pub canonical: &'a GlossaryEntry,
    pub alternatives: Vec<&'a GlossaryEntry>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(source: &str, translation: &str, context: &str) -> GlossaryEntry {
        GlossaryEntry {
            source_term: source.into(),
            preferred_translation: translation.into(),
            context: context.into(),
        }
    }

    #[test]
    fn retrieves_only_terms_present_in_passage() {
        let mut glossary = Glossary::default();
        glossary.add(entry("Shadow World", "دنیای سایه‌ها", "world-building"));
        glossary.add(entry("Shadow", "سایه", "general"));
        glossary.add(entry("Parabatai", "پاراباتای", "title"));

        let hits = glossary.relevant_to_text("He returned to the Shadow World before dawn.");
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].source_term, "Shadow World");
        assert_eq!(hits[1].source_term, "Shadow");
    }

    #[test]
    fn exact_lookup_normalizes_persian_letter_variants() {
        let mut glossary = Glossary::default();
        glossary.add(entry("كتاب", "کتاب", "object"));

        assert!(glossary.find_exact_term("کتاب").is_some());
    }

    #[test]
    fn reports_conflicting_preferred_translations() {
        let mut glossary = Glossary::default();
        glossary.add(entry("High Warlock", "جادوگر اعظم", "title"));
        glossary.add(entry("High Warlock", "وارلاک اعظم", "title"));

        let conflicts = glossary.conflicting_terms();
        assert_eq!(conflicts.len(), 2);
        assert_eq!(conflicts[0].alternatives.len(), 1);
    }

    #[test]
    fn checked_upsert_prevents_silent_approved_translation_changes() {
        let mut glossary = Glossary::default();
        glossary
            .upsert(entry("Duke", "دوک اعظم", "title"), false)
            .unwrap();
        let error = glossary
            .upsert(entry("duke", "دوک", "edited"), false)
            .unwrap_err();
        assert!(matches!(
            error,
            GlossaryCanonError::TranslationConflict { .. }
        ));
        glossary
            .upsert(entry("duke", "دوک", "edited"), true)
            .unwrap();
        assert_eq!(glossary.entries().len(), 1);
        assert_eq!(
            glossary
                .find_exact_term("DUKE")
                .unwrap()
                .preferred_translation,
            "دوک"
        );
    }

    #[test]
    fn explicit_replacement_collapses_preexisting_duplicate_terms() {
        let mut glossary = Glossary::default();
        glossary.add(entry("Duke", "دوک اعظم", "old title"));
        glossary.add(entry("duke", "دوک", "other title"));

        let update = glossary
            .upsert(entry("DUKE", "دوک", "approved title"), true)
            .unwrap();

        assert_eq!(update, GlossaryUpdate::Replaced);
        assert_eq!(glossary.entries_for_term("duke").len(), 1);
        assert_eq!(
            glossary.find_exact_term("duke"),
            Some(&entry("DUKE", "دوک", "approved title"))
        );
    }
}
