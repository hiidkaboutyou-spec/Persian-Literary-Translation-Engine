# Phase 24 Research — Literary Precision & Persian Polish

Date: 2026-09-18

## Goal

Improve the last-mile quality and safety of English-to-Persian literary translation without replacing the Rust-first architecture, duplicating already-canonical context/glossary systems, or turning optional research models into hidden runtime requirements.

Phase 24 focuses on three concrete gaps that remained after Phases 18–23:

1. deterministic Persian typography/orthography evidence that is safe for literary text;
2. bounded, human-authorized application of review-proposed paragraph revisions;
3. an implementation-independent EPUB reopen check so the same EPUB library is not both writer and sole structural reader.

## Research survey

### brothersincode/virastar

Upstream: https://github.com/brothersincode/virastar

Reviewed implementation: MIT, JavaScript, current source banner v0.22.1 (2025-04-13).

Useful ideas include Persian/Arabic character normalization, ZWNJ cleanup, prefix/suffix spacing diagnostics, punctuation spacing, ellipsis/quote handling, tatweel cleanup, and preserving URLs/markup.

Decision: **design reference, not a runtime dependency**. Small deterministic rules are safer to own natively in Rust. Literary punctuation and expressive marks are not safe to normalize automatically, so Phase 24 splits low-risk Unicode cleanup from advisory diagnostics.

### rezkam/persian

Upstream: https://github.com/rezkam/persian

License: MIT.

Useful ideas: Arabic-to-Persian character/digit maps and narrow spacing normalization.

Decision: **design/reference only**. Do not add Python to the runtime for a few deterministic mappings.

### DevinSterling/rbook

Upstream: https://github.com/DevinSterling/rbook

Selected exact crate: `rbook = 0.7.10`

License: Apache-2.0. The selected crate forbids unsafe code in its crate-level lint configuration and exposes an independent EPUB 2/3 parser/reader.

Decision: **approved only as a document-engine dev/CI dependency**. It independently reopens Persian EPUB output produced through the canonical BookForge boundary. It must not replace BookForge, enter application/runtime persistence, or become required to translate/export in normal product execution.

The Phase 24 Cargo-lock bootstrap resolved exactly one newly locked package, `rbook 0.7.10`; its required crates were already present in the workspace graph. The committed lockfile is subsequently required with `--locked`.

### booknlp/booknlp

Upstream: https://github.com/booknlp/booknlp

Source license: MIT. Published package metadata reviewed at version 1.0.7.

Potential value for fiction:
- character-name clustering;
- pronoun/coreference evidence;
- quote detection and speaker attribution;
- character/event annotations.

Risks:
- heavy Python model stack (Torch, TensorFlow, spaCy, Transformers);
- original repository has very low commit count and older environment guidance;
- source-code license does not by itself resolve all model/training-data provenance questions;
- book-scale coreference/speaker attribution is probabilistic and can merge or misattribute characters.

Decision: **not installed**. Candidate only for a future rights-safe, optional sidecar benchmark against the project's existing literary-intelligence stack. Its output could only be evidence and could never create canon automatically.

### shon-otmazgin/fastcoref

Upstream: https://github.com/shon-otmazgin/fastcoref

Software license: MIT. The reviewed F-Coref and LingMess model cards advertise MIT licenses.

Potential value: English pronoun/coreference evidence with a more actively maintained implementation than BookNLP.

Limitation: it does not itself solve fiction quotation speaker attribution, and it still adds a Torch/Transformers model stack.

Decision: **not installed**. Keep as an optional benchmark competitor if character/coreference tests demonstrate a measurable gap.

### YutongWang1216/DocMTAgent (DelTA)

Upstream: https://github.com/YutongWang1216/DocMTAgent

License: Apache-2.0.

Useful architecture: proper-noun records, bilingual summary, long-term retrieval and short-term exemplar memory.

Decision: **architecture reference only**. Phase 18 already owns selective long-novel retrieval and explicit context authority. Phase 24 must not add a parallel memory system.

### YutongWang1216/LoongDocMT

Upstream: https://github.com/YutongWang1216/LoongDocMT

Research paper: Loong: A Human-Like Long Document Translation Agent with Observe-and-Act Adaptive Context Selection (2026).

Useful idea: Essence–Exemplar–Entity context selection rather than sending all history.

Repository review at Phase 24 found only a minimal README/one-commit public repository and no reviewed LICENSE file.

Decision: **research only; no code or dependency adoption**. The conceptual context categories may inform future benchmarks, but cannot justify integrating unlicensed or incomplete code.

### UrgenProchnoff/prozetta

Upstream: https://github.com/UrgenProchnoff/prozetta

License: MIT.

Useful ideas: whole-book extraction before translation, a human-reviewable book “passport”, centralized decisions that should not drift per chunk, glossary review, and explicit pipeline-state visualization.

Decision: **design reference only**. The project already has deterministic intelligence, canon, project state and a Tauri product surface; adding its Node application would duplicate owners.

### yihong0618/bilingual_book_maker

Upstream: https://github.com/yihong0618/bilingual_book_maker

License: MIT.

Useful ideas: relevant-only glossary injection, append-only session context, bounded handoff summaries and explicit protection against compaction “death loops”.

