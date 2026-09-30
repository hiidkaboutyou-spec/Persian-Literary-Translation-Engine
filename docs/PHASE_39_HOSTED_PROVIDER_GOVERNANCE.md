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

## Account/project evidence capture procedure (2026-09-30)

GitHub-first implementation research found the official `openai/terraform-provider-openai` (Apache-2.0) exposes read-only data sources for the exact project-scoped controls needed here: `openai_project_data_retention`, `openai_project_rate_limits`, `openai_project_spend_alerts`, and `openai_project_spend_limit`. The provider source also keeps Administration API credentials separate and requires `OPENAI_ADMIN_KEY` for these data sources.

Decision: use these interfaces only as an **operator-side, read-only evidence path** when an authorized Admin API key is available. Do not install Terraform/provider binaries into this repository, do not run them in project CI, do not commit Terraform state, project IDs, Admin API keys, recipient addresses, raw account metadata, or command output. Do not create/update spend limits, alerts, rate limits, model permissions, or retention settings as part of evidence collection.

Safe evidence workflow:

1. Run the official provider in a temporary local directory outside the repository, pinned to an explicitly reviewed provider version/revision.
2. Supply the Admin API key only through the environment or an external secret manager; never through `.tf`, shell history, project files, CI variables, or captured screenshots.
3. Query only the four read-only data sources above for the intended project. No Terraform resources and no `apply`.
4. Redact project/account identifiers and notification recipients before preserving evidence.
5. Record only the minimum normalized facts needed by Phase 39: retention mode; relevant model rate limits; whether project spend alerts exist and their non-identifying thresholds; whether a hard project spend limit exists and its threshold; capture date; provider revision; and source type.
6. Obtain **separate authoritative evidence** for data residency, optional data-sharing/feedback state, and endpoint-specific application-state behavior. Absence from the Terraform data sources is not evidence that these controls are disabled.
7. Treat any inaccessible, ambiguous, or stale field as UNKNOWN. Never infer ZDR from public eligibility, infer residency from retention mode, or infer a hard budget cap from spend alerts.

Current OpenAI documentation distinguishes spend alerts from hard spend limits: alerts notify but do not stop traffic, while a hard project spend limit can reject requests after the tracked threshold is reached. Phase 39 therefore records these as separate governance facts.

This procedure is intentionally not executable in repository CI because account evidence requires an external Admin API credential and may expose organization metadata. The repository stores only the redacted governance result, never the credential or raw provider state.

### Terraform state safety hardening (2026-09-30)

GitHub-first source review plus Terraform's own state-security guidance identified an additional evidence-capture hazard: a read-only data source can still persist returned fields in Terraform state. The OpenAI provider's project-control data sources expose raw-response material for diagnostics, so **read-only does not mean state-free or safe to retain**. Terraform also documents that local state is plaintext by default and that plans/backend configuration can contain sensitive material.

Phase 39 therefore tightens the operator-side Terraform option:

- treat every Terraform state, plan, crash/recovery state, `.terraform/` directory, stdout/stderr capture, and provider diagnostic as potentially sensitive account metadata;
- use a disposable working directory outside this repository and never a repository checkout, Git worktree, CI workspace, shared artifact directory, or synchronized folder;
- do not use a remote backend for this evidence probe; the goal is not durable infrastructure state, and uploading account metadata would create an unnecessary additional retention surface;
- pass credentials only through a short-lived environment/secret-manager path and never via backend arguments, `.tfvars`, plan files, shell-history-bearing command arguments, or committed configuration;
- extract only the normalized allow-listed Phase-39 facts, redact identifiers/recipients, and verify the redacted record before it crosses into the repository;
- destroy the disposable directory, including `terraform.tfstate*`, saved plans, `.terraform/`, lock/debug logs and crash files, after the normalized facts are captured;
- if the operator cannot guarantee this isolation and cleanup, **do not use Terraform**; capture the same facts through an authorized administrative UI/API path and preserve only the normalized redacted evidence.

No Terraform/provider dependency is added to this repository. No provider execution is required to merge Phase-39 documentation, and this hardening does not weaken the existing UNKNOWN/fail-closed rule.

## Redacted governance evidence record

Preserve only a normalized, non-secret record. Every field that is not supported by authoritative project/account evidence is `UNKNOWN` and remains blocking; public product capability or defaults are never substituted for project state.

Required fields:

