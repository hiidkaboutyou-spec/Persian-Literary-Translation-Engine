# Repository guidance

## Project purpose

This repository is a professional English-to-Persian literary translation engine. It must preserve authorial voice, narrative intent, character identity and psychology, emotional impact, dialogue authenticity, relationship evolution, terminology consistency, and cultural meaning while producing publication-oriented Persian.

The product flow is:

```text
Literary Intelligence
        ↓
Translation Runtime
        ↓
Quality Gate
        ↓
Human Review
        ↓
Publishing
```

This project is independent from every other repository. Do not import assumptions, code, architecture, terminology, or project memory from the Jeonghan Daily Review Bot or any unrelated project.

## Architecture rules

- Rust is the primary implementation language. The active workspace, CLI, tests, and release build live under `engine/`.
- Keep document ingestion, segmentation, literary/project memory retrieval, provider execution, revision, deterministic quality checks, artifact generation, and RTL DOCX/EPUB publishing as explicit boundaries.
- Keep translation providers replaceable and provider-neutral; credential-free deterministic execution must remain available.
- Treat glossary, character bible, relationship context, and translation memory as durable literary intelligence with stable, documented schemas.
- Quality gates must report evidence and must not silently rewrite or accept degraded output.
- Use Python only for document parsing, model-backed analysis/evaluation, document tooling, or isolated developer tooling when a Rust solution is impractical. Keep every Python adapter isolated from the Rust domain model and orchestration.
- Preserve compatibility of persisted project memory, runtime manifests, CLI behavior, and published artifacts unless a migration path is included.
- Human review remains an explicit stage; automation must not claim literary approval on a reviewer's behalf.
- Heavy models and external tools must remain optional unless a numbered roadmap phase explicitly promotes them after benchmark, licensing, resource, privacy, and failure-mode review.

## Approved external integrations

- BookForge is the approved structured EPUB boundary. Keep `bookforge-core` and `bookforge-epub` revision-pinned as documented in `docs/EXTERNAL_INTEGRATIONS.md`; map their IR into native `document-engine` types and never persist BookForge types in project schemas.
- When the BookForge feature is enabled, a BookForge EPUB validation/parsing failure is actionable and must not silently fall back to the legacy parser. The legacy path exists only for explicit builds without the feature. Malformed archive/decompression preflight failures may be normalized to the public `CorruptedFile` contract, but validation must remain strict.
- COMET/XCOMET/DocCOMET are optional external quality evidence only. Keep them behind the isolated `quality-engine::comet` process boundary; do not import PyTorch/COMET into the Rust runtime, auto-download models in default CI, or use a COMET score as human approval.
- Lingua is approved only as optional English/Persian diagnostic evidence. Keep the crate exactly pinned as documented, disable its default all-language feature set, enable only English/Persian models, and keep `language-diagnostics` off by default. Lingua must not change deterministic quality results, rewrite text, reject intentional multilingual prose by itself, or act as human approval.
- EPUBCheck 5.3.0 is the Phase 20 publication-conformance gate because the product currently targets the stable W3C EPUB 3.3 Recommendation. Do not vendor its distribution. Keep the installer version/checksum pinned and install only under ignored local tool storage. EPUBCheck 5.4.x evaluates EPUB 3 against EPUB 3.4 rules; treat it as future-compatibility research while EPUB 3.4 remains a Candidate Recommendation unless a deliberate standards migration is approved.
- `projectmem` 0.3.3 is approved only as optional **developer-side coding memory**, as documented in `docs/PROJECT_MEMORY_INTEGRATIONS.md`. It is not translation/runtime memory and must never be imported or invoked by production Rust paths. Use the repository's safe initialization profile: no Git hooks, no watcher, no history backfill, no global-memory inheritance, and no automatic `AGENTS.md`/`CLAUDE.md` edits. Its absence or failure must never block build, translation, review, or export.
- PMC and projectmem have different ownership: PMC is the intended curated durable engineering knowledge vault; projectmem is operational issue/attempt/fix/decision history. Canonical repository docs remain authoritative when either local tool is unavailable. Never put manuscripts, generated book translations, credentials, or private reviewer material in projectmem.
- ContextWeaver is an architecture reference, not a dependency. Stable IDs, bounded selective context, resume fingerprints, review history, and canon ownership stay native to this repository unless a future gap analysis proves otherwise.
- Do not copy code from `TranslateBooksWithLLMs`; selective glossary injection is already native in `memory-engine`, and any licensing change must be deliberate.
- TransAgents may inform agent-role separation but is not a runtime dependency. Provider/model judgments remain separate from deterministic quality checks and human review.
- BGE-M3 is approved only through the optional Rust/FastEmbed boundaries introduced for Phase 18/19. Model weights are never downloaded by normal builds or default CI, deterministic native behavior remains available without the model, and BGE evidence never owns canon or human approval.
- Phase 19 literary review stays in the native `literary-review-engine` plus the optional `tools/literary-alignment` process boundary. Source/target alignment is monotonic, bounded, schema-validated, and advisory. A failed/missing aligner must not block translation, mutate canon, or be interpreted as a clean review.
- Hazm is **blocked**, not approved, while its required NLTK dependency is affected by an unpatched security advisory. Reconsider only after a patched compatible NLTK release exists and a fresh dependency/security audit passes. Do not add an advisory waiver merely to enable Hazm.
- Vecalign and SentWeave are Phase 19 research references only. Their alignment design may inform native code, but do not add their Python/Cython stacks while the native Rust aligner plus the already-approved BGE boundary satisfies the measured requirement.
- DadmaTools remains conditional on a measured Persian NLP gap after native review/benchmarking. SacreBLEU 2.6.0/chrF2++ is approved only through the optional Phase 21 benchmark sidecar over project-supplied references; it must not download external corpora through project workflows or act as literary approval.
- Serena, Global Agent Memory, MemoryWiki, and automatic chat-history memory systems are project-memory research references only unless a new gap analysis changes that decision. Do not install multiple overlapping memory systems by default.