Decision: **reference only**. The project already implements relevant-only glossary lookup (`Glossary::relevant_to_text`) and Context Packet v2. Do not duplicate these mechanisms merely because another repository independently validates the same design.

### Literary refinement research

ACL 2026: “What Does LLM Refinement Actually Improve? A Systematic Study on Document-Level Literary Translation” (ACL 2026 long paper 268; arXiv:2605.13368).

The reported robust recipe is document-aware translation followed by segment-level refinement; repeated document-level refinement was less consistently useful, while gains were concentrated more in fluency/style/terminology than adequacy.

Phase 24 therefore does **not** add a free-form “rewrite the chapter” agent. Instead it closes the existing review loop around already-bounded `RevisionProposal` objects:

- the reviewer proposes;
- the proposal is non-mutating by default;
- only an explicit human acceptance action can apply it;
- only one unambiguous target paragraph may be patched;
- the stored review must still be fresh;
- the proposed paragraph is rechecked with deterministic structural/Persian evidence before mutation;
- the existing manual revision ledger records the previous text;
- mutation marks quality/review evidence stale, requiring a new review pass.

## Implemented Phase 24 boundaries

### Native Persian typography evidence

`text-normalization` now has a separate publication-quality surface. Existing `normalize()` comparison semantics remain untouched.

Low-risk `polish_persian_unicode()` can normalize:
- Arabic yeh/kaf variants to Persian characters;
- Arabic-Indic digits to Persian digits;
- decorative tatweel;
- duplicate/structurally invalid ZWNJ.

It deliberately does not rewrite literary punctuation, expressive repeated marks, ordinary affix spacing or prose style.

`inspect_persian_typography()` reports review evidence for:
- Arabic letter/digit variants;
- tatweel;
- duplicate/misplaced ZWNJ;
- separated `می/نمی` prefixes;
- repeated spaces;
- spaces before punctuation.

The literary-review engine records this as a narrow `PersianNaturalness` evidence channel while explicitly stating that fluency, idiom, voice, rhythm and style remain unevaluated by the native typography checker.

### Bounded human-accepted review patches

A fresh literary-review finding may expose concrete `suggested_text`. The new acceptance boundary rejects:
- stale reviews;
- missing/unknown findings;
- findings without concrete suggested text;
- multi-target/ambiguous findings;
- mismatched paragraph structure;
- proposals that fail critical native structural verification.

No proposal is auto-applied. Accepted changes use the normal manual-revision path and therefore preserve previous text/history and stale the old review.

### Independent EPUB differential validation

`rbook 0.7.10` is pinned as a document-engine **dev dependency only**. A generated rights-safe EPUB fixture is ingested/exported by the canonical BookForge path, then reopened independently with strict rbook parsing. The regression verifies readable spine content plus Persian RTL/language output.

This supplements EPUBCheck and BookForge; it does not create a new publication owner.

## Explicit non-adoptions

- Hazm remains blocked by the previously recorded NLTK security-advisory path until a fresh audit proves it safe.
- DadmaTools remains conditional on a demonstrated Persian NLP gap.
- BookNLP/FastCoref are not product dependencies.
- No Node/Python Persian text-cleaner runtime is added.
- No second memory/vector database is added.
- No free-form whole-chapter refinement agent is added.
- No automated finding becomes canon or human approval.
- No model suggestion is applied without a user/human action.

## Phase 24 exit criteria

Phase 24 is canonical only after:

1. the exact committed engine lockfile includes `rbook 0.7.10` and all Phase-24 Cargo commands validate with `--locked`;
2. native Persian typography tests pass;
3. application-level literary review records Persian typography evidence without falsely claiming full naturalness;
4. explicit review-proposal acceptance is tested as non-mutating until human action, single-paragraph bounded, revision-audited and staleness-producing;
5. strict independent rbook reopen succeeds for a rights-safe Persian RTL EPUB fixture;
6. rbook remains dev-only and BookForge remains the runtime EPUB owner;
7. desktop commands/UI preserve the human acceptance gate and local-content security invariants;
8. Rust CI, Security, Phase 18–23, Phase 22 Desktop, and Project Memory gates remain green on the final PR head;
9. the final PR is reviewed/merged and exact completion SHAs/run IDs are recorded.

## Canonical completion record

Phase 24 became canonical through PR #104.

- final reviewed head: `e8a0420a57aa4a6663fda8d32bd4336069237421`;
- merge commit: `2fa48dfd39437f79e6ac9db949f54e6595f2cc0a`;
- Phase 24 run `35354157358`: success;
- Rust CI `35354157248`: success;
- Security `35354157239`: success;
- Phase 18 `35354157288`, Phase 19 `35354157340`, Phase 20 `35354157189`, Phase 21 `35354157322`, Phase 22 `35354157247`, Phase 23 `35354157250`, and Project Memory Tooling `35354157261`: success.

During final validation, three independent lock snapshots had to be synchronized because the new internal `literary-review-engine -> text-normalization` path dependency is observed by the engine workspace, the isolated desktop workspace, and the isolated literary-alignment tool. The final engine lockfile was restored from the exact GitHub Actions-resolved artifact before merge rather than approximated manually.
