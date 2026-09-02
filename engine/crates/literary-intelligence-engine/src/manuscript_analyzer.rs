use crate::errors::LiteraryIntelligenceError;
use crate::models::manuscript_intelligence::*;
use character_engine::CharacterBible;
use document_engine::{Manuscript, Paragraph};
use memory_engine::glossary::Glossary;
use std::collections::{BTreeMap, BTreeSet};
use text_normalization::normalize_case_insensitive;

const ANALYZER_NAME: &str = "deterministic-manuscript-analyzer";
const ANALYZER_VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct AnalysisCanon<'a> {
    pub characters: &'a CharacterBible,
    pub glossary: &'a Glossary,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnalysisConfig {
    pub max_evidence_per_seed: usize,
    pub max_character_seeds: usize,
    pub max_terminology_seeds: usize,
    pub max_context_chars_per_chapter: usize,
    pub minimum_unknown_name_occurrences: usize,
    pub minimum_recurring_phrase_occurrences: usize,
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            max_evidence_per_seed: 8,
            max_character_seeds: 512,
            max_terminology_seeds: 256,
            max_context_chars_per_chapter: 4_000,
            minimum_unknown_name_occurrences: 2,
            minimum_recurring_phrase_occurrences: 3,
        }
    }
}

pub trait ManuscriptAnalyzer {
    fn analyze(
        &self,
        manuscript: &Manuscript,
        canon: AnalysisCanon<'_>,
    ) -> Result<ManuscriptIntelligence, LiteraryIntelligenceError>;
}

#[derive(Debug, Clone)]
pub struct DeterministicManuscriptAnalyzer {
    config: AnalysisConfig,
}

impl Default for DeterministicManuscriptAnalyzer {
    fn default() -> Self {
        Self::new(AnalysisConfig::default())
    }
}

impl DeterministicManuscriptAnalyzer {
    pub fn new(config: AnalysisConfig) -> Self {
        Self { config }
    }
}

#[derive(Debug, Clone)]
struct Candidate {
    display: String,
    count: usize,
    evidence: Vec<EvidenceRef>,
    aliases: BTreeSet<String>,
    approved: bool,
}

impl Candidate {
    fn new(display: String, approved: bool) -> Self {
        Self {
            display,
            count: 0,
            evidence: Vec::new(),
            aliases: BTreeSet::new(),
            approved,
        }
    }

    fn record(&mut self, evidence: EvidenceRef, limit: usize) {
        self.count += 1;
        if self.evidence.len() < limit
            && !self
                .evidence
                .iter()
                .any(|existing| existing.paragraph_id == evidence.paragraph_id)
        {
            self.evidence.push(evidence);
        }
    }
}

