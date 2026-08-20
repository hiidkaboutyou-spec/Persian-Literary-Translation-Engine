# AI Memory Pipeline

## Goal

Create a publishing-quality Persian literary translation assistant that remembers project decisions without replacing the translator/editor.

## Memory Layers

### 1. Translation Memory
Stores approved source/target segments.

### 2. Glossary
Stores controlled terminology and preferred translations.

### 3. Character Memory
Stores voice, personality, dialogue patterns and emotional rules.

### 4. Style Memory
Stores formatting, narration tone and project-specific rules.

### 5. Semantic Retrieval
Uses embeddings to retrieve relevant previous decisions before generation.

## Retrieval Flow

User text
-> segment analysis
-> glossary lookup
-> translation memory similarity search
-> character/style retrieval
-> generation model
-> quality checks
-> approved memory update

## Design Principle

The system should retrieve previous human-approved decisions instead of blindly generating new translations.
