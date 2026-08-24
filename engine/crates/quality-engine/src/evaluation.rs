#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminologyRule {
    pub source_term: String,
    pub preferred_translation: String,
}

impl TerminologyRule {
    pub fn new(source_term: impl Into<String>, preferred_translation: impl Into<String>) -> Self {
        Self {
            source_term: source_term.into(),
            preferred_translation: preferred_translation.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QualityEvaluation {
    pub score: f32,
    pub blocking_errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl QualityEvaluation {
    pub fn passes(&self) -> bool {
        self.blocking_errors.is_empty()
    }
}

pub fn evaluate_translation(
    source: &str,
    output: &str,
    terminology: &[TerminologyRule],
) -> QualityEvaluation {
    let mut blocking_errors = Vec::new();
    let mut warnings = Vec::new();

    if source.trim().is_empty() {
        blocking_errors.push("source passage is empty".to_string());
    }
    if output.trim().is_empty() {
        blocking_errors.push("translation output is empty".to_string());
    }

    let normalized_output = normalize(output);
    for marker in [
        "project context",
        "passage",
        "glossary preserve these decisions",
        "translation memory use as style continuity evidence not as mandatory wording",
    ] {
        if normalized_output.contains(marker) {
            blocking_errors.push(format!("provider prompt leakage detected: {marker}"));
        }
    }

    if !source.trim().is_empty() && !output.trim().is_empty() {
        let source_chars = source.chars().filter(|ch| !ch.is_whitespace()).count();
        let output_chars = output.chars().filter(|ch| !ch.is_whitespace()).count();
        if source_chars > 0 {
            let ratio = output_chars as f32 / source_chars as f32;
            if ratio < 0.20 {
                warnings.push(format!(
                    "translation may be severely truncated: output/source character ratio {ratio:.2}"
                ));
            } else if ratio > 5.0 {
                warnings.push(format!(
                    "translation expanded unusually: output/source character ratio {ratio:.2}"
                ));
            }
        }

        let source_paragraphs = paragraph_count(source);
        let output_paragraphs = paragraph_count(output);
        if source_paragraphs >= 4 && output_paragraphs * 2 < source_paragraphs {
            warnings.push(format!(
                "paragraph structure may have collapsed: source={source_paragraphs}, output={output_paragraphs}"
            ));
        }
    }

    let normalized_source = normalize(source);
    for rule in terminology {
        let source_term = normalize(&rule.source_term);
        let preferred = normalize(&rule.preferred_translation);
        if source_term.is_empty() || preferred.is_empty() {
            continue;
        }
        if contains_term(&normalized_source, &source_term)
            && !contains_term(&normalized_output, &preferred)
        {
            warnings.push(format!(
                "preferred terminology missing for '{}': expected '{}'",
                rule.source_term, rule.preferred_translation
            ));
        }
    }

    let penalty = blocking_errors.len() as f32 * 0.50 + warnings.len() as f32 * 0.10;
    let score = (1.0 - penalty).clamp(0.0, 1.0);

    QualityEvaluation {
        score,
        blocking_errors,
        warnings,
    }
}

fn paragraph_count(text: &str) -> usize {
    text.split("\n\n")
        .filter(|paragraph| !paragraph.trim().is_empty())
        .count()
}

fn contains_term(normalized_text: &str, normalized_term: &str) -> bool {
    format!(" {normalized_text} ").contains(&format!(" {normalized_term} "))
}

use text_normalization::normalize_case_insensitive as normalize;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_translation_passes() {
        let result = evaluate_translation(
            "The High Warlock smiled.",
            "جادوگر اعظم لبخند زد.",
            &[TerminologyRule::new("High Warlock", "جادوگر اعظم")],
        );

        assert!(result.passes());
        assert!(result.warnings.is_empty());
        assert_eq!(result.score, 1.0);
    }

    #[test]
    fn empty_output_is_blocking() {
        let result = evaluate_translation("Some source text", "   ", &[]);
        assert!(!result.passes());
        assert!(result
            .blocking_errors
            .iter()
            .any(|error| error.contains("output is empty")));
    }

    #[test]
    fn prompt_leakage_is_blocking() {
        let result = evaluate_translation(
            "Alec answered.",
            "PROJECT CONTEXT\ncharacter voice\n\nPASSAGE\nالک جواب داد.",
            &[],
        );
        assert!(!result.passes());
        assert!(result
            .blocking_errors
            .iter()
            .any(|error| error.contains("prompt leakage")));
    }

    #[test]
    fn missing_preferred_terminology_is_reported() {
        let result = evaluate_translation(
            "The High Warlock smiled.",
            "وارلاک لبخند زد.",
            &[TerminologyRule::new("High Warlock", "جادوگر اعظم")],
        );
        assert!(result.passes());
        assert!(result
            .warnings
            .iter()
            .any(|warning| warning.contains("preferred terminology missing")));
    }

    #[test]
    fn severe_truncation_is_reported() {
        let source = "This is a deliberately long source passage with many words and details that should not disappear during translation. It continues with additional narrative material, dialogue, reactions, atmosphere, and emotional context.";
        let result = evaluate_translation(source, "کوتاه", &[]);
        assert!(result
            .warnings
            .iter()
            .any(|warning| warning.contains("severely truncated")));
    }

    #[test]
    fn persian_unicode_variants_do_not_trigger_false_terminology_warning() {
        let result = evaluate_translation("book", "كتاب", &[TerminologyRule::new("book", "کتاب")]);
        assert!(result.warnings.is_empty());
    }
}