impl ManuscriptAnalyzer for DeterministicManuscriptAnalyzer {
    fn analyze(
        &self,
        manuscript: &Manuscript,
        canon: AnalysisCanon<'_>,
    ) -> Result<ManuscriptIntelligence, LiteraryIntelligenceError> {
        validate_config(&self.config)?;

        let alias_index = approved_alias_index(canon.characters);
        let mut candidates: BTreeMap<String, Candidate> = BTreeMap::new();
        let mut unknown_candidate_count = 0usize;
        let mut phrase_occurrences: BTreeMap<String, Candidate> = BTreeMap::new();
        let mut approved_term_occurrences: BTreeMap<String, Candidate> = BTreeMap::new();
        let mut dialogue_paragraphs = 0usize;
        let mut paragraph_count = 0usize;
        let mut sentence_count = 0usize;
        let mut word_count = 0usize;
        let mut dialogue_conventions = BTreeSet::new();
        let mut saw_latin = false;
        let mut saw_non_latin = false;

        for chapter in &manuscript.chapters {
            for scene in &chapter.scenes {
                for paragraph in &scene.paragraphs {
                    paragraph_count += 1;
                    let text = paragraph.original_text.trim();
                    let words = words(text);
                    word_count += words.len();
                    sentence_count += count_sentences(text);
                    let is_dialogue = dialogue_style(text, &mut dialogue_conventions);
                    dialogue_paragraphs += usize::from(is_dialogue);
                    for character in text.chars().filter(|character| character.is_alphabetic()) {
                        if character.is_ascii_alphabetic() {
                            saw_latin = true;
                        } else {
                            saw_non_latin = true;
                        }
                    }

                    let evidence = evidence_ref(chapter.id.as_str(), scene.id.as_str(), paragraph);
                    let normalized_text = padded_normalized(text);
                    let mut approved_hits = BTreeSet::new();
                    for (name_or_alias, canonical) in &alias_index {
                        if normalized_text.contains(&format!(" {name_or_alias} ")) {
                            approved_hits.insert(canonical.clone());
                        }
                    }
                    for canonical in approved_hits {
                        let key = normalize_case_insensitive(&canonical);
                        let entry = candidates
                            .entry(key)
                            .or_insert_with(|| Candidate::new(canonical.clone(), true));
                        for registered in canon.characters.aliases().iter().filter(|alias| {
                            normalize_case_insensitive(&alias.canonical_name)
                                == normalize_case_insensitive(&canonical)
                        }) {
                            entry.aliases.insert(registered.alias.clone());
                        }
                        entry.record(evidence.clone(), self.config.max_evidence_per_seed);
                    }

                    for candidate in extract_capitalized_candidates(text) {
                        let normalized = normalize_case_insensitive(&candidate);
                        if normalized.is_empty() || alias_index.contains_key(&normalized) {
                            continue;
                        }
                        if !candidates.contains_key(&normalized) {
                            if unknown_candidate_count >= self.config.max_character_seeds {
                                continue;
                            }
                            unknown_candidate_count += 1;
                        }
                        candidates
                            .entry(normalized)
                            .or_insert_with(|| Candidate::new(candidate, false))
                            .record(evidence.clone(), self.config.max_evidence_per_seed);
                    }

                    for entry in canon.glossary.relevant_to_text(text) {
                        let key = normalize_case_insensitive(&entry.source_term);
                        approved_term_occurrences
                            .entry(key)
                            .or_insert_with(|| Candidate::new(entry.source_term.clone(), true))
                            .record(evidence.clone(), self.config.max_evidence_per_seed);
                    }

                    record_recurring_phrases(
                        &words,
                        &evidence,
                        &mut phrase_occurrences,
                        self.config.max_evidence_per_seed,
                        self.config.max_terminology_seeds.saturating_mul(16),
                    );
                }
            }
        }

        merge_name_aliases(&mut candidates, self.config.max_evidence_per_seed);
        candidates.retain(|_, candidate| {
            candidate.approved || candidate.count >= self.config.minimum_unknown_name_occurrences
        });

        let mut character_seeds = build_character_seeds(&manuscript.book.id, candidates);
        let relationship_seeds = build_relationship_seeds(
            manuscript,
            &character_seeds,
            canon.characters,
            self.config.max_evidence_per_seed,
        );
        attach_relationship_refs(&mut character_seeds, &relationship_seeds);

        let terminology_seeds = build_terminology_seeds(
            &manuscript.book.id,
            &character_seeds,
            phrase_occurrences,
            approved_term_occurrences,
            canon.glossary,
            &self.config,
        );
        let chapter_maps = build_chapter_maps(
            manuscript,
            &character_seeds,
            &relationship_seeds,
            &terminology_seeds,
        );
        let literary_profile = LiteraryProfile {
            observed: ObservedLiteraryProfile {
                paragraph_count,
                sentence_count,
                average_sentence_words: ratio(word_count, sentence_count),
                dialogue_density: ratio(dialogue_paragraphs, paragraph_count),
                prose_density: ratio(
                    paragraph_count.saturating_sub(dialogue_paragraphs),
                    paragraph_count,
                ),
                dialogue_conventions: dialogue_conventions.into_iter().collect(),
                code_switching_detected: saw_latin && saw_non_latin,
                chapter_count: manuscript.chapters.len(),
                average_scenes_per_chapter: ratio(
                    manuscript
                        .chapters
                        .iter()
                        .map(|chapter| chapter.scenes.len())
                        .sum(),
                    manuscript.chapters.len(),
                ),
            },
            inferred: InferredLiteraryProfile::default(),
        };
        let initialization = build_initialization_proposal(
            manuscript,
            &character_seeds,
            &relationship_seeds,
            &terminology_seeds,
            canon,
            self.config.max_context_chars_per_chapter,
        );

        let intelligence = ManuscriptIntelligence {
            schema_version: MANUSCRIPT_INTELLIGENCE_SCHEMA_VERSION,
            manuscript_id: manuscript.book.id.clone(),
            manuscript_title: manuscript.book.title.clone(),
            literary_profile,
            character_seeds,
            relationship_seeds,
            chapter_maps,
            terminology_seeds,
            analysis: AnalysisMetadata {
                analyzer: ANALYZER_NAME.into(),
                analyzer_version: ANALYZER_VERSION.into(),
                deterministic: true,
                retained_evidence_limit: self.config.max_evidence_per_seed,
                character_seed_limit: self.config.max_character_seeds,
                terminology_limit: self.config.max_terminology_seeds,
            },
            source: manuscript.source.clone(),
            initialization,
        };
        intelligence.validate()?;
        Ok(intelligence)
    }
}

