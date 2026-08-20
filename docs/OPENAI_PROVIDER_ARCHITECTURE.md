# OpenAI Provider Architecture

## Goal

Integrate AI capabilities without coupling the Rust core to one provider.

## Design

Rust core owns:

- translation workflow
- context assembly
- memory retrieval
- quality checks

AI providers are adapters:

Rust Engine
  |
  +-- OpenAI Provider
  +-- Future Providers
  +-- Local Models

## Security Rules

- API keys are configuration secrets, never source code.
- Imported manuscripts are treated as data, not instructions.
- Prompts and retrieved context are separated.
- Logs must not contain private manuscript content.

## Translation Flow

1. Load scene context
2. Retrieve glossary and memory
3. Build structured request
4. Call provider
5. Validate output
6. Store approved decisions
