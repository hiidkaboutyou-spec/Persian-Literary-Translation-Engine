use character_engine::{build_character_context, CharacterBible, RelationshipProfile};
use memory_engine::glossary::Glossary;
use memory_engine::{
    build_context_packet_v2, hybrid_memory_candidates, stable_evidence_id, ContextAuthority,
    ContextCandidate, ContextKind, ContextPacketConfig, ContextPacketV2, MemoryContextConfig,
    SemanticRetrievalStatus, SemanticSidecar, TranslationMemory,
};

use crate::{
    deterministic_speaker_context, model_coreference_context, CoreferenceError,
    CoreferenceResponse, ManuscriptIntelligence,
};

const NEIGHBOR_EXCERPT_CHARS: usize = 900;

#[derive(Debug, Clone, Copy)]
pub struct NeighborContext<'a> {
    pub id: &'a str,
    pub title: &'a str,
    pub text: &'a str,
}

#[derive(Debug)]
pub struct ChapterContextPacketInput<'a> {
    pub document_title: &'a str,
    pub chapter_id: &'a str,
    pub chapter_title: &'a str,
    pub source_text: &'a str,
    pub previous: Option<NeighborContext<'a>>,
    pub next: Option<NeighborContext<'a>>,
    pub characters: &'a CharacterBible,
    pub glossary: &'a Glossary,
    pub translation_memory: &'a TranslationMemory,
    pub intelligence: &'a ManuscriptIntelligence,
    pub reviewed_literary_lines: &'a [String],
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChapterContextPacketBuild {
    pub packet: ContextPacketV2,
    pub semantic: SemanticRetrievalStatus,
}

