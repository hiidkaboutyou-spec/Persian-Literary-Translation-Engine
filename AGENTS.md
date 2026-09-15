# Repository guidance

## Project purpose

This repository is a professional English-to-Persian literary translation engine. It must preserve authorial voice, narrative intent, character identity and psychology, emotional impact, dialogue authenticity, relationship evolution, terminology consistency, and cultural meaning while producing publication-oriented Persian.

The product flow is:

```text
Literary Intelligence
        ↓
Translation Runtime
        ↓
Quality Gate
        ↓
Human Review
        ↓
Publishing
```

This project is independent from every other repository. Do not import assumptions, code, architecture, terminology, or project memory from the Jeonghan Daily Review Bot or any unrelated project.

## Architecture rules

- Rust is the primary implementation language. The active workspace, CLI, tests, and release build live under `engine/`.
- Keep document ingestion, segmentation, literary/project memory retrieval, provider execution, revision, deterministic quality checks, artifact generation, and RTL DOCX publishing as explicit boundaries.
- Keep translation providers replaceable and provider-neutral; credential-free deterministic execution must remain available.
- Treat glossary, character bible, relationship context, and translation memory as durable literary intelligence with stable, documented schemas.
- Quality gates must report evidence and must not silently rewrite or accept degraded output.
- Use Python only for document parsing, model-backed analysis/evaluation, document tooling, or isolated developer tooling when a Rust solution is impractical. Keep every Python adapter isolated from the Rust domain model and orchestration.
- Preserve compatibility of persisted project memory, runtime manifests, CLI behavior, and published artifacts unless a migration path is included.
- Human review remains an explicit stage; automation must not claim literary approval on a reviewer's behalf.
- Heavy models and external tools must remain optional unless a numbered roadmap phase explicitly promotes them after benchmark, licensing, resource, privacy, and failure-mode review.

## Approved external integrations

- BookForge is the approved structured EPUB boundary. Keep `bookforge-core` and `bookforge-epub` revision-pinned as documented in `docs/EXTERNAL_INTEGRATIONS.md`; map their IR into native `document-engine` types and never persist BookForge types in project schemas.
- When the BookForge feature is enabled, a BookForge EPUB validation/parsing failure is actionable and must not silently fall back to the legacy parser. The legacy path exists only for explicit builds without the feature. Malformed archive/decompression preflight failures may be normalized to the public `CorruptedFile` contract, but validation must remain strict.
- COMET/XCOMET/DocCOMET are optional external quality evidence only. Keep them behind the isolated `quality-engine::comet` process boundary; do not import PyTorch/COMET into the Rust runtime, auto-download models in default CI, or use a COMET score as human approval.
- Lingua is approved only as optional English/Persian diagnostic evidence. Keep the crate exactly pinned as documented, disable its default all-language feature set, enable only English/Persian models, and keep `language-diagnostics` off by default. Lingua must not change deterministic quality results, rewrite text, reject intentional multilingual prose by itself, or act as human approval.
- EPUBCheck is approved as an optional external publication validator. Do not vendor its distribution. Keep the installer version/checksum pinned, install only under ignored local tool storage, and keep absence of Java/EPUBCheck from breaking ingestion, translation, DOCX export, or existing runtime behavior.
- `projectmem` 0.3.3 is approved only as optional **developer-side coding memory**, as documented in `docs/PROJECT_MEMORY_INTEGRATIONS.md`. It is not translation/runtime memory and must never be imported or invoked by production Rust paths. Use the repository's safe initialization profile: no Git hooks, no watcher, no history backfill, no global-memory inheritance, and no automatic `AGENTS.md`/`CLAUDE.md` edits. Its absence or failure must never block build, translation, review, or export.
- PMC and projectmem have different ownership: PMC is the intended curated durable engineering knowledge vault; projectmem is operational issue/attempt/fix/decision history. Canonical repository docs remain authoritative when either local tool is unavailable. Never put manuscripts, generated book translations, credentials, or private reviewer material in projectmem.
- ContextWeaver is an architecture reference, not a dependency. Stable IDs, bounded selective context, resume fingerprints, review history, and canon ownership stay native to this repository unless a future gap analysis proves otherwise.
- Do not copy code from `TranslateBooksWithLLMs`; selective glossary injection is already native in `memory-engine`, and any licensing change must be deliberate.
- TransAgents may inform agent-role separation but is not a runtime dependency. Provider/model judgments remain separate from deterministic quality checks and human review.
- FlagEmbedding/BGE-M3, Hazm, DadmaTools, Vecalign, and SacreBLEU are researched phase-scoped candidates, not blanket-approved dependencies. Follow `docs/EXTERNAL_INTEGRATIONS.md` and the roadmap; do not install them merely because they are useful in isolation.
- Serena, Global Agent Memory, MemoryWiki, and automatic chat-history memory systems are project-memory research references only unless a new gap analysis changes that decision. Do not install multiple overlapping memory systems by default.