## Phase 20 publication invariants

- The publication target is EPUB 3.3 until a deliberate standards migration says otherwise.
- Preserve source structure by reconstructing through the revision-pinned BookForge model; do not regenerate EPUB from flattened chapter text.
- Persist and use explicit source block provenance. Publication mapping is `BookForge block ID -> translated text`.
- Native project/application code owns completeness for literary translation units: every translated EPUB heading/paragraph must match its source block provenance exactly, and missing/mismatched native provenance must fail closed. Never guess by paragraph position or count.
- The lower `document-engine`/BookForge boundary validates only explicit mappings it receives: unknown IDs, duplicate IDs, empty translated blocks, damaged marker tokens, or invalid rebuilt structure fail closed. BookForge-only package/navigation/page-furniture blocks that are not native literary translation units may remain source-derived.
- Structural marker tokens such as `<m...>` / `</m...>` and `<r.../>` are immutable translation placeholders. Translation/revision/review must preserve marker identity, count, pairing, and relative order.
- BookForge owns source-aware XHTML reconstruction and target `dc:language` / XHTML `lang` / `xml:lang` rewriting. The native Phase 20 layer owns only the missing RTL publication metadata (`dir="rtl"` on XHTML roots and `page-progression-direction="rtl"` on the OPF spine).
- Images, CSS, navigation, links, footnotes/endnotes, and non-translatable resources remain source-derived and must survive round trip unless a documented publication transformation explicitly owns them.
- Publication export must be deterministic on the same source/artifacts, and exporting must not mutate the imported source EPUB.
- The default translation style remains `literary`. The `adult-intimacy` fidelity profile is explicit opt-in only and requires caller confirmation that every participant in sexual content is an adult. Never infer this confirmation. The profile preserves source explicitness/markedness, consent/refusal/coercion and power cues, agency/referents, sensory channels, POV, emotional intensity, and pacing; it must flag both sanitization and amplification and must not sexualize nonsexual source text. Its review evidence is never canon or human approval.

## Phase 21 benchmark invariants

- Committed literary benchmark source/reference text must be project-owned/rights-safe or have explicit dataset-level redistribution rights recorded in repository research.
- A GitHub repository license does not automatically license underlying third-party novels, subtitles, tweets, or other corpus material.
- Deterministic benchmark anchors are narrow regression assertions, not overall literary-quality scores.
- Every claimed contrastive failure dimension must have at least one anchor/evidence path that the deliberately degraded variant actually fails.
- Human scorecards remain a separate evidence channel and record reviewer expertise. Keep each evaluation pass focused; Phase 21 caps a case/pass at four dimensions.
- Reference metrics such as chrF2++ and neural metrics such as COMET/XCOMET remain advisory evidence. Never set a universal score threshold that marks literary output human-approved.
- Normal Rust build/test must not download external corpora or model weights. Optional benchmark sidecars stay isolated and failure-safe.
- Do not add a generic Persian NLP stack until the Phase 21 benchmark demonstrates a concrete failure class that the current native/evidence stack cannot measure or diagnose.

## Phase 22 desktop invariants

