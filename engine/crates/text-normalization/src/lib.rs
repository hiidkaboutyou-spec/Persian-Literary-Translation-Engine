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


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PersianTypographyIssueKind {
    ArabicLetterVariant,
    ArabicDigitVariant,
    Kashida,
    DuplicateZwnj,
    InvalidZwnj,
    PrefixSpacing,
    SpaceBeforePunctuation,
    MultipleSpaces,
}

impl PersianTypographyIssueKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ArabicLetterVariant => "Arabic letter variant",
            Self::ArabicDigitVariant => "Arabic-Indic digit variant",
            Self::Kashida => "decorative kashida/tatweel",
            Self::DuplicateZwnj => "duplicate zero-width non-joiner",
            Self::InvalidZwnj => "misplaced zero-width non-joiner",
            Self::PrefixSpacing => "می/نمی prefix separated by a normal space",
            Self::SpaceBeforePunctuation => "space before punctuation",
            Self::MultipleSpaces => "repeated spaces",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersianTypographyIssue {
    pub kind: PersianTypographyIssueKind,
    /// Character offset, not a byte offset. This keeps evidence safe for UTF-8 UI display.
    pub char_index: usize,
    pub suggestion: Option<String>,
}

/// Returns true when the text contains at least one Persian/Arabic-script letter.
///
/// This deliberately checks letters rather than the whole Arabic Unicode block,
/// so punctuation or Arabic-Indic digits alone do not cause a target to be
/// treated as Persian prose.
pub fn contains_persian_letters(text: &str) -> bool {
    text.chars().any(is_persian_script_letter)
}

/// Applies only deterministic, low-risk Unicode cleanup suitable for Persian
/// publication text.
///
/// This function intentionally does **not** rewrite punctuation style, collapse
/// expressive marks, add/remove ordinary spaces around affixes, or otherwise
/// "improve" literary prose. Those cases are diagnostics only.
pub fn polish_persian_unicode(text: &str) -> String {
    let mapped = text
        .chars()
        .filter_map(|ch| match ch {
            'ي' | 'ى' => Some('ی'),
            'ك' => Some('ک'),
            '٠' => Some('۰'),
            '١' => Some('۱'),
            '٢' => Some('۲'),
            '٣' => Some('۳'),
            '٤' => Some('۴'),
            '٥' => Some('۵'),
            '٦' => Some('۶'),
            '٧' => Some('۷'),
            '٨' => Some('۸'),
            '٩' => Some('۹'),
            '\u{0640}' => None, // tatweel/kashida is decorative, not lexical content
            other => Some(other),
        })
        .collect::<Vec<_>>();

    let mut out = String::with_capacity(text.len());
    for (index, ch) in mapped.iter().copied().enumerate() {
        if ch != '\u{200c}' {
            out.push(ch);
            continue;
        }

        let previous = index.checked_sub(1).and_then(|i| mapped.get(i)).copied();
        let next = mapped.get(index + 1).copied();
        let invalid = previous.is_none()
            || next.is_none()
            || previous.is_some_and(char::is_whitespace)
            || next.is_some_and(char::is_whitespace)
            || previous.is_some_and(is_spacing_punctuation)
            || next.is_some_and(is_spacing_punctuation)
            || previous == Some('\u{200c}');
        if !invalid {
            out.push(ch);
        }
    }
    out
}

/// Produces narrow, deterministic typography evidence without rewriting text.
///
/// The diagnostics are intentionally conservative. A finding means "review this
/// typography/orthography surface", not "the sentence is bad Persian".
pub fn inspect_persian_typography(text: &str) -> Vec<PersianTypographyIssue> {
    let chars = text.chars().collect::<Vec<_>>();
    let mut issues = Vec::new();

    for (index, ch) in chars.iter().copied().enumerate() {
        let suggestion = match ch {
            'ي' | 'ى' => Some("ی".to_string()),
            'ك' => Some("ک".to_string()),
            '٠' => Some("۰".to_string()),
            '١' => Some("۱".to_string()),
            '٢' => Some("۲".to_string()),
            '٣' => Some("۳".to_string()),
            '٤' => Some("۴".to_string()),
            '٥' => Some("۵".to_string()),
            '٦' => Some("۶".to_string()),
            '٧' => Some("۷".to_string()),
            '٨' => Some("۸".to_string()),
            '٩' => Some("۹".to_string()),
            '\u{0640}' => Some(String::new()),
            _ => None,
        };
        let kind = match ch {
            'ي' | 'ى' | 'ك' => Some(PersianTypographyIssueKind::ArabicLetterVariant),
            '٠'..='٩' => Some(PersianTypographyIssueKind::ArabicDigitVariant),
            '\u{0640}' => Some(PersianTypographyIssueKind::Kashida),
            _ => None,
        };
        if let Some(kind) = kind {
            issues.push(PersianTypographyIssue {
                kind,
                char_index: index,
                suggestion,
            });
        }

        if ch == '\u{200c}' {
            let previous = index.checked_sub(1).and_then(|i| chars.get(i)).copied();
            let next = chars.get(index + 1).copied();
            if previous == Some('\u{200c}') {
                issues.push(PersianTypographyIssue {
                    kind: PersianTypographyIssueKind::DuplicateZwnj,
                    char_index: index,
                    suggestion: Some(String::new()),
                });
            } else if previous.is_none()
                || next.is_none()
                || previous.is_some_and(char::is_whitespace)
                || next.is_some_and(char::is_whitespace)
                || previous.is_some_and(is_spacing_punctuation)
                || next.is_some_and(is_spacing_punctuation)
            {
                issues.push(PersianTypographyIssue {
                    kind: PersianTypographyIssueKind::InvalidZwnj,
                    char_index: index,
                    suggestion: Some(String::new()),
                });
            }
        }

        if ch == ' ' {
            if chars.get(index + 1).is_some_and(|next| *next == ' ') {
                issues.push(PersianTypographyIssue {
                    kind: PersianTypographyIssueKind::MultipleSpaces,
                    char_index: index + 1,
                    suggestion: Some(String::new()),
                });
            }
            if chars.get(index + 1).is_some_and(|next| is_spacing_punctuation(*next)) {
                issues.push(PersianTypographyIssue {
                    kind: PersianTypographyIssueKind::SpaceBeforePunctuation,
                    char_index: index,
                    suggestion: Some(String::new()),
                });
            }
        }
    }

    detect_prefix_spacing(&chars, &mut issues);
    issues
}

