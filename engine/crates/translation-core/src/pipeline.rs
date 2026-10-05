use crate::provider::{PassKind, ProviderError, ProviderRequest, TranslationProvider};

const DEFAULT_MAX_PASSAGE_CHARS: usize = 24_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PipelineStage {
    DocumentAnalysis,
    ContextBuilding,
    Translation,
    Revision,
    QualityReview,
    Export,
}

#[derive(Debug, Clone)]
pub struct TranslationPipeline {
    pub stages: Vec<PipelineStage>,
    max_passage_chars: usize,
}

#[derive(Debug, Clone)]
pub struct PipelineInput {
    pub source_text: String,
    pub target_language: String,
    pub context: String,
}

#[derive(Debug, Clone)]
pub struct PipelineOutput {
    pub translated_text: String,
    pub revised_text: String,
    pub quality_review: String,
    pub provider: String,
}

impl TranslationPipeline {
    pub fn default_literary_pipeline() -> Self {
        Self {
            stages: vec![
                PipelineStage::DocumentAnalysis,
                PipelineStage::ContextBuilding,
                PipelineStage::Translation,
                PipelineStage::Revision,
                PipelineStage::QualityReview,
                PipelineStage::Export,
            ],
            max_passage_chars: DEFAULT_MAX_PASSAGE_CHARS,
        }
    }

    pub fn with_max_passage_chars(mut self, max_passage_chars: usize) -> Self {
        self.max_passage_chars = max_passage_chars.max(1);
        self
    }

    /// Execute all provider-facing literary passes while bounding the amount of
    /// passage text sent in any single provider request. Oversized passages prefer
    /// nearby paragraph and sentence boundaries, then whitespace, and finally a
    /// Unicode-safe character boundary. Chunks are reassembled before the next pass.
    /// This keeps long chapters from becoming one unbounded API request while
    /// preserving deterministic chapter order.
    pub fn execute<P: TranslationProvider + ?Sized>(
        &self,
        provider: &P,
        input: PipelineInput,
    ) -> Result<PipelineOutput, ProviderError> {
        let translated = self.execute_pass(
            provider,
            PassKind::Translate,
            &input.source_text,
            &input.target_language,
            &input.context,
        )?;

        let revised = self.execute_pass(
            provider,
            PassKind::Revise,
            &translated,
            &input.target_language,
            &input.context,
        )?;

        let quality_review = self.execute_pass(
            provider,
            PassKind::QualityReview,
            &revised,
            &input.target_language,
            &input.context,
        )?;

        Ok(PipelineOutput {
            translated_text: translated,
            revised_text: revised,
            quality_review,
            provider: provider.name().to_owned(),
        })
    }

    fn execute_pass<P: TranslationProvider + ?Sized>(
        &self,
        provider: &P,
        pass: PassKind,
        source_text: &str,
        target_language: &str,
        context: &str,
    ) -> Result<String, ProviderError> {
        let chunks = split_passage(source_text, self.max_passage_chars);
        let mut outputs = Vec::with_capacity(chunks.len());

        for chunk in chunks {
            let response = provider.execute(&ProviderRequest {
                pass: pass.clone(),
                source_text: chunk,
                target_language: target_language.to_owned(),
                context: context.to_owned(),
            })?;
            outputs.push(response.text);
        }

        Ok(outputs.join(""))
    }
}

fn split_passage(text: &str, max_chars: usize) -> Vec<String> {
    if text.chars().count() <= max_chars {
        return vec![text.to_owned()];
    }

    let mut chunks = Vec::new();
    let mut remaining = text;

    while remaining.chars().count() > max_chars {
        let hard_end = remaining
            .char_indices()
            .nth(max_chars)
            .map(|(index, _)| index)
            .unwrap_or(remaining.len());
        let candidate = &remaining[..hard_end];
        let boundary = preferred_semantic_boundary(candidate, max_chars).unwrap_or(hard_end);

        chunks.push(remaining[..boundary].to_owned());
        remaining = &remaining[boundary..];
    }

    if !remaining.is_empty() {
        chunks.push(remaining.to_owned());
    }
    chunks
}

