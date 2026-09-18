use character_engine::CharacterBible;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use text_normalization::normalize_case_insensitive;

const MAX_EXPLICIT_CUE_DISTANCE_CHARS: usize = 96;
const MAX_CONTEXT_QUOTES: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuoteStyle {
    StraightDouble,
    CurlyDouble,
    CurlySingle,
    Guillemets,
    LeadingDash,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuoteSpan {
    pub start_char: usize,
    pub end_char: usize,
    pub text: String,
    pub style: QuoteStyle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttributionMethod {
    ExplicitNameSpeechVerb,
    ExplicitAliasSpeechVerb,
    PronounSpeechVerbUnresolved,
    AmbiguousExplicitCandidates,
    NoSpeakerCue,
}

impl AttributionMethod {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ExplicitNameSpeechVerb => "explicit_name_speech_verb",
            Self::ExplicitAliasSpeechVerb => "explicit_alias_speech_verb",
            Self::PronounSpeechVerbUnresolved => "pronoun_speech_verb_unresolved",
            Self::AmbiguousExplicitCandidates => "ambiguous_explicit_candidates",
            Self::NoSpeakerCue => "no_speaker_cue",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpeakerAttribution {
    pub quote_id: String,
    pub paragraph_id: String,
    pub quote: QuoteSpan,
    pub speaker: Option<String>,
    pub mention: Option<String>,
    pub method: AttributionMethod,
    pub evidence: Option<String>,
}

impl SpeakerAttribution {
    pub fn resolved(&self) -> bool {
        self.speaker.is_some()
    }
}

#[derive(Debug, Clone)]
struct LexToken {
    raw: String,
    normalized: String,
    start_char: usize,
    end_char: usize,
}

#[derive(Debug, Clone)]
struct NamePattern {
    canonical: String,
    display: String,
    tokens: Vec<String>,
    alias: bool,
}

#[derive(Debug, Clone)]
struct ExplicitMentionCue {
    canonical: String,
    mention: String,
    alias: bool,
    start_char: usize,
    end_char: usize,
    name_then_verb: bool,
    verb_then_name: bool,
    name_then_verb_end_char: Option<usize>,
    name_then_verb_evidence: Option<String>,
    verb_then_name_evidence: Option<String>,
}

#[derive(Debug, Clone)]
struct PronounSpeechCue {
    start_char: usize,
    end_char: usize,
    pronoun_then_verb: bool,
    verb_then_pronoun: bool,
}

#[derive(Debug, Clone)]
struct ExplicitCandidate {
    canonical: String,
    mention: String,
    alias: bool,
    subject_pattern: bool,
    evidence: String,
}

pub fn attribute_speakers(
    paragraph_id: &str,
    text: &str,
    characters: &CharacterBible,
) -> Vec<SpeakerAttribution> {
    let quotes = detect_quotes(text);
    if quotes.is_empty() {
        return Vec::new();
    }

    let chars = text.chars().collect::<Vec<_>>();
    let tokens = lexical_tokens(text);
    let patterns = character_patterns(characters);
    let all_quotes = quotes.clone();
    let explicit_cues = extract_explicit_mention_cues(&tokens, &patterns, &all_quotes);
    let pronoun_cues = extract_pronoun_speech_cues(&tokens, &all_quotes);
    quotes
        .into_iter()
        .enumerate()
        .map(|(index, quote)| {
            attribute_one_quote(
                paragraph_id,
                index,
                &quote,
                &all_quotes,
                &chars,
                &explicit_cues,
                &pronoun_cues,
            )
        })
        .collect()
}

pub fn deterministic_speaker_context(
    source_id: &str,
    text: &str,
    characters: &CharacterBible,
) -> Option<String> {
    let resolved = attribute_speakers(source_id, text, characters)
        .into_iter()
        .filter(SpeakerAttribution::resolved)
        .take(MAX_CONTEXT_QUOTES)
        .collect::<Vec<_>>();
    if resolved.is_empty() {
        return None;
    }

    let mut lines = vec![
        "SPEAKER MAP — deterministic explicit evidence only; unresolved dialogue is omitted:"
            .to_string(),
    ];
    for item in resolved {
        let speaker = item.speaker.as_deref().unwrap_or_default();
        let excerpt = truncate_chars(item.quote.text.trim(), 90);
        lines.push(format!(
            "- {:?} → {} ({})",
            excerpt,
            speaker,
            item.method.as_str()
        ));
    }
    Some(lines.join("\n"))
}

fn attribute_one_quote(
    paragraph_id: &str,
    quote_index: usize,
    quote: &QuoteSpan,
    all_quotes: &[QuoteSpan],
    chars: &[char],
    explicit_cues: &[ExplicitMentionCue],
    pronoun_cues: &[PronounSpeechCue],
) -> SpeakerAttribution {
    let mut candidates = Vec::new();

    for cue in explicit_cues {
        let Some(side) = mention_side(cue.start_char, cue.end_char, quote) else {
            continue;
        };
        if side.distance > MAX_EXPLICIT_CUE_DISTANCE_CHARS
            || has_intervening_quote(cue.start_char, cue.end_char, quote, all_quotes)
        {
            continue;
        }

        // Before a quote, "verb + name" is frequently an object ("asked
        // Reza, ..."), so only the subject-like "name + verb" pattern is
        // accepted. After a quote, both common literary tag orders are
        // allowed, but subject-like candidates outrank inverted tags.
        let accepted = match side.position {
            MentionPosition::Before => {
                cue.name_then_verb
                    && cue.name_then_verb_evidence.is_some()
                    && cue.name_then_verb_end_char.is_some_and(|verb_end| {
                        !contains_hard_sentence_boundary(chars, verb_end, quote.start_char)
                    })
            }
            MentionPosition::After => cue.name_then_verb || cue.verb_then_name,
        };
        if !accepted {
            continue;
        }

        let subject_pattern = cue.name_then_verb;
        let evidence = if cue.name_then_verb {
            cue.name_then_verb_evidence.clone()
        } else {
            cue.verb_then_name_evidence.clone()
        }
        .unwrap_or_default();

        candidates.push(ExplicitCandidate {
            canonical: cue.canonical.clone(),
            mention: cue.mention.clone(),
            alias: cue.alias,
            subject_pattern,
            evidence,
        });
    }

    // Subject-like explicit patterns outrank verb->name patterns when both
    // occur around the same quote (e.g. "Mina asked Reza, \"Ready?\"").
    if candidates.iter().any(|candidate| candidate.subject_pattern) {
        candidates.retain(|candidate| candidate.subject_pattern);
    }
    let mut by_character = BTreeMap::<String, ExplicitCandidate>::new();
    for candidate in candidates {
        by_character
            .entry(normalize_case_insensitive(&candidate.canonical))
            .or_insert(candidate);
    }

    if by_character.len() == 1 {
        if let Some(candidate) = by_character.values().next() {
            return SpeakerAttribution {
                quote_id: format!("{paragraph_id}:quote-{quote_index}"),
                paragraph_id: paragraph_id.to_string(),
                quote: quote.clone(),
                speaker: Some(candidate.canonical.clone()),
                mention: Some(candidate.mention.clone()),
                method: if candidate.alias {
                    AttributionMethod::ExplicitAliasSpeechVerb
                } else {
                    AttributionMethod::ExplicitNameSpeechVerb
                },
                evidence: Some(candidate.evidence.clone()),
            };
        }
    }

    if by_character.len() > 1 {
        return unresolved(
            paragraph_id,
            quote_index,
            quote,
            AttributionMethod::AmbiguousExplicitCandidates,
            Some(
                by_character
                    .into_values()
                    .map(|candidate| candidate.canonical)
                    .collect::<Vec<_>>()
                    .join(" | "),
            ),
        );
    }

    if has_nearby_pronoun_speech_cue(pronoun_cues, quote, all_quotes) {
        return unresolved(
            paragraph_id,
            quote_index,
            quote,
            AttributionMethod::PronounSpeechVerbUnresolved,
            None,
        );
    }

    unresolved(
        paragraph_id,
        quote_index,
        quote,
        AttributionMethod::NoSpeakerCue,
        None,
    )
}

fn unresolved(
    paragraph_id: &str,
    quote_index: usize,
    quote: &QuoteSpan,
    method: AttributionMethod,
    evidence: Option<String>,
) -> SpeakerAttribution {
    SpeakerAttribution {
        quote_id: format!("{paragraph_id}:quote-{quote_index}"),
        paragraph_id: paragraph_id.to_string(),
        quote: quote.clone(),
        speaker: None,
        mention: None,
        method,
        evidence,
    }
}

#[derive(Debug, Clone, Copy)]
enum MentionPosition {
    Before,
    After,
}

#[derive(Debug, Clone, Copy)]
struct MentionSide {
    position: MentionPosition,
    distance: usize,
}

fn mention_side(start: usize, end: usize, quote: &QuoteSpan) -> Option<MentionSide> {
    if end <= quote.start_char {
        Some(MentionSide {
            position: MentionPosition::Before,
            distance: quote.start_char.saturating_sub(end),
        })
    } else if start >= quote.end_char {
        Some(MentionSide {
            position: MentionPosition::After,
            distance: start.saturating_sub(quote.end_char),
        })
    } else {
        None
    }
}

fn character_patterns(characters: &CharacterBible) -> Vec<NamePattern> {
    let mut patterns = Vec::new();
    for profile in characters.profiles() {
        patterns.push(NamePattern {
            canonical: profile.name.clone(),
            display: profile.name.clone(),
            tokens: normalized_name_tokens(&profile.name),
            alias: false,
        });
    }
    for alias in characters.aliases() {
        patterns.push(NamePattern {
            canonical: alias.canonical_name.clone(),
            display: alias.alias.clone(),
            tokens: normalized_name_tokens(&alias.alias),
            alias: true,
        });
    }
    patterns.sort_by(|left, right| {
        right
            .tokens
            .len()
            .cmp(&left.tokens.len())
            .then_with(|| left.canonical.cmp(&right.canonical))
            .then_with(|| left.display.cmp(&right.display))
    });
    patterns
}

fn normalized_name_tokens(value: &str) -> Vec<String> {
    lexical_tokens(value)
        .into_iter()
        .map(|token| token.normalized)
        .collect()
}

fn lexical_tokens(text: &str) -> Vec<LexToken> {
    let chars = text.chars().collect::<Vec<_>>();
    let mut tokens = Vec::new();
    let mut start = None;

    for (index, ch) in chars.iter().copied().enumerate() {
        let lexical = ch.is_alphanumeric() || ch == '\'' || ch == '’' || ch == '-';
        match (start, lexical) {
            (None, true) => start = Some(index),
            (Some(begin), false) => {
                push_token(&chars, begin, index, &mut tokens);
                start = None;
            }
            _ => {}
        }
    }
    if let Some(begin) = start {
        push_token(&chars, begin, chars.len(), &mut tokens);
    }
    tokens
}

fn push_token(chars: &[char], start: usize, end: usize, tokens: &mut Vec<LexToken>) {
    let raw = chars[start..end].iter().collect::<String>();
    let normalized = normalize_case_insensitive(&raw);
    if !normalized.is_empty() {
        tokens.push(LexToken {
            raw,
            normalized,
            start_char: start,
            end_char: end,
        });
    }
}

fn detect_quotes(text: &str) -> Vec<QuoteSpan> {
    let chars = text.chars().collect::<Vec<_>>();
    let mut spans = Vec::new();
    collect_paired_quotes(
        &chars,
        '"',
        '"',
        QuoteStyle::StraightDouble,
        &mut spans,
    );
    collect_paired_quotes(&chars, '“', '”', QuoteStyle::CurlyDouble, &mut spans);
    collect_paired_quotes(&chars, '‘', '’', QuoteStyle::CurlySingle, &mut spans);
    collect_paired_quotes(&chars, '«', '»', QuoteStyle::Guillemets, &mut spans);
    collect_leading_dash_quotes(&chars, &mut spans);

    spans.sort_by_key(|span| (span.start_char, span.end_char));
    spans.dedup_by_key(|span| (span.start_char, span.end_char));
    spans
}

fn collect_leading_dash_quotes(chars: &[char], spans: &mut Vec<QuoteSpan>) {
    let mut line_start = 0usize;
    while line_start < chars.len() {
        let line_end = chars[line_start..]
            .iter()
            .position(|ch| *ch == '\n')
            .map(|offset| line_start + offset)
            .unwrap_or(chars.len());

        let first_non_space = (line_start..line_end)
            .find(|index| !chars[*index].is_whitespace());
        if let Some(dash_index) = first_non_space.filter(|index| chars[*index] == '—') {
            let mut content_start = dash_index + 1;
            while content_start < line_end && chars[content_start].is_whitespace() {
                content_start += 1;
            }
            if content_start < line_end {
                spans.push(QuoteSpan {
                    start_char: content_start,
                    end_char: line_end,
                    text: chars[content_start..line_end].iter().collect(),
                    style: QuoteStyle::LeadingDash,
                });
            }
        }

        if line_end == chars.len() {
            break;
        }
        line_start = line_end + 1;
    }
}

fn collect_paired_quotes(
    chars: &[char],
    open: char,
    close: char,
    style: QuoteStyle,
    spans: &mut Vec<QuoteSpan>,
) {
    if open == close {
        let positions = chars
            .iter()
            .enumerate()
            .filter_map(|(index, ch)| (*ch == open).then_some(index))
            .collect::<Vec<_>>();
        for pair in positions.chunks_exact(2) {
            let start = pair[0] + 1;
            let end = pair[1];
            if start <= end {
                spans.push(QuoteSpan {
                    start_char: start,
                    end_char: end,
                    text: chars[start..end].iter().collect(),
                    style,
                });
            }
        }
        return;
    }

    let mut open_position = None;
    for (index, ch) in chars.iter().copied().enumerate() {
        if ch == open && open_position.is_none() {
            open_position = Some(index);
        } else if ch == close {
            if let Some(open_index) = open_position.take() {
                let start = open_index + 1;
                let end = index;
                if start <= end {
                    spans.push(QuoteSpan {
                        start_char: start,
                        end_char: end,
                        text: chars[start..end].iter().collect(),
                        style,
                    });
                }
            }
        }
    }
}

fn span_inside_any_quote(start: usize, end: usize, quotes: &[QuoteSpan]) -> bool {
    quotes
        .iter()
        .any(|quote| start >= quote.start_char && end <= quote.end_char)
}

fn has_intervening_quote(
    mention_start: usize,
    mention_end: usize,
    quote: &QuoteSpan,
    all_quotes: &[QuoteSpan],
) -> bool {
    all_quotes.iter().any(|other| {
        if other.start_char == quote.start_char && other.end_char == quote.end_char {
            return false;
        }
        if mention_end <= quote.start_char {
            other.start_char >= mention_end && other.end_char <= quote.start_char
        } else if mention_start >= quote.end_char {
            other.start_char >= quote.end_char && other.end_char <= mention_start
        } else {
            false
        }
    })
}

fn contains_hard_sentence_boundary(chars: &[char], start: usize, end: usize) -> bool {
    if start >= end || start >= chars.len() {
        return false;
    }
    chars[start..end.min(chars.len())]
        .iter()
        .any(|ch| matches!(*ch, '.' | '!' | '?' | '؟'))
}

fn extract_explicit_mention_cues(
    tokens: &[LexToken],
    patterns: &[NamePattern],
    all_quotes: &[QuoteSpan],
) -> Vec<ExplicitMentionCue> {
    let mut cues = Vec::new();

    for pattern in patterns {
        if pattern.tokens.is_empty() || pattern.tokens.len() > tokens.len() {
            continue;
        }
        for start in 0..=tokens.len() - pattern.tokens.len() {
            let end = start + pattern.tokens.len();
            if !tokens[start..end]
                .iter()
                .map(|token| token.normalized.as_str())
                .eq(pattern.tokens.iter().map(String::as_str))
            {
                continue;
            }

            let start_char = tokens[start].start_char;
            let end_char = tokens[end - 1].end_char;
            if span_inside_any_quote(start_char, end_char, all_quotes) {
                continue;
            }

            let name_then_verb_token = tokens
                .get(end)
                .filter(|token| is_speech_verb(&token.normalized));
            let name_then_verb = name_then_verb_token.is_some();
            let verb_then_name = start
                .checked_sub(1)
                .and_then(|idx| tokens.get(idx))
                .is_some_and(|token| is_speech_verb(&token.normalized));
            if !name_then_verb && !verb_then_name {
                continue;
            }

            let name_then_verb_evidence = name_then_verb.then(|| {
                format!(
                    "{} {}",
                    pattern.display,
                    tokens.get(end).map(|token| token.raw.as_str()).unwrap_or("")
                )
            });
            let verb_then_name_evidence = verb_then_name.then(|| {
                format!(
                    "{} {}",
                    tokens
                        .get(start.saturating_sub(1))
                        .map(|token| token.raw.as_str())
                        .unwrap_or(""),
                    pattern.display
                )
            });

            cues.push(ExplicitMentionCue {
                canonical: pattern.canonical.clone(),
                mention: pattern.display.clone(),
                alias: pattern.alias,
                start_char,
                end_char,
                name_then_verb,
                verb_then_name,
                name_then_verb_end_char: name_then_verb_token.map(|token| token.end_char),
                name_then_verb_evidence,
                verb_then_name_evidence,
            });
        }
    }

    cues
}

fn extract_pronoun_speech_cues(
    tokens: &[LexToken],
    all_quotes: &[QuoteSpan],
) -> Vec<PronounSpeechCue> {
    const PRONOUNS: [&str; 6] = ["he", "she", "they", "i", "we", "you"];
    let mut cues = Vec::new();

    for (index, token) in tokens.iter().enumerate() {
        if !PRONOUNS.contains(&token.normalized.as_str())
            || span_inside_any_quote(token.start_char, token.end_char, all_quotes)
        {
            continue;
        }
        let pronoun_then_verb = tokens
            .get(index + 1)
            .is_some_and(|next| is_speech_verb(&next.normalized));
        let verb_then_pronoun = index
            .checked_sub(1)
            .and_then(|idx| tokens.get(idx))
            .is_some_and(|prev| is_speech_verb(&prev.normalized));
        if pronoun_then_verb || verb_then_pronoun {
            cues.push(PronounSpeechCue {
                start_char: token.start_char,
                end_char: token.end_char,
                pronoun_then_verb,
                verb_then_pronoun,
            });
        }
    }

    cues
}

fn has_nearby_pronoun_speech_cue(
    cues: &[PronounSpeechCue],
    quote: &QuoteSpan,
    all_quotes: &[QuoteSpan],
) -> bool {
    cues.iter().any(|cue| {
        let Some(side) = mention_side(cue.start_char, cue.end_char, quote) else {
            return false;
        };
        if side.distance > MAX_EXPLICIT_CUE_DISTANCE_CHARS
            || has_intervening_quote(cue.start_char, cue.end_char, quote, all_quotes)
        {
            return false;
        }
        match side.position {
            MentionPosition::Before => cue.pronoun_then_verb,
            MentionPosition::After => cue.pronoun_then_verb || cue.verb_then_pronoun,
        }
    })
}

fn is_speech_verb(value: &str) -> bool {
    matches!(
        value,
        "say"
            | "says"
            | "said"
            | "ask"
            | "asks"
            | "asked"
            | "reply"
            | "replies"
            | "replied"
            | "answer"
            | "answers"
            | "answered"
            | "whisper"
            | "whispers"
            | "whispered"
            | "murmur"
            | "murmurs"
            | "murmured"
            | "mutter"
            | "mutters"
            | "muttered"
            | "shout"
            | "shouts"
            | "shouted"
            | "yell"
            | "yells"
            | "yelled"
            | "cry"
            | "cries"
            | "cried"
            | "exclaim"
            | "exclaims"
            | "exclaimed"
            | "add"
            | "adds"
            | "added"
            | "observe"
            | "observes"
            | "observed"
            | "call"
            | "calls"
            | "called"
    )
}

fn truncate_chars(value: &str, max_chars: usize) -> String {
    let mut out = value.chars().take(max_chars).collect::<String>();
    if value.chars().count() > max_chars {
        out.push('…');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use character_engine::CharacterProfile;

    fn bible() -> CharacterBible {
        let mut bible = CharacterBible::new();
        bible.add(CharacterProfile {
            name: "Mina".into(),
            voice_notes: "quiet and precise".into(),
            personality_notes: "guarded".into(),
        });
        bible.add(CharacterProfile {
            name: "Reza".into(),
            voice_notes: "warm".into(),
            personality_notes: "patient".into(),
        });
        bible.add_alias("Mina", "Min");
        bible
    }

    #[test]
    fn resolves_explicit_name_after_quote() {
        let result = attribute_speakers("p1", "\"Stay here,\" Mina said.", &bible());
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].speaker.as_deref(), Some("Mina"));
        assert_eq!(
            result[0].method,
            AttributionMethod::ExplicitNameSpeechVerb
        );
    }

    #[test]
    fn resolves_explicit_name_before_quote() {
        let result = attribute_speakers("p1", "Mina whispered, \"Stay here.\"", &bible());
        assert_eq!(result[0].speaker.as_deref(), Some("Mina"));
    }

    #[test]
    fn resolves_alias_without_changing_canonical_identity() {
        let result = attribute_speakers("p1", "“Stay,” Min replied.", &bible());
        assert_eq!(result[0].speaker.as_deref(), Some("Mina"));
        assert_eq!(result[0].mention.as_deref(), Some("Min"));
        assert_eq!(
            result[0].method,
            AttributionMethod::ExplicitAliasSpeechVerb
        );
    }

    #[test]
    fn vocative_inside_quote_is_not_mistaken_for_speaker() {
        let result = attribute_speakers("p1", "\"Reza, stay,\" Mina said.", &bible());
        assert_eq!(result[0].speaker.as_deref(), Some("Mina"));
    }

    #[test]
    fn ask_object_before_quote_is_not_mistaken_for_speaker() {
        let result = attribute_speakers("p1", "Mina asked Reza, \"Ready?\"", &bible());
        assert_eq!(result[0].speaker.as_deref(), Some("Mina"));
    }

    #[test]
    fn pronoun_speech_cue_remains_unresolved() {
        let result = attribute_speakers("p1", "\"Stay,\" she said.", &bible());
        assert!(result[0].speaker.is_none());
        assert_eq!(
            result[0].method,
            AttributionMethod::PronounSpeechVerbUnresolved
        );
    }

    #[test]
    fn equal_distance_alias_collision_fails_closed() {
        let mut colliding = bible();
        colliding.add_alias("Mina", "Reza");
        let result = attribute_speakers("p1", "\"Stay,\" Reza said.", &colliding);
        assert!(result[0].speaker.is_none());
        assert_eq!(
            result[0].method,
            AttributionMethod::AmbiguousExplicitCandidates
        );
    }

    #[test]
    fn quote_local_boundaries_isolate_multiple_quotes() {
        let result = attribute_speakers(
            "p1",
            "\"Stay,\" Mina said. \"No,\" Reza replied.",
            &bible(),
        );
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].speaker.as_deref(), Some("Mina"));
        assert_eq!(result[1].speaker.as_deref(), Some("Reza"));
    }

    #[test]
    fn conflicting_local_explicit_speakers_fail_closed() {
        let result = attribute_speakers(
            "p1",
            "Mina said, Reza said, \"Stay.\"",
            &bible(),
        );
        assert!(result[0].speaker.is_none());
        assert_eq!(
            result[0].method,
            AttributionMethod::AmbiguousExplicitCandidates
        );
    }

    #[test]
    fn pre_quote_tag_across_sentence_boundary_is_not_reused() {
        let result = attribute_speakers(
            "p1",
            "Mina said. \"No,\" Reza replied.",
            &bible(),
        );
        assert_eq!(result[0].speaker.as_deref(), Some("Reza"));
    }

    #[test]
    fn curly_single_quotes_are_supported_without_treating_apostrophes_as_quotes() {
        let result = attribute_speakers("p1", "‘Stay,’ Mina said. I don't know.", &bible());
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].speaker.as_deref(), Some("Mina"));
        assert_eq!(result[0].quote.style, QuoteStyle::CurlySingle);
    }

    #[test]
    fn multiple_leading_dash_lines_are_detected_without_speaker_guessing() {
        let result = attribute_speakers(
            "p1",
            "— Stay here.\n— I will.",
            &bible(),
        );
        assert_eq!(result.len(), 2);
        assert!(result.iter().all(|item| item.speaker.is_none()));
        assert!(result
            .iter()
            .all(|item| item.quote.style == QuoteStyle::LeadingDash));
    }

    #[test]
    fn guillemets_are_supported() {
        let result = attribute_speakers("p1", "«Stay,» Mina said.", &bible());
        assert_eq!(result[0].speaker.as_deref(), Some("Mina"));
        assert_eq!(result[0].quote.style, QuoteStyle::Guillemets);
    }

    #[test]
    fn leading_dash_dialogue_is_detected_but_not_guessed() {
        let result = attribute_speakers("p1", "— Stay here.", &bible());
        assert_eq!(result.len(), 1);
        assert!(result[0].speaker.is_none());
        assert_eq!(result[0].quote.style, QuoteStyle::LeadingDash);
    }

    #[test]
    fn many_quotes_reuse_precomputed_explicit_cues() {
        let mut text = String::new();
        for index in 0..200 {
            text.push_str(&format!(
                "\"Line {index},\" Mina said. \"Reply {index},\" Reza replied. "
            ));
        }

        let result = attribute_speakers("large-chapter", &text, &bible());
        assert_eq!(result.len(), 400);
        assert!(result.iter().all(SpeakerAttribution::resolved));
        assert_eq!(result[0].speaker.as_deref(), Some("Mina"));
        assert_eq!(result[1].speaker.as_deref(), Some("Reza"));
    }

    #[test]
    fn context_contains_only_resolved_explicit_speakers() {
        let context = deterministic_speaker_context(
            "chapter-1",
            "\"Stay,\" Mina said. \"No,\" she replied.",
            &bible(),
        )
        .unwrap();
        assert!(context.contains("Mina"));
        assert!(!context.contains("she"));
    }
}
