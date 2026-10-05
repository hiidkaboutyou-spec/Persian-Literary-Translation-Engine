---
name: verified-agent-orchestration
description: Evidence-bound planning, execution, tool provenance, and completion rules for long-running or external-tool-assisted engineering work.
---

# Verified Agent Orchestration

Use this skill together with `agent-session-safety` for scheduled/hourly runs, broad next-stage work, multi-step agent execution, or decisions about adding external agent/MCP/developer tools.

This is developer infrastructure only. It must never become a translation-runtime dependency, a manuscript memory authority, or a path for private manuscript/reviewer content.

## 1. Separate planning from mutation

For work with material architecture, dependency, rights, security, privacy, provider, artifact, or release impact:

1. recover current repository/GitHub/phase truth;
2. define the user-visible product outcome;
3. build a decision-complete checklist with acceptance evidence and rollback;
4. only then enter mutation.

Read-only exploration may be parallel. Exploratory workers must not mutate source, project data, provider state, publishing artifacts, or releases.

## 2. Keep a resumable execution receipt

Long work must leave a compact durable receipt in the existing repository/project-memory surface with only:

- product goal;
- canonical main SHA;
- active branch/PR stack and phase prerequisites;
- completed/open checklist items;
- strongest observed evidence;
- blocker, rollback, and exact next action.

Do not create a second product/runtime state store for agent progress. Never store manuscript text, translations, reviewer notes, credentials, provider prompts containing private prose, or reveal-key material in agent receipts.

## 3. Completion requires independent evidence

An agent's own "done" status is not acceptance evidence.

Before closing substantive work:
- rerun the predeclared acceptance evidence;
- inspect actual workflow/artifact results;
- use an independent convergence/adversarial pass for high-impact changes;
- distinguish implemented, exact-head CI verified, artifact/runtime verified, human/editorial reviewed, merged, and release/real-project verified.

If evidence fails, continue from the first failed or unproven checkpoint.

## 4. Risk-classify execution

Reuse `agent-session-safety` classes:
- read-only;
- repository write;
- external side effect;
- destructive/irreversible.

Writes require repo/branch/path/phase verification. External/destructive actions require explicit authorization plus project-specific privacy/rights/acceptance rules. Autonomous/bypass modes must never silently bypass dangerous actions.

## 5. External tool and MCP provenance gate

Before adding or connecting a harness, MCP server, local agent shell, knowledge base, telemetry/evaluation platform, model trainer, or developer service, record:

- measured product/engineering gap;
- adopt / adapt-pattern / reject / defer;
- canonical upstream owner/source and maintenance state;
- version/pin strategy;
- code/model/data license and rights impact;
- authentication, network, telemetry, and data scope;
- manuscript/private-review exposure risk;
- runtime/CI coupling;
- rollback/removal path.

Prefer first-party/official provider servers and official registries when MCP is actually needed. A directory entry is discovery evidence, not installation approval.

Do not install a second orchestration harness when repository-native planning, project memory, safety, and verification already solve the need.

## 6. Trace -> evaluation -> training boundary

Operational traces can support evaluation only when intentionally text-free or rights-safe.

- Never send user manuscript/translation/reviewer prose or secrets to external trace/eval/training services by default.
- Prefer project-owned synthetic fixtures, deterministic metrics, and explicit human-reviewed evaluation artifacts.
- Traces may identify failure classes and benchmark cases; they do not grant model/provider admission.
- Fine-tuning is a separate phase requiring rights-safe data, benchmark evidence, license/privacy/security review, rollback, and human literary evaluation.
- No automatic production model replacement from trace-derived scores.

## 7. Local-first tools stay developer-only

A local agent OS, terminal, note system, or remote control layer may assist development but must not become:
- a production translation dependency;
- Character Bible / canon / glossary / translation-memory authority;
- publishing/export authority;
- mandatory CI infrastructure;
- a route that copies manuscript/private review data outside the project workspace.

Failure of optional developer tooling must not block normal engine operation.

## Provenance

This contract independently adapts useful public patterns observed on 2026-10-05 from:
- `code-yeongyu/lazycodex`: plan/execute separation, durable progress, independently verified completion;
- `avibe-bot/avibe`: local-first session/task/watch concepts and durable task history;
- `ys-ll/uniterm`: explicit read/write/dangerous execution classification and auditing;
- `overmind-core/overmind`: traces -> datasets -> evaluations before training;
- `VoltAgent/official-mcp-servers`: official/first-party MCP provenance preference.

No upstream implementation code is vendored by this skill.
