# External agent/tooling adoption review — 2026-10-05

Scope: evaluate the supplied repositories/resources for the Persian Literary Translation Engine against its existing Rust architecture, privacy/rights boundaries, human-review authority, publishing workflow, and developer harness.

## Harness state recovered from current GitHub truth

Open PR #148 contains a mature repository-owned evidence/session/production-acceptance harness and its CI validator, but those harness files are not yet canonical on current `main`. Current `main` still has the project-next-stage protocol and extensive phase invariants, but not the #148 evidence/session skill set.

This branch ports only the focused harness surfaces that remain applicable onto the current `main`, then adds the new external-tool orchestration policy. It does not merge #148 wholesale or import unrelated/stale branch changes.

## Decision matrix

| Source | Useful pattern | Decision | Why / boundary |
| --- | --- | --- | --- |
| `code-yeongyu/lazycodex` | explicit plan -> execution handoff, durable progress, independent completion verification | **Adapt pattern** | The engine already has AGENTS, project memory, next-stage protocol and phase-specific gates. A second harness would duplicate state/hooks and increase orchestration ambiguity. |
| `avibe-bot/avibe` | local-first sessions/tasks/watches and durable task history | **Defer integration; developer-only concept** | It may be useful on a workstation but must not become translation runtime, publishing authority, or a route for manuscript data. |
| `nzbdav-dev/nzbdav` | streaming/seek/cache architecture | **Reject for current scope** | NZB/WebDAV media streaming does not solve a measured ingestion, translation, review, EPUB/DOCX/PDF, or desktop gap. Upstream also declares the project no longer maintained. |
| `tianma-if/edgeever` | developer knowledge base, MCP CRUD, revision history | **Defer / optional developer-only** | It must not replace Character Bible, glossary, translation memory, project state, or native project memory. AGPL-3.0 code is not vendored. |
| `ys-ll/uniterm` | read/write/dangerous command classes, approvals, auditability, bounded execution | **Adapt pattern** | Useful safety semantics without adding a terminal dependency to Rust/CI/runtime. |
| `overmind-core/overmind` | trace -> dataset -> evaluation -> optional training lifecycle | **Adapt evaluation lifecycle only** | Root/server is AGPL-3.0; SDK/CLI subtree is MIT. No external telemetry or training service is added. Any future use must be rights-safe/text-free by default and benchmark-gated. |
| `yihui-dev/awesome-opus5-5-videos` / osp.fyi | high-quality prompt examples for motion graphics/explainers | **Defer to future UI/demo/marketing work** | Useful visual inspiration, not translation-core evidence. No reason to couple it to engine/runtime. |
| `VoltAgent/official-mcp-servers` | official/first-party MCP discovery | **Adopt as provenance policy** | MCP servers can read/act on data; official ownership, scopes, license, secrets, network behavior and rollback must be reviewed before connection. No server is installed merely from the list. |

## Concrete implementation

This branch:
1. restores the mandatory harness files that `AGENTS.md` already references;
2. restores CI validation so those contracts cannot silently disappear again;
3. adds `verified-agent-orchestration` for plan/mutation separation, independent completion evidence, tool provenance, action risk classification, and trace/eval privacy boundaries;
4. adds no translation/runtime dependency.

## Explicit non-changes

- No Rust dependency, provider, model, sidecar, database, MCP server, terminal, knowledge base, telemetry SDK, trainer, manuscript upload path, or publishing schema is added.
- No Character Bible/glossary/translation-memory authority changes.
- No private text is exported.
- No production provider admission is granted.
- No phase is skipped.

## Revisit criteria

A direct integration requires a measured gap plus comparative evidence for maintenance, license/rights, security, privacy, platform cost, architecture fit, quality/reliability gain, and rollback.


## 2026-10-05 follow-up: Paperclip, Jev Ultrafast, Laya, AX, Hindsight

| Source | Useful pattern | Decision for this engine | Boundary |
| --- | --- | --- | --- |
| `paperclipai/paperclip` (MIT) | role/authority separation, approvals, budgets/hard stops, heartbeats, audit receipts, conservative no-replay semantics | **Adapt strongly** | Developer-agent governance only. Do not add Paperclip as a second project/task/persistence authority. |
| `browser-use/jev-ultrafast` (MIT) | observed indexed actions, freshness/occlusion validation, no mutation retry, independent verifier | **Adapt browser contract; defer runtime** | Useful for UI QA/research. It requires external TypeSafe/text-model credentials for its live path and has DOM coverage limits; never make it a translation dependency. |
| `NandhaKishorM/laya` (Apache-2.0 code) | multilingual choice/score/yes-no, confidence calibration, abstention | **Research benchmark only** | Python package pulls Torch/Transformers/Safetensors/Hugging Face checkpoints. No measured literary routing gap currently justifies this stack in production. |
| `google/ax` (Apache-2.0) | sandbox/workspace fencing, resource limits, suspend/resume | **Adapt concepts; reject direct install** | Current upstream is pre-stable and requires Kubernetes + Agent Substrate; disproportionate for this desktop/Rust product and repository workflow. |
| `vectorize-io/hindsight` (MIT) | world/experience/observation memory types, semantic/BM25/graph/temporal recall, reranking | **Shadow retrieval research only** | Native Character Bible/glossary/translation memory/human review already own authority. Test multi-arm retrieval through existing retrieval-ranking metrics before considering another memory service. |

### Hindsight vs current literary memory

The strongest transferable idea is multi-arm recall, not an external source of truth. The engine already has authority-aware literary memory and has added offline retrieval-ranking metrics before RAG expansion. Therefore any Hindsight-inspired experiment should implement or simulate additional retrieval arms behind the existing metrics first. It must prove better ranking/recall without stale or cross-character contamination before an external service is considered.

Hindsight-style generated observations, opinions, or mental models must remain inferred evidence and can never mutate Character Bible canon or human-review authority.

### Laya admission criteria

Laya becomes relevant only if a concrete bounded decision problem appears (for example review triage or routing) where deterministic logic/current providers are measurably inadequate. Admission requires a rights-safe labeled corpus, per-class error analysis, calibrated abstention, latency/RAM/cold-start measurements, long-input testing, checkpoint/data license review, and proof that low-confidence output fails closed.

No Laya/Torch/Transformers/Hugging Face dependency or checkpoint is added by this review.

### Jev browser contract

Any future browser-assisted QA/research must act only on current observed controls, validate target freshness before input, never retry an uncertain mutation, and independently verify the final state. A model emitting `DONE` is not evidence that a DOCX/EPUB/UI/provider task succeeded.

### Paperclip/AX governance boundary

The useful features are approval/authority separation, budgets, audit receipts, workload fencing and resumability. The engine does not need another control plane. Paperclip, AX, Kubernetes and Agent Substrate remain outside runtime/CI unless a future scale/isolation benchmark demonstrates a concrete gap.

### Additional runtime/dependency changes

None. This follow-up adds governance and admission rules only; it adds no model, server, browser runtime, cluster, database, or manuscript data path.
