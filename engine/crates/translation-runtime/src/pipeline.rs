use crate::{TranslationProvider, TranslationRuntimeError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TranslationExecutionRequest {
    pub source_text: String,
    pub literary_context: LiteraryExecutionContext,
    pub context_snapshot: ContextSnapshot,
    pub decision_trace: DecisionTrace,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiteraryExecutionContext {
    pub character_context: String,
    pub scene_context: String,
    pub relationship_context: String,
    pub emotional_context: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextSnapshot {
    pub previous_chapter_state: String,
    pub character_state: String,
    pub relationship_state: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecisionTrace {
    pub decisions: Vec<String>,
    pub rationale: Vec<String>,
    pub confidence: f32,
    pub human_overrides: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TranslationOutput {
    pub translated_text: String,
}

pub struct TranslationPipeline<P> {
    provider: P,
}

impl<P> TranslationPipeline<P>
where
    P: TranslationProvider,
{
    pub fn new(provider: P) -> Self {
        Self { provider }
    }

    pub fn execute(
        &self,
        request: TranslationExecutionRequest,
    ) -> Result<TranslationOutput, TranslationRuntimeError> {
        if request.source_text.is_empty() {
            return Err(TranslationRuntimeError::InvalidContext(
                "source text cannot be empty".into(),
            ));
        }

        let translated_text = self.provider.translate(&request.source_text)?;

        Ok(TranslationOutput { translated_text })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TranslationRuntimeError;

    struct MockProvider;

    impl TranslationProvider for MockProvider {
        fn translate(&self, source: &str) -> Result<String, TranslationRuntimeError> {
            Ok(format!("translated:{source}"))
        }
    }

    #[test]
    fn execution_request_preserves_context_and_trace_contract() {
        let request = TranslationExecutionRequest {
            source_text: "text".into(),
            literary_context: LiteraryExecutionContext {
                character_context: "character".into(),
                scene_context: "scene".into(),
                relationship_context: "relationship".into(),
                emotional_context: "emotion".into(),
            },
            context_snapshot: ContextSnapshot {
                previous_chapter_state: "chapter".into(),
                character_state: "state".into(),
                relationship_state: "relation".into(),
            },
            decision_trace: DecisionTrace {
                decisions: vec!["choice".into()],
                rationale: vec!["reason".into()],
                confidence: 0.9,
                human_overrides: vec![],
            },
        };

        assert_eq!(request.decision_trace.decisions[0], "choice");
        assert_eq!(request.context_snapshot.character_state, "state");
    }

    #[test]
    fn invalid_source_context_is_rejected() {
        let result = TranslationPipeline::new(MockProvider).execute(TranslationExecutionRequest {
            source_text: String::new(),
            literary_context: LiteraryExecutionContext {
                character_context: String::new(),
                scene_context: String::new(),
                relationship_context: String::new(),
                emotional_context: String::new(),
            },
            context_snapshot: ContextSnapshot {
                previous_chapter_state: String::new(),
                character_state: String::new(),
                relationship_state: String::new(),
            },
            decision_trace: DecisionTrace {
                decisions: vec![],
                rationale: vec![],
                confidence: 0.0,
                human_overrides: vec![],
            },
        });

        assert!(matches!(result, Err(TranslationRuntimeError::InvalidContext(_))));
    }
}
