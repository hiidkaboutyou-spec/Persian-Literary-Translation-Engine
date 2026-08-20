# Rust Only Development Policy

The core application is implemented in Rust.

Goals:

- Keep the translation engine native and fast
- Avoid dependency on scripting languages for core logic
- Build maintainable modules
- Keep parsing, memory, pipeline, and export systems in Rust

External services may be connected through APIs, but the application logic remains Rust-first.
