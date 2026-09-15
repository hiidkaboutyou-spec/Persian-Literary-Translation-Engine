# External Translation Integrations

This document records the approved external integrations for the Persian Literary Translation Engine and the boundary each dependency is allowed to cross.

## Principles

- Rust remains the product core and owns project state, literary intelligence, translation orchestration, deterministic quality gates, review state, and publishing.
- External projects are integrated only where they provide a concrete capability that is stronger than the native implementation.
- External model scores are evidence, never human approval and never permission to silently rewrite a manuscript.
- Exact upstream revisions or package versions are pinned. Upgrades require tests and a provenance review.
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

BookForge currently declares Rust 1.88 and edition 2024. CI therefore uses the stable toolchain and the dependency stays revision-pinned.

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

Install locally from the repository root:

```bash
bash tools/comet-evaluator/install.sh
```

The installer creates `.venv/comet`, prefers Python 3.11/3.10 when available, installs the pinned COMET package, and syntax-checks the bridge. It intentionally does **not** download a model checkpoint. The selected COMET/XCOMET model is downloaded only when evaluation is explicitly invoked.

The JSON protocol accepts a batch of stable item IDs, source text, candidate Persian translations, and optional references. It returns per-item scores, a system score, and XCOMET error spans when the selected model exposes them.

COMET rules:

- COMET output is advisory quality evidence, not the final literary-quality decision.
- A COMET score must never mark a chapter as human-approved.
- Model errors, unavailable checkpoints, Hugging Face authentication, or lack of GPU must not corrupt project state.
- Default credential-free tests do not download COMET models.
- Deterministic omission, terminology, structure, prompt-leakage, continuity, and Persian-specific checks remain authoritative gates where applicable.

## ContextWeaver — architecture reference, no dependency

ContextWeaver concepts were reviewed for long-form context packets, stable segment identity, revision history, and resumability. No runtime dependency is approved because the engine already implements the relevant product primitives natively:

- stable document/chapter/scene/paragraph identity;
- source and assembled-context fingerprints for resume safety;
- passage-relevant translation-memory retrieval;
- selective glossary injection under a bounded context budget;
- character and relationship context;
- versioned review/revision history and canon promotion.

Adding another context framework would duplicate ownership and create schema/persistence conflicts. Future ContextWeaver ideas can be adopted independently when a concrete gap is demonstrated.

## TranslateBooksWithLLMs — design reference only

No source code is copied or linked. Its selective glossary/context ideas overlap with `memory-engine`, which already injects only glossary entries relevant to the current source passage and bounds memory/context size. Its AGPL licensing is also inappropriate for accidental code copying into the current dependency graph without a deliberate licensing decision.

## TransAgents — research/agent-role reference only

TransAgents can inform separation of translator, editor, fidelity, voice, and naturalness roles, but it is not a runtime dependency. The engine keeps provider execution replaceable and keeps deterministic quality checks separate from model judgments.

## Upgrade policy

Before changing a pinned external revision or package version:

1. read upstream release/security notes and license changes;
2. compare public API and persistence implications;
3. run `cargo fmt`, Clippy, the full workspace test suite, release build, CLI smoke tests, and `cargo audit`;
4. run EPUB regression fixtures when BookForge changes;
5. run the COMET protocol tests and a manually authorized model smoke test when COMET changes;
6. record the new revision/version and reason in this document and the implementation status;
7. merge only after CI/security checks pass.
