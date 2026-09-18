use character_engine::{CharacterBible, CharacterProfile};
use literary_intelligence_engine::{attribute_speakers, AttributionMethod};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
struct Corpus {
    schema_version: u32,
    provenance: Provenance,
    cases: Vec<Case>,
}

#[derive(Debug, Deserialize)]
struct Provenance {
    owner: String,
    license: String,
    synthetic: bool,
    redistribution_allowed: bool,
}

#[derive(Debug, Deserialize)]
struct Case {
    id: String,
    text: String,
    characters: Vec<CharacterFixture>,
    expected: Vec<ExpectedAttribution>,
}

#[derive(Debug, Deserialize)]
struct CharacterFixture {
    name: String,
    aliases: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ExpectedAttribution {
    speaker: Option<String>,
    method: AttributionMethod,
}

fn corpus_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../benchmarks/phase25/speaker-corpus-v1.json")
}

#[test]
fn phase25_project_owned_speaker_corpus_is_rights_safe_and_detects_regressions() {
    let raw = std::fs::read_to_string(corpus_path()).expect("phase25 corpus");
    let corpus: Corpus = serde_json::from_str(&raw).expect("valid phase25 corpus");

    assert_eq!(corpus.schema_version, 1);
    assert_eq!(corpus.provenance.owner, "Persian-Literary-Translation-Engine");
    assert_eq!(
        corpus.provenance.license,
        "project-owned synthetic benchmark"
    );
    assert!(corpus.provenance.synthetic);
    assert!(corpus.provenance.redistribution_allowed);
    assert!(corpus.cases.len() >= 10);

    let mut resolved = 0usize;
    let mut incorrect_resolved = 0usize;
    let mut expected_unresolved = 0usize;

    for case in corpus.cases {
        let mut bible = CharacterBible::new();
        for fixture in &case.characters {
            bible.add(CharacterProfile {
                name: fixture.name.clone(),
                voice_notes: String::new(),
                personality_notes: String::new(),
            });
            for alias in &fixture.aliases {
                bible
                    .add_alias_checked(fixture.name.clone(), alias.clone())
                    .expect("benchmark aliases must not collide");
            }
        }

        let actual = attribute_speakers(&case.id, &case.text, &bible);
        assert_eq!(
            actual.len(),
            case.expected.len(),
            "quote count changed for case {}",
            case.id
        );

        for (index, (actual, expected)) in actual.iter().zip(&case.expected).enumerate() {
            assert_eq!(
                actual.method, expected.method,
                "attribution method changed for {} quote {}",
                case.id, index
            );
            assert_eq!(
                actual.speaker.as_deref(),
                expected.speaker.as_deref(),
                "speaker changed for {} quote {}",
                case.id,
                index
            );

            match (&actual.speaker, &expected.speaker) {
                (Some(actual), Some(expected)) if actual == expected => resolved += 1,
                (Some(_), Some(_)) | (Some(_), None) => incorrect_resolved += 1,
                (None, None) => expected_unresolved += 1,
                (None, Some(_)) => {}
            }
        }
    }

    assert_eq!(
        incorrect_resolved, 0,
        "high-precision baseline must not create a wrong speaker label"
    );
    assert!(
        resolved >= 8,
        "benchmark must prove useful explicit-speaker coverage"
    );
    assert!(
        expected_unresolved >= 2,
        "benchmark must preserve fail-closed unresolved examples"
    );
}
