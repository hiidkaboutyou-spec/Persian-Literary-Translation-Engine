# Implementation Status

## Vision
A production-grade Persian literary translation engine with a Rust core.

The system is designed around:
- document understanding before translation
- character voice preservation
- translation memory
- glossary consistency
- multi-pass quality control
- publication-ready export

## Current Rust Architecture

Implemented and validated on `main` through Phase 17:

### Core Crates
- **translation-core** — Provider-neutral translation pipeline with EchoProvider (deterministic testing) and OpenAIProvider (production). Multi-pass: translate → revise → quality review. Bounded passage chunking for oversized chapters.
- **document-engine** — Ingestion of TXT, Markdown, DOCX, EPUB, and text-based PDF files. Structured `Manuscript → Book → Chapter → Scene → Paragraph` model with source provenance. Extensible parser registry. Character-safe text chunking. Persian RTL DOCX export. Phase 17 routes normal EPUB ingestion through revision-pinned BookForge IR while mapping back into native document types; the legacy parser is an explicit `--no-default-features` compatibility path, never a silent fallback.
- **memory-engine** — Durable translation memory and glossary with JSON persistence. Passage-relevant runtime retrieval using Jaccard similarity with negation polarity detection and diversity filtering.
- **character-engine** — Character bible with profiles, aliases, word-boundary-aware matching, and relationship context. JSON persistence.
- **quality-engine** — Deterministic quality gate: empty output, prompt leakage, truncation, paragraph collapse, and terminology drift detection. Cross-chapter consistency auditing. Phase 17 adds an optional typed COMET JSON/process sidecar for advisory model-based quality evidence without changing human-approval rules.
- **project-engine** — Project manifest with chapter state tracking, schema versioning, and JSON persistence.
- **human-review-workflow** — Versioned literary-intelligence review ledger, deterministic proposal identity/reconciliation, validated lifecycle transitions, typed canon conflicts/resolutions, promotion plans, and audit lineage. It owns review decisions but not canonical stores or file mutation.
- **project-engine** — Project manifest plus atomic review-ledger writes and recoverable multi-file promotion transactions for Character Bible, Glossary, and review audit state.
- **literary-intelligence-engine** — Literary decision models plus deterministic `Manuscript` analysis. Produces versioned character and relationship seeds, chapter maps, terminology candidates, observed literary-profile metrics, bounded evidence references, conflict reporting, and non-mutating initialization proposals.
- **advanced-literary-analysis** — Optional provider-assisted literary analysis (Phase 15). Bounded, fingerprint-identified analysis units; a provider-neutral `LiteraryAnalysisProvider` (deterministic mock + OpenAI); versioned injection-resistant prompts; deterministic validation of structured findings; derived confidence with disagreement surfaced; fingerprint-keyed cache/resume; and review-eligible `Literary` proposals that flow through the Phase 14 ledger without ever becoming canon.
- **literary-reference-knowledge** — Reference sources, editorial guidelines, and validation rules with UUID identity.
- **project-engine `application` layer (Phase 16)** — One application boundary above the domain engines: `ApplicationService` with project create/open/import/snapshot, deterministic + advanced analysis orchestration, review and canon promotion reuse, character/glossary APIs, translation lifecycle, typed errors with recovery hints, project events, atomic persistence, locking/recovery, fingerprint staleness detection, bounded audit history, UI-ready JSON models, and capabilities reporting.
- **text-normalization** — Shared Persian/Arabic text normalization, negation detection, and similarity scoring. Eliminates duplication across memory, quality, and character engines.

### Phase 17 External Integration State

Merged via PR #90 at commit `6a4b8807d8c54878f1f10db5cab1f1290fcc60fb` after lockfile, rustfmt, Clippy, compatibility build, COMET script checks, full workspace tests, cargo audit, release CLI build, CLI smoke, and Security passed.

Canonical Phase 17 integrations:

- BookForge revision `23f8c9d3c97a06f48e13424698441bfb4b037844` for strict EPUB ingestion at the document boundary.
- `unbabel-comet==2.2.7` behind an isolated optional Python sidecar; model checkpoints are not downloaded by default.
- external-integration provenance/licensing/privacy/upgrade rules in `docs/EXTERNAL_INTEGRATIONS.md` and `AGENTS.md`.

