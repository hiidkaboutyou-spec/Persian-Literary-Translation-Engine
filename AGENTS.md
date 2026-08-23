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
- Use Python only for document parsing or PDF/EPUB/DOCX/TXT processing when a Rust solution is impractical. Keep any Python adapter isolated from the Rust domain model and orchestration.
- Preserve compatibility of persisted project memory, runtime manifests, CLI behavior, and published artifacts unless a migration path is included.
- Human review remains an explicit stage; automation must not claim literary approval on a reviewer's behalf.

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

## Forbidden actions

- Do not replace literary translation with unreviewed word-for-word or generic machine translation.
- Do not bypass deterministic quality gates or mark automated output as human-approved.
- Do not silently discard glossary, character, relationship, or translation-memory decisions.
- Do not introduce Python into the core runtime when Rust can reasonably implement the requirement.
- Do not commit manuscripts, generated translations, credentials, or private review material.
- Do not break CLI, persistence, manifest, or publishing contracts without migration and documentation.
- Do not force-push shared branches, bypass failing CI/security checks, or mix this repository with another product.

## Development workflow

1. Inspect `README.md`, relevant architecture/roadmap documents, the affected Rust modules, tests, and current GitHub state.
2. Make the smallest coherent change on a focused branch.
3. Add tests for behavior, literary-decision consistency, and regressions.
4. Run formatting, Clippy, tests, build, and task-specific smoke checks; fix failures.
5. Review the diff for secret/manuscript exposure, persistence compatibility, provider coupling, and quality-gate regressions.
6. Open a concise pull request describing behavior, contracts, and validation.
7. Merge only after required CI and security checks pass and the change is safe; otherwise record the blocker and leave the pull request open.
