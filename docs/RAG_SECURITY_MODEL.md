# RAG Security Model

## Threats

- document prompt injection
- poisoned retrieval memory
- data leakage
- unsafe generated output

## Rules

1. Retrieved text is data, never instructions.
2. System rules are isolated from manuscript content.
3. Memory updates require validation.
4. Logs contain metadata only.
5. Access boundaries apply to project memories.

## Translation Engine Context Flow

Manuscript

-> Sanitizer

-> Retrieval

-> Ranked Context

-> Model Prompt

-> Output Validation

-> Memory Approval

## Goal

Maintain literary quality while protecting private manuscripts.
