use character_engine::{CharacterBible, CharacterProfile, RelationshipProfile};
use document_engine::{ingest_file, Book, DocumentFormat, Manuscript, SourceLocation};
use literary_intelligence_engine::{
    AnalysisCanon, AnalysisConfig, Confidence, ConfidenceLevel, DeterministicManuscriptAnalyzer,
    ManuscriptAnalyzer, SeedStatus, TerminologyCategory,
};
use memory_engine::glossary::{Glossary, GlossaryEntry};
use std::collections::BTreeMap;
use std::fs;

fn fiction_fixture() -> tempfile::NamedTempFile {
    let file = tempfile::Builder::new().suffix(".txt").tempfile().unwrap();
    fs::write(
        file.path(),
        "Chapter 1\nElizabeth Bennet carried the silver key to Netherfield.\n\n\"Darcy, wait,\" Elizabeth said.\n\n***\n\nDarcy watched Elizabeth Bennet hide the silver key.\n\nChapter 2\nLizzy found the silver key beside Darcy.\n\n— Miss Bennet, Darcy said.\n\nChapter 3\nDarcy returned the silver key to Lizzy.\n\nLizzy and Darcy crossed the courtyard.\n\nکتاب مقدس کنار در بود. کتاب مقدس گم شد. کتاب مقدس پیدا شد.",
    )
    .unwrap();
    file
}

fn canon() -> (CharacterBible, Glossary) {
    let mut bible = CharacterBible::new();
    bible.add(CharacterProfile {
        name: "Elizabeth Bennet".into(),
        voice_notes: "approved restrained wit".into(),
        personality_notes: "approved canon".into(),
    });
    bible.add_alias("Elizabeth Bennet", "Elizabeth");
    bible.add_alias("Elizabeth Bennet", "Lizzy");
    bible.add(CharacterProfile {
        name: "Darcy".into(),
        voice_notes: "approved formal reserve".into(),
        personality_notes: "approved canon".into(),
    });
    let mut relationship = RelationshipProfile::new("Elizabeth Bennet", "Darcy");
    relationship.dynamic_notes = "approved guarded respect".into();
    bible.add_relationship(relationship);

    let mut glossary = Glossary::default();
    glossary.add(GlossaryEntry {
        source_term: "silver key".into(),
        preferred_translation: "کلید نقره‌ای".into(),
        context: "approved object name".into(),
    });
    glossary.add(GlossaryEntry {
        source_term: "كتاب مقدس".into(),
        preferred_translation: "کتاب مقدس".into(),
        context: "approved terminology".into(),
    });
    (bible, glossary)
}

#[test]
fn multi_chapter_analysis_seeds_context_with_provenance_without_promoting_canon() {
    let file = fiction_fixture();
    let manuscript = ingest_file(file.path()).unwrap();
    let (bible, glossary) = canon();
    let result = DeterministicManuscriptAnalyzer::default()
        .analyze(
            &manuscript,
            AnalysisCanon {
                characters: &bible,
                glossary: &glossary,
            },
        )
        .unwrap();

    assert_eq!(result.chapter_maps.len(), 3);
    assert!(!result.initialization.mutates_canon);
    let lizzy = result
        .character_seeds
        .iter()
        .find(|seed| seed.canonical_name_candidate == "Elizabeth Bennet")
        .unwrap();
    assert!(lizzy.aliases.contains(&"Lizzy".to_string()));
    assert!(lizzy.matches_approved_character);
    assert_eq!(lizzy.status, SeedStatus::Inferred);
    assert!(lizzy.appearance_count >= 5);
    assert!(lizzy.evidence.iter().all(|evidence| {
        !evidence.chapter_id.is_empty()
            && !evidence.scene_id.is_empty()
            && !evidence.paragraph_id.is_empty()
            && evidence.source.chapter.is_some()
    }));

    let relationship = result
        .relationship_seeds
        .iter()
        .find(|seed| {
            [seed.character_a.as_str(), seed.character_b.as_str()].contains(&"Elizabeth Bennet")
                && [seed.character_a.as_str(), seed.character_b.as_str()].contains(&"Darcy")
        })
        .unwrap();
    assert!(relationship.matches_approved_relationship);
    assert!(relationship.interaction_count >= 3);
    assert!(
        text_normalization::normalize_case_insensitive(&relationship.character_a)
            <= text_normalization::normalize_case_insensitive(&relationship.character_b)
    );

    let silver_key = result
        .terminology_seeds
        .iter()
        .find(|seed| seed.source_expression == "silver key")
        .unwrap();
    assert_eq!(
        silver_key.category,
        TerminologyCategory::RecurringExpression
    );
    assert_eq!(
        silver_key.approved_translation.as_deref(),
        Some("کلید نقره‌ای")
    );

    let persian_term = result
        .terminology_seeds
        .iter()
        .find(|seed| {
            text_normalization::normalize(&seed.source_expression)
                == text_normalization::normalize("کتاب مقدس")
        })
        .unwrap();
    assert_eq!(
        persian_term.approved_translation.as_deref(),
        Some("کتاب مقدس")
    );
    assert!(result.literary_profile.observed.dialogue_density > 0.0);
    assert_eq!(result.literary_profile.inferred.narrative_pov, None);
}

