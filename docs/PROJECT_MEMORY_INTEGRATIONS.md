# Project Memory Integrations

This document records the research and boundaries for developer/agent project memory. It is intentionally separate from `docs/AI_MEMORY_PIPELINE.md` and the Rust runtime memory architecture.

## Memory ownership

The project has three distinct memory domains. They must not be merged into one persistence system.

### 1. Translation runtime memory — product data

Owned by the Rust engine: translation memory, glossary, character and relationship canon, literary findings, review decisions, checkpoints, source/context fingerprints, and project state. This data influences actual book translation and therefore remains under native typed schemas and human-review rules.

### 2. Project Memory Core (PMC) — curated engineering truth

PMC is intended to hold durable, reviewed engineering knowledge in a local Obsidian-compatible vault: architecture decisions, constraints, current state, plans, debugging lessons, and handoffs. `docs/PMC_BOOTSTRAP.md` is the repository-side seed until a local PMC vault is connected.

### 3. projectmem — operational coding history

`projectmem` is an optional developer-side event history for issues, attempts, fixes, decisions, notes, and pre-edit checks. It supplements PMC; it does not replace repository documentation or become authoritative product/runtime state.

## Selected companion: projectmem

Upstream: `riponcm/projectmem`

Selected release:

```text
projectmem==0.3.3
release commit: e8d73137acde6f091ef6196f88ba5eccf6eb0e8a
```

License: MIT.

Why selected:

- purpose-built for persistent coding-agent memory rather than general chat memory;
- explicitly supports Codex and MCP;
- local-first storage with no required cloud service;
- typed operational events distinguish issues, failed attempts, fixes, decisions, and notes;
- pre-edit history checks can prevent repeating a known failed engineering approach;
- small top-level Python dependency surface (`typer`, `mcp`, `watchdog`);
- upstream exposes opt-outs for hooks, watcher, backfill, global memory, bridge-file edits, and MCP-config output;
- can run completely outside the Rust translation runtime.

### Approved boundary

The repository integration under `tools/projectmem/` is optional developer tooling only.

Safe initialization must use the upstream opt-outs for:

- Git hooks;
- background watcher;
- Git-history backfill;
- cross-project/global-memory inheritance;
- automatic `AGENTS.md` / `CLAUDE.md` modification;
- automatic MCP configuration output.

Projectmem may create `.projectmem/` local memory and derived structural data and may register the repository in its machine-local project registry for MCP routing. Those effects are development tooling, not application state.

Projectmem must never:

- be imported or invoked by the Rust product runtime;
- own translation memory, glossary, character canon, review state, or application persistence;
- block a build, translation, review, or export because the tool is absent;
- receive proprietary manuscript text, generated book translations, credentials, API keys, or private reviewer material;
- silently supersede canonical repository docs or reviewed PMC knowledge.

## Research: Serena — reference only

Upstream: `oraios/serena`.

Serena is mature and actively maintained and has useful project-memory ideas, including Markdown memories, project/global scopes, explicit memory references, onboarding, and progressive disclosure. It also provides semantic code retrieval/editing and language-server tooling.

It is not selected as a direct project dependency because:

- current Serena application code from v2 onward is GPL-3.0-or-later, while its SolidLSP component is MIT;
- the full application brings a substantially broader code-intelligence/editing surface than this project needs for memory;
- that surface overlaps with the coding agent itself and would create unnecessary ownership/coupling.

Useful design ideas may be adopted independently without importing Serena application code.

## Research: Global Agent Memory — reference only

Upstream: `ozankasikci/global-agent-memory`.

MIT-licensed and explicitly local-first/project-aware, with Obsidian, SQLite/vector search, and MCP support. It overlaps strongly with the intended PMC role, is considerably newer/smaller than projectmem, and adds more operational/storage surface than required for the immediate problem. Keep as a reference unless a measured PMC/projectmem gap appears.

## Research: MemoryWiki — reference only

Upstream: `MemoryWiki/MemoryWiki`.

The Markdown-native local-first knowledge model is relevant, but the project is currently small and GitHub does not expose a resolved SPDX license for the repository. Do not copy or depend on its code without a separate license/provenance review.

## Research: automatic chat/session memory tools — not selected

Tools centered on automatic transcript/session capture (for example Claude-specific memory layers) solve a different problem. This project needs curated engineering truth plus typed operational coding history, not archival chat history. Automatic transcript capture also increases privacy and secret-retention risk. Do not add such a tool by default.

## Integration files

- `tools/projectmem/requirements.txt` — exact selected package version.
- `tools/projectmem/install.sh` — isolated virtual-environment installer and version smoke check.
- `tools/projectmem/init-safe.sh` — non-invasive initialization profile.
- `tools/projectmem/print-codex-config.sh` — prints, but never edits, local Codex MCP configuration.
- `.github/workflows/project-memory-tooling.yml` — isolated safety/installation smoke test.
- `docs/PMC_BOOTSTRAP.md` — durable PMC seed and current memory-boundary decisions.

## Upgrade policy

Before changing the selected projectmem version:

1. read release notes and security/privacy changes;
2. re-check license and Python requirements;
3. inspect changes to init defaults, hooks, watcher, global memory, bridge-file behavior, MCP routing, storage, and secret redaction;
4. keep safe opt-outs explicit even if upstream defaults change;
5. run the dedicated project-memory tooling CI smoke test;
6. verify that normal Rust CI requires no projectmem installation;
7. record the new version/release commit and rationale here and in `docs/PMC_BOOTSTRAP.md`.

PMC remains the intended curated long-lived knowledge vault. Projectmem remains an optional operational companion.
