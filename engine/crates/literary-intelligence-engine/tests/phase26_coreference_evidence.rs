use character_engine::{CharacterBible, CharacterProfile};
use literary_intelligence_engine::{canonical_coreference_links, CoreferenceResponse};
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
    response: CoreferenceResponse,
    expected: Vec<ExpectedLink>,
}

#[derive(Debug, Deserialize)]
struct CharacterFixture {
    name: String,
    aliases: Vec<String>,
}

#[derive(Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
struct ExpectedLink {
    mention: String,
    canonical: String,
}

fn corpus_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../benchmarks/phase26/coreference-corpus-v1.json")
}

#[test]
fn phase26_project_owned_coreference_corpus_is_rights_safe_and_fail_closed() {
    let raw = std::fs::read_to_string(corpus_path()).expect("phase26 corpus");
    let corpus: Corpus = serde_json::from_str(&raw).expect("valid phase26 corpus");

    assert_eq!(corpus.schema_version, 1);
    assert_eq!(
        corpus.provenance.owner,
        "Persian-Literary-Translation-Engine"
    );
    assert_eq!(
        corpus.provenance.license,
        "project-owned synthetic benchmark"
    );
    assert!(corpus.provenance.synthetic);
    assert!(corpus.provenance.redistribution_allowed);
    assert!(corpus.cases.len() >= 6);

    let mut mapped_cases = 0usize;
    let mut fail_closed_cases = 0usize;

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

        let links = canonical_coreference_links(&case.id, &case.text, &bible, &case.response)
            .unwrap_or_else(|error| panic!("{} failed protocol validation: {error}", case.id));
        let mut actual = links
            .iter()
            .flat_map(|link| {
                link.linked_mentions.iter().map(|mention| ExpectedLink {
                    mention: mention.text.clone(),
                    canonical: link.canonical_character.clone(),
                })
            })
            .collect::<Vec<_>>();
        actual.sort();

        let mut expected = case.expected;
        expected.sort();
        assert_eq!(
            actual, expected,
            "coreference mapping changed for {}",
            case.id
        );

        if actual.is_empty() {
            fail_closed_cases += 1;
        } else {
            mapped_cases += 1;
        }
    }

    assert!(
        mapped_cases >= 4,
        "benchmark must prove useful anchored coreference coverage"
    );
    assert!(
        fail_closed_cases >= 2,
        "benchmark must preserve ambiguous/unanchored fail-closed behavior"
    );
}
