# NVIDIA Acceleration Architecture

## Goal

Keep the Persian Literary Translation Engine Rust-first while allowing optional NVIDIA acceleration for heavy AI workloads.

## Design Principle

The Rust core remains responsible for:

- translation workflow
- memory retrieval
- glossary rules
- character consistency
- quality checks

GPU systems are acceleration providers, not the source of truth.

## Future Pipeline

Rust Core

-> Inference Adapter

-> NVIDIA Runtime Options

- TensorRT / TensorRT-LLM based inference
- embedding generation acceleration
- batch processing

## Use Cases

### Local Mode

Run supported local models with GPU acceleration.

### Batch Translation

Process large projects faster while preserving ordering and memory consistency.

### Embeddings

Accelerate semantic memory indexing.

## Security Rules

Never send private manuscripts to external GPU services without explicit user configuration.

## Implementation Boundary

Create a Rust trait:

GpuInferenceProvider

Possible implementations:

- CPU provider
- NVIDIA provider
- Cloud provider

The translation engine remains portable.
