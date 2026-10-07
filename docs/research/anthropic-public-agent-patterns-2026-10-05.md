# Anthropic public agent patterns — 2026-10-05

## Scope

This note records which Claude/Anthropic-related GitHub material was considered for the Persian Literary Translation Engine and what was actually adopted.

The objective is to improve engineering continuity and mutation safety around long-running agent work without importing Claude internals or adding an Anthropic runtime dependency to the translation product.

## Sources reviewed

### Adopt/adapt

1. `anthropics/claude-agent-sdk-python`
   - license: MIT;
   - relevant public patterns: resumable session stores, session summaries, hooks around tool use, and explicit tool-permission callbacks;
   - decision: **adapt concepts only** into repository-owned engineering guidance.

2. `anthropics/claude-plugins-official`
   - relevant components reviewed: automation-recommender material, hook-pattern references, Hookify, and repository-instruction management;
   - relevant component licenses observed: Apache-2.0;
   - decision: **adapt concepts only**. Existing `AGENTS.md`, phase contracts, project memory, and human-review boundaries remain authoritative.

### Rejected as implementation sources

- repositories publishing leaked/system-prompt dumps;
- decompiled or reverse-engineered Claude Code mirrors;
- unclear-license prompt/code collections.

They can be discovery leads, but are not copied into this repository.

## Gap identified

The evidence-first harness covers debugging and verification well, while this project also has unusually long-lived phase stacks, artifact validation, project memory, provider governance, human review, and resumable whole-book work.

Scheduled/resumed engineering runs therefore need an explicit contract for:

- validating saved branch/PR/phase state against current GitHub truth;
- keeping a bounded high-signal working set;
- preventing stale phase assumptions from crossing session boundaries;
- gating provider/cloud/release/destructive actions before execution;
- checking the actual returned result after mutations;
- avoiding blind retries after ambiguous writes;
- leaving a privacy-safe handoff for the next run.

## Implemented adaptation

Added `.agents/skills/agent-session-safety/SKILL.md` and activated it from `AGENTS.md` and `project-next-stage`.

The agent-harness validator now requires the skill.

No runtime dependency, model, provider, secret, manuscript upload path, publishing schema, or translation behavior is changed.

## Expected benefit

Future hourly/deep agent runs should be less likely to:

- resume on stale stacked-PR assumptions;
- repeat a write/provider side effect after uncertain status;
- lose an unresolved artifact/human-review requirement during context compression;
- push work past the actual canonical phase frontier;
- store private manuscript/reviewer content in engineering memory;
- claim previous-session evidence without fresh repository validation.

This change strengthens engineering automation only. It is not evidence of improved literary quality or publication behavior by itself.
