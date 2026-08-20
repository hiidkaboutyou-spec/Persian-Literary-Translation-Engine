#[derive(Debug, Clone)]
pub struct TranslationRequest {
    pub source: String,
    pub target_language: String,
}

#[derive(Debug, Clone)]
pub struct TranslationContext {
    pub glossary_enabled: bool,
    pub character_memory_enabled: bool,
}

pub fn prepare_translation(request: TranslationRequest, context: TranslationContext) -> String {
    format!(
        "Prepared {} translation with glossary={} character_memory={}",
        request.target_language, context.glossary_enabled, context.character_memory_enabled
    )
}
