# Rust Translation Engine Architecture

## Core principle

The engine is not a simple machine translator. It is a literary translation system with memory, style analysis and quality control.

## Crates

- translator-core: orchestration pipeline
- memory-engine: translation memory and retrieval
- character-engine: character voice and psychology profiles
- quality-engine: consistency and review checks

## Pipeline

Source Document
-> Parser
-> Scene Analyzer
-> Knowledge Retrieval
-> Character Context
-> Translation Generation
-> Quality Review
-> Publication Export

## Research influences

The design borrows ideas from Rust-native translation engines and local inference systems while keeping literary adaptation as the main goal.
