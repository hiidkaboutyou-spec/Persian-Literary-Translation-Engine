# AI Runtime Architecture

## Goal

Build a Rust-first literary translation runtime that can use cloud AI and optional local acceleration.

## Provider Boundary

Rust owns:

- context assembly
- memory retrieval
- glossary enforcement
- character voice rules
- quality checks

AI providers only generate responses.

## OpenAI Path

Adapter based integration:

Rust Core -> AI Provider Trait -> OpenAI Adapter

Rules:

- never hardcode keys
- never log manuscripts
- isolate user documents from system instructions

## NVIDIA Path

Optional local acceleration layer:

Rust Core -> Local Inference Adapter -> NVIDIA Runtime

Possible uses:

- embedding generation
- local models
- batch processing
- private offline workflows

## Design Principle

Cloud models improve quality.
Local acceleration improves privacy and cost control.
The engine must support both.
