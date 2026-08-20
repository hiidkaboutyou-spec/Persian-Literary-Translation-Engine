# Hybrid Memory Search Architecture

## Goal

Improve literary translation quality by retrieving previous approved decisions before generation.

## Search Strategy

Use multiple signals:

1. Exact terminology lookup
- Glossary
- Character names
- Fixed translations

2. Keyword retrieval
- Important phrases
- Dialogue patterns
- Previous chapters

3. Semantic retrieval
- Similar scenes
- Emotional context
- Character behavior patterns

## Pipeline

Input segment

-> glossary lookup
-> BM25/full text search
-> vector similarity search
-> ranking/fusion
-> context package for Rust translation engine

## Privacy

Never send complete private manuscripts to telemetry.
Keep ownership boundaries through database policies.

## Rust Boundary

The Rust core owns orchestration. Storage providers are adapters.