/// Shared context assembly for CLI and ApplicationService. The function only
/// consumes canon/evidence owned by existing engines and emits a bounded packet;
/// it does not promote inferred data or mutate any source of truth.
fn build_chapter_context_packet_internal(
    input: ChapterContextPacketInput<'_>,
    config: &ContextPacketConfig,
    semantic_sidecar: Option<&SemanticSidecar>,
    coreference_candidate: Option<ContextCandidate>,
) -> ChapterContextPacketBuild {
    let mut candidates = Vec::new();

    let metadata = format!(
        "document_title={}\nchapter_title={}",
        input.document_title, input.chapter_title
    );
    candidates.push(ContextCandidate::new(
        stable_evidence_id("chapter-metadata", &[input.chapter_id, &metadata]),
        ContextKind::Reference,
        ContextAuthority::Deterministic,
        metadata,
        "document/chapter identity for the current translation unit",
        1.0,
    ));

    for profile in input.characters.relevant_to_text(input.source_text) {
        let text = build_character_context(profile);
        candidates.push(ContextCandidate::new(
            stable_evidence_id("character-canon", &[&profile.name, &text]),
            ContextKind::Character,
            ContextAuthority::Canonical,
            text,
            "canonical character is present in the current source unit",
            1.0,
        ));
    }

    if let Some(text) =
        deterministic_speaker_context(input.chapter_id, input.source_text, input.characters)
    {
        candidates.push(
            ContextCandidate::new(
                stable_evidence_id("speaker-map", &[input.chapter_id, &text]),
                ContextKind::Character,
                ContextAuthority::Deterministic,
                text,
                "high-precision explicit quote-speaker evidence; unresolved dialogue is omitted",
                0.94,
            )
            .with_evidence_ids([input.chapter_id.to_string()]),
        );
    }

    if let Some(candidate) = coreference_candidate {
        candidates.push(candidate);
    }

    for relationship in input.characters.relevant_relationships(input.source_text) {
        let text = relationship_context(relationship);
        candidates.push(ContextCandidate::new(
            stable_evidence_id(
                "relationship-canon",
                &[&relationship.character_a, &relationship.character_b, &text],
            ),
            ContextKind::Relationship,
            ContextAuthority::Canonical,
            text,
            "both endpoints of a canonical relationship are present in the current unit",
            1.0,
        ));
    }

    let memory_candidates = hybrid_memory_candidates(
        input.source_text,
        input.translation_memory,
        input.glossary,
        &MemoryContextConfig::default(),
        semantic_sidecar,
    );
    let semantic = memory_candidates.semantic;
    candidates.extend(memory_candidates.candidates);
    candidates.extend(manuscript_context_candidates(
        input.intelligence,
        input.chapter_id,
    ));

    if let Some(previous) = input.previous {
        let excerpt = tail_chars(previous.text, NEIGHBOR_EXCERPT_CHARS);
        if !excerpt.trim().is_empty() {
            let text = format!(
                "PREVIOUS CHAPTER CONTINUITY — source evidence from {}:\n{}",
                previous.title, excerpt
            );
            candidates.push(
                ContextCandidate::new(
                    stable_evidence_id("previous-chapter", &[previous.id, &text]),
                    ContextKind::LocalContinuity,
                    ContextAuthority::Deterministic,
                    text,
                    "bounded tail of the immediately preceding source chapter",
                    0.82,
                )
                .with_evidence_ids([previous.id.to_string()]),
            );
        }
    }

    if let Some(next) = input.next {
        let excerpt = head_chars(next.text, NEIGHBOR_EXCERPT_CHARS);
        if !excerpt.trim().is_empty() {
            let text = format!(
                "NEXT CHAPTER CONTINUITY — source evidence from {}:\n{}",
                next.title, excerpt
            );
            candidates.push(
                ContextCandidate::new(
                    stable_evidence_id("next-chapter", &[next.id, &text]),
                    ContextKind::LocalContinuity,
                    ContextAuthority::Deterministic,
                    text,
                    "bounded head of the immediately following source chapter",
                    0.74,
                )
                .with_evidence_ids([next.id.to_string()]),
            );
        }
    }

    for (index, line) in input.reviewed_literary_lines.iter().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let text = format!(
            "REVIEWED LITERARY FINDING — human-approved context:\n{}",
            line.trim()
        );
        candidates.push(
            ContextCandidate::new(
                stable_evidence_id(
                    "reviewed-literary",
                    &[input.chapter_id, &index.to_string(), line],
                ),
                ContextKind::TranslationDecision,
                ContextAuthority::HumanApproved,
                text,
                "approved or edited literary-review finding scoped to this chapter",
                1.0,
            )
            .with_evidence_ids([input.chapter_id.to_string()]),
        );
    }

    ChapterContextPacketBuild {
        packet: build_context_packet_v2(input.chapter_id, input.source_text, &candidates, config),
        semantic,
    }
}

/// Shared context assembly with the already-approved optional semantic retrieval sidecar.
/// Coreference evidence is absent unless callers opt into the dedicated Phase-26 API.
pub fn build_chapter_context_packet_with_semantic(
    input: ChapterContextPacketInput<'_>,
    config: &ContextPacketConfig,
    semantic_sidecar: Option<&SemanticSidecar>,
) -> ChapterContextPacketBuild {
    build_chapter_context_packet_internal(input, config, semantic_sidecar, None)
}

/// Opt-in Phase-26 context assembly. External/model coreference remains Inferred evidence:
/// it is accepted only after strict protocol validation and only when a cluster has one
/// unambiguous canonical name/approved-alias anchor from CharacterBible.
pub fn build_chapter_context_packet_with_semantic_and_coreference(
    input: ChapterContextPacketInput<'_>,
    config: &ContextPacketConfig,
    semantic_sidecar: Option<&SemanticSidecar>,
    coreference_response: &CoreferenceResponse,
) -> Result<ChapterContextPacketBuild, CoreferenceError> {
    let coreference_candidate =
        model_coreference_context(input.source_text, input.characters, coreference_response)?
            .map(|text| {
                ContextCandidate::new(
                    stable_evidence_id(
                        "coreference-map",
                        &[input.chapter_id, coreference_response.model.as_str(), &text],
                    ),
                    ContextKind::Character,
                    ContextAuthority::Inferred,
                    text,
                    "optional coreference-model evidence anchored to exactly one canonical character; never canon",
                    0.76,
                )
                .with_evidence_ids([input.chapter_id.to_string()])
            });

    Ok(build_chapter_context_packet_internal(
        input,
        config,
        semantic_sidecar,
        coreference_candidate,
    ))
}