## Coding standards

- Follow idiomatic stable Rust and the module patterns already present in `engine/`.
- Keep domain types explicit, serializable where persisted, and separated from provider and filesystem concerns.
- Prefer deterministic behavior, bounded passage processing, actionable errors, and no hidden global state.
- Avoid `unwrap`/`expect` in production paths when errors can be propagated with context.
- Never log, commit, or embed API keys, proprietary manuscripts, private translations, or reviewer data.
- Update documentation and examples when CLI, configuration, schemas, or output contracts change.
- Add focused unit tests and end-to-end regression coverage for behavior changes.

## Testing and validation

Run from `engine/` before proposing a merge:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --release -p literary-engine
cargo run --quiet -p literary-engine -- --help
cargo run --quiet -p literary-engine -- --version
```

For pipeline changes, also run a credential-free `EchoProvider` smoke test through ingestion, translation, quality gate, artifact generation, and `manuscript.docx` export. Run `cargo audit` when dependencies change. Test each affected document format and verify Persian RTL output when parsing or publishing code changes. Real provider tests require explicit credentials and must never expose manuscript content or secrets.

For BookForge changes, exercise EPUB ingestion with the default feature and verify an explicit `--no-default-features` document-engine build where practical. For COMET changes, test the JSON protocol without downloading a model in default CI; a real model smoke test requires explicit local setup and any model-specific license/authentication approval.

For Lingua changes, run:

```bash
cargo test -p quality-engine --features language-diagnostics
```

The normal workspace build must still pass with the feature disabled. For EPUBCheck wrapper changes, syntax-check both scripts without downloading the distribution in default CI. A real publication-validation smoke test should use a project-owned EPUB fixture or generated output.

For projectmem integration changes, run the dedicated `Project Memory Tooling` workflow. Its smoke test must confirm the selected package version and prove that safe initialization does not create/alter Git hooks, create `AGENTS.md`/`CLAUDE.md`, or start watcher state. Do not make normal Rust CI depend on projectmem.

## Forbidden actions

- Do not replace literary translation with unreviewed word-for-word or generic machine translation.
- Do not bypass deterministic quality gates or mark automated output as human-approved.
- Do not silently discard glossary, character, relationship, or translation-memory decisions.
- Do not introduce Python into the core runtime when Rust can reasonably implement the requirement.
- Do not turn optional ML models, external validators, or developer-memory tools into hidden runtime requirements.
- Do not commit manuscripts, generated translations, credentials, private review material, downloaded model checkpoints, or downloaded EPUBCheck binaries.
- Do not put manuscript/translation content into PMC or projectmem as a substitute for native runtime memory.
- Do not break CLI, persistence, manifest, or publishing contracts without migration and documentation.
- Do not force-push shared branches, bypass failing CI/security checks, or mix this repository with another product.

## Development workflow

1. Inspect `README.md`, relevant architecture/roadmap documents, the affected Rust modules, tests, and current GitHub state.
2. Make the smallest coherent change on a focused branch.
3. Add tests for behavior, literary-decision consistency, and regressions.
4. Run formatting, Clippy, tests, build, and task-specific smoke checks; fix failures.
5. Review the diff for secret/manuscript exposure, persistence compatibility, provider coupling, external-dependency provenance, runtime-cost changes, licensing, and quality-gate regressions.
6. Open a concise pull request describing behavior, contracts, and validation.
7. Merge only after required CI and security checks pass and the change is safe; otherwise record the blocker and leave the pull request open.