fn validate_config(config: &AnalysisConfig) -> Result<(), LiteraryIntelligenceError> {
    if config.max_evidence_per_seed == 0 {
        return Err(LiteraryIntelligenceError::Validation(
            "max_evidence_per_seed must be greater than zero".into(),
        ));
    }
    if config.max_terminology_seeds == 0 {
        return Err(LiteraryIntelligenceError::Validation(
            "max_terminology_seeds must be greater than zero".into(),
        ));
    }
    if config.max_character_seeds == 0 {
        return Err(LiteraryIntelligenceError::Validation(
            "max_character_seeds must be greater than zero".into(),
        ));
    }
    Ok(())
}

fn approved_alias_index(bible: &CharacterBible) -> BTreeMap<String, String> {
    let mut index = BTreeMap::new();
    for profile in bible.profiles() {
        index.insert(
            normalize_case_insensitive(&profile.name),
            profile.name.clone(),
        );
    }
    for alias in bible.aliases() {
        index.insert(
            normalize_case_insensitive(&alias.alias),
            alias.canonical_name.clone(),
        );
    }
    index
}

fn evidence_ref(chapter_id: &str, scene_id: &str, paragraph: &Paragraph) -> EvidenceRef {
    EvidenceRef {
        chapter_id: chapter_id.into(),
        scene_id: scene_id.into(),
        paragraph_id: paragraph.id.clone(),
        source: paragraph.source.clone(),
    }
}

fn padded_normalized(text: &str) -> String {
    format!(" {} ", normalize_case_insensitive(text))
}

fn words(text: &str) -> Vec<String> {
    text.split_whitespace()
        .filter_map(|raw| {
            let trimmed = raw.trim_matches(|character: char| {
                !character.is_alphanumeric()
                    && character != '\''
                    && character != '’'
                    && character != '-'
            });
            (!trimmed.is_empty()).then(|| trimmed.to_string())
        })
        .collect()
}

fn count_sentences(text: &str) -> usize {
    let count = text
        .chars()
        .filter(|character| matches!(character, '.' | '!' | '?' | '؟'))
        .count();
    if text.trim().is_empty() {
        0
    } else {
        count.max(1)
    }
}

fn dialogue_style(text: &str, conventions: &mut BTreeSet<String>) -> bool {
    let trimmed = text.trim_start();
    let mut dialogue = false;
    if trimmed.starts_with('"') || trimmed.starts_with('“') || trimmed.starts_with('”') {
        conventions.insert("double_quotes".into());
        dialogue = true;
    }
    if trimmed.starts_with('\'') || trimmed.starts_with('‘') || trimmed.starts_with('’') {
        conventions.insert("single_quotes".into());
        dialogue = true;
    }
    if trimmed.starts_with('—') || trimmed.starts_with('-') {
        conventions.insert("leading_dash".into());
        dialogue = true;
    }
    if text.contains('«') || text.contains('»') {
        conventions.insert("guillemets".into());
        dialogue = true;
    }
    dialogue
}

