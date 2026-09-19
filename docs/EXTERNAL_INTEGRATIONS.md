# External Translation Integrations

This document records approved external integrations for the Persian Literary Translation Engine, the boundary each dependency is allowed to cross, and researched candidates that are intentionally deferred.

## Principles

- Rust remains the product core and owns project state, literary intelligence, translation orchestration, deterministic quality gates, review state, and publishing.
- External projects are integrated only where they provide a concrete capability stronger or safer than duplicating the capability natively.
- External model scores and probabilistic diagnostics are evidence, never human approval and never permission to silently rewrite a manuscript.
- Exact upstream revisions or package versions are pinned where practical. Upgrades require tests and provenance/security review.
- Heavy models and optional tooling are not downloaded by normal CI unless a dedicated reproducible validation explicitly requires them.
- Manuscript text, translations, reviewer notes, credentials, and project memory are not logged or sent anywhere except to a provider explicitly selected for that operation.

## BookForge — active EPUB ingestion and Phase 20 reconstruction dependency

Upstream: `JunjoSick/bookforge`

Approved revision:

```text
23f8c9d3c97a06f48e13424698441bfb4b037844
```

License: MIT.

`document-engine` enables the `bookforge-epub` feature by default and maps BookForge IR back into native document types. BookForge types must not leak into project persistence, translation memory, literary intelligence, review ledgers, or public application contracts.

The previous built-in EPUB reader remains an explicit compatibility build path, not a silent fallback. When BookForge is enabled and rejects an EPUB, ingestion returns an actionable error.

Phase 20 also uses BookForge for source-aware reconstruction. The product persists only native stable source provenance (`block_id`) and supplies an explicit block-ID-to-translation map at export. BookForge owns inline-marker-aware XHTML reconstruction, target primary `dc:language`, and XHTML `lang`/`xml:lang` rewriting. The native document layer adds only the RTL publication metadata not supplied by that boundary (`dir="rtl"` and OPF spine `page-progression-direction="rtl"`).

Publication export must fail closed when expected source block provenance is missing, unknown, duplicated, empty, or mismatched. Do not regenerate EPUB from flattened translated chapter text and do not guess block alignment by paragraph order.

## COMET / XCOMET / DocCOMET — optional quality evidence sidecar

Upstream: `Unbabel/COMET`

Pinned package:

```text
unbabel-comet==2.2.7
```

License: Apache-2.0 for the package; individual checkpoints may have separate terms.

COMET remains behind the isolated `quality-engine::comet` process boundary. The installer does not download a model checkpoint. COMET output is advisory evidence only and cannot mark text human-approved.

## Lingua — optional English/Persian language diagnostics

Upstream: `pemistahl/lingua-rs`
Pinned crate: `1.8.0`
License: Apache-2.0.

Only English/Persian model features are enabled and the feature is off by default. Lingua cannot rewrite, approve, or independently reject multilingual literary output.

## EPUBCheck — Phase 20 publication conformance validator

Upstream: `w3c/epubcheck`

Authoritative Phase 20 package:

```text
EPUBCheck 5.3.0
SHA-256 6c07e68584b2e2ce2f89fe06e1246dfead3eb36b46b340e7d93524f29dcff6c5
```

License: BSD-3-Clause.

Phase 20 targets the W3C EPUB 3.3 Recommendation. EPUBCheck 5.3.0 explicitly validates EPUB 3.3 and is therefore the publication-conformance gate for this phase. The distribution is not vendored; the checksum-pinned installer places it only under ignored local tool storage. Java/EPUBCheck must not become a requirement for ingestion, translation, literary review, or DOCX export.

EPUBCheck 5.4.0 is newer, but its EPUB 3 validation tracks EPUB 3.4. EPUB 3.4 is still a Candidate Recommendation at the Phase 20 research date (2026-09-17). Treat 5.4.x as future-compatibility evidence; do not silently migrate the product standard until EPUB 3.4 is stable or a deliberate migration is approved and tested.

## FastEmbed + BGE-M3 — optional semantic evidence boundary

FastEmbed crate selected by the project:

```text
fastembed = 6.1.0
```

BGE upstream: `FlagOpen/FlagEmbedding`.

