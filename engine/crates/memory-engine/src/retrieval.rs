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
            // Literary passages are often much longer than a stored translation-memory
            // example. Score both the whole passage and local sentence/token windows so
            // a highly relevant line of dialogue is not diluted by surrounding prose.
            let source_score = passage_similarity(query, &entry.source);
            let context_score = passage_similarity(query, &entry.context) * config.context_weight;
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
    let diversity_threshold = config.diversity_threshold.clamp(0.0, 1.0);

    for candidate in candidates {
        if config.deduplicate_translations {
            let translation_key = normalize(&candidate.entry.translation);
            if !translation_key.is_empty() && seen_translations.contains(&translation_key) {
                continue;
            }
        }

        // A threshold of 1.0 is documented as "disabled". Keep that promise even
        // for exact duplicate source examples, which have similarity 1.0.
        let too_similar = diversity_threshold < 1.0
            && selected.iter().any(|existing| {
                similarity(&candidate.entry.source, &existing.entry.source) >= diversity_threshold
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

/// Scores a stored memory against both the full passage and smaller local units.
/// This matters for fiction because a paragraph may contain narration, action and
/// dialogue while the useful memory often corresponds to only one sentence or beat.
fn passage_similarity(passage: &str, candidate: &str) -> f32 {
    let mut best = similarity(passage, candidate);
    if best >= 1.0 || candidate.trim().is_empty() {
        return best;
    }

    for segment in passage.split(['.', '!', '?', '…', ';', ':', '\n', '\r']) {
        if segment.trim().is_empty() {
            continue;
        }
        best = best.max(similarity(segment, candidate));
        if best >= 1.0 {
            return best;
        }
    }

    // Some source files have weak punctuation after extraction. A bounded token
    // window still lets a short remembered phrase surface from a long raw paragraph.
    let words: Vec<&str> = passage.split_whitespace().collect();
    let candidate_words = candidate.split_whitespace().count().max(1);
    let window_size = (candidate_words * 2).clamp(4, 24);

    if words.len() > window_size {
        for window in words.windows(window_size) {
            let local = window.join(" ");
            best = best.max(similarity(&local, candidate));
            if best >= 1.0 {
                return best;
            }
        }
    }

    best
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
    fn retrieves_a_local_line_from_a_long_literary_passage() {
        let entries = vec![
            entry("his hands were shaking", "دست‌هایش می‌لرزید", "fear", &[]),
            entry(
                "sunlight filled the kitchen",
                "نور آفتاب آشپزخانه را پر کرده بود",
                "setting",
                &[],
            ),
        ];

        let query = "He tried to smile as though nothing had happened. His hands were shaking. She noticed, but neither of them said a word about it.";
        let hits = rank_memory(&entries, query, &RetrievalConfig::default());

        assert!(!hits.is_empty());
        assert_eq!(hits[0].entry.source, "his hands were shaking");
        assert!(hits[0].score >= 1.0);
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
            entry(
                "he whispered softly",
                "آرام زمزمه کرد",
                "intimate dialogue",
                &[],
            ),
            entry(
                "she whispered softly",
                "آرام زمزمه کرد",
                "quiet dialogue",
                &[],
            ),
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
            entry(
                "he gave her a quiet smile",
                "لبخند آرامی به او زد",
                "tender",
                &[],
            ),
            entry(
                "he gave him a quiet smile",
                "لبخند آرامی نثارش کرد",
                "tender",
                &[],
            ),
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

        assert!(hits
            .iter()
            .any(|hit| hit.entry.source == "his voice softened"));
        assert_eq!(
            hits.iter()
                .filter(|hit| hit.entry.source.contains("quiet smile"))
                .count(),
            1
        );
    }

    #[test]
    fn diversity_threshold_one_really_disables_source_filtering() {
        let entries = vec![
            entry("I know", "می‌دونم", "casual dialogue", &[]),
            entry("I know", "می‌دانم", "formal dialogue", &[]),
        ];
        let config = RetrievalConfig {
            min_score: 0.05,
            diversity_threshold: 1.0,
            deduplicate_translations: false,
            ..Default::default()
        };

        let hits = rank_memory(&entries, "I know", &config);
        assert_eq!(hits.len(), 2);
    }
}