fn extract_capitalized_candidates(text: &str) -> Vec<String> {
    let tokens = text
        .split_whitespace()
        .filter_map(|raw| {
            let value = raw.trim_matches(|character: char| {
                !character.is_alphanumeric()
                    && character != '\''
                    && character != '’'
                    && character != '-'
            });
            if value.is_empty() {
                None
            } else {
                let boundary_after =
                    raw.ends_with(['.', ',', '!', '?', '؟', ';', ':', '"', '”', '’']);
                Some((value.to_string(), boundary_after))
            }
        })
        .collect::<Vec<_>>();
    let mut result = Vec::new();
    let mut index = 0;
    while index < tokens.len() {
        if is_capitalized_name_token(&tokens[index].0) {
            let start = index;
            index += 1;
            while index < tokens.len()
                && index - start < 3
                && !tokens[index - 1].1
                && is_capitalized_name_token(&tokens[index].0)
            {
                index += 1;
            }
            let mut candidate = tokens[start..index]
                .iter()
                .map(|(token, _)| token.as_str())
                .collect::<Vec<_>>()
                .join(" ");
            let parts: Vec<&str> = candidate.split_whitespace().collect();
            if parts.len() > 1 && is_honorific(parts[0]) {
                candidate = parts[1..].join(" ");
            }
            if !candidate.is_empty() && !is_name_stopword(&candidate) {
                result.push(candidate);
            }
        } else {
            index += 1;
        }
    }
    result
}

fn is_capitalized_name_token(token: &str) -> bool {
    let mut characters = token.chars();
    matches!(characters.next(), Some(first) if first.is_uppercase())
        && characters.any(|character| character.is_alphabetic())
}

fn is_honorific(value: &str) -> bool {
    matches!(
        value.trim_end_matches('.').to_ascii_lowercase().as_str(),
        "mr" | "mrs" | "ms" | "miss" | "dr" | "doctor" | "sir" | "lady" | "lord" | "professor"
    )
}

fn is_name_stopword(value: &str) -> bool {
    matches!(
        normalize_case_insensitive(value).as_str(),
        "a" | "an"
            | "and"
            | "as"
            | "at"
            | "but"
            | "chapter"
            | "epilogue"
            | "for"
            | "he"
            | "her"
            | "his"
            | "i"
            | "if"
            | "in"
            | "it"
            | "its"
            | "my"
            | "no"
            | "not"
            | "of"
            | "on"
            | "or"
            | "our"
            | "prologue"
            | "she"
            | "so"
            | "that"
            | "the"
            | "their"
            | "then"
            | "there"
            | "they"
            | "this"
            | "to"
            | "we"
            | "what"
            | "when"
            | "where"
            | "who"
            | "why"
            | "you"
            | "your"
    )
}

fn merge_name_aliases(candidates: &mut BTreeMap<String, Candidate>, evidence_limit: usize) {
    let multi_names: Vec<(String, Vec<String>)> = candidates
        .iter()
        .filter_map(|(key, candidate)| {
            let parts = normalize_case_insensitive(&candidate.display)
                .split_whitespace()
                .map(str::to_string)
                .collect::<Vec<_>>();
            (parts.len() > 1).then(|| (key.clone(), parts))
        })
        .collect();
    let single_keys: Vec<String> = candidates
        .iter()
        .filter(|(key, candidate)| !key.contains(' ') && !candidate.approved)
        .map(|(key, _)| key)
        .cloned()
        .collect();

    for single_key in single_keys {
        let matches: Vec<_> = multi_names
            .iter()
            .filter(|(_, parts)| parts.contains(&single_key))
            .map(|(key, _)| key.clone())
            .collect();
        if matches.len() != 1 {
            continue;
        }
        let Some(single) = candidates.remove(&single_key) else {
            continue;
        };
        if let Some(full) = candidates.get_mut(&matches[0]) {
            full.count += single.count;
            full.aliases.insert(single.display);
            for evidence in single.evidence {
                if full.evidence.len() < evidence_limit
                    && !full
                        .evidence
                        .iter()
                        .any(|existing| existing.paragraph_id == evidence.paragraph_id)
                {
                    full.evidence.push(evidence);
                }
            }
        }
    }
}

fn stable_id(namespace: &str, value: &str) -> String {
    const OFFSET: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x100000001b3;
    let hash = format!("{namespace}\0{}", normalize_case_insensitive(value))
        .as_bytes()
        .iter()
        .fold(OFFSET, |hash, byte| {
            (hash ^ u64::from(*byte)).wrapping_mul(PRIME)
        });
    format!("seed-{hash:016x}")
}