fn preferred_semantic_boundary(candidate: &str, max_chars: usize) -> Option<usize> {
    // Avoid creating a very small chunk just because an early paragraph/sentence
    // boundary exists. Natural boundaries are preferred only in the final 40%
    // of the current capacity; otherwise keep filling toward the hard limit.
    let min_chars = max_chars.saturating_mul(3) / 5;
    let min_byte = candidate
        .char_indices()
        .nth(min_chars)
        .map(|(index, _)| index)
        .unwrap_or(0);

    let paragraph = [
        candidate.rfind("\r\n\r\n").map(|index| index + 4),
        candidate.rfind("\n\n").map(|index| index + 2),
    ]
    .into_iter()
    .flatten()
    .filter(|index| *index >= min_byte)
    .max();
    if paragraph.is_some() {
        return paragraph;
    }

    let ends_at_sentence_boundary = candidate
        .trim_end()
        .chars()
        .rev()
        .find(|character| !is_sentence_closer(*character))
        .is_some_and(is_sentence_terminator);
    if ends_at_sentence_boundary && candidate.trim_end().len() >= min_byte {
        return Some(candidate.len());
    }

    let sentence = candidate
        .char_indices()
        .rev()
        .find_map(|(index, character)| {
            if index < min_byte || !character.is_whitespace() {
                return None;
            }
            let before = candidate[..index].trim_end();
            before
                .chars()
                .rev()
                .find(|character| !is_sentence_closer(*character))
                .is_some_and(is_sentence_terminator)
                .then_some(index + character.len_utf8())
        });
    if sentence.is_some() {
        return sentence;
    }

    candidate
        .char_indices()
        .rev()
        .find(|(_, character)| character.is_whitespace())
        .map(|(index, character)| index + character.len_utf8())
        .filter(|index| *index > 0)
}

fn is_sentence_terminator(character: char) -> bool {
    matches!(character, '.' | '!' | '?' | '؟' | '。' | '！' | '？')
}