- The desktop UI is an adapter over `project_engine::application::ApplicationService`. Never orchestrate engine crates, project JSON files, canon stores, translation checkpoints, or publication artifacts directly from JavaScript.
- Keep `desktop/src-tauri` outside the core `engine/` Cargo workspace. Tauri/platform WebView dependencies must not become prerequisites for CLI/runtime builds.
- Use only locally bundled frontend assets. Do not add remote pages, CDN scripts/fonts, or a production localhost web server.
- Keep CSP restrictive and never render manuscript/provider/reviewer content with `innerHTML`.
- File/folder selection belongs to the bounded Rust/native-dialog bridge. Do not grant general frontend filesystem access without a separately measured need/security review.
- Provider credentials are session-only unless a future secrets-store phase explicitly designs encrypted OS-native persistence. Never write API keys to project files, frontend storage, logs, Git, PMC, or projectmem.
- Long-running desktop commands must execute outside the UI event loop and call the existing synchronous application facade. Pause/resume/progress semantics remain application-owned.
- Every UI-exposed provider/model option must map truthfully to application configuration; do not expose inert controls.
- Tauri updater remains disabled until signed artifacts and a trusted update endpoint are explicitly configured and verified.
- Public macOS distribution requires code signing/notarization. CI may prove an unsigned/ad-hoc app bundle, but never claim it is a notarized public release.
- Phase 22 is not canonical without a committed independent desktop lockfile and `--locked` validation of its exact dependency graph.

## Optional product-identity skill

- The reviewed `ip-as-logo` Agent Skill is vendored at `tools/agent-skills/ip-as-logo/SKILL.md` from upstream commit `acb834c717bcd0a487c49732d08397ba280d690b`.
- Use it only for an explicitly requested mascot, product-identity, app-icon exploration, or related visual-branding task. It is developer/design guidance, not application runtime behavior.
- Before using it, read `tools/agent-skills/ip-as-logo/SOURCE.md` and the vendored `SKILL.md`. Do not auto-sync from upstream or run an installer.
- The skill must never become a dependency of translation, review, project persistence, EPUB/DOCX publishing, CI core tests, or provider configuration.
- Generated visual candidates are non-canonical until a human explicitly selects them. Do not silently replace the current app icon or product identity.
- Updating the vendored skill requires a new pinned upstream commit plus provenance, license, and security review.

## Coding standards

- Follow idiomatic stable Rust and the module patterns already present in `engine/`.
- Keep domain types explicit, serializable where persisted, and separated from provider and filesystem concerns.
- Prefer deterministic behavior, bounded passage processing, actionable errors, and no hidden global state.
- Avoid `unwrap`/`expect` in production paths when errors can be propagated with context.
- Never log, commit, or embed API keys, proprietary manuscripts, private translations, or reviewer data.
- Update documentation and examples when CLI, configuration, schemas, or output contracts change.
- Add focused unit tests and end-to-end regression coverage for behavior changes.

## Testing and validation