fn build_character_seeds(
    book_id: &str,
    candidates: BTreeMap<String, Candidate>,
) -> Vec<CharacterSeed> {
    let mut seeds = candidates
        .into_values()
        .filter_map(|candidate| {
            let first_appearance = candidate.evidence.first()?.clone();
            Some(CharacterSeed {
                id: stable_id(&format!("{book_id}:character"), &candidate.display),
                canonical_name_candidate: candidate.display,
                aliases: candidate.aliases.into_iter().collect(),
                first_appearance,
                appearance_count: candidate.count,
                confidence: Confidence::from_evidence(candidate.count),
                evidence: candidate.evidence,
                likely_narrative_role: None,
                speech_register_observations: Vec::new(),
                recurring_lexical_patterns: Vec::new(),
                personality_observations: Vec::new(),
                relationship_refs: Vec::new(),
                status: SeedStatus::Inferred,
                matches_approved_character: candidate.approved,
            })
        })
        .collect::<Vec<_>>();
    seeds.sort_by(|left, right| {
        left.first_appearance
            .source
            .chapter
            .cmp(&right.first_appearance.source.chapter)
            .then_with(|| {
                left.first_appearance
                    .source
                    .paragraph
                    .cmp(&right.first_appearance.source.paragraph)
            })
            .then_with(|| {
                left.canonical_name_candidate
                    .cmp(&right.canonical_name_candidate)
            })
    });
    seeds
}

fn paragraph_has_character(text: &str, seed: &CharacterSeed) -> bool {
    let normalized = padded_normalized(text);
    std::iter::once(&seed.canonical_name_candidate)
        .chain(seed.aliases.iter())
        .any(|name| normalized.contains(&format!(" {} ", normalize_case_insensitive(name))))
}

fn build_relationship_seeds(
    manuscript: &Manuscript,
    characters: &[CharacterSeed],
    bible: &CharacterBible,
    evidence_limit: usize,
) -> Vec<RelationshipSeed> {
    let mut pairs: BTreeMap<(usize, usize), Vec<EvidenceRef>> = BTreeMap::new();
    let mut counts: BTreeMap<(usize, usize), usize> = BTreeMap::new();
    for chapter in &manuscript.chapters {
        for scene in &chapter.scenes {
            for paragraph in &scene.paragraphs {
                let present = characters
                    .iter()
                    .enumerate()
                    .filter(|(_, seed)| paragraph_has_character(&paragraph.original_text, seed))
                    .map(|(index, _)| index)
                    .collect::<Vec<_>>();
                for (offset, left) in present.iter().enumerate() {
                    for right in present.iter().skip(offset + 1) {
                        let pair = (*left.min(right), *left.max(right));
                        *counts.entry(pair).or_default() += 1;
                        let evidence = pairs.entry(pair).or_default();
                        if evidence.len() < evidence_limit {
                            evidence.push(evidence_ref(&chapter.id, &scene.id, paragraph));
                        }
                    }
                }
            }
        }
    }

    let mut seeds = pairs
        .into_iter()
        .map(|((left, right), evidence)| {
            let left_character = &characters[left];
            let right_character = &characters[right];
            let (a, b) = if normalize_case_insensitive(&left_character.canonical_name_candidate)
                <= normalize_case_insensitive(&right_character.canonical_name_candidate)
            {
                (left_character, right_character)
            } else {
                (right_character, left_character)
            };
            let approved = bible.relationships().iter().any(|relationship| {
                normalized_pair(&relationship.character_a, &relationship.character_b)
                    == normalized_pair(&a.canonical_name_candidate, &b.canonical_name_candidate)
            });
            let pair_value = format!("{}\0{}", a.id, b.id);
            let interaction_count = counts[&(left, right)];
            RelationshipSeed {
                id: stable_id("relationship", &pair_value),
                character_a_id: a.id.clone(),
                character_b_id: b.id.clone(),
                character_a: a.canonical_name_candidate.clone(),
                character_b: b.canonical_name_candidate.clone(),
                relationship_label_candidate: None,
                forms_of_address: Vec::new(),
                interaction_count,
                evidence,
                confidence: Confidence::from_evidence(interaction_count),
                status: SeedStatus::Inferred,
                matches_approved_relationship: approved,
            }
        })
        .collect::<Vec<_>>();
    seeds.sort_by(|left, right| {
        left.character_a
            .cmp(&right.character_a)
            .then_with(|| left.character_b.cmp(&right.character_b))
    });
    seeds
}

