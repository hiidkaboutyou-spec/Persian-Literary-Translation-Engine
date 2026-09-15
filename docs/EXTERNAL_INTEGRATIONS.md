# External Translation Integrations

This document records approved external integrations for the Persian Literary Translation Engine, the boundary each dependency is allowed to cross, and researched candidates that are intentionally deferred.

## Principles

- Rust remains the product core and owns project state, literary intelligence, translation orchestration, deterministic quality gates, review state, and publishing.
- External projects are integrated only where they provide a concrete capability that is stronger than the native implementation.
- External model scores and probabilistic diagnostics are evidence, never human approval and never permission to silently rewrite a manuscript.
- Exact upstream revisions or package versions are pinned. Upgrades require tests and a provenance review.
- Heavy models and optional tooling are never downloaded by default CI unless a dedicated reproducible test explicitly requires them.
- Manuscript text, translations, reviewer notes, credentials, and project memory must not be logged or sent anywhere except to a provider explicitly selected for that operation.

## BookForge — active EPUB ingestion dependency

Upstream: `JunjoSick/bookforge`

Approved revision:

```text
23f8c9d3c97a06f48e13424698441bfb4b037844
```

License: MIT.

`document-engine` enables the `bookforge-epub` feature by default and pins `bookforge-core` plus `bookforge-epub` to that exact revision. BookForge is used only at the document boundary. Its IR is mapped into the engine's existing `ParsedDocument` / `Manuscript` types; BookForge types must not leak into project persistence, translation memory, literary intelligence, review ledgers, or public application contracts.

Why it is integrated:

- structured OPF/spine/XHTML parsing instead of relying only on the previous hand-rolled XML scanning path;
- archive validation and bounded archive reads;
- stable EPUB block/section structure;
- explicit handling of protected spans and non-translatable page furniture in the upstream IR;
- a stronger foundation for future deterministic EPUB rebuild/validation.

Synthetic BookForge metadata/nav/NCX sections are excluded from translation input. `PageFurniture` and `Code` blocks are also excluded. Other visible literary blocks are mapped into the engine's current paragraph/heading model.

The previous built-in reader remains available when `document-engine` is compiled without the `bookforge-epub` feature. It is a compatibility path, not a silent fallback: when the BookForge feature is enabled and BookForge rejects an EPUB, ingestion returns an actionable error instead of quietly switching parsers.

BookForge archive/decompression preflight failures that describe a malformed ZIP remain strict BookForge failures but are normalized to the public `DocumentError::CorruptedFile` contract. Well-formed archives with invalid EPUB structure remain `InvalidStructure`.

## COMET / XCOMET / DocCOMET — optional quality evidence sidecar

Upstream: `Unbabel/COMET`

Pinned Python package:

```text
unbabel-comet==2.2.7
```

Package license: Apache-2.0. Individual model checkpoints can have separate licenses and access requirements; those must be accepted by the user before use.

COMET is deliberately not linked into the Rust process. The adapter is split into:

- `quality-engine::comet::CometSidecar` — typed Rust JSON/process boundary;
- `tools/comet-evaluator/evaluate.py` — isolated Python evaluator;
- `tools/comet-evaluator/requirements.txt` — pinned Python dependency;
- `tools/comet-evaluator/install.sh` — local virtual-environment installer.

The installer creates `.venv/comet` and intentionally does **not** download a model checkpoint. The selected COMET/XCOMET model is downloaded only when evaluation is explicitly invoked.

COMET rules:

- COMET output is advisory quality evidence, not the final literary-quality decision.
- A COMET score must never mark a chapter as human-approved.
- Model errors, unavailable checkpoints, authentication, or lack of GPU must not corrupt project state.
- Default credential-free tests do not download COMET models.
- Deterministic omission, terminology, structure, prompt-leakage, continuity, and Persian-specific checks remain authoritative gates where applicable.

## Lingua — optional English/Persian language diagnostics

Upstream: `pemistahl/lingua-rs`

Pinned crate:

```text
lingua = 1.8.0
```

License: Apache-2.0.

Integration rules:

- dependency is optional behind the `quality-engine` feature `language-diagnostics`;
- default features are disabled so the project does **not** pull all 75 language models;
- only the `english` and `persian` model features are enabled;
- the default project build remains unchanged when the feature is not requested;
- `diagnose_persian_output` returns dominant-language/confidence/span evidence without changing the existing deterministic `evaluate_translation` result;
- mixed-language span detection is treated as advisory because upstream documents it as experimental;
- quoted English, names, titles, code, and intentionally multilingual prose can be legitimate, so the diagnostic can never independently reject, rewrite, approve, or canonize text.

This integration exists to make obvious provider failures such as untranslated English output easier to detect without turning a probabilistic classifier into a literary judge.

## EPUBCheck — optional publication conformance validator

Upstream: `w3c/epubcheck`

Pinned production release:

```text
EPUBCheck 5.3.0
SHA-256 6c07e68584b2e2ce2f89fe06e1246dfead3eb36b46b340e7d93524f29dcff6c5
```

