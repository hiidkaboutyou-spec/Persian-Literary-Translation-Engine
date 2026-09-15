# projectmem developer-memory integration

This directory provides an **optional, developer-side** integration with [`riponcm/projectmem`](https://github.com/riponcm/projectmem). It is not linked to the Rust workspace and is not part of document ingestion, translation, quality gating, project persistence, human review, or publishing.

## Why this tool

The project already has three different kinds of memory, and they must stay separate:

1. **Translation runtime memory** — translation memory, glossary, character/relationship canon, literary context, checkpoints, and book/project state owned by the Rust engine.
2. **PMC** — curated durable engineering knowledge intended for a local Obsidian-compatible Project Memory Core vault.
3. **projectmem** — operational coding history: issues, failed attempts, fixes, decisions, notes, and pre-edit history checks for coding agents.

`projectmem` was selected as a companion rather than a new source of product truth. Its event history can help an agent avoid repeating failed engineering approaches, while canonical architecture/state continues to live in repository docs and, when connected locally, PMC.

## Provenance

- upstream: `riponcm/projectmem`
- selected release: `0.3.3`
- release commit: `e8d73137acde6f091ef6196f88ba5eccf6eb0e8a`
- license: MIT
- Python compatibility used here: 3.10–3.12

The package is installed in an isolated `.venv/projectmem` virtual environment. `.venv/` is ignored by Git.

## Install

From the repository root:

```bash
bash tools/projectmem/install.sh
```

The installer prefers Python 3.12, 3.11, then 3.10, verifies that the installed top-level package is exactly `0.3.3`, and runs a CLI smoke check.

## Safe initialization

Use:

```bash
bash tools/projectmem/init-safe.sh
```

Safe initialization deliberately disables:

- Git hook installation
- background file watcher
- Git-history backfill
- cross-project/global-memory inheritance
- automatic edits to `AGENTS.md` or `CLAUDE.md`
- automatic MCP-config output

This keeps projectmem outside the translation runtime and prevents it from silently changing repository workflow. It may still create its normal local `.projectmem/` memory files, a derived structure cache, and a machine-local project-registry entry used for MCP routing.

Raw/runtime projectmem files such as `events.jsonl`, watcher state, logs, and derived structure data should remain uncommitted. Never put manuscript text, translations, API keys, credentials, reviewer-private content, or other secrets into projectmem events.

## Codex MCP connection

After local installation, print the exact Codex configuration block for the current clone:

```bash
bash tools/projectmem/print-codex-config.sh
```

It prints a block of the form:

```toml
[mcp_servers.projectmem]
command = "/absolute/path/to/repo/.venv/projectmem/bin/python"
args = ["-m", "projectmem.mcp_server"]
```

That block belongs in the local Codex configuration. Repository automation intentionally does **not** edit a developer's home-directory Codex configuration.

## Failure boundary

If projectmem is missing, broken, disabled, or unconfigured, the translation engine must still build, test, translate, review, and export normally. Do not introduce imports or subprocess calls from Rust production paths into this tool.

For architecture and selection rationale, see `docs/PROJECT_MEMORY_INTEGRATIONS.md`. For the future PMC vault seed, see `docs/PMC_BOOTSTRAP.md`.