BGE-M3 supports multilingual/cross-lingual embeddings and long inputs. The project uses it only through optional Rust tooling for Phase 18 retrieval and Phase 19 alignment evidence.

Rules:

- model weights are not vendored;
- normal compilation/default CI must not download weights;
- deterministic native retrieval/review remains available without BGE;
- BGE output cannot create canon IDs, own project memory, approve text, or silently mutate output;
- model-backed process boundaries must be timeout/failure-safe;
- Linux and Apple Silicon arm64 compatibility are tested.

## Phase 19 Native Literary Alignment — canonical

Phase 19 implements monotonic alignment natively in Rust and exposes optional embedding-backed execution through `tools/literary-alignment`.

Supported alignment shapes include 1:1, 1:N, N:1, N:M and source/target gaps. The Rust core validates returned unit identity, index bounds, complete ordered coverage, finite values, and schema before evidence is accepted.

A missing/failing/malformed aligner does not block translation and does not become a clean review result. Review artifacts record whether evidence was requested, unavailable, completed, or failed.

Phase 19 is canonical via PR #95, merge `d073dab10c0965197745a6cbc7b8e56c946835e8`.

## Hazm — blocked by unpatched dependency advisory

Upstream: `roshan-research/hazm`
Current evaluated version: `0.12.1`
License: MIT.
Python requirement: `>=3.12,<3.14`.
Mandatory dependency includes `nltk ^3.9.0`.

Potential value: Persian normalization/tokenization/POS/syntax diagnostics.

Decision: **blocked**, not merely deferred. The mandatory compatible NLTK dependency was affected by the recorded High-severity `GHSA-8mgp-746c-j5xp` / `CVE-2026-81726` at the Phase 19 audit point. Do not add an audit waiver merely to enable Hazm. Re-evaluate only after a patched compatible NLTK release exists and a fresh dependency/security audit passes.

## Vecalign — algorithm/design reference only

Upstream: `thompsonb/vecalign`
Core license: Apache-2.0.

Vecalign is a strong reference for multilingual monotonic sentence alignment, including one-to-many/many-to-one behavior and document-scale alignment. It is not installed because its Python/Cython/compiler surface is unnecessary while the native Rust aligner satisfies the measured requirement; bundled Bleualign dev/test datasets also have separate GPL licensing and must not be copied blindly.

## SentWeave 0.3.3 — audited research reference only

Upstream: `amajdalawi/sentweave`
Release commit: `61e9af7086a4339448ab4b2c25b8a2071961d123`
License: Apache-2.0.
PyPI sdist SHA-256:

```text
ef6414bdd1d7fa4064f31fdf1b446c7f2601955777fcdf64988fc09bca9d2940
```

A one-off research workflow validated hash-pinned installation, dependency audit, algorithm smoke, Linux, and Apple Silicon. Decision: reference only; native Rust covers the need with less operational surface.

## DadmaTools — conditional research candidate

License: Apache-2.0.
Potential value: Persian NER/POS/dependency/ezafe diagnostics.
Decision: defer unless a benchmark demonstrates a specific capability gap after the native Phase 19 stack.

## ContextWeaver — architecture reference, no dependency

ContextWeaver concepts were reviewed for long-form context packets, stable segment identity, revision history, and resumability. Native Context Packet v2 owns those responsibilities.

## TranslateBooksWithLLMs — design reference only

No source code is copied or linked. Selective glossary/context ideas overlap native memory architecture and its licensing does not justify importing code.

## TransAgents — research/agent-role reference only

TransAgents can inform translator/editor/fidelity/voice/naturalness role separation, but its orchestration and memory architecture are not runtime dependencies. Provider judgments remain separate from deterministic quality checks and human review.

## SacreBLEU / chrF++ — Phase 21 optional benchmark evidence

Upstream: `mjpost/sacrebleu`.

Pinned package:

```text
sacrebleu==2.6.0
```

Release date: 12 January 2026.
Upstream tag/release commit reviewed: `2277caccfc7b956671a6a09f1646f62250034157`.
License: Apache-2.0.
Python requirement: >=3.9.

Phase 21 exposes SacreBLEU only through `tools/sacrebleu-evaluator`. The boundary computes chrF2++ over project-supplied hypothesis/reference strings, returns JSON evidence, and does not download SacreBLEU/WMT test sets. Normal build/translation/review/publication does not require Python or SacreBLEU.

