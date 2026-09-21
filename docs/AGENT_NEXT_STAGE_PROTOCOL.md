# Deep Next-Stage Execution Protocol

## Purpose

This protocol defines what a request such as **"do the next stage"**, **"continue the project deeply"**, or an equivalent instruction means for this repository.

The goal is **maximum safe coherent progress toward the product goal**, not the smallest possible diff. A next-stage run should complete a substantial, internally coherent slice of the real project frontier while preserving existing invariants, compatibility, privacy, and reviewability.

This is a developer/agent workflow only. It must never become a runtime dependency of the literary engine.

## Core rule

Choose the **largest coherent stage that can be researched, implemented, validated, and converged safely** from the repository's actual current frontier.

Do not:
- invent a phase from memory;
- skip over unfinished prerequisite PRs;
- optimize for a cosmetic or documentation-only change when a larger implementation slice is safely achievable;
- bundle unrelated improvements merely to make the diff larger;
- claim completion when meaningful exit criteria remain open.

A large stage may span multiple modules and tests. When reviewability requires it, use a small stack of explicitly dependent PRs rather than one unsafe mega-diff.

## 1. Recover authoritative context first

Before substantive work:

1. Read `AGENTS.md`, `README.md`, relevant architecture/roadmap/status documents, and the affected code/tests.
2. Load PMC/project memory and developer-side projectmem when available and permitted.
3. Verify the real GitHub state:
   - current default-branch HEAD;
   - open PRs and their bases/heads;
   - stacked dependencies;
   - required CI/check status;
   - recent task-relevant commits.
4. Treat current repository/GitHub evidence as implementation authority. Memory is context, not proof.
5. If durable memory and GitHub disagree, record the mismatch and follow verified GitHub state.

Never choose a next stage from phase numbering alone.

## 2. Identify the real frontier

Build a short frontier map:

- what is canonical on `main`;
- what is implemented but still non-canonical in open/stacked PRs;
- which exit criteria remain incomplete;
- which blockers or regressions exist;
- which roadmap item becomes reachable only after those prerequisites.

If a non-canonical prerequisite is already substantially implemented, prefer completing, validating, or safely extending that chain over starting an unrelated later phase.

## 3. Deep research is mandatory when it can change the design

Before implementation, research any material uncertainty. Prefer primary sources:

- upstream repository source and release history;
- official documentation/specifications;
- exact licenses for code, data, models, and bundled assets;
- security advisories and dependency metadata;
- relevant standards;
- measured repository-local benchmarks and existing research notes.

For a proposed external dependency/tool/model, explicitly assess:

1. exact problem/gap it solves;
2. whether the repository already solves the same problem;
3. adopt vs adapt-ideas-only vs reject vs defer;
4. maintenance/activity and release recency;
5. license compatibility, including model/data rights separately from repository code;
6. security advisories and transitive risk;
7. Apple Silicon / supported-platform impact;
8. runtime/build/CI size and resource cost;
9. privacy/network behavior;
10. failure modes and rollback/removal path.

Do not add a dependency because it is popular or appears in a list.

## 4. Scope selection: largest safe coherent slice

Select the stage that maximizes product progress subject to all of these:

- one clear product outcome;
- bounded architectural surface;
- prerequisites known;
- regression surface testable;
- migration/compatibility story defined when persisted contracts change;
- no unresolved security/license/privacy blocker;
- no hidden dependence on unavailable credentials or proprietary manuscript text;
- reviewable diff or explicit stacked-PR decomposition.

Prefer an end-to-end vertical slice over many disconnected foundations.

The stage should normally include implementation **and** the tests, migrations, docs, validation, and operational evidence needed to make that implementation trustworthy.

## 5. Define success before coding

Record:

- problem and user/product outcome;
- non-goals;
- current evidence;
- architectural boundaries;
- compatibility/invariant constraints;
- task breakdown and dependencies;
- validation matrix;
- exit criteria.

When Spec Kit is available, use the full quality path for substantial work:

`specify -> clarify -> plan -> checklist -> tasks -> analyze -> implement -> converge`

Repeat implementation and convergence until there is no material spec gap.

