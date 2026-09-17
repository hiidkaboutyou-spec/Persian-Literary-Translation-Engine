# Persian NLP Ecosystem Research — 2026-09-17

This document records the repository's durable adoption decisions for Persian-language GitHub projects reviewed for the English-to-Persian literary translation engine.

The governing rule is **gap first, dependency second**. Popularity or breadth is not enough. A tool enters the runtime only when it closes a measured capability gap and survives license, maintenance, security, platform, privacy, failure-mode, and quality review. Heavy Python/model stacks remain optional sidecars unless a numbered roadmap phase deliberately promotes them.

## Executive decision

Do **not** install a generic "Persian NLP stack" into the product core. The engine already owns normalization, long-novel context, character/relationship canon, semantic retrieval, literary review, quality evidence, document reconstruction, and human approval boundaries. Adding overlapping toolkits would create duplicate ownership and can damage literary voice through aggressive normalization/formalization.

Current decisions:

- **Hazm — blocked.** Useful toolkit, but its active dependency path requires NLTK while the reviewed NLTK advisory `GHSA-8mgp-746c-j5xp` / `CVE-2026-81726` has no patched version as of this review. Do not waive the advisory. Re-evaluate only after a compatible patched NLTK path exists and a fresh audit is green.
- **ParsBench — Phase 21 reference/candidate, not installed.** Functionally valuable for Persian-aware evaluation, but current ParsBench 0.3.0 directly depends on Hazm 0.12.1 and NLTK, so it currently inherits the same security blocker. Re-evaluate in Phase 21 after the dependency path is safe.
- **DadmaTools — conditional optional sidecar.** Apache-2.0 and substantially more current than older Persian NLP stacks. Its NER/POS/dependency/ezafe-style signals could become useful diagnostic evidence, but the package is large and model-heavy. Integrate only if the Phase 21 corpus proves a measurable review gap that native Rust + the existing BGE boundary cannot close.
- **Parsivar — reference only.** MIT, but older Python toolkit, direct overlap with existing normalization/tokenization responsibilities, and the NLTK/security path makes runtime adoption unjustified.
- **Persian-text-preprocessing — reject as runtime dependency.** MIT but currently tied to an old Python/Hazm/Parsivar/NLTK stack. Its informal-to-formal transformation is particularly unsafe as a default literary preprocessing step because it can erase character register and colloquial voice.
- **FarsNet wrapper/database — pending separate data-license review.** Wrapper code is MIT, but the FarsNet lexical database is documented separately under CC BY-SA. Do not bundle lexical data or promote it into canon until redistribution/share-alike obligations and dataset provenance are explicitly reviewed.
- **Virastyar — no direct code integration.** GPL-3.0 and architecturally tied to an older desktop/Office ecosystem. It may be a conceptual reference for Persian orthography/editing rules, but copying code/rules/data requires separate license review.
- **ParsBERT — benchmark/reference only.** Apache-2.0 ecosystem and useful as a Persian-specific semantic baseline, but it is a heavy older BERT runtime and overlaps the approved Rust/FastEmbed BGE-M3 semantic boundary. No second embedding/model stack without a measured advantage.
- **rust-persian-tools — reference/benchmark only for now.** MIT, current Rust implementation, and relevant utilities for Arabic/Persian character conversion and ZWNJ. However, `text-normalization` already owns comparison-safe Persian normalization. Adding a second normalizer now would create conflicting ownership. Reconsider only if a Phase 21/22 benchmark exposes a missing locale utility.
- **Persian stopword packages — do not use in literary source/target processing.** Removing Persian function words can destroy syntax, rhythm, register, polarity cues, and voice. Stopword lists may be used only in isolated research features where deletion cannot alter translation input/output or canon.
- **Synthetic-data generators (PersianSyntheticData / Persian-PromptWright) — research only.** Synthetic examples can help stress-test tooling, but generated data is never reference truth and must not become literary canon or benchmark gold without human provenance review.
- **Persian datasets/catalogs — discovery sources only until dataset-by-dataset rights review.** `farsinlp.github.io`, `Farsi-datasets`, `Persian_NLP_Datasets`, and awesome lists are useful indexes, not blanket licenses. Every corpus needs its own provenance, redistribution, commercial-use, and derivative-work check before entering Phase 21.
- **Persian QA/sentiment projects — out of current scope.** Their task objectives do not directly close a literary translation fidelity gap. Use only as research references if a later benchmark calls for those signals.
- **OCR projects — future ingestion candidates only.** Persian OCR can be valuable for image-only/scanned PDFs, but that is a document-ingestion gap, not a Phase 20 publication dependency. Benchmark on project-owned scans before choosing a tool; keep OCR output provenance explicit and reviewable.
- **ASR/TTS projects and Persian speech datasets — out of current product scope.** Revisit only if audio/transcript ingestion becomes a product requirement.
- **JavaScript Persian date/tools packages — product/UI utilities only.** Do not pull Node/JS packages into the Rust translation engine for capabilities already owned natively.
- **Fonts/RTL framework repositories — Phase 22 UI/distribution scope.** Review each font/license individually. Do not assume a CDN collection has redistribution rights for every bundled commercial-looking font.
- **awesome-persian / awesome-persian-nlp-ir / Awesome-Persian-LLM — research indexes only.** Useful discovery surfaces, never runtime dependencies or provenance substitutes.

