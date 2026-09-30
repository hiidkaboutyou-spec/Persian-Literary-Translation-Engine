---
name: evidence-first-engineering
description: Mandatory evidence-first workflow for bugs, features, refactors, dependency changes, and any claim that work is fixed, complete, safe, or production-ready.
---

# Evidence-First Engineering

Use this skill for any substantive code change, and always for:
- production/runtime bugs or unexpected behavior;
- failed CI/tests;
- performance/reliability regressions;
- dependency/tool/model adoption;
- persisted-contract or publishing changes;
- any task where the final answer would say "fixed", "done", "works", "safe", or "production-ready".

Repository `AGENTS.md` and project-specific skills remain higher authority.

## 1. Preflight before mutation

Recover current truth before changing code:

1. read `AGENTS.md` and task-relevant roadmap/status/handoff docs;
2. inspect canonical `main` HEAD;
3. inspect open/stacked PRs and prerequisite branches;
4. inspect recent task-relevant commits;
5. inspect the concrete runtime/publishing path, not only the obvious helper or trait;
6. identify existing tests and CI gates for that path.

Do not edit first and investigate afterward.

## 2. Define the observable outcome

Write down what must become true from the user's or product's point of view.

Separate:
- **symptom**;
- **root-cause hypothesis**;
- **acceptance evidence**.

A green unit test is not automatically acceptance evidence.

## 3. Reproduce or gather equivalent evidence

For a bug, reproduce it before fixing whenever feasible.

Preferred order:
1. exact local reproduction;
2. regression test that fails for the same reason;
3. CI/runtime/artifact evidence proving the failure path;
4. deterministic minimal harness matching the concrete production type/path.

If reproduction is impossible, state why and gather the strongest available evidence. Do not invent a cause.

## 4. Research before designing when uncertainty is material

Use external research when behavior depends on a file format, platform, provider, model, library, standard, or dependency.

Prefer:
- official specifications/documentation;
- upstream source, issues, PRs, releases/changelogs;
- security advisories;
- exact code/model/data licenses;
- maintained real implementations;
- repository-local benchmarks and prior research.

For any external tool/model/repository, record:
- exact gap;
- adopt vs adapt idea vs reject/defer;
- activity/maintenance;
- license/rights;
- platform/resource impact;
- privacy/network behavior;
- failure/rollback/removal path.

Popularity is not evidence.

## 5. Confirm one root cause before the real fix

Do not stack speculative fixes.

Use one explicit hypothesis and test it with the smallest safe experiment. If it fails, return to investigation.

After three materially different failed fix attempts, stop and review the architecture/coupling before another patch.

## 6. Regression-first implementation

For a bug:
1. add the smallest regression reproducing the demonstrated failure;
2. confirm it fails for the expected reason;
3. implement the smallest coherent root-cause fix;
4. rerun the focused regression.

For a feature:
1. define acceptance contracts first;
2. implement against them;
3. test failure modes and compatibility, not only happy-path output.

Mocks may isolate external systems, but at least one test must exercise the concrete integration boundary when the bug lives there.

## 7. Verification ladder

Run evidence in increasing scope:

1. focused regression;
2. related crate/module/integration tests;
3. repository baseline from `AGENTS.md`;
4. affected phase/format/security/platform workflows;
5. artifact/runtime acceptance when behavior changes a generated book, provider flow, persistence contract, or desktop path;
6. exact-head CI;
7. post-merge `main` evidence where merge/release/runtime behavior can differ.

Read the actual result and failing step. Do not infer success from an adjacent workflow.

## 8. Independent convergence review

Before merge compare the current diff against:
- original user/product outcome;
- root cause;
- regression reproduction;
- architecture and persisted-schema invariants;
- privacy/rights constraints;
- rollback;
- acceptance evidence.

Ask:
- Did we test the same concrete path that failed?
- Could a trait implementation/adapter/provider/publisher bypass the tested helper?
- Could a generated artifact be structurally valid but visually/semantically wrong?
- Could CI differ from release/runtime/platform behavior?
- Did the change silently weaken a human-review or fail-closed boundary?

Fix material gaps before merge.

## 9. Completion-claim gate

Never claim "fixed", "complete", "safe", "working", or equivalent unless fresh evidence supports that exact claim.

Distinguish:
- **implemented**;
- **focused regression verified**;
- **full/affected CI verified**;
- **artifact/runtime verified**;
- **merged to main**;
- **release/real-project behavior verified**.

If a human/editorial or real-book check remains necessary, state that instead of claiming full success.

## 10. Handoff

Record:
- root cause/problem;
- evidence;
- research decision;
- changed surfaces;
- regression/acceptance coverage;
- tests/CI actually run;
- what is and is not proven;
- migration/rollback;
- next verified frontier.
