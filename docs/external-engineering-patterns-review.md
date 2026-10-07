# External Engineering Patterns Review

## Purpose

Track reusable engineering patterns from mature open-source projects without copying incompatible implementations.

## Adopted Principles

### 1. Parser isolation

Document ingestion should remain isolated from literary interpretation and translation execution.

Adoption:
- document parsing owns structure extraction
- literary intelligence owns meaning analysis
- translation runtime owns execution

### 2. Pipeline boundaries

Large systems should expose stable boundaries between stages.

Adoption:
- explicit contracts between engines
- no hidden cross-layer mutation
- deterministic validation points

### 3. Quality gates

Validation should happen before publishing and deployment.

Adoption:
- formatting checks
- linting
- tests
- security checks
- container validation

### 4. Dependency discipline

External libraries should be added only when they improve reliability without breaking architecture.

Decision rule:
- prefer mature maintained dependencies
- avoid unnecessary coupling
- keep domain logic independent

## Future Evaluation Targets

- document format adapters
- testing strategies
- benchmark infrastructure
- observability patterns
- release automation
