# Phase 18 Research — Context Packet v2 and Long-Novel Retrieval

Status: branch-scoped research and implementation work for `phase-18-context-packet-v2`. Nothing in this document is canonical on `main` until the Phase 18 pull request is merged and verified.

## Problem statement

The existing engine already owns deterministic translation memory, glossary retrieval, character/relationship canon, reviewed literary findings, checkpoint fingerprints, and project persistence. The missing capability is not another end-to-end RAG framework. The concrete gaps are:

1. a typed and inspectable context packet instead of one opaque context string;
2. explicit provenance and authority for every context item;
3. strict context budgeting for long novels;
4. stable source/canon/packet fingerprints;
5. optional multilingual semantic recall/reranking when lexical overlap is weak;
6. future hierarchical scene/chapter/arc/book summaries without surrendering canon ownership.

## Existing native capabilities that must remain authoritative

- `memory-engine` already owns translation-memory and glossary retrieval.
- `character-engine::CharacterBible` already owns canonical characters, aliases, and relationship context.
- human-review/literary-intelligence layers already own reviewed/canonical findings.
- `project-engine` already invalidates resumable checkpoints when source/context fingerprints change.
- persisted Rust schemas remain the product data contract.

External retrieval systems therefore may rank evidence, but they must not create or silently mutate canon.

## Repositories evaluated

### FIERsity/ContextWeaver — design reference only

Useful ideas:

- typed context packets;
- previous/next local context;
- glossary/entity/reference fields;
- stable IDs and evidence links;
- revision/supersession concepts;
- summary digests for refresh/reuse.

Decision: do not depend on it. It is a young Python system and overlaps the engine's existing orchestration, memory, review, and persistence ownership. Phase 18 adopts the narrow ideas natively in Rust.

### parthsarthi03/raptor — hierarchical-summary reference only

Useful idea: recursive/hierarchical summarization allows retrieval at more than one semantic scale.

Decision: do not import the RAPTOR stack. Literary structure already gives this project safer natural hierarchy boundaries: scene -> chapter -> arc -> whole book. Future summary records should use those explicit structural boundaries, provenance, and refresh fingerprints instead of opaque unsupervised clustering as canon.

### microsoft/graphrag — architecture reference only

Useful ideas: local/global retrieval separation, community-style summaries, and explicit provenance.

Decision: no dependency. GraphRAG adds extraction/orchestration and graph ownership that would duplicate `CharacterBible`, relationship canon, literary review, and project persistence. Its model-driven graph must not compete with human-approved canon.

### OSU-NLP-Group/HippoRAG — retrieval research reference only

Useful idea: multi-hop retrieval over related knowledge.

Decision: no dependency. Its knowledge-graph/PageRank memory model is broader than the measured gap and would introduce a second relationship/entity truth system.

### HKUDS/LightRAG — retrieval research reference only

Useful ideas: hybrid vector/graph retrieval and multiple storage backends.

Decision: no dependency. Storage/orchestration ownership overlaps native project and memory engines.

### qhjqhj00/MemoRAG — research reference only

Useful idea: using long-memory retrieval clues to broaden recall.

Decision: no dependency. Generative retrieval clues can hallucinate associations and are inappropriate as authoritative evidence for canon-sensitive literary translation.

### TsinghuaC3I/LongRAG — chunking/retrieval reference only

Useful idea: larger semantic retrieval units can preserve more local coherence than very small chunks.

Decision: concept only. Phase 18 will prefer structurally bounded scene/chapter evidence and explicit budgets.

### anush008/fastembed-rs — selected optional semantic implementation boundary

Why selected:

- native Rust library, fitting the Rust-first repository;
- Apache-2.0 source license;
- active current release `6.1.0` at research time;
- supports multilingual BGE-M3 and BGE reranker families;
- can use a project-local cache;
- lets us isolate model-backed ranking in a separate tool instead of importing Python/PyTorch into the product core.

Selected integration shape:

- `tools/semantic-retrieval/` is outside the engine workspace;
- `fastembed = 6.1.0` is exact-version pinned and optional behind feature `bge`;
- default tool build does not compile FastEmbed;
- engine/runtime does not require the tool;
- model weights are never committed and are not downloaded by normal workspace CI;
- the sidecar receives only bounded candidate IDs/text and returns rankings for those same IDs;
- the core rejects unknown/duplicate returned IDs;
- raw model scores are advisory; fusion uses rank-based reciprocal-rank fusion rather than assuming cross-model score calibration;
- deterministic retrieval remains the fallback when semantic ranking is absent or fails.

Current FastEmbed BGE-M3 implementation used by the optional tool is the quantized `gpahal/bge-m3-onnx-int8` model exposed by FastEmbed. The selected reranker is `BGERerankerV2M3` as exposed by FastEmbed. Model-weight licensing/provenance must be reviewed separately before enabling model downloads in a distribution or managed environment; source-library licensing alone is not sufficient approval for bundled weights.

## Native Context Packet v2 design

`memory-engine::context_v2` introduces a typed candidate/item packet with:

- schema version;
- stable evidence IDs;
- explicit kind and authority;
- inclusion reason and relevance;
- evidence IDs/provenance;
- strict total/item-count/item-length budgets;
- deterministic ordering and deduplication;
- SHA-256 source fingerprint;
- SHA-256 canon fingerprint;
- SHA-256 packet fingerprint;
- selected/excluded counts and truncation metadata.

Authority precedence is deliberately separate from relevance. Human-approved/canonical evidence must not lose to a high semantic score from advisory memory.

Kinds include glossary, translation decision, character, relationship, local continuity, scene/chapter/arc/book summaries, translation memory, references, and advisory context. These types allow future orchestration to budget classes of context without parsing prompt text.

The old `build_memory_context()` API remains available during migration. Phase 18 must not force a flag-day persistence/runtime migration.

## Hierarchical summary policy

Phase 18 may add summaries only with explicit ownership and evidence:

- scene summaries reference the scene/source units they summarize;
- chapter summaries reference scene/source evidence;
- arc summaries reference chapter/scene evidence;
- whole-book summaries reference lower-level evidence;
- summaries carry source/context digests so stale summaries can be refreshed;
- inferred summaries never outrank human-approved/canonical decisions;
- summaries are context evidence, not silent canon mutation.

## Safety and performance policy

- Never require semantic models for normal translation.
- Never auto-download model weights in default CI or on ordinary engine startup.
- Never send an entire proprietary manuscript to semantic ranking when a bounded candidate set is sufficient.
- Never log manuscript passages, API credentials, model credentials, or private review text.
- Keep semantic candidate count bounded; current protocol hard limit is 512.
- Preserve stable IDs and reject semantic responses that invent IDs.
- On sidecar failure, orchestration must retain deterministic retrieval rather than fail the translation product.
- Benchmark semantic retrieval before enabling it by default in any future product surface.

## Completion criteria for Phase 18

Phase 18 is not complete merely because these primitives compile. Completion requires:

1. Context Packet v2 integrated into the translation orchestration without breaking CLI/application parity.
2. Character/relationship, glossary, TM, reviewed decisions, local continuity, and summary evidence represented with correct authority/provenance.
3. Existing resume/checkpoint invalidation wired to the packet fingerprint or an equivalent deterministic packet-derived context fingerprint.
4. hierarchical summaries persisted/reused with evidence and stale-data detection, or explicitly scoped out with roadmap follow-up if benchmark proves unnecessary;
5. semantic ranking remaining optional and deterministic fallback tested;
6. regression coverage for long novels, repeated names, negation/polarity, relationship changes, timeline changes, and terminology conflicts;
7. full Rust CI/security/audit/release smoke green before merge.
