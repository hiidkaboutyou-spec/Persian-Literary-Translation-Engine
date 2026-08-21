use std::cmp::Ordering;
use std::collections::HashSet;

use crate::models::MemoryEntry;

#[derive(Debug, Clone, Copy)]
pub struct RetrievalHit<'a> {
    pub entry: &'a MemoryEntry,
    pub score: f32,
}

#[derive(Debug, Clone)]
pub struct RetrievalConfig {
    pub max_results: usize,
    pub min_score: f32,
    pub context_weight: f32,
    pub tag_weight: f32,
    /// Prevents near-identical source examples from crowding out useful variety.
    /// Set to 1.0 to disable source-level diversity filtering.
    pub diversity_threshold: f32,
    /// Avoids returning multiple memories that collapse to the same Persian wording.
    pub deduplicate_translations: bool,
}

impl Default for RetrievalConfig {
    fn default() -> Self {
        Self {
            max_results: 6,
            min_score: 0.12,
            context_weight: 0.30,
            tag_weight: 0.15,
            diversity_threshold: 0.82,
            deduplicate_translations: true,
        }
    }
}

pub fn rank_memory<'a>(
    entries: &'a [MemoryEntry],
    query: &str,
    config: &RetrievalConfig,
) -> Vec<RetrievalHit<'a>> {
    if query.trim().is_empty() || config.max_results == 0 {
        return Vec::new();
    }

    let mut candidates: Vec<_> = entries
        .iter()
        .filter_map(|entry| {
            let source_score = similarity(query, &entry.source);
            let context_score = similarity(query, &entry.context) * config.context_weight;
            let tag_score = entry
                .tags
                .iter()
                .map(|tag| similarity(query, tag))
                .fold(0.0_f32, f32::max)
                * config.tag_weight;

            let exact_bonus = if normalize(query) == normalize(&entry.source) {
                0.30
            } else {
                0.0
            };

            let score = (source_score + context_score + tag_score + exact_bonus).min(1.0);
            (score >= config.min_score).then_some(RetrievalHit { entry, score })
        })
        .collect();

    candidates.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(Ordering::Equal)
            .then_with(|| a.entry.source.len().cmp(&b.entry.source.len()))
    });

    let mut selected: Vec<RetrievalHit<'a>> = Vec::with_capacity(config.max_results);
    let mut seen_translations = HashSet::new();

    for candidate in candidates {
        if config.deduplicate_translations {
            let translation_key = normalize(&candidate.entry.translation);
            if !translation_key.is_empty() && seen_translations.contains(&translation_key) {
                continue;
            }
        }

        let too_similar = selected.iter().any(|existing| {
            similarity(&candidate.entry.source, &existing.entry.source)
                >= config.diversity_threshold
        });
        if too_similar {
            continue;
        }

        if config.deduplicate_translations {
            seen_translations.insert(normalize(&candidate.entry.translation));
        }
        selected.push(candidate);

        if selected.len() >= config.max_results {
            break;
        }
    }

    selected
}

pub fn similarity(a: &str, b: &str) -> f32 {
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

    let phrase_bonus = if a_norm.contains(&b_norm) || b_norm.contains(&a_norm) {
        0.20
    } else {
        0.0
    };

    (jaccard + phrase_bonus).min(1.0)
}

pub fn normalize(text: &str) -> String {
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

    fn entry(source: &str, translation: &str, context: &str, tags: &[&str]) -> MemoryEntry {
        MemoryEntry {
            source: source.into(),
            translation: translation.into(),
            context: context.into(),
            tags: tags.iter().map(|tag| (*tag).into()).collect(),
        }
    }

    #[test]
    fn normalizes_arabic_and_persian_letter_variants() {
        assert_eq!(normalize("مي‌رود"), normalize("می رود"));
        assert_eq!(normalize("كتاب"), normalize("کتاب"));
    }

    #[test]
    fn ranks_semantically_related_memory_first() {
        let entries = vec![
            entry(
                "a quiet smile",
                "لبخندی آرام",
                "gentle dialogue",
                &["tender"],
            ),
            entry(
                "he slammed the door",
                "در را محکم کوبید",
                "argument",
                &["anger"],
            ),
        ];

        let hits = rank_memory(
            &entries,
            "quiet smile in a gentle moment",
            &RetrievalConfig::default(),
        );
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].entry.translation, "لبخندی آرام");
    }

    #[test]
    fn exact_source_match_receives_maximum_relevance() {
        let entries = vec![entry(
            "I missed you",
            "دلم برات تنگ شده بود",
            "dialogue",
            &[],
        )];
        let hits = rank_memory(&entries, "I missed you", &RetrievalConfig::default());
        assert_eq!(hits[0].score, 1.0);
    }

    #[test]
    fn respects_score_threshold_and_result_limit() {
        let entries = vec![
            entry("soft voice", "صدای آرام", "dialogue", &[]),
            entry("soft touch", "لمس آرام", "intimacy", &[]),
            entry("storm outside", "طوفان بیرون", "weather", &[]),
        ];
        let config = RetrievalConfig {
            max_results: 1,
            min_score: 0.10,
            ..Default::default()
        };
        let hits = rank_memory(&entries, "soft voice", &config);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].entry.source, "soft voice");
    }

    #[test]
    fn removes_duplicate_persian_wording_from_prompt_evidence() {
        let entries = vec![
            entry("he whispered softly", "آرام زمزمه کرد", "intimate dialogue", &[]),
            entry("she whispered softly", "آرام زمزمه کرد", "quiet dialogue", &[]),
            entry("a soft laugh", "خنده‌ای آرام", "tender moment", &[]),
        ];

        let hits = rank_memory(
            &entries,
            "soft quiet dialogue",
            &RetrievalConfig {
                min_score: 0.05,
                ..Default::default()
            },
        );

        assert_eq!(
            hits.iter()
                .filter(|hit| hit.entry.translation == "آرام زمزمه کرد")
                .count(),
            1
        );
    }

    #[test]
    fn preserves_varied_examples_instead_of_near_duplicate_sources() {
        let entries = vec![
            entry("he gave her a quiet smile", "لبخند آرامی به او زد", "tender", &[]),
            entry("he gave him a quiet smile", "لبخند آرامی نثارش کرد", "tender", &[]),
            entry("his voice softened", "صدایش نرم‌تر شد", "tender", &[]),
        ];

        let hits = rank_memory(
            &entries,
            "quiet tender moment smile voice",
            &RetrievalConfig {
                min_score: 0.05,
                diversity_threshold: 0.70,
                deduplicate_translations: false,
                ..Default::default()
            },
        );

        assert!(hits.iter().any(|hit| hit.entry.source == "his voice softened"));
        assert_eq!(
            hits.iter()
                .filter(|hit| hit.entry.source.contains("quiet smile"))
                .count(),
            1
        );
    }
}
