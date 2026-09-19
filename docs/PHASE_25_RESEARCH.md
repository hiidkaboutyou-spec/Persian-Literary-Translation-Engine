# Phase 25 Research — Narrative Speaker & Coreference Intelligence

Date: 2026-09-18

## Goal

Improve English-fiction understanding before translation by adding high-precision, evidence-backed quotation speaker attribution without creating a second character/canon owner or introducing a hidden Python/model runtime.

The concrete gap after Phase 24 is narrow but important:

- `CharacterBible` reliably retrieves canonical characters by explicit names and approved aliases;
- deterministic manuscript analysis detects dialogue conventions and character occurrences;
- Context Packet v2 can carry canonical voice/relationship context;
- but the system did not explicitly map a direct quotation to its speaker, and therefore could not distinguish which canonical character voice belongs to which quote;
- pronoun/coreference and implicit turn-taking remain probabilistic and must not be guessed into canon.

Phase 25 therefore starts with a **high-precision native baseline**, a rights-safe benchmark, and an explicit future sidecar admission gate.

## Research findings

### Two-stage deterministic quotation attribution

Muzny et al., *A Two-stage Sieve Approach for Quote Attribution* (EACL 2017) models the problem as:

1. quote -> mention;
2. mention -> speaker entity.

The paper demonstrates that precision-ranked deterministic sieves can be tuned for high precision, reporting a 95.6% speaker-label precision configuration on its development setting. Useful concepts include:

- explicit quote/mention/speech-verb patterns;
- exact name/alias matching;
- preferring nearby cues;
- separating mention detection from entity resolution;
- allowing uncertain cases to remain unresolved rather than forcing a label.

Phase 25 adopts these design ideas independently; it does not copy external implementation code.

### ACL 2023: four distinct subtasks

Vishnubhotla et al. describe literary quotation attribution as four interconnected tasks:

- character identification;
- coreference resolution;
- quotation identification;
- speaker attribution.

This supports keeping the native explicit-speaker baseline separate from future coreference/model work. A speaker label obtained from an explicit name + speech verb does not justify a general pronoun coreference system.

### ModernBookNLP / 2026 joint scoring

Michel, Attali & Epure, *Fast and Accurate Quotation Attribution in Literary Texts* (2026), report a joint-scoring encoder formulation with strong PDNC performance and release `gasmichel/ModernBookNLP_QA`.

Reviewed repository facts:

- root requirements include Torch 2.6, Transformers 4.57.6, spaCy 3.8.14, torch-geometric and additional scientific dependencies;
- the modified `ModernBookNLP` fork contains the original BookNLP MIT license;
- the repository root does not provide a reviewed standalone LICENSE file;
- model checkpoints are downloaded separately from Hugging Face and their redistribution/deployment terms must be reviewed independently;
- the research package is therefore materially heavier than the Rust core and has a model-provenance boundary.

Decision: **research/benchmark candidate only**. Do not install or make it a runtime dependency in Phase 25.

### BookNLP

`booknlp/booknlp` remains useful as a research reference for quotation detection, speaker attribution, character entities and coreference. Its speaker model and coreference stack are model-backed and historically optimized for literary English.

Decision: **reference/optional future benchmark only**. The project does not need BookNLP to implement explicit name+speech-verb evidence.

### FastCoref

`shon-otmazgin/fastcoref` remains a possible optional benchmark for English pronoun/coreference resolution. Software and the previously reviewed F-Coref/LingMess model cards advertise MIT licensing.

Decision: **not installed**. It may be evaluated later only if the Phase-25 benchmark is expanded with rights-safe pronoun/coreference cases and a measurable gap justifies the model stack.

### Maverick

`SapienzaNLP/maverick-coref` is technically relevant, including LitBank-trained models, but the reviewed repository/data/software terms are CC BY-NC-SA 4.0.

Decision: **blocked from product integration**. Research citation/reference only.

### BookCoref

`SapienzaNLP/bookcoref` provides book-scale character coreference evaluation and explicitly describes full-book/split evaluation modes.

Reviewed terms: data and software are CC BY-NC-SA 4.0.

Decision: **research only**. Do not vendor, train on, or make it a product benchmark dependency.

### Renard

`CompNet/Renard` is a maintained modular character-network extraction pipeline and demonstrates useful pipeline decomposition ideas.

Reviewed implementation:

- GPL-3.0-only;
- Python;
- Torch/Transformers plus NLTK and other heavy dependencies.

Decision: **architecture reference only**. No code/dependency adoption.

### LitBank

`dbamman/litbank` contains entity, event, coreference and quotation annotations over public-domain literary texts.

Reviewed dataset license: **CC BY 4.0**.

Decision:

- acceptable as a future external/reference benchmark with attribution;
- not required by permanent CI;
- Phase 25's committed CI corpus remains project-owned synthetic data so tests are network-free, deterministic and provenance-simple.

### PDNC

