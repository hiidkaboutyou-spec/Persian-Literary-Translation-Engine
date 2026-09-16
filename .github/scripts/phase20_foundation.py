from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    p = Path(path)
    text = p.read_text()
    if old not in text:
        raise SystemExit(f"expected text not found in {path}: {old[:120]!r}")
    p.write_text(text.replace(old, new, 1))


Path("engine/crates/translation-core/src/style.rs").write_text(
    '''use std::fmt;
use std::str::FromStr;

pub const DEFAULT_STYLE_PROFILE_ID: &str = "literary";
pub const ADULT_INTIMACY_STYLE_PROFILE_ID: &str = "adult-intimacy";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TranslationStyleProfile {
    Literary,
    AdultIntimacy,
}

impl Default for TranslationStyleProfile {
    fn default() -> Self {
        Self::Literary
    }
}

impl TranslationStyleProfile {
    pub fn id(self) -> &'static str {
        match self {
            Self::Literary => DEFAULT_STYLE_PROFILE_ID,
            Self::AdultIntimacy => ADULT_INTIMACY_STYLE_PROFILE_ID,
        }
    }

    pub fn requires_adult_confirmation(self) -> bool {
        matches!(self, Self::AdultIntimacy)
    }

    /// Empty for the default profile so existing literary runs keep their
    /// established prompt/context behavior and resume fingerprint.
    pub fn context_block(self) -> &'static str {
        match self {
            Self::Literary => "",
            Self::AdultIntimacy => concat!(
                "ADULT INTIMACY FIDELITY PROFILE (confirmed-adult source material only)\\n",
                "Treat this as a fidelity contract, never as an instruction to make prose more sexual. ",
                "Do not sexualize nonsexual source material. Preserve the source passage's exact degree of ",
                "sexual/sensual explicitness and pragmatic markedness: do not censor, euphemize, summarize, ",
                "sanitize, romanticize, intensify, vulgarize, or add detail. Preserve whether each expression is ",
                "veiled, sensual/implicit, clear but non-detailed, explicit/direct, or strongly marked/colloquial, ",
                "using natural Persian of comparable force rather than dictionary-literal wording. Preserve the ",
                "source's euphemistic, neutral/direct, or dysphemistic function when relevant.\\n",
                "Keep POV, narrative distance, character-specific idiolect, dialogue register, pauses, unfinished ",
                "speech, humor, awkwardness, tenderness, discomfort, shame, desire, vulnerability, and emotional ",
                "subtext at source strength. Preserve all source evidence of consent, permission, hesitation, refusal, ",
                "withdrawal, reassurance, coercion, resistance, voluntary submission, commands, and power imbalance ",
                "without reinterpreting one as another. Preserve who performs each action, its recipient, physical ",
                "sequence, spatial continuity, cause-and-reaction chain, and any aftermath.\\n",
                "Preserve only sensory channels and embodied interiority actually present in the source; invent no ",
                "touch, smell, taste, sound, bodily response, metaphor, dialogue, thought, or emotion. Match source ",
                "pacing and sentence rhythm, including deliberate repetition and acceleration/deceleration. Persian ",
                "must read as idiomatic contemporary literary prose appropriate to the source: avoid accidental ",
                "clinical, childish, archaic, coy, flowery, or mechanically obscene diction unless the source itself ",
                "uses that register. Avoid canned erotic clichés and forced synonym variation.\\n",
                "Every later revision and quality pass is bound by the same parity rules and must not sanitize or ",
                "amplify an initially faithful translation. Preserve structural marker tokens such as <m1>...</m1> ",
                "and <r1/> exactly in identifier, count, pairing, and relative order."
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StyleProfileError(pub String);

impl fmt::Display for StyleProfileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

impl std::error::Error for StyleProfileError {}

impl FromStr for TranslationStyleProfile {
    type Err = StyleProfileError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "" | "literary" | "literary-fiction" => Ok(Self::Literary),
            "adult-intimacy" | "explicit-adult" | "smut" => Ok(Self::AdultIntimacy),
            other => Err(StyleProfileError(format!(
                "unsupported translation style profile '{other}'; expected literary or adult-intimacy"
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn literary_default_is_prompt_neutral() {
        let profile = TranslationStyleProfile::default();
        assert_eq!(profile.id(), "literary");
        assert!(profile.context_block().is_empty());
        assert!(!profile.requires_adult_confirmation());
    }

    #[test]
    fn adult_profile_is_fidelity_not_amplification() {
        let profile: TranslationStyleProfile = "adult-intimacy".parse().unwrap();
        let block = profile.context_block().to_ascii_lowercase();
        assert!(profile.requires_adult_confirmation());
        assert!(block.contains("do not censor"));
        assert!(block.contains("do not sexualize nonsexual"));
        assert!(block.contains("must not sanitize or"));
        assert!(block.contains("consent"));
        assert!(block.contains("dysphemistic"));
        assert!(block.contains("<m1>...</m1>"));
    }

    #[test]
    fn unknown_profile_fails_closed() {
        assert!("make-it-hotter".parse::<TranslationStyleProfile>().is_err());
    }
}
'''
)

