use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsistencyObservation {
    pub key: String,
    pub value: String,
    pub location: String,
}

impl ConsistencyObservation {
    pub fn new(
        key: impl Into<String>,
        value: impl Into<String>,
        location: impl Into<String>,
    ) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            location: location.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsistencyConflict {
    pub key: String,
    pub variants: Vec<String>,
    pub locations: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConsistencyResult {
    pub score: f32,
    pub warnings: Vec<String>,
    pub conflicts: Vec<ConsistencyConflict>,
}

/// Audits repeated literary decisions such as character names, honorifics,
/// titles, catchphrases and recurring terminology. A key is considered stable
/// only when all observations use the same normalized value.
pub fn audit_consistency(observations: &[ConsistencyObservation]) -> ConsistencyResult {
    if observations.is_empty() {
        return ConsistencyResult {
            score: 1.0,
            warnings: Vec::new(),
            conflicts: Vec::new(),
        };
    }

    let mut grouped: BTreeMap<String, Vec<&ConsistencyObservation>> = BTreeMap::new();
    for observation in observations {
        grouped
            .entry(normalize_key(&observation.key))
            .or_default()
            .push(observation);
    }

    let mut conflicts = Vec::new();
    let mut stable_groups = 0usize;

    for (key, group) in &grouped {
        let variants: BTreeSet<String> = group
            .iter()
            .map(|observation| normalize_value(&observation.value))
            .collect();

        if variants.len() <= 1 {
            stable_groups += 1;
            continue;
        }

        let locations = group
            .iter()
            .map(|observation| observation.location.clone())
            .collect();

        conflicts.push(ConsistencyConflict {
            key: key.clone(),
            variants: variants.into_iter().collect(),
            locations,
        });
    }

    let group_count = grouped.len();
    let score = if group_count == 0 {
        1.0
    } else {
        stable_groups as f32 / group_count as f32
    };

    let warnings = conflicts
        .iter()
        .map(|conflict| {
            format!(
                "Translation drift for '{}': {}",
                conflict.key,
                conflict.variants.join(" | ")
            )
        })
        .collect();

    ConsistencyResult {
        score,
        warnings,
        conflicts,
    }
}

/// Backward-compatible empty audit used by callers that have not yet wired
/// observations into the quality engine.
pub fn check_consistency() -> ConsistencyResult {
    audit_consistency(&[])
}

fn normalize_key(text: &str) -> String {
    text.trim().to_lowercase()
}

fn normalize_value(text: &str) -> String {
    text_normalization::normalize(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stable_repeated_decisions_score_perfectly() {
        let observations = vec![
            ConsistencyObservation::new("High Warlock", "جادوگر اعظم", "chapter-1"),
            ConsistencyObservation::new("high warlock", "جادوگر اعظم", "chapter-4"),
        ];

        let audit = audit_consistency(&observations);
        assert_eq!(audit.score, 1.0);
        assert!(audit.conflicts.is_empty());
    }

    #[test]
    fn detects_translation_drift_across_chapters() {
        let observations = vec![
            ConsistencyObservation::new("High Warlock", "جادوگر اعظم", "chapter-1"),
            ConsistencyObservation::new("High Warlock", "وارلاک اعظم", "chapter-6"),
            ConsistencyObservation::new("Parabatai", "پاراباتای", "chapter-2"),
            ConsistencyObservation::new("Parabatai", "پاراباتای", "chapter-7"),
        ];

        let audit = audit_consistency(&observations);
        assert_eq!(audit.score, 0.5);
        assert_eq!(audit.conflicts.len(), 1);
        assert_eq!(audit.conflicts[0].key, "high warlock");
        assert_eq!(audit.warnings.len(), 1);
    }

    #[test]
    fn ignores_persian_unicode_variants_when_meaning_is_same() {
        let observations = vec![
            ConsistencyObservation::new("book", "كتاب", "chapter-1"),
            ConsistencyObservation::new("book", "کتاب", "chapter-2"),
        ];

        let audit = audit_consistency(&observations);
        assert_eq!(audit.score, 1.0);
        assert!(audit.conflicts.is_empty());
    }
}