fn normalized_pair(a: &str, b: &str) -> (String, String) {
    let a = normalize_case_insensitive(a);
    let b = normalize_case_insensitive(b);
    if a <= b {
        (a, b)
    } else {
        (b, a)
    }
}

fn attach_relationship_refs(characters: &mut [CharacterSeed], relationships: &[RelationshipSeed]) {
    for character in characters {
        character.relationship_refs = relationships
            .iter()
            .filter(|relationship| {
                relationship.character_a_id == character.id
                    || relationship.character_b_id == character.id
            })
            .map(|relationship| relationship.id.clone())
            .collect();
    }
}

fn record_recurring_phrases(
    tokens: &[String],
    evidence: &EvidenceRef,
    phrases: &mut BTreeMap<String, Candidate>,
    evidence_limit: usize,
    candidate_limit: usize,
) {
    let normalized = tokens
        .iter()
        .map(|token| normalize_case_insensitive(token))
        .filter(|token| !token.is_empty())
        .collect::<Vec<_>>();
    for width in 2..=3 {
        for phrase in normalized.windows(width) {
            if phrase.iter().all(|token| is_phrase_stopword(token)) {
                continue;
            }
            let value = phrase.join(" ");
            if !phrases.contains_key(&value) && phrases.len() >= candidate_limit {
                continue;
            }
            phrases
                .entry(value.clone())
                .or_insert_with(|| Candidate::new(value, false))
                .record(evidence.clone(), evidence_limit);
        }
    }
}

fn is_phrase_stopword(token: &str) -> bool {
    matches!(
        token,
        "a" | "an" | "and" | "at" | "for" | "in" | "of" | "on" | "or" | "the" | "to"
    )
}

fn build_terminology_seeds(
    book_id: &str,
    characters: &[CharacterSeed],
    phrases: BTreeMap<String, Candidate>,
    approved_terms: BTreeMap<String, Candidate>,
    glossary: &Glossary,
    config: &AnalysisConfig,
) -> Vec<TerminologySeed> {
    let mut seeds = characters
        .iter()
        .map(|character| TerminologySeed {
            id: stable_id(
                &format!("{book_id}:term"),
                &character.canonical_name_candidate,
            ),
            source_expression: character.canonical_name_candidate.clone(),
            occurrence_count: character.appearance_count,
            category: TerminologyCategory::CharacterName,
            evidence: character.evidence.clone(),
            confidence: character.confidence.clone(),
            status: SeedStatus::Inferred,
            approved_translation: glossary
                .find_exact_term(&character.canonical_name_candidate)
                .map(|entry| entry.preferred_translation.clone()),
        })
        .collect::<Vec<_>>();
    let mut seen = seeds
        .iter()
        .map(|seed| normalize_case_insensitive(&seed.source_expression))
        .collect::<BTreeSet<_>>();

    for candidate in approved_terms.into_values() {
        let normalized = normalize_case_insensitive(&candidate.display);
        if !seen.insert(normalized) {
            continue;
        }
        seeds.push(TerminologySeed {
            id: stable_id(&format!("{book_id}:term"), &candidate.display),
            source_expression: candidate.display.clone(),
            occurrence_count: candidate.count,
            category: TerminologyCategory::RecurringExpression,
            evidence: candidate.evidence,
            confidence: Confidence::from_evidence(candidate.count),
            status: SeedStatus::Inferred,
            approved_translation: glossary
                .find_exact_term(&candidate.display)
                .map(|entry| entry.preferred_translation.clone()),
        });
    }

    let character_terms = characters
        .iter()
        .flat_map(|character| {
            std::iter::once(&character.canonical_name_candidate).chain(character.aliases.iter())
        })
        .map(|value| normalize_case_insensitive(value))
        .collect::<BTreeSet<_>>();
    let mut recurring = phrases
        .into_values()
        .filter(|candidate| candidate.count >= config.minimum_recurring_phrase_occurrences)
        .filter(|candidate| !character_terms.contains(&candidate.display))
        .collect::<Vec<_>>();
    recurring.sort_by(|left, right| {
        right
            .count
            .cmp(&left.count)
            .then_with(|| left.display.cmp(&right.display))
    });

    for candidate in recurring {
        if seeds.len() >= config.max_terminology_seeds {
            break;
        }
        if !seen.insert(normalize_case_insensitive(&candidate.display)) {
            continue;
        }
        let approved_translation = glossary
            .find_exact_term(&candidate.display)
            .map(|entry| entry.preferred_translation.clone());
        seeds.push(TerminologySeed {
            id: stable_id(&format!("{book_id}:term"), &candidate.display),
            source_expression: candidate.display,
            occurrence_count: candidate.count,
            category: TerminologyCategory::RecurringExpression,
            evidence: candidate.evidence,
            confidence: Confidence::from_evidence(candidate.count),
            status: SeedStatus::Inferred,
            approved_translation,
        });
    }
    seeds.sort_by(|left, right| left.source_expression.cmp(&right.source_expression));
    seeds.truncate(config.max_terminology_seeds);
    seeds
}

