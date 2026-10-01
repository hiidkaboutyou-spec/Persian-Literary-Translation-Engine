# Cycle 14 Book IR foundation research and implementation

Date: 2026-10-01. Base: `main@ed0f844f601733c655b2e5b84bbbad6e67f1a847`. Tracks #138.

## Current truth and blocker

Cycle 14 requires one canonical editable manuscript that can regenerate DOCX, EPUB and PDF. The existing document model has useful source identities, but the publishing path still consumes format-oriented `Chapter.content` strings and cannot explicitly represent protected LTR spans such as URLs, email, ISBN, code or user-approved Latin text. PR #147 fixes the immediate DOCX mixed-script symptom, but its heuristic is deliberately not the canonical editing model.

## Research queries and evidence

GitHub/web queries executed:
- `GitHub ebook intermediate representation AST EPUB DOCX book model Rust`
- `GitHub Pandoc AST DOCX EPUB document model protected inline spans`
- `GitHub typst document model inline text spans epub docx`
- prior Cycle 14 queries recorded in `CYCLE_14_DOCX_MIXED_SCRIPT_RESEARCH.md`.

Compared approaches:
- Pandoc AST: mature Block/Inline separation and generic spans. Strong architecture reference, but adopting Pandoc as runtime authority would add a large external tool boundary and does not preserve our project identities automatically.
- zacharydenton/boko: explicit format -> semantic IR -> format architecture. Good confirmation of exporter-independent IR; direct dependency is unnecessary for our Persian/editor domain.
- sukamenev/booktrans: stable block IDs survive translation. Strong fit for our identity requirement; implementation remains reference-only.
- CasualOffice/opendoc: normalized editable DOCX model with loss-aware round-trip and deterministic rendering. Valuable future DOCX/render research candidate, but much broader than this Book IR slice and requires separate license/security/API evaluation before adoption.
- existing project models: already use serde, stable source block IDs and deterministic Rust domain types. Lowest-risk path is a repository-owned IR with no new dependency.

## Decision

Add a small versioned, format-neutral Book IR inside `document-engine`.

The first schema intentionally contains only:
- stable book/block IDs;
- chapter headings, paragraphs and scene breaks;
- ordered inline runs;
- semantic direction (`rtl`, `ltr`, `auto`);
- explicit protected-run categories (URL, email, ISBN, identifier, code, file path, explicit Latin, provenance);
- optional language metadata;
- fail-closed schema-version validation and duplicate-ID rejection.

This is a foundation, not a migration. Existing import/export APIs remain unchanged. No provider, model, network service, font, normalization library or new Cargo dependency is activated.

## Regression evidence added

Unit tests require:
1. serde JSON round-trip preserves schema version, stable block identity, LTR direction and URL protection;
2. duplicate stable block IDs are rejected;
3. unknown future schema versions fail closed.

## Next gates

1. exact-head Rust CI / fmt / clippy / tests must pass;
2. add deterministic adapters from current translated chapters into Book IR without changing output;
3. replace PR #147's exporter-only directional heuristic with explicit Book IR runs once adapter fixtures prove equivalence;
4. preserve stable identities through edit -> export -> re-import;
5. only then share publication tokens across DOCX/EPUB/PDF and web preview.

No merge is justified until exact-head CI is green.
