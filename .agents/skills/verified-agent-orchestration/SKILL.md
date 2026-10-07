---
name: verified-agent-orchestration
description: Evidence-bound planning, agent governance, execution, external-tool admission, memory/decision-model boundaries, and verified completion for long-running engineering work.
---

# Verified Agent Orchestration

Use this skill with `agent-session-safety` for scheduled/hourly runs, broad next-stage work, multi-agent execution, browser-assisted engineering, or decisions about adding an agent harness, MCP server, persistent-memory system, decision model, browser agent, or cluster orchestrator.

This is developer infrastructure only. It must never become translation runtime, literary-memory authority, publishing authority, or a path for private manuscript/reviewer data.

## 1. Separate planning from mutation

For material architecture, dependency, rights, security, privacy, provider, artifact, release, or cost decisions:

1. recover current repository/GitHub/phase truth;
2. define the product outcome;
3. define acceptance evidence, rollback, and a bounded work budget;
4. only then mutate.

Read-only exploration may run in parallel. Exploratory workers must not mutate source, project data, provider state, review ledgers, publishing artifacts, or releases.

## 2. Role, assignment, wake, and authority are different

An agent's title/role, reporting line, task assignment, wake/heartbeat, mention, or prior successful run never grants missing permission.

Before a mutation verify:
- the task and current phase still own the work;
- the actor has the required repository/product authority;
- the target branch/path/project is correct;
- human-only, rights-holder, approval, literary-review, release, and reveal-key boundaries remain intact.

Never delegate to another worker merely to bypass a denied capability. Preserve an unresolved blocker instead of manufacturing authority.

## 3. Bounded budgets and hard stops

Long work must declare task-appropriate limits such as:
- materially different fix attempts;
- provider/model or external service calls;
- browser/live calls;
- parallel workers;
- artifact generations;
- PR/change scope.

Near a soft budget, narrow to the critical acceptance path. At the hard limit, stop new speculative work, preserve evidence, and hand off the blocker. Budget exhaustion never justifies skipping tests or weakening quality/rights/privacy gates.

## 4. Unknown external effects are not failures

For repository writes, provider calls, browser mutations, deployments, releases, artifact uploads, signatures, or other side effects:
- record intent before execution where supported;
- inspect the returned receipt/outcome afterward;
- distinguish success, explicit failure, and unknown outcome;
- never replay a mutation merely because its result is uncertain;
- fence/reconcile the earlier attempt first.

Replaying an audit/evidence record must be side-effect free.

## 5. Browser-assisted work uses observed bounded actions

If browser automation is used for research, UI QA, provider diagnostics, or future import/source tooling:
- actions must target currently observed controls/state;
- supported operation types and compatible targets must be explicit;
- model output must never become arbitrary selectors, coordinates, shell commands, or executable JavaScript;
- target freshness/visibility must be checked before mutation;
- uncertain browser mutations are not retried;
- final success requires an independent outcome verifier; `DONE` is not proof;
- manuscript, credentials, reviewer material, and private project text stay out of generic browser traces.

A paid browser-decision provider must never become required for normal engine operation.

## 6. Decision-model admission gate

Fast typed-decision systems such as Laya are research candidates only.

Before any production/tooling admission:
1. define a specific bounded classification/routing gap;
2. build a rights-safe project-owned benchmark;
3. compare against the current deterministic/current-provider baseline;
4. measure per-class errors, calibration, selective accuracy/abstention, latency, RAM, cold start, and long-input behavior;
5. fail closed or abstain on low confidence;
6. review code/checkpoint/data licenses and model-download supply chain;
7. keep model downloads and heavyweight ML stacks out of default Rust CI/runtime unless a dedicated phase explicitly approves them.

A confidence score can never create canon, approve literary quality, select a winning translation automatically, sign reviewer evidence, or grant provider admission.

## 7. Persistent-memory admission gate

Hindsight-style persistent memory may be evaluated only as shadow evidence.

- Native Character Bible, relationship state, glossary, translation memory, project state, human review, and repository truth remain authoritative.
- External categories such as world/experience/observation are retrieval labels, not authority levels.
- Generated observations/opinions/mental models can never promote themselves into canon.
- Evaluate multi-arm retrieval ideas (semantic/vector, text/BM25, graph/entity, temporal) through the repository's existing retrieval-ranking metrics before adding a service.
- Measure ranking quality, recall, stale-memory behavior, deletion/revocation, latency, storage growth, and cross-chapter contamination.
- Never send manuscript, translation, private reviewer text, or secrets to a hosted memory system by default.
- Any local memory system must remain optional and fail without breaking normal translate/review/export behavior.

## 8. Cluster/sandbox orchestrator admission gate

Paperclip/AX-style orchestration may support developer operations only after a measured need.

Require:
- one authoritative task/state owner;
- isolated filesystem/workspace and explicit network scope;
- resource/cost limits and hard stops;
- approval boundaries and auditable writes;
- resumable/checkpoint semantics with idempotent recovery;
- rollback that restores repository/GitHub/project state as sole authority.

Do not add Kubernetes, Agent Substrate, or another control plane merely to run existing repository work.

## 9. Completion requires independent evidence

An agent's own completion claim is not acceptance evidence.

Before closing substantive work:
- rerun the predeclared acceptance evidence;
- inspect real workflow/artifact results;
- use an independent convergence/adversarial pass for high-impact changes;
- distinguish implemented, exact-head CI verified, artifact/runtime verified, human/editorial reviewed, merged, and release/real-project verified.

## 10. External tool/MCP provenance gate

Before adding a harness, MCP server, memory system, decision model, browser agent, local shell, telemetry/eval platform, trainer, or developer service, record:
- measured gap and adopt/adapt/reject/defer decision;
- canonical upstream and maintenance state;
- version/pin strategy;
- code/model/data rights and licenses;
- authentication/network/telemetry/data scope;
- private-text/secrets exposure;
- model/download/supply-chain behavior;
- runtime/CI coupling and rollback.

Prefer official/first-party integrations when an external connection is actually required. Discovery is not installation approval.

## Provenance

This contract independently adapts public patterns reviewed on 2026-10-05 from:
- `paperclipai/paperclip`: authority/approval separation, bounded budgets, heartbeat/task discipline, audit receipts, no-replay handling for unknown effects;
- `browser-use/jev-ultrafast`: observed indexed actions, target freshness, no mutation retry, independent completion verification;
- `NandhaKishorM/laya`: typed decisions, confidence calibration and abstention;
- `google/ax`: sandbox/workspace fencing, resource limits, suspend/resume;
- `vectorize-io/hindsight`: typed persistent memory and multi-arm semantic/text/graph/temporal retrieval;
- prior reviewed LazyCodex, Avibe, uniTerm, Overmind and official MCP patterns.

No upstream implementation code, private data, or model weights are vendored by this skill.
