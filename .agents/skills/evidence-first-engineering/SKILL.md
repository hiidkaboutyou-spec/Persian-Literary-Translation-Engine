---
name: evidence-first-engineering
description: Mandatory evidence-first workflow for bugs, features, refactors, dependency changes, and any claim that work is fixed, complete, safe, or production-ready.
---

# Evidence-First Engineering

Use this skill for any substantive code change, and always for:
- runtime bugs or unexpected behavior;
- failed CI/tests;
- performance/reliability regressions;
- dependency/tool/model adoption;
- persisted-contract or publishing changes;
- any task where the final answer would say "fixed", "done", "works", "safe", or "production-ready".

Repository `AGENTS.md` and project-specific skills remain higher authority.

## Preflight before mutation

Before changing code:
1. read `AGENTS.md` and task-relevant roadmap/status/handoff docs;
2. inspect canonical `main` HEAD;
3. inspect open/stacked PRs and prerequisite branches;
4. inspect recent task-relevant commits;
5. inspect the concrete runtime/publishing path, not only a helper/trait;
6. identify existing tests and CI gates for that path.

Do not edit first and investigate afterward.

## Define the observable outcome

Separate:
- the visible symptom/product outcome;
- the root-cause hypothesis;
- the acceptance evidence that would prove the outcome.

A green unit test is not automatically acceptance evidence.

## Reproduce or gather equivalent evidence

For bugs, reproduce before fixing whenever feasible.

Preferred order:
1. exact local reproduction;
2. regression test failing for the same reason;
3. CI/runtime/artifact evidence proving the failure path;
4. deterministic harness matching the concrete implementation path.

If reproduction is impossible, state why and gather the strongest evidence. Do not invent a cause.

## Research before design when uncertainty is material

Use current external research for file formats, platforms, providers, models, standards, dependencies, licensing, rights, security, or privacy when they can affect the design.

Prefer official specifications/docs, upstream source/issues/PRs/releases, exact licenses, security advisories, maintained implementations, and repository-local benchmarks.

For external tools/models record:
- exact gap;
- adopt vs adapt vs reject/defer;
- maintenance/activity;
- code/model/data rights;
- platform/resource impact;
- privacy/network behavior;
- rollback/removal path.

Popularity is not evidence.

## One root cause, one hypothesis

Do not stack speculative fixes. State one hypothesis and test it with the smallest safe experiment.

After three materially different failed fixes, stop local patching and review the architecture/coupling before another attempt.

## Regression-first implementation

For a bug:
1. add the smallest regression reproducing the demonstrated failure;
2. confirm it fails for the expected reason;
3. implement the smallest coherent root-cause fix;
4. rerun the focused regression.

For a feature:
1. define acceptance contracts first;
2. implement against them;
3. test failure modes and compatibility.

Mocks may isolate external systems, but at least one test must exercise the concrete integration boundary when the defect lives there.

## Verification ladder

Verify progressively:
1. focused regression;
2. related crate/module/integration tests;
3. repository baseline from `AGENTS.md`;
4. affected phase/format/security/platform workflows;
5. artifact/runtime/editorial acceptance when applicable;
6. exact-head PR CI;
7. post-merge `main`/release evidence when behavior can differ after merge.

Read actual results and failing steps. Do not infer success from an adjacent workflow.

## Independent convergence review

Before merge compare the current diff to:
- original product outcome;
- root cause;
- regression reproduction;
- architecture/persisted-contract invariants;
- privacy/rights constraints;
- rollback;
- required acceptance evidence.

Ask whether an adapter/provider/publisher/trait implementation bypasses the tested helper, whether an artifact can be structurally valid but user-visible wrong, whether CI differs from release/runtime behavior, and whether any human-review/fail-closed boundary was weakened.

Fix material gaps before merge.

## Completion-claim gate

Never claim "fixed", "complete", "safe", or "working" unless fresh evidence supports that exact level.

Distinguish:
- implemented;
- focused regression verified;
- affected/full CI verified;
- artifact/runtime verified;
- human/editorial quality reviewed;
- merged to main;
- release/real-project behavior verified.

If a human/editorial or real-book check remains, state that explicitly.

## Handoff

Record root cause/problem, evidence, external research decision, changed surfaces, regression/acceptance coverage, tests/CI actually run, what is and is not proven, migration/rollback, and the next verified frontier.
