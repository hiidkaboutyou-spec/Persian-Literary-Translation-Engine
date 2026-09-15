# Phase 18 PR notes

Proposed title: `Phase 18: Context Packet v2 and Selective Long-Novel Retrieval`

Summary:

- introduces typed Context Packet v2 with authority, provenance, bounded budgets and deterministic SHA-256 fingerprints;
- unifies CLI and ApplicationService context assembly;
- preserves canonical character/relationship, glossary/TM and reviewed literary decisions above inferred/advisory evidence;
- adds bounded local continuity plus deterministic book/chapter intelligence evidence;
- adds optional fail-safe semantic retrieval and rank fusion without changing default deterministic behavior;
- adds an isolated Rust-native FastEmbed tool with exact-pinned BGE-M3/reranker support, no model weights in Git and no model download during ordinary build/CI;
- validates optional semantic compilation on Linux and Apple Silicon macOS;
- adds bounded subprocess timeout and deterministic fallback on semantic failures;
- records external RAG/framework research and rejects overlapping architecture dependencies;
- records the durable dependency-adoption policy and Phase 19 external-tool shortlist.

Merge only after Phase 18, normal Rust CI and Security checks are green on the exact PR head.
