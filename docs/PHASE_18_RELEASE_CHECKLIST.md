# Phase 18 Release Checklist

Phase 18 may merge only when all items below are satisfied on the pull-request head.

- [ ] Context Packet v2 is the shared CLI/ApplicationService context policy.
- [ ] Deterministic retrieval remains functional with semantic retrieval disabled or unavailable.
- [ ] Semantic ranking is opt-in, bounded, provenance-preserving, and cannot invent candidate IDs.
- [ ] Semantic subprocess execution is timeout-bounded and failure falls back to deterministic retrieval.
- [ ] BGE-M3 / BGE reranker build is exact-version locked; normal workspace builds do not download model weights.
- [ ] Apple Silicon `macos-15` compile check passes for the optional BGE tool.
- [ ] Context/source/canon fingerprints invalidate stale resume state safely.
- [ ] Character, relationship, glossary/TM, literary review, chapter/book evidence and local continuity preserve authority/provenance.
- [ ] Long-book/repeated-name/negation/relationship/timeline/terminology regression coverage is green or explicitly represented by equivalent existing regression tests.
- [ ] `cargo fmt`, Clippy `-D warnings`, affected tests, full workspace tests, `cargo audit`, release build and CLI smoke are green.
- [ ] Security workflow is green.
- [ ] No temporary write-enabled workflow remains in the repository.
- [ ] No manuscript, translation, credentials, model weights or private review data are committed.
- [ ] Roadmap/status/PMC docs are updated with branch-aware state before merge and canonical state after merge.
