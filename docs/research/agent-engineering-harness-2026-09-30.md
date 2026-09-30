# Agent engineering harness research — 2026-09-30

## Goal

Make engineering-agent work repeatable enough that a plausible patch cannot be described as complete merely because narrow tests pass.

Required behavior:
- research material uncertainty before design;
- establish root cause before bug fixes;
- reproduce failures where feasible;
- test concrete runtime/provider/publisher boundaries;
- verify progressively;
- separate deterministic correctness from literary/editorial quality;
- never claim more than the evidence proves.

## Reviewed upstreams

### OpenAI Plugins / Superpowers

The former `openai/skills` catalog is now deprecated. Current OpenAI guidance points to `openai/plugins` for Codex plugin examples.

The official OpenAI Plugins repository currently packages Superpowers:
- path: `plugins/superpowers`
- observed manifest version: `6.3.0`
- upstream: `obra/superpowers`
- license: MIT

Useful methods:
- systematic root-cause debugging;
- regression-first/TDD;
- evidence before completion claims;
- planning/execution/convergence;
- pre-merge review.

Decision: **adapt the methods into repository-owned skills, do not vendor the entire generic plugin into the engine**.

Why:
- this repository already has extensive domain-specific `AGENTS.md`, phase invariants and next-stage protocol;
- literary quality, human-review authority, document artifacts and cross-platform release acceptance require project-specific gates;
- a generic plugin should remain developer/agent tooling, never a Rust/runtime dependency.

### AGENTS.md

Reviewed `agentsmd/agents.md` (MIT), the open repository-local guidance format.

Decision: keep durable policy in root `AGENTS.md` and repeatable procedures in `.agents/skills/`.

### GitHub Spec Kit

Spec Kit is already approved here as optional developer-side tooling. Current upstream includes independent bug-fixing and idea-assessment processes in addition to spec-driven development.

Decision: keep it optional and complementary. Repository-owned evidence gates remain mandatory even when Spec Kit is unavailable.

## Architecture decision

Add:

1. `.agents/skills/evidence-first-engineering/SKILL.md`
   - evidence-first debugging, regression and verification workflow.

2. `.agents/skills/literary-production-acceptance/SKILL.md`
   - project-specific end-to-end proof for manuscript workflows, literary quality, formats/artifacts, persistence/canon, providers, application orchestration and releases.

3. a mandatory activation section in `AGENTS.md`.

## Important distinction

A passing parser test does not prove an exported book is publication-correct.
A passing deterministic quality gate does not prove Persian literary quality.
A Linux CI pass does not prove Apple Silicon or Windows behavior.
A model response does not become human-approved canon.

The harness therefore separates implementation, deterministic verification, artifact verification, editorial verification and release/real-project evidence.

## External plugin policy

The Superpowers plugin is useful as optional agent-environment assistance. Repository-owned skills remain authoritative and must work when the plugin is absent.

No application dependency, network service, secret, manuscript upload, model stack or runtime requirement is added by this harness.


## Implementation record

### Current PR state

The evidence-first engineering harness is implemented in PR #148 on branch `chore/evidence-first-agent-harness-2026-09-30`.

At the time of this handoff:
- PR: `#148 chore: add evidence-first engineering harness`
- current recorded head before this documentation update: `44fb6cea738c371b9a13fd405b64d7918e89f4c4`
- base: `main@ed0f844f601733c655b2e5b84bbbad6e67f1a847`
- merge state: open and mergeable;
- runtime impact: none; this is developer/agent process infrastructure only.

### Files added or changed

- `.agents/skills/evidence-first-engineering/SKILL.md`
  - requires current-truth preflight;
  - requires root-cause/reproduction evidence before bug fixes;
  - requires primary/upstream research for material file-format/platform/provider/model/dependency decisions;
  - requires regression-first implementation;
  - requires progressive verification and evidence-calibrated completion claims.

- `.agents/skills/literary-production-acceptance/SKILL.md`
  - requires end-to-end manuscript/product acceptance, not helper-only success;
  - adds DOCX/EPUB/PDF artifact and round-trip expectations;
  - requires mixed Persian/Latin and RTL verification when relevant;
  - protects provider, canon, review, persistence/resume and `ApplicationService` boundaries;
  - separates deterministic correctness from human/editorial literary quality.

- `AGENTS.md`
  - activates both skills as mandatory for substantive engineering work;
  - requires convergence against the original product outcome before merge;
  - forbids collapsing implementation/CI/artifact/editorial/release evidence into one generic "done" claim.

- `tools/validate_agent_harness.py`
  - validates required skills;
  - validates frontmatter names/descriptions;
  - validates root `AGENTS.md` activation;
  - validates the mandatory evidence-first contract.

- `.github/workflows/agent-harness.yml`
  - adds a lightweight, network-independent contract gate for agent tooling changes.

- this research note
  - records the upstream comparison, decisions, implementation, evidence and handoff.

### External research and decisions

Reviewed:
- current `openai/plugins` Codex plugin packaging;
- Superpowers `6.3.0`, upstream `obra/superpowers`, MIT;
- `agentsmd/agents.md`, MIT;
- GitHub Spec Kit;
- deprecated `openai/skills`, which now redirects readers toward current plugin guidance.

Decision:
- adapt systematic debugging, regression-first development, verification-before-completion and review discipline;
- keep those rules repository-owned;
- do not vendor the whole generic framework into the Rust engine;
- do not add a provider/model/runtime package/service/secret;
- keep optional generic agent plugins outside the product runtime.

### Branch-convergence note

During implementation, overlapping harness files appeared on the same working branch from concurrent work. They were reviewed instead of force-overwritten.

The final intended structure was deliberately consolidated to one generic skill plus one project-specific acceptance skill:
- `evidence-first-engineering`;
- `literary-production-acceptance`.

Duplicate/overlapping skill references were removed from the final intended contract. No force-push was used.

### Verification snapshot

On PR head `44fb6cea738c371b9a13fd405b64d7918e89f4c4`, the dedicated `Agent Harness Contract` completed successfully.

A later full check-run snapshot showed:
- 35 total checks;
- 33 successful;
- 0 failed;
- 2 still in progress;
- the two remaining checks were macOS desktop integrity paths.

Because required checks were still running, the PR was intentionally **not merged** at that point. This is the behavior the new evidence-first contract is intended to enforce.

After this documentation commit, exact-head checks must be read again before merge; the older snapshot must not be reused as proof for the new head.

### Acceptance meaning for this repository

Future claims must use the strongest actually-proven level, for example:
- implemented;
- focused regression verified;
- affected phase/full CI verified;
- generated artifact structurally validated;
- round-trip/reopen verified;
- human/editorial quality reviewed;
- merged to `main`;
- release/real-project behavior verified.

Examples of claims that are explicitly insufficient:
- "DOCX fixed" because an XML helper unit test passed;
- "translation quality improved" because a few samples look better;
- "EPUB round-trip works" because export returned success;
- "resume works" because checkpoint serialization passed.

### Ongoing durable-handoff rule

Future substantial work should record in-repository:
- product goal or root cause;
- research and upstream evidence;
- adopt/adapt/reject decisions;
- exact changed surfaces;
- regression and acceptance coverage;
- CI actually run;
- artifact/editorial evidence when applicable;
- what remains unproven;
- migration/rollback;
- the next verified frontier.

No manuscript text, generated private translation, reviewer prose, credentials or other private content belongs in this engineering record.