Decision: **approved as optional reference-overlap evidence, never as the literary judge or an acceptance threshold**. The committed Phase 21 reference corpus is synthetic/project-owned; third-party corpora remain independently licensed/provenanced assets and are not implicitly approved by this tool decision.

## Tauri 2 — Phase 22 desktop product boundary

Selected reviewed versions:

- `tauri = 2.11.5`
- `tauri-build = 2.6.3`
- `tauri-plugin-dialog = 2.7.2`
- distribution CI uses Tauri CLI `2.11.4`

License: Tauri and official dialog plugin are MIT OR Apache-2.0.

Ownership boundary:

- Tauri owns native window/WebView integration, bounded IPC transport, native dialogs and application bundling.
- `ApplicationService` remains the only product/domain orchestration facade.
- Tauri types are not persisted in project schemas.
- the desktop crate remains outside the `engine/` workspace so Tauri is not a core runtime/build dependency.
- the frontend loads local bundled content only and receives no general filesystem capability.
- provider secrets are not persisted by the integration.
- signing/notarization and future updater credentials are external release secrets.

Decision: **approved for Phase 22 product surface only**. Do not move literary logic into Tauri commands or frontend code.

Research record: `docs/PHASE_22_RESEARCH.md`.

## cargo-cyclonedx 0.5.9 — Phase 23 release SBOM tooling

Upstream: `CycloneDX/cyclonedx-rust-cargo`.
Selected tool/version: `cargo-cyclonedx 0.5.9`.
License: Apache-2.0.

Approved boundary:

- CI/tagged-release tooling only;
- installed with an exact version and `--locked`;
- reads Cargo metadata plus the committed dependency graph to create CycloneDX JSON;
- uses `SOURCE_DATE_EPOCH` from the repository commit timestamp for reproducible SBOM metadata;
- target-specific release SBOMs are published beside CLI binaries;
- deprecated slash-form license expressions present in historical transitive crate metadata are preserved as named-license warnings; Phase 23 intentionally does not rewrite third-party metadata or use `--license-strict` as an SBOM availability gate;
- the tool is never a translation, review, desktop-runtime, or publishing dependency.

A generated SBOM is component inventory evidence; it is not a vulnerability-free guarantee.

Research: `docs/PHASE_23_RESEARCH.md`.

## GitHub Artifact Attestations — Phase 23 tagged-release provenance

Integration: GitHub Actions `actions/attest@v4`.

Approved boundary:

- tagged public release artifacts only;
- `actions/attest` is pinned to reviewed commit `1e69f48acb82d1966a394da916b4c1698aa569d6` rather than a movable major tag;
- publish job receives narrowly scoped `id-token: write`, `attestations: write`, and `artifact-metadata: write`;
- provenance claims link release bytes to repository/workflow/commit/build context;
- per-binary SBOM attestations link each CLI binary to its CycloneDX document;
- routine PR validation artifacts are not treated as public trusted releases merely because CI built them.

Attestation is provenance evidence, not a substitute for code review/security assessment.

## Apple Developer ID / Notary Service — credentialed release boundary, not yet activated

Apple Developer ID signing and notarization are required before calling the desktop application a normal public direct-download macOS release.

Current status:

- unsigned/ad-hoc Apple Silicon app build is validated;
- no Developer ID certificate/private key is stored in the repository;
- no App Store Connect/Notary API private key is stored in the repository;
- notarization/stapling is not simulated;
- activation requires real human-controlled credentials and a dedicated credentialed release validation.

## Tauri updater — deliberately disabled pending trust root

Tauri v2's updater requires signed update artifacts. Signature verification cannot be disabled.

Current decision:

- `tauri-plugin-updater` is not installed;
- `createUpdaterArtifacts` is not enabled;
- no production update endpoint is configured;
- Phase-23 CI verifies this fail-closed state;
- activation requires a real Tauri signing private key, committed public verification key, trusted HTTPS endpoint, recovery/storage policy, and end-to-end update verification.

## ip-as-logo Agent Skill — optional developer/design integration

Upstream: `s1dashu/ip-as-logo-skill`.
Pinned upstream commit: `acb834c717bcd0a487c49732d08397ba280d690b`.
License: MIT.
Vendored path: `tools/agent-skills/ip-as-logo/`.

