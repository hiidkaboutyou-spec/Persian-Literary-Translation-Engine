# Phase 33 Research — Provider Admission Governance

## Purpose

Phase 32 ends with comparative human evidence and deliberately emits `production_admission: not_granted`. Phase 33 adds the next missing safety layer: a deterministic, fail-closed admission assessment that checks whether a candidate provider has enough governance evidence to be *eligible for a separate owner decision*. It does not authorize a provider, change `LITERARY_ENGINE_PROVIDER`, add a production provider, or send a private manuscript anywhere.

## Why this is the next stage

The Phase 32 handoff explicitly requires a later stage to address privacy/retention, data handling, rights, reliability/failure behavior, cost/limits, and explicit project-owner authorization. Human literary preference alone cannot answer those questions.

NIST AI RMF 1.0 and the Generative AI Profile frame deployment as a lifecycle risk-management decision rather than a benchmark-only decision. NIST's AIRC likewise emphasizes testing, evaluation, verification, validation, governance, privacy, security, reliability, and documentation. Phase 33 uses that principle narrowly: quality evidence and operational/governance evidence are separate inputs, and missing governance evidence is a blocker rather than an invitation to infer favorable terms.

References:
- NIST AI RMF: https://www.nist.gov/itl/ai-risk-management-framework
- NIST AI RMF Generative AI Profile (NIST AI 600-1): https://nvlpubs.nist.gov/nistpubs/ai/NIST.AI.600-1.pdf
- NIST AI Resource Center: https://airc.nist.gov/

## Atria-specific due diligence snapshot — 2026-09-23

Official Atria API documentation currently establishes useful technical facts: `Atria-Dawn-Preview` is text-only, exposes a 256K context window, supports Chat Completions / Messages / Responses, documents a 1–65,536 output-token limit, uses API keys, and documents rate-limit behavior including HTTP 429 and `Retry-After`.

Official source reviewed:
- https://api.atria-asi.ai/docs

However, the reviewed public API documentation did **not** establish the project-critical hosted-service answers needed before private book text could be sent: exact prompt/output retention period, whether API inputs/outputs may be used for training or service improvement, deletion procedure/SLA, data residency/subprocessors, incident-response commitments, confidentiality/rights treatment for unpublished manuscripts, or a stable published pricing/cost contract. The correct state for those fields is therefore `unknown`, not an inferred `acceptable`.

This is especially important because the hosted API and the open-weight model are different deployment choices. An open-weight license does not itself describe the hosted API's data handling. Phase 33 does not conflate them.

## Admission profile schema

A governance profile must identify the provider/model, review date, terms/privacy evidence locations, and evidence records for:

- data retention;
- training/service-improvement use;
- data residency;
- deletion process;
- incident response;
- reliability and failure behavior;
- rate limits;
- cost limits;
- rights and confidentiality.

Each item is explicitly `acceptable`, `unacceptable`, or `unknown`, plus a non-empty evidence note. Unknown is a first-class state.

## Fail-closed behavior

`provider_admission_cmd.rs` validates that the input dossier still preserves the Phase 32 non-admission contract (`automatic_winner = null`, human-comparative-evidence-only, explicit admission required). The candidate provider must be one of the systems actually represented in that dossier. Any `unknown` or `unacceptable` governance item blocks eligibility.

Even when every item is acceptable, the output remains:

- `production_admission: "not_granted"`;
- `requires_explicit_owner_authorization: true`;
- `selector_changed: false`.

This intentionally creates a two-key boundary: evidence can make a provider eligible for an owner decision, but evidence cannot impersonate owner authorization.

## Integration boundary

Phase 33's governance core is compiled and exercised by a dedicated integration test using `#[path = "../src/provider_admission_cmd.rs"]`. It is intentionally **not wired into the normal production CLI selector in this phase**. That prevents an assessment artifact from becoming a backdoor provider activation path. A later explicit owner-authorization/runtime-integration phase may expose a command only after the project owner makes the decision and all required evidence is acceptable.

## Privacy and manuscript boundary

- No private manuscript is committed, logged, or transmitted by Phase 33.
- No Atria/OpenAI inference call exists in the Phase 33 module.
- No API key or provider environment variable is read.
- No provider selector is changed.
- Admission artifacts are local metadata/evidence only.
- Output creation refuses to overwrite an existing admission artifact.

## Exit criteria

1. Admission assessment core compiles and passes on Linux and Apple Silicon.
2. Existing Phase 32 blind-review tests remain green.
3. Static CI proves the Phase 33 module contains no provider execution/selector/API-key references.
4. Unknown governance evidence demonstrably blocks Atria eligibility.
5. A provider absent from the Phase 32 dossier is rejected.
6. Production admission remains impossible in this phase.
