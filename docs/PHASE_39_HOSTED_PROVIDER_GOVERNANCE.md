# Phase 39 — Hosted Provider Governance Evidence

Date: 2026-09-27
Status: research/evidence gate; production admission NOT GRANTED
Base: main@ed0f844f601733c655b2e5b84bbbad6e67f1a847

## Purpose

Record authoritative evidence required before any hosted provider can be considered for representative English→Persian literary qualification. This phase does not enable a provider, upload a private manuscript, or grant production admission.

## Canonical preflight

Phase 38 is canonical via PR #134 and main merge 914781a103679dbb9964583668bbeabb0104f879. Its handoff requires hosted-provider data-handling evidence plus representative human EN→FA literary evaluation as the next frontier. The existing Phase 33 governance boundary remains authoritative.

## OpenAI evidence snapshot

Evidence reviewed 2026-09-29:

- OpenAI Services Agreement: https://openai.com/policies/services-agreement/
- Service Terms (updated 2026-09-21): https://openai.com/policies/service-terms/
- Business data privacy/security: https://openai.com/business-data/
- Zero Data Retention announcement (2026-08-19): https://openai.com/index/offering-zero-data-retention-for-frontier-models/
- ZDR with Private Safety Processing: https://developers.openai.com/api/docs/guides/private-safety-processing
- Project data-retention controls: https://developers.openai.com/api/docs/guides/terraform/project-controls
- OpenAI API deprecations (Evals transition): https://developers.openai.com/api/docs/deprecations

### Supported by public authoritative evidence

1. API/business customer content is not used to develop or improve OpenAI services unless the customer explicitly agrees.
2. The customer retains Input rights and, to the extent permitted by law, owns Output; the customer remains responsible for having rights/licenses/permissions for Input.
3. Standard API retention and endpoint/application-state behavior are configuration-dependent; OpenAI states API inputs/outputs are removed from logs after 30 days under standard practices unless legal/security exceptions apply.
4. Eligible organizations can use Zero Data Retention. Current documentation makes clear that ZDR is project/configuration specific and that eligibility/approval is required.
5. Data-retention mode and data-residency compatibility can depend on the organization/project configuration.
6. Current Service Terms contain API-specific IP indemnity limits, including where the customer lacked rights to Input or ignored relevant safety/citation/filtering controls.

### Still UNKNOWN for this project's real API account/project

Public documentation does **not** prove any of the following for the project's actual OpenAI organization/project:

- ZDR approval or selected retention mode;
- project data-residency region;
- whether optional data-sharing/feedback is enabled;
- endpoint-specific application-state retention for the intended API path;
- approved budget/spend ceiling and current rate limits;
- organization-specific contractual/DPA terms beyond public terms.

These must remain UNKNOWN/fail-closed until account/project evidence is captured outside manuscript data.

## Evaluation-platform decision (2026-09-29)

Do **not** adopt the hosted OpenAI Evals platform for Phase 39. OpenAI announced its deprecation on 2026-06-03; existing evals become read-only on 2026-10-31 and the dashboard/API are scheduled to shut down on 2026-11-30. Adding a new hosted evaluation dependency now would create avoidable migration and retention surface while the repository already has a local blind-review/provenance chain.

Use the repository-owned Phase 32–38 evaluation path for blind human EN→FA qualification. Hosted evaluation frameworks may be researched as references, but they must not become required evidence storage or reviewer authority in this phase.

## Persian literary rubric evidence

GitHub research on 2026-09-29 found two useful idea sources that do not justify runtime adoption:

- `mshojaei77/ParsiEval` / Persian Fluency Bench (MIT): useful rubric ideas for formal Persian register, native fluency and explicit translationese penalties.
- `niktaas/TAAROFBENCH` (CC0 dataset): useful evidence that Persian cultural pragmatics can diverge sharply from literal-semantic quality and should be reviewed explicitly.

Adapt ideas only; do not copy benchmark code or make either repository a runtime dependency. Human literary review remains authoritative.

## Admission rule

Do not send private, unpublished, licensed, or otherwise rights-sensitive book text to a hosted provider merely to qualify it. First establish account/project governance evidence. Initial quality qualification must use public-domain, project-owned, synthetic, or otherwise explicitly rights-safe passages.

A hosted provider is not production-admitted merely because:
- public terms are acceptable;
- a model scores well;
- ZDR exists as a product capability;
- CI passes.

Production admission requires the existing governance assessment plus explicit owner authorization.

## Representative EN→FA literary evaluation plan

Use the existing blind-review/provenance chain:

1. Build a small rights-safe corpus representing dialogue, narration, idiom/pragmatics, register, character voice, ambiguity, imagery, and long-context continuity.
2. Freeze source passages and evaluation rubric before provider outputs are revealed.
3. Generate candidate translations without reviewer-identifying metadata in the blind bundle.
4. Human reviewers score fidelity, Persian naturalness, literary voice, character consistency, register, translationese/calque artifacts, idiom/pragmatics and cultural intent, omissions/additions, and continuity.
5. Preserve Phase 36–38 bundle/reveal/provenance rules and record the human decision dossier.
6. Compare against the current baseline; do not promote on aggregate score alone if a hard literary/safety criterion regresses.

## Dependency/tooling decision

No new runtime dependency is justified for this phase. Existing Phase 32/33 evaluation/governance and Phase 36–38 blind-review/provenance machinery already provide the required repository primitives. External Persian NLP benchmarks may inform rubric design but cannot replace human literary evaluation.

Keep unrelated Dependabot major upgrades and Cycle 14 publishing tooling out of this phase.

## Exit criteria

Phase 39 can advance only when:

- authoritative account/project retention evidence is recorded without secrets;
- data residency and optional data-sharing state are known;
- intended endpoint/model retention behavior is documented;
- budget/rate-limit controls are known;
- a rights-safe representative EN→FA blind human evaluation is completed;
- the resulting dossier passes the existing governance assessment;
- production activation, if ever requested, receives separate explicit owner authorization.

## Next action

Capture non-secret account/project data-control evidence, then run the rights-safe representative EN→FA qualification through the existing blind-review pipeline. Until then: production admission NOT GRANTED.
