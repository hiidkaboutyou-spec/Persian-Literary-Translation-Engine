use crate::glossary::Glossary;
use crate::{RetrievalConfig, TranslationMemory};

#[derive(Debug, Clone)]
pub struct MemoryContextConfig {
    pub max_memory_hits: usize,
    pub max_glossary_entries: usize,
    pub max_chars: usize,
    pub min_memory_score: f32,
}

impl Default for MemoryContextConfig {
    fn default() -> Self {
        Self {
            max_memory_hits: 5,
            max_glossary_entries: 12,
            max_chars: 6_000,
            min_memory_score: 0.16,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryContext {
    pub text: String,
    pub memory_hits: usize,
    pub glossary_hits: usize,
    pub truncated: bool,
}

/// Builds compact prompt context for one source passage. Glossary decisions are
/// emitted first because names, titles and recurring terminology are hard
/// constraints; previous translation examples follow as softer style evidence.
pub fn build_memory_context(
    source_text: &str,
    memory: &TranslationMemory,
    glossary: &Glossary,
    config: &MemoryContextConfig,
) -> MemoryContext {
    if source_text.trim().is_empty() || config.max_chars == 0 {
        return MemoryContext {
            text: String::new(),
            memory_hits: 0,
            glossary_hits: 0,
            truncated: false,
        };
    }

    let glossary_hits = glossary
        .relevant_to_text(source_text)
        .into_iter()
        .take(config.max_glossary_entries)
        .collect::<Vec<_>>();

    let retrieval = RetrievalConfig {
        max_results: config.max_memory_hits,
        min_score: config.min_memory_score,
        ..RetrievalConfig::default()
    };
    let memory_hits = memory.search_with_config(source_text, &retrieval);

    let mut sections = Vec::new();
    if !glossary_hits.is_empty() {
        let lines = glossary_hits
            .iter()
            .map(|entry| {
                if entry.context.trim().is_empty() {
                    format!("- {} => {}", entry.source_term, entry.preferred_translation)
                } else {
                    format!(
                        "- {} => {} [{}]",
                        entry.source_term,
                        entry.preferred_translation,
                        entry.context.trim()
                    )
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        sections.push(format!("GLOSSARY — preserve these decisions:\n{lines}"));
    }

    if !memory_hits.is_empty() {
        let lines = memory_hits
            .iter()
            .map(|hit| {
                let context = hit.entry.context.trim();
                if context.is_empty() {
                    format!(
                        "- {:?} => {:?} (relevance {:.2})",
                        hit.entry.source, hit.entry.translation, hit.score
                    )
                } else {
                    format!(
                        "- {:?} => {:?} | context: {} (relevance {:.2})",
                        hit.entry.source, hit.entry.translation, context, hit.score
                    )
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        sections.push(format!(
            "TRANSLATION MEMORY — use as style/continuity evidence, not as mandatory wording:\n{lines}"
        ));
    }

    let full = sections.join("\n\n");
    let (text, truncated) = truncate_at_char_boundary(&full, config.max_chars);

    MemoryContext {
        text,
        memory_hits: memory_hits.len(),
        glossary_hits: glossary_hits.len(),
        truncated,
    }
}

fn truncate_at_char_boundary(text: &str, max_chars: usize) -> (String, bool) {
    if text.chars().count() <= max_chars {
        return (text.to_string(), false);
    }

    let mut truncated = text.chars().take(max_chars).collect::<String>();
    if let Some(last_newline) = truncated.rfind('\n') {
        if last_newline > max_chars / 2 {
            truncated.truncate(last_newline);
        }
    }
    truncated.push_str("\n[context truncated]");
    (truncated, true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::glossary::GlossaryEntry;
    use crate::MemoryEntry;

    fn glossary_entry(source: &str, translation: &str, context: &str) -> GlossaryEntry {
        GlossaryEntry {
            source_term: source.into(),
            preferred_translation: translation.into(),
            context: context.into(),
        }
    }

    #[test]
    fn prioritizes_hard_glossary_decisions_before_memory_examples() {
        let mut glossary = Glossary::default();
        glossary.add(glossary_entry("High Warlock", "جادوگر اعظم", "title"));

        let mut memory = TranslationMemory::new();
        memory.add(MemoryEntry::new(
            "the High Warlock smiled".into(),
            "جادوگر اعظم لبخند زد".into(),
            "narration".into(),
        ));

        let context = build_memory_context(
            "The High Warlock smiled.",
            &memory,
            &glossary,
            &MemoryContextConfig::default(),
        );

        assert_eq!(context.glossary_hits, 1);
        assert_eq!(context.memory_hits, 1);
        assert!(
            context.text.find("GLOSSARY").unwrap()
                < context.text.find("TRANSLATION MEMORY").unwrap()
        );
    }

    #[test]
    fn excludes_irrelevant_glossary_entries() {
        let mut glossary = Glossary::default();
        glossary.add(glossary_entry("Parabatai", "پاراباتای", "title"));
        glossary.add(glossary_entry("Portal", "پرتال", "magic"));

        let context = build_memory_context(
            "He opened a Portal.",
            &TranslationMemory::new(),
            &glossary,
            &MemoryContextConfig::default(),
        );

        assert!(context.text.contains("Portal => پرتال"));
        assert!(!context.text.contains("Parabatai"));
    }

    #[test]
    fn respects_prompt_budget_without_cutting_utf8() {
        let mut glossary = Glossary::default();
        glossary.add(glossary_entry(
            "Shadow World",
            "دنیای سایه‌ها",
            "recurring world-building terminology",
        ));

        let config = MemoryContextConfig {
            max_chars: 42,
            ..MemoryContextConfig::default()
        };
        let context = build_memory_context(
            "Shadow World",
            &TranslationMemory::new(),
            &glossary,
            &config,
        );

        assert!(context.truncated);
        assert!(context.text.is_char_boundary(context.text.len()));
        assert!(context.text.contains("[context truncated]"));
    }

    #[test]
    fn empty_source_produces_no_context() {
        let context = build_memory_context(
            "   ",
            &TranslationMemory::new(),
            &Glossary::default(),
            &MemoryContextConfig::default(),
        );
        assert!(context.text.is_empty());
        assert_eq!(context.memory_hits, 0);
        assert_eq!(context.glossary_hits, 0);
    }
}