- `captured_at` and evidence provenance/revision;
- `retention_mode`: organization default / ZDR / MAM / enhanced variant / none / UNKNOWN;
- `data_residency_region` and whether regional **storage** and **processing** are each evidenced;
- `data_sharing.api_inputs_outputs`, `data_sharing.evals_fine_tuning`, and `data_sharing.feedback` as independent states;
- intended `endpoint`, model snapshot/family, processing mode, and regional base URL where applicable;
- endpoint `store` behavior, ZDR eligibility, application-state behavior, and prompt-cache behavior relevant to that model/request;
- project spend alerts and hard spend limit as separate controls;
- relevant project/model rate limits;
- unresolved contractual/account-specific facts as explicit `UNKNOWN` entries.

Interpretation rules:

1. `store: false` is not evidence that ZDR is enabled. ZDR is an organization/project retention control and changes endpoint behavior only when actually enabled.
2. ZDR eligibility is request-wide: model, endpoint, tools and third-party services must all be compatible. ZDR-ineligible capabilities may retain application state even on a ZDR project.
3. Data residency storage and regional processing are distinct. A regional endpoint or storage region alone does not prove regional processing.
4. Prompt caching is application state and must be evaluated for the selected model/request. Do not infer cache behavior from the base endpoint alone.
5. Third-party MCP/tool traffic inherits the third party's own retention/residency policy; therefore Phase 39 qualification should use no third-party network tool unless separately governed.
6. Do not record project IDs, organization IDs, Admin API keys, notification recipients, raw API responses, screenshots containing identifiers, or manuscript text in this evidence record.

For the first rights-safe qualification, prefer a plain text-only `/v1/responses` or `/v1/chat/completions` request with no hosted/remote tools and no persistent conversation object. Freeze the exact endpoint/model/processing/cache parameters alongside the rubric before candidate generation. This is a qualification profile, not production admission.

## Qualification freeze checkpoint (2026-09-30)

The rights-safe corpus does not need to be invented or imported from a third party. Reuse the repository-owned synthetic `benchmarks/phase21/corpus-v1.json` (Git blob `893359b53c49619f4e808c4d8f89eebd1463eda4` on the current canonical base). Its provenance explicitly states that all English passages, Persian references, and contrastive degradations were written for this repository and that no third-party literary, subtitle, social-media, or proprietary manuscript text is included.

This corpus already exercises the Phase-39 literary failure classes that matter for an initial gate: negation/subtext, formal-vs-intimate register, sarcasm, named-place terminology continuity, agency/sequence, concrete-detail omission, English-shaped calques, and short-window entity continuity. Reusing it avoids a new dataset dependency, new licensing uncertainty, and benchmark drift while preserving the existing Phase-21 schema and Phase-31 qualification CLI.

Freeze the first hosted-provider qualification as follows **before** candidate generation:

- corpus path: `benchmarks/phase21/corpus-v1.json`;
- corpus Git blob: `893359b53c49619f4e808c4d8f89eebd1463eda4` (re-fetch and record a new immutable identifier if the corpus intentionally changes);
- all eight cases; no cherry-picking after outputs are seen;
- existing Phase-21 dimensions/anchors are deterministic challenge evidence only, never the literary verdict;
- candidate generation uses the existing `qualify-provider` pipeline; comparison uses `blind-compare`; review/dossier/provenance uses the canonical Phase 32–38 chain;
- human comparison must explicitly consider semantic fidelity, Persian naturalness/translationese, voice/subtext, relationship register, idiom/pragmatics, terminology/entity continuity, omissions/additions, and overall reading quality;
- ties/defer remain valid; no automatic winner and no score-only promotion;
- the reveal key remains unavailable to reviewers until all judgments are complete;
- provider/model snapshot, endpoint, processing mode, `store`, cache policy, and governance-evidence revision are frozen in the run record before generation;
- no hosted/remote tools, persistent conversation object, private manuscript, or rights-sensitive text is permitted in this first qualification.

Current OpenAI data-control documentation was rechecked on 2026-09-30. It confirms that Responses and Chat Completions can be ZDR-eligible but that ZDR must actually be enabled for the organization/project; `store: false` alone does not enable ZDR. It also documents endpoint/application-state exceptions, including prompt caching, and states that third-party network services have their own retention policies. OpenAI's current data-sharing documentation separately controls feedback, eval/fine-tuning data, and API inputs/outputs at organization/project scope. These facts reinforce the existing fail-closed account-evidence gate; they do not substitute for real project settings.

### Execution boundary

The corpus/rubric is now frozen enough for a reproducible qualification, but **provider execution remains blocked** until the redacted account/project governance record has no blocking UNKNOWN for retention/ZDR, residency, data sharing, intended endpoint/application-state behavior, budget, and relevant rate limits. Once that evidence exists, run the frozen corpus without changing cases/rubric in response to observed outputs, then carry the resulting blind bundle through the existing authenticated reviewer/reveal evidence path.

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
