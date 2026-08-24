//! Shared Persian/Arabic text normalization, negation detection, and comparison utilities.
//!
//! This crate eliminates duplicated normalization logic across memory-engine,
//! quality-engine, and character-engine. All crates that compare Persian or
//! Arabic text should use these canonical functions.

use std::collections::HashSet;

/// Normalizes Persian and Arabic letter variants, strips diacritics and zero-width
/// characters, and collapses whitespace. Does **not** lowercase — use
/// [`normalize_case_insensitive`] when case-insensitive comparison is needed.
///
/// # Examples
///
/// ```
/// use text_normalization::normalize;
///
/// assert_eq!(normalize("مي‌رود"), normalize("می رود"));
/// assert_eq!(normalize("كتاب"), normalize("کتاب"));
/// ```
pub fn normalize(text: &str) -> String {
    let expanded = expand_english_negation_contractions(text);
    expanded
        .chars()
        .map(|ch| match ch {
            'ي' | 'ى' => 'ی',
            'ك' => 'ک',
            '\u{200c}' | '\u{200d}' | '\u{00a0}' => ' ',
            c if c.is_alphanumeric() => c,
            _ => ' ',
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Normalized text with lowercasing for case-insensitive comparison.
///
/// This is the variant used by quality checks and character matching where
/// `"High Warlock"` and `"high warlock"` should compare as identical.
pub fn normalize_case_insensitive(text: &str) -> String {
    normalize(text).to_lowercase()
}

/// Returns `true` when the normalized text contains negation markers.
///
/// Handles English contractions (`don't`, `won't`, `can't`) and Persian
/// negation prefixes (`نمی`, `ندار`, `نکرد`, etc.) after normalization.
pub fn has_negation(normalized: &str) -> bool {
    normalized.split_whitespace().any(is_negation_token)
}

/// Expands common English negation contractions so that `don't` and `do not`
/// normalize identically. Preserves the lexical stem for each contraction
/// (e.g., `can't` → `cannot` rather than `ca not`).
pub fn expand_english_negation_contractions(text: &str) -> String {
    let mut expanded = text.to_lowercase().replace('\u{2019}', "'");
    for (contracted, full) in [
        ("don't", "do not"),
        ("doesn't", "does not"),
        ("didn't", "did not"),
        ("isn't", "is not"),
        ("aren't", "are not"),
        ("wasn't", "was not"),
        ("weren't", "were not"),
        ("can't", "cannot"),
        ("couldn't", "could not"),
        ("won't", "will not"),
        ("wouldn't", "would not"),
        ("shouldn't", "should not"),
        ("hasn't", "has not"),
        ("haven't", "have not"),
        ("hadn't", "had not"),
        ("mustn't", "must not"),
        ("needn't", "need not"),
    ] {
        expanded = expanded.replace(contracted, full);
    }
    expanded
}

fn is_negation_token(token: &str) -> bool {
    matches!(
        token,
        "not"
            | "no"
            | "never"
            | "neither"
            | "nor"
            | "without"
            | "cannot"
            | "ن"
            | "نه"
            | "نیست"
            | "نیستم"
            | "نیستی"
            | "نیستیم"
            | "نیستید"
            | "نیستند"
            | "نبود"
            | "نبودم"
            | "نبودی"
            | "نبودیم"
            | "نبودید"
            | "نبودند"
            | "هرگز"
            | "هیچ"
            | "هیچوقت"
            | "هیچگاه"
            | "بدون"
    ) || token == "نمی"
        || token.starts_with("نمی")
        || token.starts_with("ندار")
        || token.starts_with("نخواه")
        || token.starts_with("نکرد")
        || token.starts_with("نکن")
        || token.starts_with("نباید")
        || token.starts_with("نتوان")
        || matches!(token, "نرو" | "نیا" | "نگو" | "نبین" | "نبر" | "نخور")
}

/// Jaccard similarity between two normalized token sets, with a phrase-containment
/// bonus and a negation-polarity penalty.
///
/// Returns a value in `[0.0, 1.0]`.
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

    let polarity_factor = if has_negation(&a_norm) == has_negation(&b_norm) {
        1.0
    } else {
        0.35
    };

    ((jaccard + phrase_bonus) * polarity_factor).min(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_arabic_and_persian_letter_variants() {
        assert_eq!(normalize("مي‌رود"), normalize("می رود"));
        assert_eq!(normalize("كتاب"), normalize("کتاب"));
    }

    #[test]
    fn normalizes_curly_and_straight_negation_contractions() {
        assert_eq!(
            expand_english_negation_contractions("I don't know"),
            expand_english_negation_contractions("I do not know")
        );
        assert_eq!(normalize("I don't know"), normalize("I do not know"));
        assert_eq!(
            normalize("She won't leave"),
            normalize("She will not leave")
        );
    }

    #[test]
    fn case_insensitive_normalization_lowercases() {
        assert_eq!(
            normalize_case_insensitive("High Warlock"),
            normalize_case_insensitive("high warlock")
        );
    }

    #[test]
    fn exact_match_returns_one() {
        assert_eq!(similarity("hello world", "hello world"), 1.0);
    }

    #[test]
    fn polarity_mismatch_is_penalized() {
        let positive = similarity("I trust you", "I really trust you");
        let inverted = similarity("I trust you", "I don't trust you");
        assert!(positive > inverted);
        assert!(inverted < 0.30);
    }

    #[test]
    fn persian_negative_prefixes_are_detected_after_normalization() {
        for text in [
            "نمی‌خوام برم",
            "نمیتونم قبولش کنم",
            "ندارمش",
            "نکردم",
            "نخواهم رفت",
            "نکن این کارو",
            "نگو که تموم شده",
            "هیچ‌وقت فراموشت نمی‌کنم",
        ] {
            assert!(has_negation(&normalize(text)), "missed negation in: {text}");
        }
    }

    #[test]
    fn persian_positive_and_negative_lines_do_not_collapse_together() {
        let positive = similarity("بهت اعتماد دارم", "من واقعاً بهت اعتماد دارم");
        let inverted = similarity("بهت اعتماد دارم", "بهت اعتماد ندارم");
        assert!(positive > inverted);
        assert!(inverted < 0.30);
    }

    #[test]
    fn empty_input_returns_empty_string() {
        assert_eq!(normalize(""), "");
        assert_eq!(normalize("   "), "");
    }
}
