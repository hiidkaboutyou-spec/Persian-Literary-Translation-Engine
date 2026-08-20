# Local AI Model Strategy

## Goal

Support private translation workflows where users may want local inference.

## Architecture

Rust Translation Engine

-> Model Provider Interface

-> Local Model Adapter

-> Optional GPU Acceleration

## Requirements

- preserve translation memory
- preserve character voice
- keep glossary enforcement
- support offline workflows

## Future Components

- model loader
- embedding service
- inference scheduler
- GPU resource manager

## Rule

Local models are an option. The core engine must remain independent from any single model vendor.