### Branch-Scoped Supporting Tooling

The following is implemented on `chore/safe-quality-tooling` and is **not canonical until that branch is merged**:

- optional `quality-engine` feature `language-diagnostics` using exactly `lingua 1.8.0`, with default Lingua features disabled and only English/Persian models enabled;
- advisory `diagnose_persian_output` API that does not alter the deterministic quality gate or human-review state;
- optional EPUBCheck 5.3.0 installer/wrapper with published SHA-256 verification and ignored local `.tools/` installation;
- CI coverage for the optional language feature and EPUBCheck shell wrappers.

Heavy Phase 18–21 candidates such as BGE-M3, Hazm, DadmaTools, Vecalign, and SacreBLEU remain deliberately uninstalled until their owning phase has a benchmark, legal inputs, resource/failure constraints, and a concrete capability gap.

### CLI
- `inspect` — Document analysis with text and JSON output
- `analyze` — Credential-free manuscript intelligence with text summaries and stable schema-versioned JSON
- `analyze-advanced` — Explicit, provider-assisted literary analysis with deterministic mock provider available
- `review sync/list/show/approve/edit/reject/defer/reopen/promote` — Non-interactive review, reconciliation, conflict preview, and explicit canon promotion with stable JSON outputs
- `prepare` — Chapter preparation with text and JSON output
- `run` — Full pipeline: ingest → segment → context → translate → quality → export
- `resume` — Checkpoint-based resume with source fingerprinting
- `project create/import/status/analyze/analyze-advanced/review/translate/resume/progress/export/history` — Thin CLI adapter over the Phase 16 `ApplicationService`
- `--format json` — Machine-readable JSON output for all commands
- resume checkpoints include source and assembled-context fingerprints, so canon changes cannot silently reuse output generated under stale project knowledge

### CI/CD
- deterministic Cargo lockfile freshness check
- Format checking (rustfmt)
- Clippy with `-D warnings`
- Full workspace test suite
- optional-feature compatibility tests where external integrations are feature-gated
- BookForge explicit no-default-features compatibility build
- sidecar/wrapper syntax checks
- Cross-crate integration tests (glossary, character bible, memory, quality gates, JSON output, resume)
- 120-chapter end-to-end regression coverage
- CLI smoke tests (text and JSON modes)
- Cargo audit for dependency vulnerability scanning
- Security audit workflow
- Dependabot configuration

## Non-Negotiable Constraints
- Rust remains the core language.
- No simple machine translation wrapper.
- Preserve author intent and narrative structure.
- Build reusable translation memory across projects.
- Human review remains the approval/canon boundary.
- Optional probabilistic diagnostics and external metrics remain evidence, not authority.
- Heavy external models/tools must not become hidden runtime requirements.

## Test Coverage
- Coverage includes review lifecycle/reconciliation/stable IDs/conflict resolution, dry-run non-mutation, atomic rollback, idempotent apply, audit lineage, Unicode, CLI JSON, canon-aware resume, manuscript intelligence, full pipeline, and large-book regression behavior.
- Phase 15 coverage adds provider contract, prompt-injection resistance, evidence validation, stable identity, cache reuse/invalidation, partial failure, Unicode, review lifecycle for Literary proposals, and credential-free CLI end-to-end tests.
- Phase 16 coverage adds application lifecycle/offline workflow, snapshot accuracy, next-action determinism, review rules, character/glossary APIs, pause/resume, manual edits, translation gating, source mismatch, canon staleness, locking, corrupt-manifest recovery, events, provider config, advanced cache, JSON round-trips, Unicode, and 120-chapter regression.
- Phase 17 coverage adds strict BookForge EPUB regression fixtures, compatibility-path build coverage, COMET JSON/process protocol tests, lockfile freshness, and integration CI/security validation.
- Supporting tooling branch adds focused English/Persian language-diagnostic tests and shell validation for the EPUBCheck installer/wrapper without downloading EPUBCheck in default CI.