replace_once(
    "engine/crates/translation-core/src/lib.rs",
    "pub mod provider;\n",
    "pub mod provider;\npub mod style;\n",
)
replace_once(
    "engine/crates/translation-core/src/lib.rs",
    "pub use provider::{",
    "pub use style::{\n    StyleProfileError, TranslationStyleProfile, ADULT_INTIMACY_STYLE_PROFILE_ID,\n    DEFAULT_STYLE_PROFILE_ID,\n};\npub use provider::{",
)
replace_once(
    "engine/crates/translation-core/src/provider.rs",
    '''        format!(
            "You are the production translation engine for a long-form fiction workflow. Target language: {target_language}. {task} Treat the provided project context as binding continuity guidance when relevant. Return only the resulting passage."
        )''',
    '''        format!(
            "You are the production translation engine for a long-form fiction workflow. Target language: {target_language}. {task} Treat the provided project context as binding continuity guidance when relevant. Structural marker tokens used by the document layer (for example <m1>...</m1> and <r1/>) are immutable placeholders: preserve every marker identifier, count, pairing, and relative order exactly through translation, revision, and quality review. Return only the resulting passage."
        )''',
)

replace_once(
    "engine/crates/document-engine/src/models.rs",
    '''    #[serde(skip_serializing_if = "Option::is_none")]
    pub paragraph: Option<usize>,
''',
    '''    #[serde(skip_serializing_if = "Option::is_none")]
    pub paragraph: Option<usize>,
    /// Stable source-format block identity when the parser exposes one.
    /// EPUB/BookForge uses this to rebuild translated XHTML without guessing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub block_id: Option<String>,
''',
)
replace_once(
    "engine/crates/document-engine/src/models.rs",
    "            paragraph: None,\n",
    "            paragraph: None,\n            block_id: None,\n",
)

p = Path("engine/crates/document-engine/src/epub.rs")
text = p.read_text()
text = text.replace(
    "            section_blocks.push((kind, text));",
    "            section_blocks.push((kind, text, block.id.0.clone()));",
    1,
)
text = text.replace(
    "            .is_some_and(|(kind, _)| matches!(kind, BlockKind::Heading(_)));",
    "            .is_some_and(|(kind, _, _)| matches!(kind, BlockKind::Heading(_)));",
    1,
)
old = '''        for (kind, text) in section_blocks {
            paragraph_index += 1;
            let mut source = root.clone();
            source.resource = Some(section.href.clone());
            source.paragraph = Some(paragraph_index);
            parsed_blocks.push(ParsedBlock { kind, text, source });
        }'''
new = '''        for (kind, text, block_id) in section_blocks {
            paragraph_index += 1;
            let mut source = root.clone();
            source.resource = Some(section.href.clone());
            source.paragraph = Some(paragraph_index);
            source.block_id = Some(block_id);
            parsed_blocks.push(ParsedBlock { kind, text, source });
        }'''
if old not in text:
    raise SystemExit("EPUB tuple loop insertion point missing")
text = text.replace(old, new, 1)
p.write_text(text)

replace_once(
    "engine/crates/project-engine/src/application/error.rs",
    '''    #[error("translation is already running")]
    TranslationAlreadyRunning,
''',
    '''    #[error("invalid translation configuration: {0}")]
    InvalidTranslationConfig(String),
    #[error("translation is already running")]
    TranslationAlreadyRunning,
''',
)
replace_once(
    "engine/crates/project-engine/src/application/error.rs",
    '            Self::TranslationAlreadyRunning => "translation_already_running",\n',
    '            Self::InvalidTranslationConfig(_) => "invalid_translation_config",\n            Self::TranslationAlreadyRunning => "translation_already_running",\n',
)
replace_once(
    "engine/crates/project-engine/src/application/error.rs",
    "            Self::TranslationAlreadyRunning => RecoveryHint::ResumeExistingRun,\n",
    "            Self::InvalidTranslationConfig(_) => RecoveryHint::None,\n            Self::TranslationAlreadyRunning => RecoveryHint::ResumeExistingRun,\n",
)