/// Deterministic compatibility wrapper. Existing callers keep exactly the old
/// behavior unless they explicitly provide a semantic sidecar.
pub fn build_chapter_context_packet(
    input: ChapterContextPacketInput<'_>,
    config: &ContextPacketConfig,
) -> ContextPacketV2 {
    build_chapter_context_packet_with_semantic(input, config, None).packet
}

/// Convert deterministic manuscript intelligence into typed Phase 18 context
/// candidates without changing canon ownership. Observed evidence is
/// deterministic; inferred literary traits remain explicitly inferred.
pub fn manuscript_context_candidates(
    intelligence: &ManuscriptIntelligence,
    chapter_id: &str,
) -> Vec<ContextCandidate> {
    let mut candidates = Vec::new();

    let observed = &intelligence.literary_profile.observed;
    let observed_text = format!(
        "BOOK PROFILE — observed evidence only:\nchapters={}\naverage_scenes_per_chapter={:.2}\ndialogue_density={:.3}\nprose_density={:.3}\ncode_switching_detected={}{}",
        observed.chapter_count,
        observed.average_scenes_per_chapter,
        observed.dialogue_density,
        observed.prose_density,
        observed.code_switching_detected,
        list_line("dialogue_conventions", &observed.dialogue_conventions),
    );
    candidates.push(
        ContextCandidate::new(
            stable_evidence_id(
                "book-observed",
                &[&intelligence.manuscript_id, &observed_text],
            ),
            ContextKind::BookSummary,
            ContextAuthority::Deterministic,
            observed_text,
            "whole-manuscript observed literary profile",
            0.72,
        )
        .with_evidence_ids([intelligence.manuscript_id.clone()]),
    );

    let inferred = &intelligence.literary_profile.inferred;
    let mut inferred_lines = Vec::new();
    push_optional(
        &mut inferred_lines,
        "narrative_pov",
        inferred.narrative_pov.as_deref(),
    );
    push_optional(
        &mut inferred_lines,
        "narrative_tense",
        inferred.narrative_tense.as_deref(),
    );
    push_optional(
        &mut inferred_lines,
        "narrator_register",
        inferred.narrator_register.as_deref(),
    );
    push_optional(
        &mut inferred_lines,
        "dialogue_register",
        inferred.dialogue_register.as_deref(),
    );
    push_list(
        &mut inferred_lines,
        "recurring_imagery",
        &inferred.recurring_imagery,
    );
    push_list(
        &mut inferred_lines,
        "recurring_motifs",
        &inferred.recurring_motifs,
    );
    push_list(
        &mut inferred_lines,
        "humor_signals",
        &inferred.humor_signals,
    );
    push_list(
        &mut inferred_lines,
        "sarcasm_signals",
        &inferred.sarcasm_signals,
    );
    if !inferred_lines.is_empty() {
        let inferred_text = format!(
            "BOOK PROFILE — inferred evidence only:\n{}",
            inferred_lines.join("\n")
        );
        candidates.push(
            ContextCandidate::new(
                stable_evidence_id(
                    "book-inferred",
                    &[&intelligence.manuscript_id, &inferred_text],
                ),
                ContextKind::BookSummary,
                ContextAuthority::Inferred,
                inferred_text,
                "whole-manuscript inferred literary profile",
                0.58,
            )
            .with_evidence_ids([intelligence.manuscript_id.clone()]),
        );
    }

    if let Some(chapter) = intelligence
        .chapter_maps
        .iter()
        .find(|chapter| chapter.chapter_id == chapter_id)
    {
        let mut chapter_lines = vec![
            format!("title={}", chapter.title),
            format!("dialogue_density={:.3}", chapter.dialogue_density),
            format!("narrative_density={:.3}", chapter.narrative_density),
        ];
        push_list(
            &mut chapter_lines,
            "important_named_entities",
            &chapter.important_named_entities,
        );
        push_list(
            &mut chapter_lines,
            "recurring_terms",
            &chapter.recurring_terms,
        );
        push_list(
            &mut chapter_lines,
            "continuity_hooks",
            &chapter.continuity_hooks,
        );
        push_list(
            &mut chapter_lines,
            "unresolved_references",
            &chapter.unresolved_references,
        );
        let chapter_text = format!(
            "CHAPTER MAP — observed continuity evidence:\n{}",
            chapter_lines.join("\n")
        );
        let evidence_ids = std::iter::once(chapter.chapter_id.clone())
            .chain(chapter.scene_ids.iter().cloned())
            .collect::<Vec<_>>();
        candidates.push(
            ContextCandidate::new(
                stable_evidence_id("chapter-map", &[&chapter.chapter_id, &chapter_text]),
                ContextKind::ChapterSummary,
                ContextAuthority::Deterministic,
                chapter_text,
                "current chapter structural and continuity evidence",
                0.88,
            )
            .with_evidence_ids(evidence_ids),
        );
    }

    if let Some(seed_context) = intelligence
        .initialization
        .context_for_chapter(chapter_id)
        .filter(|value| !value.trim().is_empty())
    {
        candidates.push(
            ContextCandidate::new(
                stable_evidence_id("chapter-seeds", &[chapter_id, seed_context]),
                ContextKind::ChapterSummary,
                ContextAuthority::Inferred,
                seed_context,
                "deterministic manuscript-analysis proposal; not approved canon",
                0.64,
            )
            .with_evidence_ids([chapter_id.to_string()]),
        );
    }

    candidates
}