fn build_chapter_maps(
    manuscript: &Manuscript,
    characters: &[CharacterSeed],
    relationships: &[RelationshipSeed],
    terms: &[TerminologySeed],
) -> Vec<ChapterMap> {
    manuscript
        .chapters
        .iter()
        .map(|chapter| {
            let paragraph_count = chapter
                .scenes
                .iter()
                .map(|scene| scene.paragraphs.len())
                .sum::<usize>();
            let dialogue_count = chapter
                .scenes
                .iter()
                .flat_map(|scene| &scene.paragraphs)
                .filter(|paragraph| {
                    let mut conventions = BTreeSet::new();
                    dialogue_style(&paragraph.original_text, &mut conventions)
                })
                .count();
            let character_seed_ids = characters
                .iter()
                .filter(|seed| {
                    seed.evidence
                        .iter()
                        .any(|evidence| evidence.chapter_id == chapter.id)
                })
                .map(|seed| seed.id.clone())
                .collect::<Vec<_>>();
            let relationship_seed_ids = relationships
                .iter()
                .filter(|seed| {
                    seed.evidence
                        .iter()
                        .any(|evidence| evidence.chapter_id == chapter.id)
                })
                .map(|seed| seed.id.clone())
                .collect::<Vec<_>>();
            let recurring_terms = terms
                .iter()
                .filter(|seed| {
                    seed.evidence
                        .iter()
                        .any(|evidence| evidence.chapter_id == chapter.id)
                })
                .map(|seed| seed.source_expression.clone())
                .collect::<Vec<_>>();
            let important_named_entities = characters
                .iter()
                .filter(|seed| character_seed_ids.contains(&seed.id))
                .map(|seed| seed.canonical_name_candidate.clone())
                .collect();
            ChapterMap {
                chapter_id: chapter.id.clone(),
                source_chapter_index: chapter.index,
                title: chapter.title.clone(),
                source: chapter.source.clone(),
                scene_ids: chapter
                    .scenes
                    .iter()
                    .map(|scene| scene.id.clone())
                    .collect(),
                character_seed_ids,
                important_named_entities,
                recurring_terms,
                relationship_seed_ids,
                dialogue_density: ratio(dialogue_count, paragraph_count),
                narrative_density: ratio(
                    paragraph_count.saturating_sub(dialogue_count),
                    paragraph_count,
                ),
                unresolved_references: Vec::new(),
                continuity_hooks: Vec::new(),
            }
        })
        .collect()
}

