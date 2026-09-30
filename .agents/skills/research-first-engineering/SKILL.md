---
name: research-first-engineering
description: Mandatory evidence-first workflow for bugs, features, regressions, integrations, quality problems, and production-behavior changes. Investigate root cause, research proven patterns, prove RED before GREEN, and verify the exact executable path before claiming success.
---

# Research-First Engineering

Use this skill for every bugfix, behavior change, integration, quality regression, performance issue, format/export defect, provider problem, or request equivalent to "make this work properly".

This skill adapts the strongest ideas from Superpowers systematic debugging, TDD, and verification-before-completion to this repository. `AGENTS.md` and phase-specific invariants remain higher authority.

## Iron laws

1. **No fix before root-cause investigation.**
2. **No behavior change without a regression test or executable reproduction that proves the old behavior fails or is insufficient.**
3. **No "fixed / done / works" claim without fresh evidence for the exact claim.**
4. **Green unit tests do not prove end-to-end manuscript or publishing behavior.**
5. **Do not trust agent reports, remembered state, stale docs, or a prior CI run when current GitHub/code evidence can be inspected.**

## Phase 0 — Recover current truth

Before changing code:

- read `AGENTS.md`, affected architecture/roadmap docs, modules/tests/workflows/config;
- verify current `main` SHA and open/stacked PRs;
- inspect recent changes touching the same execution path;
- identify the concrete CLI/application/provider/export path actually used;
- identify persisted schema/version/fingerprint boundaries involved;
- distinguish canonical `main` from branch-only/pilot behavior.

If memory/docs disagree with GitHub/current code, current repository evidence is authoritative.

## Phase 1 — Reproduce and trace root cause

For a bug or quality failure:

1. capture the exact symptom and safe reproduction input;
2. read full errors/logs/artifact evidence;
3. reproduce on the closest deterministic path to the real application;
4. trace data/provenance through ingestion -> model/provider -> revision/quality -> persistence -> export as applicable;
5. inspect adapters/wrappers/feature flags, not only core helpers;
6. compare a working format/provider/path with the broken one;
7. state one concrete hypothesis backed by evidence.

If the issue cannot be reproduced, gather stronger diagnostics. Do not guess.

If three distinct fixes fail, stop patching symptoms and reassess the architecture.

## Phase 2 — Research before design

When external behavior, format standards, model/provider behavior, dependency choice, typography, EPUB/DOCX/PDF tooling, or architecture could materially affect the solution:

- search GitHub source, issues, PRs, discussions, releases;
- prefer official specs/docs and primary upstream source;
- use community reports as secondary evidence;
- compare plausible alternatives instead of copying the first implementation.

For dependencies/tools/models/data, record:
- exact gap solved;
- overlap with current architecture;
- adopt vs adapt-ideas-only vs reject/defer;
- maintenance activity;
- code/data/model license separately;
- security/transitive risk;
- Apple Silicon/Linux/Windows and Rust/Python compatibility;
- resource/build/runtime cost;
- privacy/network behavior;
- rollback/removal path.

Popularity alone is not evidence.

## Phase 3 — RED before implementation

Create the smallest failing regression or executable reproduction first.

Requirements:
- test real domain behavior rather than only mock interactions;
- use the concrete application/CLI/export boundary when wiring matters;
- for document bugs, include a minimal rights-safe fixture with the same structural condition;
- for persistence bugs, prove reopen/resume/migration behavior;
- for quality regressions, define the measurable failure before tuning prompts/models;
- verify RED fails for the intended reason before writing production code.

## Phase 4 — Minimal root-cause fix

- fix the source of the defect;
- one hypothesis at a time;
- no unrelated refactor;
- preserve provenance, human-review authority, persistence compatibility, fail-closed export, and provider neutrality;
- prefer adapting upstream ideas over adding heavy dependencies when existing architecture is sufficient.

## Phase 5 — Verification ladder

Freshly verify:
1. focused regression;
2. affected crate/module/format tests;
3. full repository baseline;
4. concrete application/CLI path;
5. affected persisted/exported artifact contract;
6. task-specific phase/security/platform workflows;
7. exact PR-head CI;
8. after merge, canonical `main` evidence when the claim is about shipped/released behavior.

For user-visible publishing changes, inspect the produced artifact rather than inferring success from internal data.

## Completion vocabulary

Use precise states:
- **root cause identified**
- **regression reproduced**
- **code fix implemented**
- **PR-head verified**
- **merged to main**
- **artifact/end-to-end path verified**
- **human/editorial quality confirmed** when a human-quality gate actually exists and has been run

Never collapse these into one "done" statement.

## Review

Before merge, independently compare the diff to:
- original symptom/quality goal;
- root-cause hypothesis;
- RED regression;
- `AGENTS.md` and phase invariants;
- compatibility/migration risk;
- rights/privacy/security impact;
- rollback.

Critical or important findings block merge.