#[test]
fn repeated_analysis_is_byte_stable_and_evidence_is_bounded() {
    let file = fiction_fixture();
    let manuscript = ingest_file(file.path()).unwrap();
    let (bible, glossary) = canon();
    let analyzer = DeterministicManuscriptAnalyzer::new(AnalysisConfig {
        max_evidence_per_seed: 2,
        ..AnalysisConfig::default()
    });

    let analyze = || {
        analyzer
            .analyze(
                &manuscript,
                AnalysisCanon {
                    characters: &bible,
                    glossary: &glossary,
                },
            )
            .unwrap()
    };
    let first = analyze();
    let second = analyze();
    assert_eq!(first, second);
    assert_eq!(
        serde_json::to_vec(&first).unwrap(),
        serde_json::to_vec(&second).unwrap()
    );
    assert!(first
        .character_seeds
        .iter()
        .all(|seed| seed.evidence.len() <= 2));
    assert!(first
        .terminology_seeds
        .iter()
        .all(|seed| seed.evidence.len() <= 2));
}

#[test]
fn approved_character_ambiguity_is_surfaced_and_excluded_from_context() {
    let file = tempfile::Builder::new().suffix(".txt").tempfile().unwrap();
    fs::write(
        file.path(),
        "Chapter 1\nLizzy Bennet met Darcy.\n\nChapter 2\nLizzy Bennet answered Darcy.",
    )
    .unwrap();
    let manuscript = ingest_file(file.path()).unwrap();
    let (bible, glossary) = canon();
    let result = DeterministicManuscriptAnalyzer::default()
        .analyze(
            &manuscript,
            AnalysisCanon {
                characters: &bible,
                glossary: &glossary,
            },
        )
        .unwrap();

    let conflict = result
        .initialization
        .conflicts
        .iter()
        .find(|conflict| conflict.inferred_value == "Lizzy Bennet")
        .unwrap();
    assert_eq!(conflict.approved_value, "Elizabeth Bennet");
    assert!(result.initialization.chapters.iter().all(|chapter| !chapter
        .context
        .contains("character candidate: Lizzy Bennet")));
}

#[test]
fn confidence_validation_rejects_unsupported_certainty_and_invalid_config() {
    assert!(Confidence {
        level: ConfidenceLevel::High,
        supporting_evidence: 0,
    }
    .validate()
    .is_err());

    let file = fiction_fixture();
    let manuscript = ingest_file(file.path()).unwrap();
    let (bible, glossary) = canon();
    let error = DeterministicManuscriptAnalyzer::new(AnalysisConfig {
        max_evidence_per_seed: 0,
        ..AnalysisConfig::default()
    })
    .analyze(
        &manuscript,
        AnalysisCanon {
            characters: &bible,
            glossary: &glossary,
        },
    )
    .unwrap_err();
    assert!(error.to_string().contains("max_evidence_per_seed"));
}

#[test]
fn synthetic_120_chapter_analysis_stays_bounded_and_deduplicated() {
    let file = tempfile::Builder::new().suffix(".txt").tempfile().unwrap();
    let text = (1..=120)
        .map(|index| {
            format!(
                "Chapter {index}\nMina met Reza beside the blue lantern.\n\n\"Reza, the blue lantern is here,\" Mina said.\n"
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(file.path(), text).unwrap();
    let manuscript = ingest_file(file.path()).unwrap();
    let result = DeterministicManuscriptAnalyzer::default()
        .analyze(
            &manuscript,
            AnalysisCanon {
                characters: &CharacterBible::new(),
                glossary: &Glossary::default(),
            },
        )
        .unwrap();

    assert_eq!(result.chapter_maps.len(), 120);
    assert!(result.character_seeds.len() < 10);
    assert!(result.terminology_seeds.len() <= 256);
    assert!(result
        .character_seeds
        .iter()
        .all(|seed| seed.evidence.len() <= 8));
    let ids = result
        .character_seeds
        .iter()
        .map(|seed| &seed.id)
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(ids.len(), result.character_seeds.len());
}

#[test]
fn empty_structured_manuscript_produces_an_empty_valid_analysis() {
    let source = SourceLocation::new("synthetic-empty.txt", DocumentFormat::Txt);
    let manuscript = Manuscript {
        book: Book {
            id: "empty-book".into(),
            title: "Empty".into(),
            author: None,
            language: Some("en".into()),
            metadata: BTreeMap::new(),
        },
        chapters: Vec::new(),
        source,
    };
    let result = DeterministicManuscriptAnalyzer::default()
        .analyze(
            &manuscript,
            AnalysisCanon {
                characters: &CharacterBible::new(),
                glossary: &Glossary::default(),
            },
        )
        .unwrap();

    assert!(result.character_seeds.is_empty());
    assert!(result.relationship_seeds.is_empty());
    assert!(result.chapter_maps.is_empty());
    assert_eq!(result.literary_profile.observed.dialogue_density, 0.0);
    assert!(result.validate().is_ok());
}
