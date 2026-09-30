---
name: translation-engine-acceptance
description: Project-specific acceptance gates for Persian Literary Translation Engine changes affecting translation quality, document formats, publishing artifacts, providers, persistence, memory, UI, or release behavior.
---

# Translation Engine Acceptance

Use together with `evidence-first-engineering` for any substantive engine change.

## General rule

Acceptance must prove the **user-visible book/project outcome**, not merely that a helper returned successfully.

## Translation quality / prompt / provider changes

For changes that can alter Persian output:

1. preserve source meaning, character identity, dialogue attribution, relationship facts, terminology and chronology;
2. run deterministic quality/fidelity checks;
3. exercise the concrete provider/runtime path that owns the change;
4. compare before/after on rights-safe committed fixtures;
5. if claiming literary/naturalness improvement, require the repository's human/editorial benchmark or explicitly label the result as unreviewed;
6. do not promote model output or inferred literary evidence to human-approved canon;
7. keep credential-free deterministic execution intact unless an explicit roadmap decision changes it.

Unit tests can prove invariants; they cannot alone prove literary quality.

## DOCX / EPUB / PDF / TXT / Markdown changes

For ingest/export/publishing changes:

1. exercise the exact format through the canonical application/runtime path;
2. use project-owned or rights-safe fixture input;
3. perform round-trip/reopen validation where supported;
4. inspect the generated artifact structurally, not only its existence;
5. for RTL/mixed-script DOCX/EPUB work, verify paragraph/run direction, ordering, styles and preserved Latin segments;
6. preserve provenance/source-location contracts;
7. fail closed on unsupported/scanned/corrupt input rather than silently degrading.

A file that opens is not automatically a correct publication artifact.

## Persisted project / memory / canon changes

Verify:
- old project data can reopen or a tested migration exists;
- schema/version/fingerprint compatibility is explicit;
- stale source/context/plan artifacts are rejected correctly;
- crash/restart/resume is idempotent;
- canon/human-review authority is not silently widened;
- no manuscript or private translation text leaks into CI, GitHub, project memory, or external telemetry.

## Provider / model / external tool adoption

Before adoption require:
- measured repository-local gap;
- exact code/model/data license;
- security/transitive review;
- Apple Silicon/Linux/Windows impact as applicable;
- resource/build/download cost;
- privacy/network behavior;
- deterministic/offline fallback;
- pinned version/revision where project policy requires it;
- rollback/removal path.

Default dependencies must not grow merely because a tool is popular.

## Desktop/application orchestration

For UI/application changes:
- test through `ApplicationService` or the canonical application boundary;
- prove project locking/state transitions;
- prove error/event models;
- prove no duplicate orchestration authority is introduced;
- validate the affected desktop workflow if the change is user-visible.

## Release / cross-platform changes

For release, packaging, architecture or platform behavior:
- run affected locked workflows;
- require exact-head CI for supported targets;
- do not infer Apple Silicon/Windows success from Linux tests;
- verify generated checksums/packages when touched.

## Final status vocabulary

Use precise claims such as:
- "implemented";
- "focused regression green";
- "affected phase CI green";
- "artifact round-trip verified";
- "human/editorial quality reviewed";
- "merged to main";
- "release/real-project behavior verified".

Do not collapse these into "done".
