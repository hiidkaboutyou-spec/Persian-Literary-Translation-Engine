# Dependency Adoption Policy

This policy is durable project guidance for all future external GitHub repositories, packages, models and tools.

Before adding an external dependency, prove a concrete gap in the existing engine. Then review:

1. maintenance/activity and release provenance;
2. license compatibility for source, bundled data and model weights separately;
3. Rust/Python/platform compatibility, including Apple Silicon where relevant;
4. dependency/runtime cost and whether the capability belongs in core, an optional feature, a sidecar, dev-only tooling or reference-only documentation;
5. privacy boundaries and whether proprietary manuscript/review text leaves the local process;
6. failure behavior, including timeout, malformed output and unavailable model/tool;
7. overlap with native memory, canon, review, persistence or orchestration ownership;
8. reproducible version/revision/checksum pinning;
9. tests, security/audit gates and an explicit fallback path when optional tooling fails;
10. benchmark evidence before making heavyweight/model-backed behavior a default.

Default preference order:

- small native Rust dependency when it provides one narrow, well-maintained capability;
- isolated optional local tool/sidecar when ML or ecosystem maturity makes native implementation impractical;
- development/benchmark-only dependency when it should never affect product runtime;
- design/reference-only use when another project overlaps architecture or has incompatible/unclear licensing.

Never adopt an end-to-end framework merely because it contains a useful subfeature. Preserve native Character Bible, Translation Memory, Glossary, Human Review, project persistence and `ApplicationService` ownership unless a deliberate roadmap migration is approved.

Automated external evidence never equals human approval and must not silently mutate canon or literary output.