License: BSD-3-Clause.

EPUBCheck is the W3C/DAISY EPUB conformance checker. It is intentionally **not vendored** into the repository and is not a dependency of the translation runtime.

Repository tooling:

- `tools/epubcheck-validator/install.sh` downloads only the pinned official release over HTTPS, verifies the published SHA-256 digest, then installs it under ignored local `.tools/` storage by default;
- `tools/epubcheck-validator/validate.sh` invokes the installed `epubcheck.jar` through Java;
- absence of Java or the local EPUBCheck installation does not break ingestion, translation, DOCX export, or any existing runtime operation;
- default CI syntax-checks the wrapper scripts but does not download the 33 MB distribution.

Phase 20 should make EPUBCheck part of the publication-readiness path for generated EPUB files. Structural conformance is a publication gate; it is not a translation-quality score.

## ContextWeaver — architecture reference, no dependency

ContextWeaver concepts were reviewed for long-form context packets, stable segment identity, revision history, and resumability. No runtime dependency is approved because the engine already implements the relevant product primitives natively. Future concepts can be adopted independently when a concrete gap is demonstrated.

## TranslateBooksWithLLMs — design reference only

No source code is copied or linked. Its selective glossary/context ideas overlap with `memory-engine`. Its AGPL licensing also makes accidental code copying inappropriate without a deliberate licensing decision.

## TransAgents — research/agent-role reference only

TransAgents can inform separation of translator, editor, fidelity, voice, and naturalness roles, but it is not a runtime dependency. The engine keeps provider execution replaceable and deterministic quality checks separate from model judgments.

## Researched candidates intentionally deferred

### FlagEmbedding / BGE-M3 — Phase 18 candidate

License: MIT.

Potential value: multilingual semantic retrieval plus reranking for long-novel memory when lexical similarity is insufficient.

Decision: **do not install or download models yet**. BGE-M3 is a substantial model dependency and must be introduced only after Phase 18 defines a stable context-packet/retrieval contract, measurable retrieval benchmarks, resource limits, provenance, and deterministic fallback behavior. It should augment native memory candidate selection, not own project memory or canon.

### Hazm — Phase 19 candidate

License: MIT.

Potential value: Persian tokenization, normalization, POS/linguistic diagnostics and other Persian NLP signals.

Decision: **defer**. Existing Rust `text-normalization` already owns core normalization. Hazm should be added only for concrete Persian-linguistic diagnostics missing from native code, in an isolated Python environment, with no automatic rewriting of literary prose and no implicit pretrained-model download in CI.

### DadmaTools — conditional Phase 19 candidate

License: Apache-2.0.

Potential value: broader Persian NER/syntax/ezafe/linguistic tooling.

Decision: **do not install by default**. It overlaps heavily with Hazm and would enlarge the Python/ML surface. Add it only if a Phase 19 benchmark demonstrates a specific capability gap that Hazm/native code cannot satisfy.

### Vecalign — Phase 19/21 candidate

Core code license: Apache-2.0. Bundled development/test datasets can carry different licenses (including GPL-licensed material) and must not be copied into the project without explicit review.

Potential value: source/translation sentence alignment for omission/addition evidence.

Decision: **defer** until an alignment benchmark and rights-safe fixtures exist. A native alignment layer over Phase 18 embeddings may ultimately be cleaner.

### SacreBLEU / chrF++ — Phase 21 candidate

License: Apache-2.0.

Potential value: reproducible reference-based BLEU/chrF/TER benchmarking.

Decision: **defer** until the project owns or can lawfully use a suitable EN→FA literary reference corpus. These metrics are benchmark evidence only and cannot judge literary naturalness, voice, or subtext by themselves.

## Other architecture references

- `sukamenev/booktrans` — useful whole-book scout, selective context, editor/verifier patterns; use ideas selectively rather than replacing native orchestration.
- Tolmach / `KazKozDev/book-translator` — useful patch-refinement/verifier concepts; AGPL means reference-only unless a deliberate licensing decision changes that boundary.
- `madpin/epublate` — useful EPUB round-trip/glossary lifecycle ideas; do not copy code until licensing/provenance is independently verified.
- ArmenianLitTranslator — useful critic-role/evaluation dimensions; research reference, not a runtime dependency.

## Upgrade policy

Before changing a pinned external revision or package version:

1. read upstream release/security notes and license changes;
2. compare public API, runtime cost, privacy, and persistence implications;
3. keep heavyweight models/downloads optional unless a numbered phase explicitly makes them required;
4. run `cargo fmt`, Clippy, the full workspace test suite, release build, CLI smoke tests, and `cargo audit`;
5. run the relevant optional-feature tests (`quality-engine --features language-diagnostics`, BookForge compatibility path, sidecar protocol tests, wrapper syntax checks);
6. run EPUB regression fixtures when BookForge changes;
7. run a manually authorized model smoke test when a model-backed sidecar changes;
8. record the new revision/version and reason in this document, the roadmap/status, and project-memory bootstrap when durable;
9. merge only after CI/security checks pass.
