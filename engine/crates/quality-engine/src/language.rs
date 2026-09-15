use std::sync::OnceLock;

use lingua::Language::{English, Persian};
use lingua::{Language, LanguageDetector, LanguageDetectorBuilder};

const MIN_ASCII_LETTERS_FOR_SUSPICIOUS_SPAN: usize = 12;
const HIGH_ENGLISH_CONFIDENCE: f64 = 0.80;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticLanguage {
    Persian,
    English,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanguageSpan {
    pub language: DiagnosticLanguage,
    pub start_byte: usize,
    pub end_byte: usize,
    pub alphabetic_characters: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LanguageDiagnostics {
    pub dominant_language: Option<DiagnosticLanguage>,
    pub persian_confidence: f64,
    pub english_confidence: f64,
    pub suspicious_english_spans: Vec<LanguageSpan>,
}

impl LanguageDiagnostics {
    /// Advisory signal only. Callers must not treat this as proof that a literary
    /// translation is invalid: quoted English, names, titles, code, and deliberate
    /// multilingual passages can all be legitimate.
    pub fn probable_english_leakage(&self) -> bool {
        self.dominant_language == Some(DiagnosticLanguage::English)
            || self.english_confidence >= HIGH_ENGLISH_CONFIDENCE
            || !self.suspicious_english_spans.is_empty()
    }
}

/// Diagnoses whether a translation output appears predominantly Persian or contains
/// substantial English spans. This function is intentionally separate from the
/// deterministic quality gate: language detection is evidence, never an approval or
/// rejection decision.
pub fn diagnose_persian_output(text: &str) -> LanguageDiagnostics {
    if text.trim().is_empty() {
        return LanguageDiagnostics {
            dominant_language: None,
            persian_confidence: 0.0,
            english_confidence: 0.0,
            suspicious_english_spans: Vec::new(),
        };
    }

    let detector = detector();
    let dominant_language = detector.detect_language_of(text).and_then(map_language);
    let persian_confidence = detector.compute_language_confidence(text, Persian);
    let english_confidence = detector.compute_language_confidence(text, English);

    let suspicious_english_spans = detector
        .detect_multiple_languages_of(text)
        .into_iter()
        .filter(|result| result.language() == English)
        .filter_map(|result| {
            let span = text.get(result.start_index()..result.end_index())?;
            let alphabetic_characters = span
                .chars()
                .filter(|character| character.is_ascii_alphabetic())
                .count();
            (alphabetic_characters >= MIN_ASCII_LETTERS_FOR_SUSPICIOUS_SPAN).then_some(
                LanguageSpan {
                    language: DiagnosticLanguage::English,
                    start_byte: result.start_index(),
                    end_byte: result.end_index(),
                    alphabetic_characters,
                },
            )
        })
        .collect();

    LanguageDiagnostics {
        dominant_language,
        persian_confidence,
        english_confidence,
        suspicious_english_spans,
    }
}

fn detector() -> &'static LanguageDetector {
    static DETECTOR: OnceLock<LanguageDetector> = OnceLock::new();
    DETECTOR.get_or_init(|| LanguageDetectorBuilder::from_languages(&[English, Persian]).build())
}

fn map_language(language: Language) -> Option<DiagnosticLanguage> {
    match language {
        Persian => Some(DiagnosticLanguage::Persian),
        English => Some(DiagnosticLanguage::English),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn persian_prose_is_identified_as_persian() {
        let diagnostics = diagnose_persian_output(
            "جونگهان کنار پنجره ایستاد و چند لحظه به باران نگاه کرد. بعد آرام برگشت و لبخند زد.",
        );

        assert_eq!(
            diagnostics.dominant_language,
            Some(DiagnosticLanguage::Persian)
        );
        assert!(diagnostics.persian_confidence > diagnostics.english_confidence);
        assert!(!diagnostics.probable_english_leakage());
    }

    #[test]
    fn english_output_is_advisory_leakage() {
        let diagnostics = diagnose_persian_output(
            "He stood by the window for a moment, watching the rain before turning back.",
        );

        assert_eq!(
            diagnostics.dominant_language,
            Some(DiagnosticLanguage::English)
        );
        assert!(diagnostics.english_confidence > diagnostics.persian_confidence);
        assert!(diagnostics.probable_english_leakage());
    }

    #[test]
    fn empty_output_produces_no_language_claim() {
        let diagnostics = diagnose_persian_output("   \n\t");
        assert_eq!(diagnostics.dominant_language, None);
        assert_eq!(diagnostics.persian_confidence, 0.0);
        assert_eq!(diagnostics.english_confidence, 0.0);
        assert!(diagnostics.suspicious_english_spans.is_empty());
        assert!(!diagnostics.probable_english_leakage());
    }
}
