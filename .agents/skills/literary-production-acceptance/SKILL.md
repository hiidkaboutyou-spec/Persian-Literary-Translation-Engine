---
name: literary-production-acceptance
description: Project-specific acceptance gates for Persian Literary Translation Engine changes affecting manuscript ingestion, providers, literary memory, quality, persistence, DOCX/EPUB/PDF, RTL typography, UI/application orchestration, or published artifacts.
---

# Literary Production Acceptance

Use together with `evidence-first-engineering` whenever a change can alter a real manuscript workflow or publication artifact.

## Principle

The acceptance target is the **end-to-end product contract**, not the nearest helper function.

Map the concrete path before testing:

`source document -> ingest/normalize -> Manuscript model -> analysis/memory/context -> provider -> revision/quality gate -> durable project state -> export -> reopen/round-trip/readability`

Only test the segments relevant to the change, but never skip the boundary where the original defect was visible.

## Format changes

### DOCX
Verify:
- paragraph/run ordering;
- Persian RTL paragraph direction;
- mixed Persian/Latin run direction;
- punctuation/numeral behavior where relevant;
- headings/lists/page-break expectations;
- reopening the generated DOCX with project tooling when supported;
- no source text loss or duplicate paragraphs.

For typography/layout claims, inspect the actual generated document structure/artifact; internal strings alone are insufficient.

### EPUB
Verify:
- archive validity;
- spine/order/resource references;
- XHTML language/direction semantics;
- CSS/RTL behavior;
- round-trip/re-ingestion when supported;
- no loss of chapter/block provenance.

### PDF
Distinguish text-based parsing from scanned/image-only limitations. Never claim OCR support unless an actual OCR path exists and was tested.

## Translation/provider changes

Verify:
- deterministic/offline provider path remains usable;
- production provider schema/response validation;
- source-to-output alignment and truncation guards;
- glossary/character/relationship context remains bounded and authoritative at the correct level;
- provider failure does not corrupt canonical memory or mark incomplete work complete;
- pause/resume/checkpoint fingerprints still prevent mixed-plan output.

Prompt/model tuning must not be called an improvement from anecdotal samples alone when a project benchmark/human gate exists.

## Memory/canon/review changes

Verify:
- inferred evidence cannot silently become canon;
- human review/promotion boundaries stay explicit;
- persisted schema compatibility or tested migration;
- reopen/resume behavior;
- provenance/fingerprint binding;
- conflict and ambiguous cases fail closed.

## Application/UI orchestration

Exercise the real `ApplicationService`/CLI adapter path, not only domain helpers.

Check project lock/state transitions, typed errors/events, interrupted operation/resume, stale source/canon invalidation, and export authority only when the current plan is complete.

## End-to-end acceptance fixture

Use rights-safe synthetic/project-owned fixtures in permanent CI. For a user-reported structural bug, create the smallest fixture that reproduces the same structure without copying private manuscript text.

Where practical, exercise:

`project create -> import -> analyze/review as required -> translate with deterministic provider -> pause/resume if touched -> export -> reopen/validate artifact`

## Baseline verification

For core Rust changes run the repository baseline in `AGENTS.md`, including format, clippy, workspace tests, release build and CLI smoke. Also run every affected phase/format/security/Apple-Silicon workflow required by `AGENTS.md`.

## Claim discipline

Do not say:
- "DOCX fixed" because XML unit tests pass;
- "translation quality improved" because one sample looks better;
- "EPUB round-trip works" because export completed;
- "resume works" because checkpoint serialization passed.

Say exactly what has been proven: focused regression, full workspace, generated artifact validation, round-trip/reopen, exact-head CI, and human/editorial review when applicable.