fn relationship_context(relationship: &RelationshipProfile) -> String {
    let mut parts = vec![format!(
        "Relationship: {} ↔ {}",
        relationship.character_a, relationship.character_b
    )];
    if !relationship.dynamic_notes.trim().is_empty() {
        parts.push(format!("Dynamic: {}", relationship.dynamic_notes.trim()));
    }
    if !relationship.address_notes.trim().is_empty() {
        parts.push(format!(
            "Forms of address: {}",
            relationship.address_notes.trim()
        ));
    }
    if !relationship.boundaries_notes.trim().is_empty() {
        parts.push(format!(
            "Continuity constraints: {}",
            relationship.boundaries_notes.trim()
        ));
    }
    parts.join("\n")
}

fn head_chars(text: &str, max_chars: usize) -> String {
    text.chars().take(max_chars).collect()
}

fn tail_chars(text: &str, max_chars: usize) -> String {
    let count = text.chars().count();
    text.chars().skip(count.saturating_sub(max_chars)).collect()
}

fn push_optional(lines: &mut Vec<String>, label: &str, value: Option<&str>) {
    if let Some(value) = value.filter(|value| !value.trim().is_empty()) {
        lines.push(format!("{label}={}", value.trim()));
    }
}

fn push_list(lines: &mut Vec<String>, label: &str, values: &[String]) {
    if !values.is_empty() {
        lines.push(format!("{label}={}", values.join(" | ")));
    }
}

