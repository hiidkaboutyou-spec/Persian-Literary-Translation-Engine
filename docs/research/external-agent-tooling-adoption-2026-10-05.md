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
