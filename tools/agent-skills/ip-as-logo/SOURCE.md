# ip-as-logo vendored source

- Upstream: https://github.com/s1dashu/ip-as-logo-skill
- Pinned commit: `acb834c717bcd0a487c49732d08397ba280d690b`
- Vendored SKILL.md Git blob: `391a7dc3214a34a4bc2c5e5c77ba40cef5709c19`
- Vendored LICENSE Git blob: `234cc2396ef9ed37c4ea1d8c6026ed2e3b6001f6`
- License: MIT
- Reviewed: 2026-09-18
- Runtime role: none
- Product role: optional developer/design guidance for mascot and product-identity exploration only

## Boundary

This skill is intentionally vendored as text instead of installed through an npm/CLI installer. The reviewed upstream revision contains one instruction document plus showcase assets and no scripts, hooks, executable code, runtime library, or model dependency. Only the instruction document and its MIT notice are vendored here.

The literary translation engine, desktop runtime, CI core gates, persistence, publishing, and provider paths MUST NOT depend on this skill. Its absence or failure must never affect translation or publication behavior.

Agents working on an explicitly requested mascot/product-identity task may read `SKILL.md` and use an available image-generation capability. Generated visual candidates are design artifacts, not canonical product identity, until a human explicitly selects one.

Do not automatically sync from upstream. Any update requires a new provenance/license/security review and a new pinned commit.