fn build_initialization_proposal(
    manuscript: &Manuscript,
    characters: &[CharacterSeed],
    relationships: &[RelationshipSeed],
    terms: &[TerminologySeed],
    canon: AnalysisCanon<'_>,
    max_context_chars: usize,
) -> InitializationProposal {
    let mut conflicts = Vec::new();
    let approved_aliases = approved_alias_index(canon.characters);
    for character in characters
        .iter()
        .filter(|seed| !seed.matches_approved_character)
    {
        let candidate_tokens = normalize_case_insensitive(&character.canonical_name_candidate)
            .split_whitespace()
            .map(str::to_string)
            .collect::<BTreeSet<_>>();
        if let Some((_, canonical)) = approved_aliases
            .iter()
            .find(|(alias, _)| candidate_tokens.contains(alias.as_str()))
        {
            conflicts.push(SeedConflict {
                kind: ConflictKind::CharacterName,
                seed_id: character.id.clone(),
                inferred_value: character.canonical_name_candidate.clone(),
                approved_value: canonical.clone(),
                message:
                    "approved character identity takes precedence over an ambiguous inferred name"
                        .into(),
            });
        }
    }
    for term in terms {
        if let Some(approved) = canon.glossary.find_exact_term(&term.source_expression) {
            if term.approved_translation.as_deref() != Some(approved.preferred_translation.as_str())
            {
                conflicts.push(SeedConflict {
                    kind: ConflictKind::Terminology,
                    seed_id: term.id.clone(),
                    inferred_value: term.approved_translation.clone().unwrap_or_default(),
                    approved_value: approved.preferred_translation.clone(),
                    message: "approved glossary entry takes precedence over inferred terminology"
                        .into(),
                });
            }
        }
    }
    conflicts.sort_by(|left, right| left.seed_id.cmp(&right.seed_id));
    let conflicting_seed_ids = conflicts
        .iter()
        .map(|conflict| conflict.seed_id.as_str())
        .collect::<BTreeSet<_>>();
    let approved_character_names = characters
        .iter()
        .filter(|character| character.matches_approved_character)
        .map(|character| normalize_case_insensitive(&character.canonical_name_candidate))
        .collect::<BTreeSet<_>>();

    let chapters = manuscript
        .chapters
        .iter()
        .map(|chapter| {
            let chapter_characters = characters
                .iter()
                .filter(|seed| seed.evidence.iter().any(|evidence| evidence.chapter_id == chapter.id))
                .collect::<Vec<_>>();
            let chapter_relationships = relationships
                .iter()
                .filter(|seed| seed.evidence.iter().any(|evidence| evidence.chapter_id == chapter.id))
                .collect::<Vec<_>>();
            let chapter_terms = terms
                .iter()
                .filter(|seed| seed.evidence.iter().any(|evidence| evidence.chapter_id == chapter.id))
                .collect::<Vec<_>>();

            let mut lines = vec!["MANUSCRIPT SEEDS — inferred evidence only; approved project memory takes precedence:".to_string()];
            for seed in chapter_characters.iter().filter(|seed| {
                !seed.matches_approved_character && !conflicting_seed_ids.contains(seed.id.as_str())
            }) {
                lines.push(format!(
                    "- character candidate: {} ({:?}, {} occurrences)",
                    seed.canonical_name_candidate, seed.confidence.level, seed.appearance_count
                ));
            }
            for seed in chapter_relationships.iter().filter(|seed| !seed.matches_approved_relationship) {
                lines.push(format!(
                    "- co-occurrence: {} ↔ {} ({} interactions; relationship type unknown)",
                    seed.character_a, seed.character_b, seed.interaction_count
                ));
            }
            for seed in chapter_terms
                .iter()
                .filter(|seed| {
                    seed.approved_translation.is_none()
                        && !(seed.category == TerminologyCategory::CharacterName
                            && approved_character_names
                                .contains(&normalize_case_insensitive(&seed.source_expression)))
                })
                .take(12)
            {
                lines.push(format!("- terminology candidate: {}", seed.source_expression));
            }
            let context = if lines.len() == 1 {
                String::new()
            } else {
                truncate_chars(&lines.join("\n"), max_context_chars)
            };
            ChapterInitializationProposal {
                chapter_id: chapter.id.clone(),
                character_seed_ids: chapter_characters.iter().map(|seed| seed.id.clone()).collect(),
                relationship_seed_ids: chapter_relationships.iter().map(|seed| seed.id.clone()).collect(),
                terminology_seed_ids: chapter_terms.iter().map(|seed| seed.id.clone()).collect(),
                context,
            }
        })
        .collect();

    InitializationProposal {
        mutates_canon: false,
        chapters,
        conflicts,
    }
}

fn truncate_chars(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.into();
    }
    text.chars().take(max_chars).collect()
}

fn ratio(numerator: usize, denominator: usize) -> f32 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f32 / denominator as f32
    }
}