replace_once(
    "engine/crates/project-engine/src/application/models.rs",
    "pub struct TranslatedChapter {\n",
    '''fn default_translation_style_profile() -> String {
    "literary".to_string()
}

pub struct TranslatedChapter {
''',
)
replace_once(
    "engine/crates/project-engine/src/application/models.rs",
    '''    pub context_fingerprint: String,
    pub paragraphs: Vec<TranslatedParagraph>,
''',
    '''    pub context_fingerprint: String,
    /// Translation style contract used for this artifact. Old artifacts default
    /// to the neutral literary profile for backward-compatible deserialization.
    #[serde(default = "default_translation_style_profile")]
    pub style_profile: String,
    pub paragraphs: Vec<TranslatedParagraph>,
''',
)

p = Path("engine/crates/project-engine/src/application/translation.rs")
text = p.read_text()
old = '''use translation_core::{
    EchoProvider, OpenAIProvider, PipelineInput, TranslationPipeline, TranslationProvider,
};'''
new = '''use translation_core::{
    EchoProvider, OpenAIProvider, PipelineInput, TranslationPipeline, TranslationProvider,
    TranslationStyleProfile,
};
use std::str::FromStr;'''
if old not in text:
    raise SystemExit("translation-core import block missing")
text = text.replace(old, new, 1)
text = text.replace(
    '''    pub target_language: String,
    /// Optional bound: translate at most this many chapters this run (used by
    /// the CLI/UI to chunk work; resume continues from checkpoints).
    pub max_chapters: Option<usize>,''',
    '''    pub target_language: String,
    /// Optional style contract. `literary` preserves the pre-Phase-20 behavior.
    pub style_profile: String,
    /// Required opt-in acknowledgement before the adult-intimacy fidelity
    /// profile may be used. This profile is never inferred automatically.
    pub adult_content_confirmed: bool,
    /// Optional bound: translate at most this many chapters this run (used by
    /// the CLI/UI to chunk work; resume continues from checkpoints).
    pub max_chapters: Option<usize>,''',
    1,
)
text = text.replace(
    '''            target_language: "fa".to_string(),
            max_chapters: None,''',
    '''            target_language: "fa".to_string(),
            style_profile: "literary".to_string(),
            adult_content_confirmed: false,
            max_chapters: None,''',
    1,
)
marker = '''    let manuscript = load_manuscript(layout)?;
    let (characters, glossary) = load_canon(layout)?;'''
replacement = '''    let manuscript = load_manuscript(layout)?;
    let style_profile = TranslationStyleProfile::from_str(&config.style_profile)
        .map_err(|error| ApplicationError::InvalidTranslationConfig(error.to_string()))?;
    if style_profile.requires_adult_confirmation() && !config.adult_content_confirmed {
        return Err(ApplicationError::InvalidTranslationConfig(
            "adult-intimacy requires explicit confirmation that every participant in sexual content is an adult; pass the confirmation only after verifying the source".into(),
        ));
    }
    let style_context = style_profile.context_block();
    let (characters, glossary) = load_canon(layout)?;'''
if marker not in text:
    raise SystemExit("translation style insertion point missing")
text = text.replace(marker, replacement, 1)
old_context = '''        let context_packet = context_build.packet;
        let context = context_packet.rendered_context.clone();
        let source_fingerprint = content_fingerprint(source_text.as_bytes());
        let context_fingerprint = context_packet.packet_fingerprint.clone();'''
new_context = '''        let context_packet = context_build.packet;
        let source_fingerprint = content_fingerprint(source_text.as_bytes());
        let (context, context_fingerprint) = if style_context.is_empty() {
            (
                context_packet.rendered_context.clone(),
                context_packet.packet_fingerprint.clone(),
            )
        } else {
            let rendered = if context_packet.rendered_context.trim().is_empty() {
                style_context.to_string()
            } else {
                format!("{}\\n\\n{}", context_packet.rendered_context, style_context)
            };
            let fingerprint_material = format!(
                "{}\\0{}\\0{}",
                context_packet.packet_fingerprint,
                style_profile.id(),
                style_context
            );
            (rendered, content_fingerprint(fingerprint_material.as_bytes()))
        };'''