fn detect_prefix_spacing(chars: &[char], issues: &mut Vec<PersianTypographyIssue>) {
    for start in 0..chars.len() {
        let boundary_before = start == 0 || !is_persian_script_letter(chars[start - 1]);

        let mi = chars.get(start) == Some(&'م')
            && chars
                .get(start + 1)
                .is_some_and(|ch| matches!(*ch, 'ی' | 'ي' | 'ى'))
            && chars.get(start + 2) == Some(&' ')
            && chars
                .get(start + 3)
                .is_some_and(|ch| is_persian_script_letter(*ch));
        if boundary_before && mi {
            issues.push(PersianTypographyIssue {
                kind: PersianTypographyIssueKind::PrefixSpacing,
                char_index: start + 2,
                suggestion: Some("\u{200c}".to_string()),
            });
        }

        let nemi = chars.get(start) == Some(&'ن')
            && chars.get(start + 1) == Some(&'م')
            && chars
                .get(start + 2)
                .is_some_and(|ch| matches!(*ch, 'ی' | 'ي' | 'ى'))
            && chars.get(start + 3) == Some(&' ')
            && chars
                .get(start + 4)
                .is_some_and(|ch| is_persian_script_letter(*ch));
        if boundary_before && nemi {
            issues.push(PersianTypographyIssue {
                kind: PersianTypographyIssueKind::PrefixSpacing,
                char_index: start + 3,
                suggestion: Some("\u{200c}".to_string()),
            });
        }
    }
}

fn is_persian_script_letter(ch: char) -> bool {
    ch.is_alphabetic()
        && (('\u{0600}'..='\u{06ff}').contains(&ch)
            || ('\u{0750}'..='\u{077f}').contains(&ch)
            || ('\u{08a0}'..='\u{08ff}').contains(&ch))
}

fn is_spacing_punctuation(ch: char) -> bool {
    matches!(ch, '،' | '؛' | '؟' | '!' | '?' | ',' | ';' | ':' | '.' | '…')
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
    fn safe_persian_polish_normalizes_only_low_risk_unicode_surfaces() {
        assert_eq!(
            polish_persian_unicode("عليـ كاتب ٣٤٥ و می‌رود!!!"),
            "علی کاتب ۳۴۵ و می‌رود!!!"
        );
        assert_eq!(polish_persian_unicode("نسخه 123"), "نسخه 123");
    }

    #[test]
    fn safe_persian_polish_preserves_valid_zwnj_and_drops_invalid_ones() {
        assert_eq!(polish_persian_unicode("می‌روم"), "می‌روم");
        assert_eq!(polish_persian_unicode("\u{200c}سلام"), "سلام");
        assert_eq!(polish_persian_unicode("سلام\u{200c} دنیا"), "سلام دنیا");
        assert_eq!(polish_persian_unicode("می‌\u{200c}روم"), "می‌روم");
    }

    #[test]
    fn typography_diagnostics_are_advisory_and_precise() {
        let issues = inspect_persian_typography("من نمي روم  ؟");
        assert!(issues
            .iter()
            .any(|issue| issue.kind == PersianTypographyIssueKind::ArabicLetterVariant));
        assert!(issues
            .iter()
            .any(|issue| issue.kind == PersianTypographyIssueKind::PrefixSpacing));
        assert!(issues
            .iter()
            .any(|issue| issue.kind == PersianTypographyIssueKind::MultipleSpaces));
        assert!(issues
            .iter()
            .any(|issue| issue.kind == PersianTypographyIssueKind::SpaceBeforePunctuation));
    }

    #[test]
    fn persian_letter_detection_ignores_punctuation_only() {
        assert!(contains_persian_letters("سلام!"));
        assert!(!contains_persian_letters("۱۲۳؟"));
        assert!(!contains_persian_letters("English only"));
    }

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
