---
name: agent-session-safety
description: Preserve reliable session continuity and gate repository mutations during long-running, scheduled, or tool-heavy agent work.
---

# Agent Session Safety

Use this skill for broad next-stage work, scheduled/hourly engineering runs, multi-step investigations, or any task that may span multiple tool calls, commits, workflows, artifacts, or resumed sessions.

This is developer/agent infrastructure. It must never become a translation-runtime dependency or a path for manuscript/private review data.

## 1. Establish a resumable working set

Before mutation, retain only the minimum current working set:

- user-visible product goal;
- canonical `main` SHA;
- active branch/PR stack and prerequisite PRs;
- current roadmap/phase frontier;
- strongest verified evidence;
- task-relevant crates/files/workflows/artifacts;
- unresolved blocker and next action.

Treat previous chat/session summaries as hints only. Current repository/GitHub evidence is authoritative.

## 2. Resume safely

When a later run continues prior work:

1. compare saved main/branch/PR references with current GitHub state;
2. re-evaluate stacked dependencies and phase prerequisites;
3. re-check task-relevant CI and review state;
4. invalidate stale assumptions when source, context, plan, schema, provider, or artifact contracts changed;
5. continue from the first unproven step rather than the last claimed step;
6. never replay a write, provider call, release, or artifact mutation merely because the previous session result is uncertain.

Resume must preserve repository and product idempotency.

## 3. Keep context bounded

Carry forward high-signal evidence:

- exact SHAs/PRs;
- accepted architecture constraints;
- root cause or product gap;
- contract/acceptance criteria;
- changed surfaces;
- workflow/artifact results;
- remaining blocker/rollback.

Drop duplicated logs, stale hypotheses, raw unrelated tool output, and transient implementation chatter.

Never compact away:
- failing CI;
- unresolved human/editorial review;
- rights/license/privacy constraints;
- stale/mixed-plan evidence;
- artifact validation failures;
- required real-book/release acceptance.

## 4. Pre-action mutation gate

Classify each action before execution:

- **read-only**: inspect/search/fetch;
- **repo-local write**: branch/file/commit/PR updates;
- **external side effect**: provider call, deployment/release, cloud upload, external evaluation;
- **destructive**: delete/overwrite/force operation or irreversible project mutation.

For repo-local writes:
- verify repo/branch/path and PR stack;
- verify the write belongs to the current phase/task;
- preserve persisted schemas, human-review authority, and rollback requirements;
- prefer reviewable branches/PRs.

For external/destructive actions:
- require explicit task authorization and all relevant repository acceptance rules;
- never infer approval from a previous unrelated run;
- never transmit user manuscript, translation, reviewer notes, or secrets unless the exact approved workflow explicitly permits it.

## 5. Post-action verification hook

After every mutation:

1. inspect the returned commit/PR/workflow/artifact result;
2. verify the intended target changed;
3. record the strongest evidence level actually achieved;
4. diagnose a failure before retrying;
5. do not loop the same mutation after an ambiguous result.

After three materially different failed fix attempts, revisit architecture/coupling instead of stacking speculative patches.

## 6. Handoff checkpoint

Before ending a substantial run, update the existing durable project/repository memory surface with:

- current `main`, active PR/branch stack and phase frontier;
- exact changes;
- validation/artifact evidence actually observed;
- unproven or human-review-dependent claims;
- blockers/risks/rollback;
- exact next action.

Never persist manuscript text, translations, private reviewer notes, API keys, credentials, or other private project content in engineering checkpoints.

## 7. Provenance boundary

The ideas here are adapted from public Anthropic material with explicit open-source licensing, especially:

- `anthropics/claude-agent-sdk-python`: session stores/resume, session summaries, hooks, and tool-permission callbacks (MIT);
- `anthropics/claude-plugins-official`: automation/hook patterns and repository-instruction management (Apache-2.0 components).

Do not vendor or copy leaked, decompiled, reverse-engineered, or unclear-license Claude prompt/code dumps. They may be used only as discovery leads; implementation must be independently designed or sourced from clearly licensed public material.