if old_context not in text:
    raise SystemExit("context fingerprint block missing")
text = text.replace(old_context, new_context, 1)
text = text.replace(
    '''            context_fingerprint,
            paragraphs: align_paragraphs(chapter, &output.quality_review),''',
    '''            context_fingerprint,
            style_profile: style_profile.id().to_string(),
            paragraphs: align_paragraphs(chapter, &output.quality_review),''',
    1,
)
p.write_text(text)

p = Path("engine/cli/src/project_cmd.rs")
text = p.read_text()
text = text.replace(
    '''       translate <dir> [--provider echo|auto|openai] [--max-chapters <n>]\\n\\
       resume <dir>                                     resume an existing translation run\\n\\''',
    '''       translate <dir> [--provider echo|auto|openai] [--style-profile literary|adult-intimacy] [--confirm-adult-characters] [--max-chapters <n>]\\n\\
       resume <dir> [--style-profile literary|adult-intimacy] [--confirm-adult-characters]  resume an existing translation run\\n\\''',
    1,
)
old = '''    Ok(TranslationConfig {
        provider,
        model: flag_value(args, "--model").map(str::to_string),
        target_language: target,
        max_chapters,
    })'''
new = '''    Ok(TranslationConfig {
        provider,
        model: flag_value(args, "--model").map(str::to_string),
        target_language: target,
        style_profile: flag_value(args, "--style-profile")
            .unwrap_or("literary")
            .to_string(),
        adult_content_confirmed: args
            .iter()
            .any(|arg| arg == "--confirm-adult-characters"),
        max_chapters,
    })'''
if old not in text:
    raise SystemExit("project translation config literal missing")
text = text.replace(old, new, 1)
p.write_text(text)

p = Path("engine/crates/literary-review-engine/src/lib.rs")
text = p.read_text()
if "    IntimacyFidelity," not in text:
    text = text.replace(
        "    TerminologyContinuity,\n}",
        "    TerminologyContinuity,\n    IntimacyFidelity,\n}",
        1,
    )
p.write_text(text)

p = Path("engine/crates/literary-review-engine/src/provider.rs")
text = p.read_text()
text = text.replace(
    '''            "Do not provide chain-of-thought; provide concise review findings and revision rationale only."''',
    '''            "For intimacy_fidelity, when requested, evaluate only confirmed-adult source material and check parity of explicitness/markedness, consent/hesitation/refusal/coercion and power cues, physical agency/referents, sensory channels, POV, emotional intensity, and pacing; flag both sanitization and amplification. Do not sexualize nonsexual source text. Do not provide chain-of-thought; provide concise review findings and revision rationale only."''',
    1,
)
text = text.replace(
    '''            "terminology_continuity",
        ];''',
    '''            "terminology_continuity",
            "intimacy_fidelity",
        ];''',
    1,
)
p.write_text(text)

p = Path("engine/crates/project-engine/src/application/literary_review.rs")
text = p.read_text()
text = text.replace(
    "            let dimensions = provider_dimensions();",
    "            let dimensions = provider_dimensions(&translated.style_profile);",
    1,
)
text = text.replace(
    "            provider_dimensions(),",
    '            provider_dimensions("literary"),',
    1,
)
old = '''fn provider_dimensions() -> Vec<ReviewDimension> {
    vec![
        ReviewDimension::SemanticFidelity,
        ReviewDimension::CharacterVoice,
        ReviewDimension::RelationshipRegister,
        ReviewDimension::PersianNaturalness,
        ReviewDimension::DialogueSubtext,
        ReviewDimension::TerminologyContinuity,
    ]
}'''
new = '''fn provider_dimensions(style_profile: &str) -> Vec<ReviewDimension> {
    let mut dimensions = vec![
        ReviewDimension::SemanticFidelity,
        ReviewDimension::CharacterVoice,
        ReviewDimension::RelationshipRegister,
        ReviewDimension::PersianNaturalness,
        ReviewDimension::DialogueSubtext,
        ReviewDimension::TerminologyContinuity,
    ];
    if style_profile == "adult-intimacy" {
        dimensions.push(ReviewDimension::IntimacyFidelity);
    }
    dimensions
}'''
if old not in text:
    raise SystemExit("provider_dimensions function missing")
text = text.replace(old, new, 1)
p.write_text(text)
