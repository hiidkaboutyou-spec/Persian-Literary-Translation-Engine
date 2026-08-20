# Engineering Decisions

## Core

The engine is built around a Rust core.

## Memory

The system keeps separate concepts:

- translation memory
- glossary memory
- character voice memory
- relationship context

## Product Direction

The target is a real user workflow:

Upload a story -> analyze -> translate -> review -> export a professional Persian manuscript.

## Avoid

- simple word replacement
- stateless translation
- losing character voices between chapters
- architecture that prevents future model providers
