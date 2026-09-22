# Phase 31 Research — Provider Qualification & Experimental Atria Adapter

Date: 2026-09-23

## Measured gap after Phase 30

The project can now run, resume, review, audit, and manually resolve a full-book pilot without a new model stack. The remaining provider gap is different: there is no safe admission path for evaluating a new model on the project's actual English→Persian literary requirements before exposing it to private manuscripts or making it selectable by the production ApplicationService.

Atria Dawn Preview is a timely candidate, but its public evaluation is centered on research, coding, tool use, delivery, and agentic tasks rather than English→Persian literary translation. Phase 31 therefore builds an experimental qualification path rather than promoting Atria into production.

## Official Atria contract verified

Primary sources:

- https://api.atria-asi.ai/docs
- https://huggingface.co/internlm/Atria-Dawn-Preview

Verified public contract on 2026-09-23:

- API model ID: `Atria-Dawn-Preview` (case-sensitive);
- OpenAI-compatible Responses endpoint: `https://api.atria-asi.ai/v1/responses`;
- Bearer authentication from `ATRIA_API_KEY`;
- 256K context window;
- text-only model input;
- `max_output_tokens` accepted in the range 1..=65,536;
- account-level request-per-minute limiting with HTTP 429 and `Retry-After`;
- public model weights/model card use the MIT license.

The official API documentation explicitly recommends validating correctness, instruction following, latency, and token usage on representative inputs before using the model for a workload.

### Privacy limitation

The public API documentation reviewed for Phase 31 does not establish a manuscript-retention/deletion guarantee suitable for silently sending private books. Therefore Phase 31 does not send user manuscripts to Atria. The only intended network workload is the repository-owned rights-safe Phase-21 benchmark corpus after the user explicitly configures an API key and invokes the qualification command.

## Literary-translation evaluation research

### Automatic metrics and LLM judges are not literary authority

Zhang et al., NAACL 2025, *How Good Are LLMs for Literary Translation, Really? Literary Translation Evaluation with Humans and LLMs*:

https://aclanthology.org/2025.naacl-long.548/

The study reports that automatic metrics perform poorly for literary translation and that a simpler human comparative scheme can be more reliable than complex error taxonomies for distinguishing strong literary translations.

Gerrits, van Noord & Guerberof Arenas, EAMT 2026, *Creativity Bias: How Machine Evaluation Struggles with Creativity in Literary Translations*:

https://aclanthology.org/2026.eamt-1.43/

The paper finds poor alignment between automatic/LLM-as-judge evaluation and professional judgments of literary creativity, including bias against creative and culturally appropriate solutions.

Decision:

- deterministic Phase-21 anchors remain challenge evidence only;
- no Atria-vs-OpenAI winner is computed automatically;
- no LLM judge is added;
- human blind review is required before any production-admission proposal.

### Comprehension and creativity should be assessed separately

Zhang et al., Findings of ACL 2026, *Beyond Reproduction: A Paired-Task Framework for Assessing LLM Comprehension and Creativity in Literary Translation*:

https://aclanthology.org/2026.findings-acl.2030/

The study shows that strong source comprehension does not imply human-level translational creativity and motivates evaluating creative potential separately from basic adequacy.

Decision:

Phase 31 reuses the existing Phase-21 dimensions and human scorecards instead of collapsing fidelity, Persian naturalness, voice, register, subtext, terminology, continuity, readability, and overall literary quality into one model-generated score.

### Document-level and professional evaluation remain important

WMT 2025 General MT and evaluation findings:

- https://aclanthology.org/2025.wmt-1.22/
- https://aclanthology.org/2025.wmt-1.24/

WMT 2025 deliberately increased test difficulty, moved further toward document-level source material, and used professional human evaluation. This supports representative, context-bearing evaluation rather than a tiny isolated sentence smoke test.

## Phase 31 architecture

### AtriaProvider

`translation-core` gains an `AtriaProvider` implementing the existing `TranslationProvider` trait.

Its contract is intentionally narrow:

- reads `ATRIA_API_KEY` only from the process environment;
- optional `ATRIA_MODEL`, defaulting to `Atria-Dawn-Preview`;
- optional bounded `ATRIA_MAX_OUTPUT_TOKENS`, default 8,192 and hard-limited to 65,536;
- calls only the documented Responses endpoint;
- uses the same production literary pass instructions/context contract as the OpenAI provider;
- does not send OpenAI-specific undocumented fields such as `store`;
- parses documented Responses text content;
- surfaces 429 with `Retry-After` context and rejects invalid limits fail-closed.

### Qualification CLI

New command:

`literary-engine qualify-provider <corpus.json> <submission.json> --provider echo|openai|atria [--model <id>] [--max-cases <n>] [--format json]`

Rules:

1. provider selection is mandatory and explicit;
2. the corpus must pass the existing rights-safe Phase-21 schema validation;
3. the default literary translate → revise → quality-review pipeline is exercised;
4. neighboring context is supplied as context, never merged into the passage;
5. candidate output is persisted in the existing submission schema;
6. deterministic anchors are evaluated and reported;
7. partial runs are labeled partial;
8. every report says `production_admission = not_granted`;
9. human blind review remains required.

The deterministic `echo` provider exists in this command so CI can exercise the complete qualification path without credentials or network access.

## Production isolation

Phase 31 deliberately does **not**:

- add `atria` to `ApplicationCapabilities.translation_providers`;
- add Atria to `configured_translation_provider` in project-engine;
- change `auto` provider selection;
- add Atria to the desktop Provider screen;
- add an Atria key to project files or browser storage;
- send private manuscripts to Atria;
- compare providers with an automatic overall winner;
- make benchmark anchors or reference metrics a literary-quality verdict.

A future production-admission phase would require a separate reviewed change after representative human results and privacy/retention terms are acceptable.

## Exit criteria

Phase 31 is complete when:

1. Atria has a separate provider implementation with no production selector wiring;
2. API key/model/output-limit handling is fail-closed and secrets never enter repository state;
3. payload/response/output-limit behavior has offline unit coverage;
4. the rights-safe qualification command works end to end with `echo` in CI;
5. full provider qualification uses the existing Phase-21 corpus and production literary pipeline;
6. partial runs are explicit and never interpreted as admission;
7. reports always require human review and never grant production admission;
8. existing OpenAI/echo automatic production behavior is unchanged;
9. no new runtime dependency is introduced;
10. Linux and Apple Silicon validation pass;
11. Rust CI, Security, Phase 21 evaluation, Desktop Product, Trusted Release, and Project Memory regressions are green on the final head;
12. Atria is not promoted to real-book translation merely because the integration compiles.
