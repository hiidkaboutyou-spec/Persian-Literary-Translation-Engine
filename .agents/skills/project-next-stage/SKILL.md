---
name: project-next-stage
description: Execute the next substantial safe stage of the Persian Literary Translation Engine with deep research, implementation, validation, convergence, and durable handoff.
---

# Project Next Stage

Use this skill when the user asks to "do the next stage", "continue deeply", "advance the project", or equivalent without prescribing a narrow implementation.

Read `AGENTS.md` and `docs/AGENT_NEXT_STAGE_PROTOCOL.md` first. Those files are authoritative.

## Required behavior

1. Reconstruct current truth from the repository and GitHub before choosing work.
2. Load available project memory, then verify it against `main`, open/stacked PRs, CI, roadmap, and code.
3. Determine the real frontier and select the **largest safe coherent slice**, not the smallest diff and not an arbitrary future phase.
4. Perform external research where it can affect architecture, dependencies, licensing, privacy, security, standards, platform support, or quality.
5. Compare alternatives; prefer adapting ideas over adding dependencies when the repository already has the needed capability.
6. Define exit criteria and a validation matrix before implementation.
7. Implement the complete vertical slice, including tests/migrations/docs needed for trustworthiness.
8. Run required validation. Never report an unexecuted gate as passing.
9. Perform a convergence review against the intended outcome. Fix material missing/partial/contradictory gaps and repeat.
10. Update durable repository/project memory with decisions and verified state, without storing manuscript/private translation content.
11. Open/maintain a reviewable PR or explicit PR stack. Do not bypass failing checks or prerequisite branches.

## Autonomy

Do not ask the user to choose routine engineering details that can be resolved from evidence. Make the best supported technical decision and record the rationale.

Ask/stop only when an irreducible product choice, unavailable authorization/secret, legal/rightsholder decision, or safety boundary cannot be determined from repository evidence.

## Size target

Aim for a stage large enough to deliver a meaningful product capability or close a real roadmap milestone. It may touch multiple modules. Do not combine unrelated work just to increase size.

When the safe frontier is blocked, spend the stage closing the blocker comprehensively rather than pretending to advance past it.

## Tool use

Use connected tools/plugins when they materially improve evidence or execution:
- GitHub for repository truth and changes;
- web/primary upstream sources for current external research;
- project memory/PMC/projectmem when available;
- other plugins only when relevant to the stage.

Tool availability must not become a production dependency.

## Final handoff

Report:
- verified frontier used;
- substantial capability completed;
- research/adoption decisions;
- files/surfaces changed;
- tests/CI actually executed and results;
- blockers or non-canonical dependencies;
- exact next frontier.

Do not describe a stage as complete if required convergence or validation remains unresolved.
