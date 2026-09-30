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
   - shared research, debugging, regression and verification workflow.

2. `.agents/skills/translation-engine-acceptance/SKILL.md`
   - project-specific proof for literary quality, formats/artifacts, persistence/canon, providers, desktop orchestration and releases.

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