Run from `engine/` before proposing a merge:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --release -p literary-engine
cargo run --quiet -p literary-engine -- --help
cargo run --quiet -p literary-engine -- --version
```

Use non-mutating lockfile verification in CI (`cargo metadata --locked` or an equivalent `--locked` command). Do not use `cargo generate-lockfile` as a freshness check because it may refresh otherwise compatible transitive dependencies and create unrelated diffs/failures.

For pipeline changes, also run a credential-free `EchoProvider` smoke test through ingestion, translation, quality gate, artifact generation, and `manuscript.docx` export. Run `cargo audit` when dependencies change. Test each affected document format and verify Persian RTL output when parsing or publishing code changes. Real provider tests require explicit credentials and must never expose manuscript content or secrets.

For BookForge changes, exercise EPUB ingestion with the default feature and verify an explicit `--no-default-features` document-engine build where practical. For COMET changes, test the JSON protocol without downloading a model in default CI; a real model smoke test requires explicit local setup and any model-specific license/authentication approval.

For Lingua changes, run:

```bash
cargo test -p quality-engine --features language-diagnostics
```

The normal workspace build must still pass with the feature disabled. For EPUBCheck wrapper changes, syntax-check both scripts without downloading the distribution in normal Rust CI. The dedicated Phase 20 publication gate may install the checksum-pinned EPUBCheck 5.3.0 distribution and Java to validate a generated rights-safe EPUB fixture.

For projectmem integration changes, run the dedicated `Project Memory Tooling` workflow. Its smoke test must confirm the selected package version and prove that safe initialization does not create/alter Git hooks, create `AGENTS.md`/`CLAUDE.md`, or start watcher state. Do not make normal Rust CI depend on projectmem.

For Phase 19 literary-review changes, run the dedicated `Phase 19 Literary Review` workflow. Credential-free tests must cover native review, artifact persistence/staleness, provider-schema validation, alignment protocol validation, and deterministic operation with alignment/provider tooling absent. Compile the optional BGE alignment adapter on Linux and Apple Silicon without fetching model weights in CI. Never convert an unavailable provider/aligner into a silent pass.

For Phase 20 publishing changes, run the dedicated `Phase 20 EPUB Round Trip` workflow. It must use only project-owned/synthetic fixture text and assets, validate input/output with EPUBCheck 5.3.0, exercise create -> import -> analyze -> EchoProvider translate -> EPUB export, byte-compare repeated exports, prove the source EPUB remains unchanged, assert language/RTL metadata, preserve representative CSS/image/link/inline markup, run the no-default-features compatibility build, and compile/test publication surfaces on Apple Silicon arm64. The final branch/PR must contain no write-enabled Phase 20 one-shot workflows.

## Forbidden actions

- Do not replace literary translation with unreviewed word-for-word or generic machine translation.
- Do not bypass deterministic quality gates or mark automated output as human-approved.
- Do not silently discard glossary, character, relationship, or translation-memory decisions.
- Do not introduce Python into the core runtime when Rust can reasonably implement the requirement.
- Do not turn optional ML models, external validators, or developer-memory tools into hidden runtime requirements.
- Do not commit manuscripts, generated translations, credentials, private review material, downloaded model checkpoints, or downloaded EPUBCheck binaries.
- Do not put manuscript/translation content into PMC or projectmem as a substitute for native runtime memory.
- Do not guess EPUB structure/provenance when native literary block provenance is incomplete; fail publication export instead.
- Do not break CLI, persistence, manifest, or publishing contracts without migration and documentation.
- Do not force-push shared branches, bypass failing CI/security checks, or mix this repository with another product.

## Development workflow

1. Inspect `README.md`, relevant architecture/roadmap documents, the affected Rust modules, tests, and current GitHub state.
2. Make the smallest coherent change on a focused branch.
3. Add tests for behavior, literary-decision consistency, and regressions.
4. Run formatting, Clippy, tests, build, and task-specific smoke checks; fix failures.
5. Review the diff for secret/manuscript exposure, persistence compatibility, provider coupling, external-dependency provenance, runtime-cost changes, licensing, and quality-gate regressions.
6. Open a concise pull request describing behavior, contracts, and validation.
7. Merge only after required CI and security checks pass and the change is safe; otherwise record the blocker and leave the pull request open.

## Phase 24 literary-precision invariants

- Preserve `text-normalization::normalize()` as the comparison/search normalization contract. Use separate publication-quality Persian APIs for typography work.
- Never auto-rewrite literary punctuation, expressive repeated marks, register, slang or prose style under the label of normalization.
- Native Persian typography diagnostics are evidence only; they cannot mark a translation human-approved.
- A literary-review revision proposal is inert until an explicit human acceptance action. Reject stale, multi-target, structurally ambiguous or non-concrete proposals.
- Accepted review patches must flow through the normal translation revision ledger and must stale old quality/review evidence.
- `rbook` is dev/CI-only independent EPUB evidence. Do not move it into normal document-engine dependencies or replace the pinned BookForge reconstruction boundary.
- Do not add BookNLP, FastCoref, Hazm, DadmaTools or another model/NLP stack to defaults without a rights-safe benchmark proving a concrete gap plus license/security review.
- Do not duplicate existing Context Packet v2 or relevant-only glossary/canon selection when adopting ideas from document-translation research.

## Phase 25 narrative-speaker invariants

- Speaker attribution belongs to `literary-intelligence-engine`; canonical character identity remains owned by `CharacterBible`.
- Resolve only supported high-precision explicit name/alias + speech-verb patterns by default. Do not guess pronoun/coreference or conversational turn-taking.
- Never infer character gender from names or use gender assumptions to resolve `he/she/they`.
- Do not treat a vocative/name inside quoted text as the speaker.
- Preserve the `asked <object>` guard and quote-local cue isolation; cues cannot cross another quotation or a hard pre-quote sentence boundary, and conflicting local character cues fail closed.
- Unresolved/ambiguous dialogue is a valid output. Never turn lack of attribution into a silent pass or fabricated character.
- Context Packet speaker maps have Deterministic authority, not Canonical/HumanApproved authority.
- Keep speaker context bounded; do not dump whole-book dialogue or model traces into provider context.
- Permanent speaker-attribution CI must remain project-owned/network-free unless a future phase explicitly adopts an external benchmark with reviewed rights.
- Do not add BookNLP, ModernBookNLP, FastCoref, Maverick, BookCoref, Renard, Torch, Transformers or spaCy to default dependencies without a new benchmark/license/security decision.
- Any future speaker/coreference sidecar is optional evidence only and must fail without breaking normal translation.

For Phase 25 changes, run the dedicated `Phase 25 Narrative Speaker Intelligence` workflow. It must cover native speaker attribution, project-owned benchmark provenance, zero incorrect resolved labels on the committed corpus, Context Packet integration, locked Linux validation and Apple Silicon compilation/tests.

## Phase 26 coreference-evidence invariants

- Coreference model output is evidence only. It must never create, merge, rename, or mutate CharacterBible canon automatically.
- A model cluster may enter translation context only when explicit canonical-name/approved-alias mentions inside that cluster resolve to exactly one existing canonical character.
- Clusters with no canonical anchor or multiple conflicting canonical anchors fail closed and are omitted.
- Validate schema, source fingerprint binding, globally unique bounded/safe mention IDs, unique bounded/safe cluster IDs, non-reused spans, aggregate mention limits, character-offset bounds, and exact source-slice text before consuming a response.
- Bound sidecar stdin/stdout/stderr execution: concurrent pipe draining, timeout, stdout/stderr byte ceilings, and explicit oversized-output failure are part of the protocol safety contract.
- Canonicalize accepted clusters/mentions by source position before bounded Context Packet rendering; model return order must not control which evidence survives the output budget.
- Coreference Context Packet items have Inferred authority. Do not upgrade them to Deterministic, Canonical, or HumanApproved merely because they contain a canonical anchor.
- Existing non-coreference translation/context paths must remain usable when the optional sidecar is missing, malformed, timed out, or disabled.
- Keep optional sidecars bounded by input size, cluster/mention limits, and timeout. Do not silently send manuscripts to a remote service.
- Permanent Phase-26 CI uses project-owned synthetic data. External corpora/models require explicit code/checkpoint/data license review before use.
- BookCoref/xCoRe/Maverick remain research-only under reviewed non-commercial terms. FastCoref is not approved until exact model/license/security/resource and measured literary long-span gain are reviewed.
- Do not add Torch, Transformers, spaCy, xCoRe, Maverick, BookCoref, FastCoref, CorPipe or another model stack to default dependencies as part of Phase 26.

For Phase 26 changes, run the dedicated `Phase 26 Long-Span Coreference Evidence` workflow plus the existing Rust/Security/Phase regression gates.

## Phase 27 real-book pilot invariants

- Treat Phase 27 as stacked/non-canonical until Phase 26 PR #111 lands and canonical `main` is verified.
- Chapter reuse requires matching source fingerprint, Context Packet fingerprint, and translation-plan fingerprint. Do not weaken this to source-only or source+context-only reuse.
- The translation-plan fingerprint must represent semantic execution choices: resolved provider/model, target language, style profile, versioned pipeline contract. Do not include `max_chapters` or other purely operational budgets.
- Rebuild completed chapter/paragraph progress from currently valid checkpoints on resume. Never increment old persisted paragraph totals for checkpoints being rediscovered.
- `max_chapters` limits newly translated chapters in the current invocation. Reusing a valid checkpoint must not consume that budget.
- Missing/legacy/mismatched plan fingerprints invalidate reuse and trigger regeneration. Never silently mix chapters produced by different provider/model/target/style plans.
- A paused partial run may contain old artifacts on disk, but project status/export authority comes from current-plan progress. Do not mark a mixed partial run complete.
- Permanent Phase-27 CI must use only project-owned synthetic manuscripts and must prove repeated bounded resume reaches exact completion and durable reopen/export.
- Real/user manuscript text and generated translation text must never be copied to GitHub, PMC/projectmem, Linear, CI logs/artifacts, or external evaluation services merely for tracking. Store only non-text operational metadata outside the native project workspace.
- Do not adopt a new document-level metric, workflow engine, cloud persistence layer, translation provider, or model stack in Phase 27 without a separate measured gap and rights/security review.
- Research findings about refinement granularity are advisory. Do not change the EN→FA production pipeline default until project-owned/real-pilot evidence demonstrates improvement.

For Phase 27 changes, run the dedicated `Phase 27 Real-Book Pilot Readiness` workflow plus Rust/Security/Phase 18–26/Desktop/Trusted Release/Project Memory regression gates.
