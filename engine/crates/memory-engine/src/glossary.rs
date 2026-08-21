use crate::retrieval::normalize;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlossaryEntry {
    pub source_term: String,
    pub preferred_translation: String,
    pub context: String,
}

#[derive(Debug, Default)]
pub struct Glossary {
    entries: Vec<GlossaryEntry>,
}

impl Glossary {
    pub fn add(&mut self, entry: GlossaryEntry) {
        self.entries.push(entry);
    }

    pub fn entries(&self) -> &[GlossaryEntry] {
        &self.entries
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
}