fn is_sentence_closer(character: char) -> bool {
    matches!(character, '"' | '\'' | '”' | '’' | '»' | ')' | ']' | '}')
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::{EchoProvider, ProviderResponse};
    use std::sync::Mutex;

    #[test]
    fn default_pipeline_runs_all_provider_passes() {
        let pipeline = TranslationPipeline::default_literary_pipeline();
        let output = pipeline
            .execute(
                &EchoProvider,
                PipelineInput {
                    source_text: "A chapter".into(),
                    target_language: "fa".into(),
                    context: "voice bible".into(),
                },
            )
            .unwrap();

        assert_eq!(output.translated_text, "A chapter");
        assert_eq!(output.revised_text, "A chapter");
        assert_eq!(output.quality_review, "A chapter");
        assert_eq!(output.provider, "echo");
    }

    #[derive(Debug, Default)]
    struct RecordingProvider {
        request_sizes: Mutex<Vec<usize>>,
    }

    impl TranslationProvider for RecordingProvider {
        fn name(&self) -> &str {
            "recording"
        }

        fn execute(&self, request: &ProviderRequest) -> Result<ProviderResponse, ProviderError> {
            self.request_sizes
                .lock()
                .unwrap()
                .push(request.source_text.chars().count());
            Ok(ProviderResponse {
                text: request.source_text.clone(),
                provider: self.name().to_owned(),
                model: None,
                usage: None,
            })
        }
    }

    #[test]
    fn oversized_passages_are_bounded_for_every_provider_call() {
        let provider = RecordingProvider::default();
        let pipeline = TranslationPipeline::default_literary_pipeline().with_max_passage_chars(10);
        let source = "1234567890abcdefghijXYZ";

        let output = pipeline
            .execute(
                &provider,
                PipelineInput {
                    source_text: source.into(),
                    target_language: "fa".into(),
                    context: String::new(),
                },
            )
            .unwrap();

        assert_eq!(output.quality_review, source);
        let request_sizes = provider.request_sizes.lock().unwrap();
        assert_eq!(request_sizes.len(), 9);
        assert!(request_sizes.iter().all(|size| *size <= 10));
    }

    #[test]
    fn splitting_is_unicode_safe_and_lossless() {
        let text = "سلام دنیا — یک متن آزمایشی";
        let chunks = split_passage(text, 5);
        assert!(chunks.iter().all(|chunk| chunk.chars().count() <= 5));
        assert_eq!(chunks.concat(), text);
    }

    #[test]
    fn splitting_prefers_whitespace_over_cutting_words() {
        let chunks = split_passage("alpha beta gamma", 10);
        assert_eq!(chunks, vec!["alpha ", "beta gamma"]);
    }

    #[test]
    fn splitting_prefers_nearby_paragraph_boundary_over_later_word_boundary() {
        let text = "123456789012345678\n\nsecond part words";
        let chunks = split_passage(text, 30);

        assert_eq!(chunks[0], "123456789012345678\n\n");
        assert_eq!(chunks.concat(), text);
        assert!(chunks.iter().all(|chunk| chunk.chars().count() <= 30));
    }

    #[test]
    fn splitting_prefers_nearby_sentence_boundary_when_no_paragraph_fits() {
        let text = "One sentence ends here. Next sentence keeps going";
        let chunks = split_passage(text, 35);

        assert_eq!(chunks[0], "One sentence ends here. ");
        assert_eq!(chunks.concat(), text);
        assert!(chunks.iter().all(|chunk| chunk.chars().count() <= 35));
    }

    #[test]
    fn early_natural_boundary_does_not_force_pathologically_small_chunk() {
        let text = "Short. alpha beta gamma delta epsilon";
        let chunks = split_passage(text, 24);

        assert_ne!(chunks[0], "Short. ");
        assert_eq!(chunks.concat(), text);
        assert!(chunks.iter().all(|chunk| chunk.chars().count() <= 24));
    }

    #[test]
    fn sentence_ending_exactly_at_hard_limit_stays_whole() {
        let sentence = format!("{}.", "a".repeat(29));
        let text = format!("{sentence} next sentence");
        let chunks = split_passage(&text, 30);

        assert_eq!(chunks[0], sentence);
        assert_eq!(chunks.concat(), text);
        assert!(chunks.iter().all(|chunk| chunk.chars().count() <= 30));
    }

    #[test]
    fn project_owned_chunk_boundary_corpus_has_no_avoidable_first_split() {
        let corpus: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../benchmarks/phase21/chunk-boundary-corpus-v1.json"
        ))
        .expect("chunk-boundary corpus should be valid JSON");
        assert_eq!(corpus["schema_version"], 1);
        assert_eq!(corpus["provenance"]["rights_safe"], true);

        let cases = corpus["cases"]
            .as_array()
            .expect("chunk-boundary corpus should contain cases");
        assert_eq!(cases.len(), 3);

        let mut avoidable_first_splits = Vec::new();
        for case in cases {
            let id = case["id"].as_str().expect("case id should be a string");
            let text = case["source"]
                .as_str()
                .expect("case source should be a string");
            let max_chars = case["max_chars"]
                .as_u64()
                .expect("max_chars should be an integer") as usize;
            let expected_first_chunk = case["expected_first_chunk"]
                .as_str()
                .expect("expected_first_chunk should be a string");

            let chunks = split_passage(text, max_chars);
            assert_eq!(chunks.concat(), text, "case '{id}' must remain lossless");
            assert!(
                chunks
                    .iter()
                    .all(|chunk| chunk.chars().count() <= max_chars),
                "case '{id}' must remain within the provider character cap"
            );
            if chunks.first().map(String::as_str) != Some(expected_first_chunk) {
                avoidable_first_splits.push(id);
            }
        }

        assert!(
            avoidable_first_splits.is_empty(),
            "avoidable first splits detected in: {avoidable_first_splits:?}"
        );
    }
}
