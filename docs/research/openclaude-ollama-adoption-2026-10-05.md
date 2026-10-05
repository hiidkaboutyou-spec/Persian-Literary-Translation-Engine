# OpenClaude / Ollama adoption decision — 2026-10-05

## Problem

The project already documents local inference as a target, but `docs/LOCAL_AI_MODEL_STRATEGY.md`
had no implemented local provider adapter. At the same time, the provider qualification and blind
human-review path already exists and deliberately separates experimentation from production admission.

## External research

OpenClaude was reviewed as an architecture/reference source, especially its provider profiles, local
Ollama path, runtime diagnostics, and no-key local workflow. Its repository license also states that
the tree contains code derived from proprietary Claude Code source and that only contributor
modifications are offered under MIT where legally permissible. No OpenClaude implementation code was
copied into this repository.

The local adapter in this change is therefore an independent implementation based on Ollama's public
native API contract:

- local HTTP service, normally `http://127.0.0.1:11434`;
- `POST /api/chat`;
- `stream: false` for one bounded response;
- explicit `num_ctx` so long-context evaluation is not accidentally performed with a tiny window;
- no API key or Authorization header;
- native prompt/evaluation token counters are used when returned.

## Adopt / adapt / reject

### ADAPT — provider abstraction + no-key local model

Useful. The existing `TranslationProvider` trait is a better fit than importing another agent
runtime, so Ollama is implemented as one more adapter.

### ADOPT — explicit context-window configuration

Useful. Local models can silently run with context too small for literary workloads. The adapter
defaults to 32,768 tokens and exposes `OLLAMA_NUM_CTX`, with fail-closed bounds.

### ADOPT — provider health through qualification rather than model reputation

Useful. `qualify-provider --provider ollama` routes a local model through the same project-owned
rights-safe literary corpus, latency/token telemetry and later blind human review used for other
candidate providers.

### REJECT — OpenClaude runtime/CLI dependency

Not needed. It would add a large Node/TypeScript agent stack to a Rust translation engine, duplicate
provider routing already owned by the project, and create unnecessary licensing/supply-chain
surface.

### REJECT — automatic production fallback to any installed Ollama model

Unsafe. "Local" and "free" do not imply sufficient English→Persian literary quality. The model must
first earn admission through project evidence.

## Implementation

- Added `OllamaProvider` in `translation-core`.
- Uses the native Ollama chat endpoint with separate system/user messages.
- Reuses the production literary pass instructions and context/pass separation.
- Reads no API key.
- Supports `OLLAMA_MODEL`, `OLLAMA_BASE_URL` and `OLLAMA_NUM_CTX`.
- Parses `prompt_eval_count` / `eval_count` into existing provider usage telemetry.
- Added `ollama` to the qualification CLI only.
- CI proves the adapter contract offline on Linux and Apple Silicon.
- CI also proves `ollama` does not enter the production provider selector/capability list.

## Validation target

A real local model should be evaluated with:

```bash
export OLLAMA_MODEL='<installed-model>'
export OLLAMA_NUM_CTX=32768
cargo run --locked -q -p literary-engine -- \
  qualify-provider ../benchmarks/phase21/corpus-v1.json /tmp/ollama-submission.json \
  --provider ollama --format json
```

The deterministic report is evidence, not an admission decision. Compare a complete Ollama
submission against the current candidate with `blind-compare`; production admission remains
`not_granted` until the project's human-review governance is satisfied.

## Rollback

Remove the qualification-only `OllamaProvider`, the `ollama` CLI arm and its CI checks. No
production provider selector, manuscript state, database schema, secret, or default behavior is
changed by this adoption.