## Why aggressive normalization is dangerous here

Literary translation is not generic information retrieval. The engine must distinguish safe comparison normalization from destructive rewriting.

Allowed native comparison normalization can reconcile Arabic/Persian code-point variants, spacing/ZWNJ variation, punctuation noise, and search-equivalent forms **for matching/evaluation**. It must not silently rewrite the source or final translation.

Forbidden default transformations include:

- informal -> formal rewriting;
- stopword deletion;
- stemming/lemmatization of text that will be sent as the literary source;
- spelling "correction" that changes intentional dialect, idiolect, slang, names, code-switching, or voice;
- normalization that collapses negation/polarity or changes quotation/dialogue structure.

Any future Persian NLP sidecar must return bounded evidence with source spans/IDs and must not mutate canon or translated artifacts automatically.

## Candidate matrix

| Project / family | Potential value | License / maintenance signal | Decision |
| --- | --- | --- | --- |
| `roshan-research/hazm` | normalization, tokenization, morphology, POS/dependency | MIT; modern project, but blocked by unpatched NLTK advisory path | **Blocked** |
| `ParsBench/ParsBench` | Persian LLM/evaluation tasks and Persian-aware normalization | Apache-2.0; 0.3.0; Python 3.12/3.13; currently depends on Hazm + NLTK | **Phase 21 candidate after security fix** |
| `Dadmatech/DadmaTools` | NER/POS/dependency/linguistic diagnostics | Apache-2.0; v2.3.6; Python 3.11-3.12; large optional Torch/Transformers stack | **Conditional sidecar after benchmark** |
| `ICTRC/Parsivar` | normalization/tokenization/stemming | MIT; older toolkit and overlapping responsibilities | **Reference only** |
| `Persian-NLP-Toolkit/Persian-text-preprocessing` | normalization/spelling/formalization | MIT; old Python stack; formalization threatens literary register | **Reject runtime** |
| `arashbehmand/farsnet` + FarsNet data | lexical senses/relations | wrapper MIT; database CC BY-SA; provenance obligations separate | **Pending data-license review** |
| `alishakiba/virastyar` | orthography/spell/editing ideas | GPL-3.0; legacy desktop architecture | **Reference only** |
| `hooshvare/parsbert` | Persian semantic/NER baseline | Apache-2.0 family; older/heavy model stack | **Benchmark only** |
| `persian-tools/rust-persian-tools` | Persian chars/ZWNJ/numbers/locale helpers | MIT; current Rust | **Reference until native gap proven** |
| Persian stopword lists | IR/topic-model preprocessing | often GPL or dataset-specific | **Never mutate literary input/output** |
| Persian OCR repos | scanned-PDF ingestion | project-specific; requires separate benchmark/license audit | **Future ingestion candidate** |
| Persian ASR/TTS | audio ingestion/output | model/data licenses vary | **Out of current scope** |
| Persian fonts/RTL frameworks | Persian UI/distribution | license varies per asset | **Phase 22 only** |

## Phase 21 re-evaluation gates

Before adopting any candidate during Literary Evaluation Corpus & Benchmarking:

1. build a rights-safe EN->FA literary evaluation corpus with explicit source/reference provenance;
2. define the exact gap to measure (e.g. ZWNJ/orthography diagnostics, entity/relationship mention recovery, Persian morphology evidence, semantic adequacy, register preservation);
3. compare the candidate against the existing native Rust baseline and existing BGE-M3 optional boundary;
4. require an improvement on the target failure class without degrading voice/register or increasing false-positive review noise;
5. run dependency/vulnerability audit and record transitive model/data licenses;
6. keep Python/model tooling isolated behind a bounded schema/process boundary;
7. no automatic canon promotion or translation rewriting;
8. keep deterministic operation available when the sidecar is absent or fails;
9. validate Linux and Apple Silicon when the tool is promoted beyond research;
10. pin exact versions/commits only after the above passes.

## Durable policy derived from this review

- Never install a Persian NLP repository merely because it is popular or comprehensive.
- Prefer one owner for each responsibility; avoid parallel normalizers, embedding stacks, canon stores, or orchestration layers.
- Treat code license and model/dataset license as separate reviews.
- A catalog/awesome list is not proof that downstream data is redistributable.
- Do not use formalization or stopword deletion as production literary preprocessing.
- Security blockers are not waived for convenience; wait for a patched path or implement the small needed capability natively.
- If a heavy toolkit is useful only for one diagnostic, expose that one diagnostic through an optional sidecar rather than importing the toolkit into the product core.

## Phase 21 corpus-candidate due diligence — 2026-09-18

The four high-interest resources identified for literary/natural-Persian work were inspected at repository level before any download or installation. They remain **research candidates only**:

| Candidate | Observed value | Repository license signal | Phase 21 decision |
| --- | --- | --- | --- |
| `omidkashefi/Mizan` | about 1M Persian-English sentence pairs described as collected from literary works | repository contains `LICENSE.md`: CC BY 4.0 | **Do not bundle or treat as benchmark gold yet.** The repository license is clear, but the parallel text is derived from third-party literary works; verify underlying-text provenance/rights and benchmark suitability separately before redistribution or CI use. |
| `royakabiri/iPerUDT` | 3,000 informal Persian sentences / 54,904 tokens with UD syntax, useful for colloquial-syntax diagnostics | repository `LICENSE`: CC0 1.0 | **Best licensing signal of the four, but diagnostic rather than EN->FA reference data.** Candidate for a bounded Phase 21 Persian-naturalness/syntax diagnostic after provenance and task-fit review; no runtime dependency is justified yet. |
| `mut-deep/Degarbayan-SC` | large colloquial paraphrase corpus described as 1.5M pairs aligned from movie subtitles | repository `LICENSE`: GPL-3.0 | **Research/reference only for now.** GPL on the repository does not by itself establish redistribution rights for third-party movie-subtitle text, and GPL is not a data-specific provenance solution. Do not vendor/download into CI until underlying subtitle rights and dataset terms are resolved. |
| `mojtabasajjadi/FarSSiM` | 1,123 informal Persian short-text pairs from tweets with semantic-relatedness/entailment annotations | no explicit `LICENSE`, `LICENSE.md`, `LICENSE.txt`, or `COPYING` file found in the reviewed repository | **Blocked for project ingestion/bundling without explicit permission/license.** Public GitHub availability is not a reuse license; tweet-source provenance also needs review. |

Practical consequence: Phase 21 should first create a small project-owned/rights-safe literary benchmark schema and scoring harness. External corpora can then be evaluated as optional evidence sources one by one. Do not let corpus availability determine the benchmark design, and do not use third-party literary/subtitle/tweet text as committed fixtures unless redistribution rights are explicit.

## Current roadmap placement

Phase 20 is canonical on `main` (PR #98; merge `1611cd731160c122baa68c9e80c1d4faeb7dfcfc`). This Persian ecosystem review therefore hands off to **Phase 21 — Literary Evaluation Corpus & Benchmarking**.

Mizan, iPerUDT, Degarbayan-SC, FarSSiM and related datasets are **benchmark/corpus candidates, not automatic runtime dependencies**. Before any download, CI installation, bundling, or use as reference truth, Phase 21 must verify dataset-level license, provenance, redistribution/commercial-use terms, source/reference quality, and whether the corpus actually measures the literary failure class in question. Generic Persian NLP packages remain subject to the gap-first/security rules above.
