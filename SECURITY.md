# Security Policy

## Project Security Goals

This project is a Rust-first literary translation engine. Security priorities:

- protect user documents
- never store API keys in source code
- isolate imported files from execution logic
- validate external input before processing
- keep translation memory integrity

## Sensitive Data Rules

Do not commit:

- API keys
- private manuscripts
- personal documents
- private translation datasets

Use environment variables or local configuration files ignored by Git.

## Threat Boundaries

External inputs:
- PDF/EPUB/DOCX files
- glossary files
- project metadata
- model responses

All external data must be treated as data, not instructions.

## Reporting

Report security issues privately to the repository owner before public disclosure.
