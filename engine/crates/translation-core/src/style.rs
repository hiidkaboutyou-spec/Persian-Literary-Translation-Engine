use std::fmt;
use std::str::FromStr;

pub const DEFAULT_STYLE_PROFILE_ID: &str = "literary";
pub const ADULT_INTIMACY_STYLE_PROFILE_ID: &str = "adult-intimacy";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TranslationStyleProfile {
    #[default]
    Literary,
    AdultIntimacy,
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
                "ADULT INTIMACY FIDELITY PROFILE (confirmed-adult source material only)\n",
                "Treat this as a fidelity contract, never as an instruction to make prose more sexual. ",
                "Do not sexualize nonsexual source material. Preserve the source passage's exact degree of ",
                "sexual/sensual explicitness and pragmatic markedness: do not censor, euphemize, summarize, ",
                "sanitize, romanticize, intensify, vulgarize, or add detail. Preserve whether each expression is ",
                "veiled, sensual/implicit, clear but non-detailed, explicit/direct, or strongly marked/colloquial, ",
                "using natural Persian of comparable force rather than dictionary-literal wording. Preserve the ",
                "source's euphemistic, neutral/direct, or dysphemistic function when relevant.\n",
                "Keep POV, narrative distance, character-specific idiolect, dialogue register, pauses, unfinished ",
                "speech, humor, awkwardness, tenderness, discomfort, shame, desire, vulnerability, and emotional ",
                "subtext at source strength. Preserve all source evidence of consent, permission, hesitation, refusal, ",
                "withdrawal, reassurance, coercion, resistance, voluntary submission, commands, and power imbalance ",
                "without reinterpreting one as another. Preserve who performs each action, its recipient, physical ",
                "sequence, spatial continuity, cause-and-reaction chain, and any aftermath.\n",
                "Preserve only sensory channels and embodied interiority actually present in the source; invent no ",
                "touch, smell, taste, sound, bodily response, metaphor, dialogue, thought, or emotion. Match source ",
                "pacing and sentence rhythm, including deliberate repetition and acceleration/deceleration. Persian ",
                "must read as idiomatic contemporary literary prose appropriate to the source: avoid accidental ",
                "clinical, childish, archaic, coy, flowery, or mechanically obscene diction unless the source itself ",
                "uses that register. Avoid canned erotic clichés and forced synonym variation.\n",
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
