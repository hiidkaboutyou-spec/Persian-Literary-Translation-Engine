# Specialized Intimacy Translation Engine

## Goal

Create a dedicated translation subsystem for intimate, romantic and mature scenes without mixing its rules with the general translation engine.

## Architecture

Main Engine

|
+-- Literary Translation Brain
|
+-- Character Voice Brain
|
+-- Intimacy Translation Brain

## Intimacy Brain Responsibilities

- scene context analysis
- emotional intent preservation
- relationship dynamic tracking
- terminology consistency
- tone matching
- sensitivity and quality checks

## Memory Layers

1. Intimacy Glossary
- approved terminology
- style decisions

2. Scene Memory
- previous similar scenes
- pacing decisions

3. Relationship Memory
- character boundaries
- emotional history

4. Voice Memory
- how each character expresses affection or conflict

## Important Rule

This module does not replace literary judgment. It provides specialized context retrieval while the main Rust pipeline controls the final workflow.

## Rust Design

Trait example:

IntimacyContextProvider

Possible implementations:
- local memory
- Supabase vector storage
- future GPU accelerated retrieval

## Security

Private manuscripts remain project-scoped. Do not expose stored private text through logs or telemetry.
