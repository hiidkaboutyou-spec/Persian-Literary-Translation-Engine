# Phase 18 Branch State

Branch: `phase-18-context-packet-v2`

Status: implementation complete enough for pull-request validation; not canonical until merged and post-merge verified.

Current branch scope:

- Context Packet v2 with typed kinds, authority, provenance, bounded budgets and SHA-256 fingerprints.
- Shared context assembly for CLI and `ApplicationService`.
- Canonical character/relationship, glossary/TM, reviewed literary findings, deterministic manuscript intelligence, local previous/next continuity and book/chapter evidence represented through the packet.
- Optional fail-safe semantic retrieval using the existing deterministic candidate pool plus rank-based fusion.
- Rust-native optional semantic tool using exact-pinned `fastembed 6.1.0` with BGE-M3 and BGE reranker support.
- Model weights are not vendored and are not downloaded by normal engine build/CI.
- Semantic failures, missing executable/model, invalid protocol data, and bounded execution timeout fall back to deterministic retrieval rather than blocking translation.
- Dedicated Ubuntu and Apple Silicon macOS compile gates for the optional semantic tool.
- Research decisions for RAG frameworks and Phase 19 candidate tooling recorded separately.

This file must be replaced or folded into canonical status/roadmap documentation after merge; do not cite it later as evidence that Phase 18 is already on `main`.
