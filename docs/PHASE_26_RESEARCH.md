# Phase 26 Research — Long-Span Literary Coreference Evidence

Date: 2026-09-19

## Goal

Improve English-fiction understanding for translation by adding a strict, optional coreference-evidence boundary for pronouns and nominal mentions that Phase 25 deliberately leaves unresolved.

Phase 26 does **not** select a production coreference model. It first establishes:

1. a rights-safe project-owned benchmark;
2. a provider-neutral process protocol;
3. strict source-offset/text validation;
4. canonical-character anchoring that cannot invent new canon;
5. bounded Context Packet integration with Inferred authority;
6. an explicit admission gate for any future model sidecar.

## Why this is the next gap

Phase 25 can attribute direct quotations when explicit name/alias + speech-verb evidence is present, but it intentionally leaves pronoun-only and implicit references unresolved.

That means passages such as:

- `Mina closed the door. She sighed.`
- `Mina entered the clinic. The doctor removed her coat.`

still need identity evidence if translation is to reliably attach the right voice/register/relationship context.

The key risk is false merging: a wrong coreference cluster can silently assign one character's pronoun, title, role, or dialogue to another. Phase 26 therefore optimizes for **fail-closed canonical anchoring**, not maximum coverage.

## External research review

### LitBank

Upstream: `dbamman/litbank`.

Relevant facts:

- 100 works of English-language fiction;
- entity, event, entity-coreference and quotation-attribution annotations;
- coreference includes personal pronouns;
- quotation speaker labels link to the same entity/coreference identities;
- dataset license: **CC BY 4.0**.

Decision:

- eligible for future attributed external/reference evaluation;
- not required by permanent CI;
- committed Phase-26 regression data remains project-owned synthetic so CI is network-free and provenance-simple.

### BOOKCOREF

Paper: ACL 2025, *BOOKCOREF: Coreference Resolution at Book Scale*.

Relevant finding: existing coreference systems degrade materially when evaluated over book-scale documents rather than shorter contexts. BOOKCOREF reports average document lengths above 200k tokens and highlights the long-document gap.

Repository/data terms reviewed: **CC BY-NC-SA 4.0**.

Decision: research/reference only. Do not vendor, train on, or make it a product benchmark/runtime dependency.

### GOLEMcoref (ACL 2026)

Upstream: `GOLEM-lab/GOLEMcoref`.

Research value:

- current gold-standard fiction coreference benchmark spanning seven languages;
- 827k annotated tokens across complete fictional stories, with English included;
- human-curated character coreference annotations in CoNLL-2012/CorefUD formats;
- useful evidence that fiction-specific annotation and cross-lingual training remain active research directions in 2026.

Reviewed repository/data terms: **CC BY-NC 4.0**.

Decision: non-commercial research/reference only. Do not vendor it into product CI or use its released model as a product dependency. It may be used only in a separate attributed research evaluation if its terms remain compatible with that evaluation.

### xCoRe

Upstream: `SapienzaNLP/xcore`; EMNLP 2025.

Research value:

- explicitly designed for short, long and cross-document coreference;
- splits long documents into contexts, resolves within-context clusters, then merges clusters across contexts;
- publishes a LitBank model and supports long-document inference.

Reviewed repository terms: **CC BY-NC-SA 4.0** for data/software.

Decision: technically highly relevant, but non-commercial terms make it research-only for this project. Do not install as a product/runtime dependency.

### FastCoref

Upstream: `shon-otmazgin/fastcoref`.

Research value:

- MIT-licensed software;
- practical Python API;
- F-Coref/LingMess model family;
- lower integration friction than large book-specific pipelines.

Risks:

- Python/model runtime and model-weight provenance remain separate review surfaces;
- ordinary FastCoref usage is not itself proof of book-scale literary performance;
- any checkpoint used in product work needs its own exact license/security/resource review.

Decision: future benchmark candidate only. Do not install in Phase 26.

### CorPipe 2026

Upstream: `ufal/crac2026-corpipe`; CRAC 2026 winning multilingual system.

Reviewed software license: **MPL-2.0**. The released pretrained CorPipe 26 model weights are separately published under **CC BY-NC-SA 4.0**, so code licensing must not be mistaken for checkpoint/product-use permission.

Research value: current multilingual coreference engineering and evaluation reference.

Limitation: not specifically a long-fiction/book-scale system and does not remove the need for literary/book-scale evaluation.

Decision: reference/benchmark candidate only; no runtime adoption in Phase 26. Exact model-weight terms remain an independent admission gate.

### NovelCR

ACL Findings 2025 introduces a large bilingual long-span novel coreference benchmark with many links spanning three or more sentences.

The reviewed public repository exposes the benchmark and statistics but an explicit redistribution/license decision was not established from the repository surface reviewed for this phase.

Decision: research reference only until dataset-level licensing/provenance is explicitly verified. Do not vendor it into CI.

## Native Phase-26 architecture

### Protocol

`CoreferenceRequest`:

- schema version;
- source unit ID;
- deterministic SHA-256-backed source fingerprint derived from unit ID + exact source text;
- exact source text.

`CoreferenceResponse`:

- schema version;
- exact echoed source fingerprint binding the response to the requested unit/text;
- bounded safe model identifier;
- clusters;
- mentions with globally unique IDs;
- character-offset spans;
- exact mention text.

The Rust boundary rejects:

- unsupported schema versions;
- empty model IDs;
- too many clusters, per-cluster mentions, or total mentions;
- empty/duplicate/unsafe cluster IDs;
- empty/duplicate/unsafe mention IDs;
- unsafe/unbounded model identifiers that could inject control text into Context Packet rendering;
- stale/mismatched source fingerprints;
- invalid/out-of-range offsets;
- duplicate spans across clusters;
- mention text that does not exactly equal the source slice at its declared offsets.

A malformed sidecar response is an error, never a silent pass.

The process boundary is also resource-bounded **before** JSON parsing:

- stdout is capped at 8 MiB by default;
- stderr is capped at 256 KiB by default;
- total mentions are capped at 32,768 by default;
- stdin/stdout/stderr are handled concurrently so a sidecar cannot deadlock the host by filling an OS pipe before reading input;
- timeout kills the child and joins the I/O workers;
- oversized output fails explicitly rather than being read unbounded into memory.

Mapped clusters/mentions are canonicalized into source-position order before Context Packet rendering so model response ordering cannot nondeterministically change which bounded evidence lines are emitted first.

### Canonical anchoring rule

An external/model cluster may produce Context Packet evidence only when:

1. at least one mention exactly matches an existing canonical character name or approved alias;
2. every explicit canonical/alias mention in that cluster resolves to the **same** canonical character.

If a cluster contains:

- no canonical anchor; or
- anchors for more than one canonical character;

the cluster is omitted entirely.

This prevents a model from creating characters or choosing between conflicting canonical identities.

### Context authority

Validated mapped coreference is still model inference.

Therefore:

- `ContextKind::Character`;
- `ContextAuthority::Inferred`;
- below Phase-25 deterministic explicit-speaker evidence;
- below canonical CharacterBible evidence;
- never HumanApproved/Canonical.

The rendered map explicitly says it is optional model evidence and never canon.

### Failure isolation

The existing deterministic/semantic context APIs remain unchanged.

Phase 26 adds a separate opt-in API:

`build_chapter_context_packet_with_semantic_and_coreference(...)`

If no coreference evidence is supplied, all existing callers preserve their previous behavior.

A future executable sidecar uses a bounded subprocess with timeout. Missing/failing optional coreference tooling must not make normal translation unusable.

## Rights-safe benchmark

Committed corpus:

`benchmarks/phase26/coreference-corpus-v1.json`

Properties:

- project-owned;
- synthetic;
- redistribution allowed;
- no external text/model download;
- exact character-offset spans.

Challenge classes:

- canonical-name anchor -> pronoun;
- approved-alias anchor -> pronoun;
- canonical anchor -> nominal mention + pronoun;
- longer multi-sentence link;
- conflicting canonical anchors -> fail closed;
- unanchored cluster -> fail closed.

Protocol unit tests separately cover malformed offsets/text and bounded sidecar timeout.

## Model admission gate

No model is approved merely because its paper reports strong F1.

A future coreference model may enter an optional product sidecar only when all are true:

1. exact source-code license reviewed;
2. exact model/checkpoint license reviewed separately;
3. training/evaluation dataset rights reviewed;
4. product use is compatible with those terms;
5. model is optional and process-isolated;
6. normal translation still works without it;
7. project-owned benchmark shows measurable gain;
8. false merges / false canonical links are reported separately from recall/coverage;
9. resource usage and timeout behavior are bounded;
10. source text is not silently transmitted to an unapproved remote service;
11. Linux and Apple Silicon behavior is validated;
12. model output remains evidence and cannot mutate CharacterBible/canon automatically.

## Explicit non-adoptions

Phase 26 does not install:

- xCoRe;
- Maverick;
- BookCoref;
- FastCoref;
- CorPipe;
- BookNLP/ModernBookNLP;
- Torch;
- Transformers;
- spaCy.

No new model/runtime package is required by the Phase-26 native protocol.

## Exit criteria

Phase 26 becomes canonical only when:

1. strict request/response protocol validation passes;
2. exact source identity fingerprint plus source offsets/text are enforced;
3. duplicate/invented spans/IDs and unsafe protocol identifiers fail closed;
4. subprocess stdout/stderr and aggregate mention counts are bounded before/while parsing;
5. canonical anchoring cannot create or choose conflicting character canon;
6. project-owned benchmark proves both useful mappings and intentional fail-closed cases;
7. mapped evidence order is deterministic from source offsets rather than model-return order;
8. Context Packet integration is opt-in and authority is Inferred;
9. existing non-coreference context APIs preserve prior behavior;
10. no external model/runtime dependency is introduced;
11. bounded timeout, oversized-output and failure behavior are tested;
12. Linux and Apple Silicon Phase-26 gates are green;
13. Rust CI, Security, Phases 18–25, Desktop, Trusted Release and Project Memory remain green;
14. final merge SHA/run IDs are recorded.


## Final timeout hardening

A final full-workspace validation exposed a scheduler-sensitive edge case in the optional coreference sidecar timeout. The original timer began after process spawn and pipe-thread setup, and checked child completion before the deadline. On a heavily descheduled CI host, a child could finish well after the configured deadline before the parent was scheduled again, and the host could accept that late completion.

The Phase-26 contract is now explicitly fail-closed:

- timeout accounting starts before process creation;
- child startup, pipe setup, host scheduling delay, and execution all consume the same configured budget;
- once the host observes that the deadline has elapsed, it terminates/reaps the child and returns `CoreferenceError::Timeout` before accepting a late completion;
- the normal translation path still falls back cleanly because model-backed coreference remains optional evidence.

This is a runtime correctness fix, not a relaxation of the timeout test.
