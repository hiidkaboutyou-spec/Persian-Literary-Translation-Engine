# Local AI Model Strategy

## Goal

Support private translation workflows where users may want local inference without coupling
the core engine to a vendor or assuming that a free/local model is good enough for literary work.

## Architecture

Rust Translation Engine

-> Model Provider Interface

-> Local Model Adapter

-> Optional GPU Acceleration

The first implemented local adapter is `OllamaProvider`. It is intentionally exposed only through
the rights-safe provider qualification path; production manuscript translation does not select
Ollama automatically.

## Ollama qualification

Configuration:

- `OLLAMA_MODEL`: required unless `--model <id>` is passed.
- `OLLAMA_BASE_URL`: optional, default `http://127.0.0.1:11434`.
- `OLLAMA_NUM_CTX`: optional, default `32768`.
- no API key is read or sent.

Example:

```bash
export OLLAMA_MODEL='<installed-model>'
export OLLAMA_NUM_CTX=32768
cargo run --locked -q -p literary-engine -- \
  qualify-provider ../benchmarks/phase21/corpus-v1.json /tmp/ollama-submission.json \
  --provider ollama --format json
```

The result must still go through the existing blind human-review and provider-admission governance.
A successful HTTP call, deterministic anchor score, or low latency does not grant production
admission by itself.

## Requirements

- preserve translation memory
- preserve character voice
- keep glossary enforcement
- support offline workflows
- keep project context and passage boundaries explicit
- expose local context-window size instead of relying on an implicit small default
- never log private manuscript text as provider diagnostics
- keep production selection fail-closed until qualification evidence exists

## Future Components

- model-specific local qualification profiles based on measured quality
- optional GPU resource diagnostics
- inference scheduling only if real full-book measurements justify it

## Rule

Local models are an option. The core engine must remain independent from any single model vendor,
and local/no-key operation must not bypass the project's literary-quality admission gates.