The Project Dialogism Novel Corpus contains 35k+ annotated quotations and is highly relevant to quotation attribution research.

Reviewed publication terms: **CC BY-NC 4.0**.

Decision: research reference only; it is not the canonical product benchmark and is not vendored into the repository.

## Phase 25 native design

### High-precision explicit baseline

The native Rust analyzer resolves a speaker only when:

- a canonical character name or approved alias appears outside the quotation;
- a known speech verb is directly adjacent in a subject-like literary attribution pattern;
- the cue is locally near the target quotation;
- the cue belongs to the target quote's local region and conflicting explicit character cues fail closed.

Supported high-precision arrangements include:

- `"..." Mina said.`
- `Mina said, "..."`
- `"..." said Mina.`
- approved alias variants of the same patterns.

Guardrails:

- names inside the quotation are not treated as the speaker (vocatives remain content);
- before a quote, `verb + name` is not accepted because it is frequently an object, e.g. `Mina asked Reza, "Ready?"`;
- when `name + speech-verb` and `speech-verb + name` candidates share the exact same speech-verb token, the inverted candidate is treated as that verb's object/addressee; distinct local character cues remain conflicting and fail closed rather than being ranked by proximity;
- another quotation cannot sit between the explicit cue and the target quote;
- a pre-quote speech tag is not reused across a hard sentence boundary;
- conflicting local explicit character cues fail closed;
- pronoun + speech-verb cues are detected but remain unresolved;
- no gender inference is performed;
- no conversational turn-taking guess is performed;
- leading-dash dialogue may be detected as dialogue without inventing a speaker.

### Context Packet integration

Resolved speaker evidence is rendered as a bounded deterministic `SPEAKER MAP` Context Packet candidate.

Authority: **Deterministic**, not Canonical.

Why:

- the character identity comes from canonical `CharacterBible`;
- the quote -> speaker link is a deterministic inference from source cues;
- it must not be promoted into canon automatically.

Only resolved explicit evidence enters the map. Pronoun/ambiguous/unattributed dialogue is omitted.

The map is capped to a bounded number of quotes and excerpts are truncated.

## Rights-safe benchmark

Committed corpus:

`benchmarks/phase25/speaker-corpus-v1.json`

Properties:

- project-owned;
- synthetic;
- redistribution allowed;
- no external text download;
- no model download.

Challenge classes include:

- pre/post quote explicit names;
- approved aliases;
- verb-name post-quote order;
- vocative-vs-speaker separation;
- `ask` object-vs-subject disambiguation;
- pronoun fail-closed behavior;
- no-speech-cue fail-closed behavior;
- curly single quotes (while straight apostrophes remain lexical, not quote delimiters);
- guillemets;
- leading-dash detection without guessing;
- multiple quotations with quote-local cue isolation;
- competing post-quote cues from distinct speech verbs, which must remain unresolved rather than picking the nearest character;
- post-quote `Mina asked Reza` object disambiguation, where both apparent cues share the same speech-verb token and only Mina remains a speaker candidate.

The regression requires zero wrong resolved speaker labels on the committed corpus and proves both useful explicit coverage and intentional unresolved behavior.

## External model admission gate

No external quotation/coreference model is approved merely because it scores well in a paper.

A future sidecar may be considered only if all of the following are true:

1. exact source-code license is reviewed;
2. exact checkpoint/model license is reviewed separately;
3. training/evaluation data terms are compatible with the intended use;
4. the model runs behind an optional process boundary;
5. normal translation remains functional without the model;
6. model output is evidence only and never canon;
7. a rights-safe benchmark demonstrates a measurable gain over the native baseline;
8. false speaker attribution is measured separately from coverage;
9. resource use, privacy and failure behavior are bounded;
10. Linux and Apple Silicon behavior is validated.

## Explicit non-adoptions

Phase 25 does not install:

- ModernBookNLP;
- BookNLP;
- FastCoref;
- Maverick;
- BookCoref;
- Renard;
- Torch;
- Transformers;
- spaCy.

No new runtime package is required for the Phase-25 native baseline.

## Phase 25 exit criteria

Phase 25 is canonical only when:

1. the native analyzer detects supported quote styles and explicit speaker cues deterministically;
2. vocatives and `asked <object>` patterns cannot create the obvious wrong-speaker regression;
3. pronoun-only and no-cue examples remain unresolved;
4. project-owned benchmark provenance is validated;
5. committed benchmark produces zero incorrect resolved speaker labels;
6. deterministic speaker evidence reaches Context Packet v2 with non-canonical authority;
7. context output is bounded and unresolved dialogue is omitted;
8. no external model/runtime dependency is introduced;
9. Linux and Apple Silicon Phase-25 validation is green;
10. Rust CI, Security, Phases 18–24, Phase 22 Desktop, Phase 23 Trusted Release and Project Memory remain green on the final head;
11. PR merge and exact final SHA/run IDs are recorded.