Reviewed functional payload is instruction text, not executable runtime code.

Approved boundary:

- explicit mascot/product-identity/app-icon exploration only;
- exact vendored `SKILL.md` and license, with upstream Git blob IDs checked in CI;
- no npm/CLI installer required;
- no automatic upstream sync;
- no access to manuscript data unless a user explicitly provides a design brief that needs non-sensitive product context;
- no dependency from Rust engine, desktop runtime, persistence, publication, or core CI;
- generated visual outputs remain candidates until a human selects them.

Research/provenance: `docs/PHASE_23_RESEARCH.md` and `tools/agent-skills/ip-as-logo/SOURCE.md`.

## rbook 0.7.10 — Phase 24 independent EPUB validation

Upstream: `DevinSterling/rbook`.
Selected exact crate: `rbook = 0.7.10`.
License: Apache-2.0.

Approved boundary:

- `document-engine` dev dependency only;
- CI/test-time independent EPUB parsing and spine/resource reading;
- strict reopen of project-owned generated fixtures and translated Persian EPUB output;
- no project persistence types, no translation/review ownership, no application dependency and no normal runtime requirement;
- BookForge remains the canonical ingestion/source-aware reconstruction dependency;
- EPUBCheck remains the standards conformance validator.

Purpose: reduce common-mode validation risk. A BookForge-generated EPUB that can only be reopened by BookForge does not provide the same independent structural evidence as a second parser.

## Persian text-cleaner references — native implementation only

Reviewed references:

- `brothersincode/virastar` — MIT;
- `rezkam/persian` — MIT.

Decision: no JS/Python runtime dependency. Phase 24 implements the small deterministic Unicode/typography subset natively in Rust, with aggressive/literary-style transformations left advisory or excluded.

## Fiction character/coreference candidates — benchmark only

### BookNLP

Upstream: `booknlp/booknlp`.
Source license: MIT.

Potential evidence: character clustering, pronoun/coreference chains and quotation speaker attribution.

Decision: not installed. The model stack is heavy (Torch/TensorFlow/spaCy/Transformers), the original package/repository has older runtime assumptions, and code-license review alone is insufficient to promote model/training provenance into the product. Any future evaluation must be isolated, rights-safe and benchmarked against existing literary intelligence.

### FastCoref

Upstream: `shon-otmazgin/fastcoref`.
Software license: MIT; reviewed F-Coref/LingMess model cards advertise MIT.

Decision: not installed. It may be an optional benchmark competitor for English coreference, but it does not itself solve quotation speaker attribution and cannot become canon/runtime without demonstrated benefit.

## Supporting-tool candidates outside translation runtime

### OpenDataLoader PDF — ingestion benchmark candidate

Potential value: structured Markdown/JSON/HTML extraction, reading-order/layout recovery, bounding boxes, tables, and OCR/hybrid handling for difficult PDFs. Do not replace `document-engine`; benchmark project-owned fixtures first and keep any Java/Python/hybrid AI tooling optional behind a narrow ingestion boundary.

### ripwire — developer-only code-intelligence candidate

Potential value: deterministic tree-sitter symbol/call graph, ranked bounded code maps, and MCP for coding agents. If evaluated, it remains developer tooling and never owns runtime/project memory or becomes required for build/translation/review/export.

### Headroom — conditional developer/research candidate

License: Apache-2.0. Potential value is coding/research context compression. Do not place lossy compression between literary evidence/context and translation/review providers without a dedicated fidelity benchmark.

## Other architecture references

- `sukamenev/booktrans` — whole-book scouting/selective context/editor-verifier patterns.
- Tolmach / `KazKozDev/book-translator` — refinement/verifier concepts; reference-only under licensing boundary.
- `madpin/epublate` — EPUB round-trip/glossary lifecycle ideas; no code copied without independent licensing/provenance review.
- ArmenianLitTranslator — critic-role/evaluation-dimension reference only.

## Upgrade / adoption policy

Before adding/upgrading any external repository/package/model/tool:

1. prove a concrete capability gap;
2. read current license, release/security notes, provenance, maintenance state, and model/data licenses;
3. compare public API, runtime cost, privacy, persistence, and failure implications;
4. prefer a reasonably small native Rust implementation when safer and easier to own;
5. keep heavyweight models/downloads optional unless a numbered phase explicitly promotes them;
6. benchmark on rights-safe/project-owned fixtures;
7. run rustfmt, Clippy, affected/full tests, release/CLI smoke where relevant, and vulnerability audits;
8. validate optional tools on Linux/macOS when expected there;
9. never convert external evidence into canon or human approval;
10. record selected/deferred decisions in roadmap/status/PMC/this document before merge.

## Phase 25 narrative-speaker candidates

### ModernBookNLP

Upstream: `gasmichel/ModernBookNLP_QA`.

Research value: 2026 joint-scoring quotation attribution with strong reported PDNC results and an MIT-licensed modified BookNLP subfolder.

Boundary:

- root research stack is Python/Torch/Transformers/spaCy/torch-geometric heavy;
- root repository licensing and separately downloaded model checkpoint terms must be reviewed independently before any integration;
- not installed and not a runtime/build requirement;
- future use, if any, must be an optional process sidecar with benchmark-proven gain and evidence-only output.

### BookNLP

Upstream: `booknlp/booknlp`; MIT source.

Research value: literary entities, quotation detection/speaker attribution and coreference.

Boundary: reference/optional future benchmark only. The Phase-25 native explicit-speaker baseline does not require its model stack.

### FastCoref

Upstream: `shon-otmazgin/fastcoref`; previously reviewed MIT software/model-card terms for F-Coref/LingMess.

Boundary: optional future English coreference benchmark only. No product dependency until a rights-safe pronoun/coreference benchmark proves a measurable gap.

### Maverick / BookCoref

Upstreams: `SapienzaNLP/maverick-coref`, `SapienzaNLP/bookcoref`.

Reviewed terms include non-commercial CC licensing. Research-only; do not vendor or integrate into product/runtime.

### Renard

Upstream: `CompNet/Renard`; GPL-3.0-only.

Research value: modular character-network pipeline design.

Boundary: architecture reference only; no dependency/code adoption.

### LitBank

Upstream: `dbamman/litbank`; dataset license CC BY 4.0.

Boundary: eligible for a future attributed external/reference benchmark. Permanent CI remains project-owned synthetic and network-free.

### PDNC

Project Dialogism Novel Corpus; CC BY-NC 4.0.

Boundary: research reference only; not a product benchmark dependency or committed corpus.

## Phase 26 coreference candidates

### LitBank

Upstream: `dbamman/litbank`; dataset license CC BY 4.0.

Use: attributed external/reference evaluation for literary entities, coreference and quotation linkage. Permanent CI remains project-owned/network-free.

### BOOKCOREF

Upstream: `SapienzaNLP/bookcoref`; ACL 2025 book-scale benchmark.

Reviewed terms: data/software CC BY-NC-SA 4.0.

Boundary: research/reference only. Do not vendor, train on, or make it a product benchmark/runtime dependency.

### xCoRe

Upstream: `SapienzaNLP/xcore`; EMNLP 2025 cross-context coreference system.

Research value: long-document and cross-context cluster merging, including a LitBank model.

Reviewed terms: CC BY-NC-SA 4.0.

Boundary: research/reference only; no product/runtime integration under current terms.

### FastCoref

Upstream: `shon-otmazgin/fastcoref`; software MIT.

Boundary: optional future benchmark candidate only. Exact checkpoint/model license, dependency/security/resource profile and literary long-span gain must be reviewed separately before any integration.

### CorPipe 2026

Upstream: `ufal/crac2026-corpipe`; software MPL-2.0.

Boundary: current multilingual coreference engineering reference. No Phase-26 runtime adoption because literary/book-scale suitability remains unproven for this project.

### NovelCR

Upstream: `tongmeihan1995/NovelCR`; ACL Findings 2025 long-span novel coreference benchmark.

Boundary: research reference only until exact dataset-level licensing/redistribution terms are explicitly verified. Public availability is not treated as permission to vendor data.

### Phase-26 process boundary

The project-owned Rust protocol accepts model output only as optional evidence. A future adapter must preserve exact source offsets/text, bounded execution, fail-closed validation, canonical-anchor mapping and normal-translation fallback. No candidate listed above is installed by Phase 26.
