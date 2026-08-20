# Production Pipeline V1

## Goal

Build a usable Persian literary translation engine with Rust as the core runtime.

## Pipeline

```text
Source Document
      |
      v
Document Ingestion
      |
      v
Chapter Segmentation
      |
      v
Character + Voice Memory
      |
      v
Glossary Retrieval
      |
      v
Translation Context Builder
      |
      v
Translation Provider
      |
      v
Quality Review
      |
      v
Publication Export
```

## Non-negotiable Requirements

- Rust remains the core implementation language.
- Character voice must remain consistent across chapters.
- Glossary decisions must be reusable.
- Translation memory must persist between sessions.
- Output must target professional Persian manuscripts.

## Current Priority Order

1. Real document ingestion (PDF/EPUB/DOCX)
2. End-to-end CLI workflow
3. Persistent memory integration
4. Quality evaluation layer
5. Publishing export
