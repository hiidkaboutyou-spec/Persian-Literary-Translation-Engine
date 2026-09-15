use memory_engine::{stable_evidence_id, ContextAuthority, ContextCandidate, ContextKind};

use crate::ManuscriptIntelligence;

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
    push_optional(&mut inferred_lines, "narrative_pov", inferred.narrative_pov.as_deref());
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
    push_list(&mut inferred_lines, "recurring_imagery", &inferred.recurring_imagery);
    push_list(&mut inferred_lines, "recurring_motifs", &inferred.recurring_motifs);
    push_list(&mut inferred_lines, "humor_signals", &inferred.humor_signals);
    push_list(&mut inferred_lines, "sarcasm_signals", &inferred.sarcasm_signals);
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
        push_list(&mut chapter_lines, "recurring_terms", &chapter.recurring_terms);
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
                stable_evidence_id(
                    "chapter-map",
                    &[&chapter.chapter_id, &chapter_text],
                ),
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
    use crate::{AnalysisCanon, DeterministicManuscriptAnalyzer, ManuscriptAnalyzer};
    use character_engine::CharacterBible;
    use document_engine::{Book, Chapter, ParsedDocument};
    use memory_engine::glossary::Glossary;

    #[test]
    fn candidates_keep_observed_and_inferred_authority_separate() {
        let document = ParsedDocument {
            book: Book {
                title: "Test".into(),
                author: None,
            },
            chapters: vec![Chapter::source(
                0,
                "Chapter 1".into(),
                "Mina whispered. The door remained open.".into(),
            )],
            ..ParsedDocument::default()
        };
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
}