When Spec Kit is unavailable, perform the same logical stages using repository-native Markdown/tasks. The tool must never be required for normal builds or runtime.

## 6. Implementation rules

- Work from the verified frontier, not a remembered one.
- Preserve existing public/persisted contracts unless the stage includes a tested migration.
- Prefer native Rust in core runtime boundaries.
- Keep optional evidence/model/tooling isolated and failure-safe.
- Add focused tests while implementing, not after the fact.
- Keep every external integration revision/version pinned when repository policy requires it.
- Do not weaken an existing guard merely to make a new feature pass.
- If a discovered prerequisite is small and tightly coupled, include it. If it changes the product goal or creates a separate risk domain, split it into a prerequisite PR.

## 7. Literary-project safety gates

In addition to normal repository rules:

- never upload or commit user manuscripts, generated book translations, private reviewer prose, or credentials;
- never send manuscript text to a new remote service without explicit product design and privacy approval;
- human review authority remains human;
- inferred/model evidence cannot silently become canon;
- publishing/export must remain fail-closed where provenance/completeness is uncertain;
- optional ML/tooling must not become a hidden default runtime requirement;
- rights-safe synthetic/project-owned fixtures are the default for permanent CI.

## 8. Validation is part of the stage

Run the repository's established validation from `AGENTS.md` plus task-specific gates.

For core Rust changes, the default baseline is:

```bash
cd engine
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --release -p literary-engine
cargo run --quiet -p literary-engine -- --help
cargo run --quiet -p literary-engine -- --version
```

Also run every affected phase/format/security/Apple-Silicon gate required by the touched surfaces.

If the current environment cannot execute a gate:
- do not claim it passed;
- inspect available CI evidence;
- make the limitation explicit in the PR;
- leave the change unmerged when the missing gate is required for safety.

## 9. Convergence

After implementation, independently compare the **current code** to the stage's success criteria.

Classify gaps as:
- missing;
- partial;
- contradictory;
- unrequested/risky.

Fix material gaps and repeat validation. Do not close a stage merely because every originally listed task was checked off.

## 10. Durable memory and handoff

At the end of a successful stage:

1. update canonical roadmap/status/architecture docs only where reality changed;
2. record important research/adoption/rejection decisions in repository research docs;
3. update PMC/project memory and projectmem when available and appropriate;
4. never place manuscript/translation/private reviewer content in engineering memory;
5. ensure the PR states:
   - what became true;
   - what did not change;
   - exact validation evidence;
   - migrations/rollback;
   - remaining blockers;
   - verified next frontier.

Avoid a second mutable "current state" file if an existing canonical status/roadmap already owns that information.

## 11. PR and merge policy

- Use a focused branch or explicit stack.
- Never force-push shared work.
- Never bypass required checks.
- Never merge merely because implementation looks complete.
- If a prerequisite PR is not canonical, preserve the dependency explicitly.
- If required CI/security evidence is unavailable or failing, leave the PR open/draft with the blocker recorded.

## Spec Kit integration policy

GitHub Spec Kit is approved here as an **optional developer-side process harness**, not as application code.

Reviewed baseline at adoption:
- upstream: https://github.com/github/spec-kit
- pinned reviewed release: `v1.0.8`
- existing-project guidance: https://github.com/github/spec-kit/blob/main/docs/guides/existing-projects.md
- Codex integration: skills under `.agents/skills`
- substantial-work flow: Specify -> Clarify -> Plan -> Checklist -> Tasks -> Analyze -> Implement -> Converge
- workflow engine can chain steps and resume interrupted runs.

Rules:
- initialize only on a reviewable branch;
- review generated files before adoption;
- do not run `--force` against an unreviewed dirty working tree;
- do not make `specify` a Rust/runtime/build prerequisite;
- upgrade deliberately and review generated diffs;
- repository `AGENTS.md` and canonical architecture/invariants remain higher authority than generic Spec Kit templates.

## Definition of a good "next stage"

A good next-stage run should leave the repository measurably closer to the product goal and in a state another engineer/agent can verify without trusting the previous chat.

It is complete only when the implementation, regression protection, safety evidence, documentation, and durable handoff agree with each other.