fn list_line(label: &str, values: &[String]) -> String {
    if values.is_empty() {
        String::new()
    } else {
        format!("\n{label}={}", values.join(" | "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AnalysisCanon, CoreferenceCluster, CoreferenceMention, CoreferenceResponse,
        DeterministicManuscriptAnalyzer, ManuscriptAnalyzer, COREFERENCE_PROTOCOL_VERSION,
    };
    use character_engine::{CharacterProfile, RelationshipProfile};
    use document_engine::{Book, Chapter, DocumentFormat, Manuscript, SourceLocation};
    use memory_engine::glossary::{Glossary, GlossaryEntry};
    use memory_engine::MemoryEntry;
    use std::collections::BTreeMap;

    fn test_manuscript(content: &str) -> Manuscript {
        Manuscript {
            book: Book {
                id: "book-test".into(),
                title: "Test".into(),
                author: None,
                language: Some("en".into()),
                metadata: BTreeMap::new(),
            },
            chapters: vec![Chapter::translated(0, "Chapter 1", content)],
            source: SourceLocation::new("test.txt", DocumentFormat::Txt),
        }
    }

    #[test]
    fn candidates_keep_observed_and_inferred_authority_separate() {
        let document = test_manuscript("Mina whispered. The door remained open.");
        let characters = CharacterBible::new();
        let glossary = Glossary::default();
        let intelligence = DeterministicManuscriptAnalyzer::default()
            .analyze(
                &document,
                AnalysisCanon {
                    characters: &characters,
                    glossary: &glossary,
                },
            )
            .unwrap();
        let chapter_id = intelligence.chapter_maps[0].chapter_id.clone();
        let candidates = manuscript_context_candidates(&intelligence, &chapter_id);

        assert!(candidates.iter().any(|candidate| {
            candidate.kind == ContextKind::BookSummary
                && candidate.authority == ContextAuthority::Deterministic
        }));
        assert!(candidates.iter().any(|candidate| {
            candidate.kind == ContextKind::ChapterSummary
                && candidate.authority == ContextAuthority::Deterministic
        }));
    }

    #[test]
    fn explicit_speaker_map_reaches_context_without_resolving_pronouns() {
        let document = test_manuscript("\"Stay,\" Mina said. \"No,\" she replied.");
        let mut characters = CharacterBible::new();
        characters.add(CharacterProfile {
            name: "Mina".into(),
            voice_notes: "quiet and precise".into(),
            personality_notes: "guarded".into(),
        });
        let glossary = Glossary::default();
        let translation_memory = TranslationMemory::new();
        let intelligence = DeterministicManuscriptAnalyzer::default()
            .analyze(
                &document,
                AnalysisCanon {
                    characters: &characters,
                    glossary: &glossary,
                },
            )
            .unwrap();
        let chapter_id = intelligence.chapter_maps[0].chapter_id.clone();
        let packet = build_chapter_context_packet(
            ChapterContextPacketInput {
                document_title: "Test",
                chapter_id: &chapter_id,
                chapter_title: "Chapter 1",
                source_text: "\"Stay,\" Mina said. \"No,\" she replied.",
                previous: None,
                next: None,
                characters: &characters,
                glossary: &glossary,
                translation_memory: &translation_memory,
                intelligence: &intelligence,
                reviewed_literary_lines: &[],
            },
            &ContextPacketConfig::default(),
        );

        assert!(packet
            .items
            .iter()
            .any(|item| item.text.contains("SPEAKER MAP")));
        assert!(packet.rendered_context.contains("Mina"));
        assert!(!packet.rendered_context.contains("she →"));
    }

    #[test]
    fn opted_in_coreference_evidence_is_inferred_and_canon_anchored() {
        let source = "Mina closed the door. She sighed.";
        let document = test_manuscript(source);
        let mut characters = CharacterBible::new();
        characters.add(CharacterProfile {
            name: "Mina".into(),
            voice_notes: "quiet and precise".into(),
            personality_notes: "guarded".into(),
        });
        let glossary = Glossary::default();
        let translation_memory = TranslationMemory::new();
        let intelligence = DeterministicManuscriptAnalyzer::default()
            .analyze(
                &document,
                AnalysisCanon {
                    characters: &characters,
                    glossary: &glossary,
                },
            )
            .unwrap();
        let chapter_id = intelligence.chapter_maps[0].chapter_id.clone();
        let response = CoreferenceResponse {
            schema_version: COREFERENCE_PROTOCOL_VERSION,
            model: "synthetic-phase26".into(),
            clusters: vec![CoreferenceCluster {
                id: "c1".into(),
                mentions: vec![
                    CoreferenceMention {
                        id: "m1".into(),
                        start_char: 0,
                        end_char: 4,
                        text: "Mina".into(),
                    },
                    CoreferenceMention {
                        id: "m2".into(),
                        start_char: 22,
                        end_char: 25,
                        text: "She".into(),
                    },
                ],
            }],
        };

        let build = build_chapter_context_packet_with_semantic_and_coreference(
            ChapterContextPacketInput {
                document_title: "Test",
                chapter_id: &chapter_id,
                chapter_title: "Chapter 1",
                source_text: source,
                previous: None,
                next: None,
                characters: &characters,
                glossary: &glossary,
                translation_memory: &translation_memory,
                intelligence: &intelligence,
                reviewed_literary_lines: &[],
            },
            &ContextPacketConfig::default(),
            None,
            &response,
        )
        .unwrap();

        let item = build
            .packet
            .items
            .iter()
            .find(|item| item.text.contains("COREFERENCE MAP"))
            .expect("coreference evidence should be present");
        assert_eq!(item.authority, ContextAuthority::Inferred);
        assert!(item.text.contains("\"She\" → Mina"));
        assert!(build.packet.rendered_context.contains("never canon"));
    }

    #[test]
    fn shared_packet_prioritizes_human_and_canonical_context_and_neighbors() {
        let document = test_manuscript("Mina met Reza near the Portal.");
        let mut characters = CharacterBible::new();
        characters.add(CharacterProfile {
            name: "Mina".into(),
            voice_notes: "quiet and precise".into(),
            personality_notes: "guarded".into(),
        });
        characters.add(CharacterProfile {
            name: "Reza".into(),
            voice_notes: "warm".into(),
            personality_notes: "patient".into(),
        });
        let mut relationship = RelationshipProfile::new("Mina", "Reza");
        relationship.address_notes = "informal in private".into();
        characters.add_relationship(relationship);
        let mut glossary = Glossary::default();
        glossary.add(GlossaryEntry {
            source_term: "Portal".into(),
            preferred_translation: "پرتال".into(),
            context: "fantasy term".into(),
        });
        let mut translation_memory = TranslationMemory::new();
        translation_memory.add(MemoryEntry::new(
            "Mina met Reza".into(),
            "مینا رضا را دید".into(),
            "prior scene".into(),
        ));
        let intelligence = DeterministicManuscriptAnalyzer::default()
            .analyze(
                &document,
                AnalysisCanon {
                    characters: &characters,
                    glossary: &glossary,
                },
            )
            .unwrap();
        let chapter_id = intelligence.chapter_maps[0].chapter_id.clone();
        let reviewed = vec!["Preserve their private informal register.".to_string()];
        let packet = build_chapter_context_packet(
            ChapterContextPacketInput {
                document_title: "Test",
                chapter_id: &chapter_id,
                chapter_title: "Chapter 1",
                source_text: "Mina met Reza near the Portal.",
                previous: Some(NeighborContext {
                    id: "prev",
                    title: "Previous",
                    text: "They had argued the previous night.",
                }),
                next: None,
                characters: &characters,
                glossary: &glossary,
                translation_memory: &translation_memory,
                intelligence: &intelligence,
                reviewed_literary_lines: &reviewed,
            },
            &ContextPacketConfig::default(),
        );

        assert!(packet
            .items
            .iter()
            .any(|item| item.authority == ContextAuthority::HumanApproved));
        assert!(packet
            .items
            .iter()
            .any(|item| item.kind == ContextKind::Character));
        assert!(packet
            .items
            .iter()
            .any(|item| item.kind == ContextKind::Relationship));
        assert!(packet
            .items
            .iter()
            .any(|item| item.kind == ContextKind::LocalContinuity));
        assert!(packet.rendered_context.contains("پرتال"));
    }
}
