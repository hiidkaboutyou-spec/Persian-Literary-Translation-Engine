# Human preference export from blind literary review — 2026-10-05

## External reference

Reviewed `FareedKhan-dev/train-llm-from-scratch` (MIT), including its preference-data, reward-model and DPO paths.

The useful transferable idea is intentionally small: represent an explicit human preference as a pair with:

- `prompt`
- `chosen`
- `rejected`

The upstream repository also implements reward modeling, DPO/IPO/SimPO/ORPO/KTO, PPO and GRPO, but those algorithms are **not** adopted here.

## Existing project evidence

This engine already has a stronger domain-specific source of preference data than generic public RLHF corpora:

1. Phase-31 candidate submissions produce comparable translations over a rights-safe literary evaluation corpus.
2. `blind-compare` randomizes Candidate A/B presentation and keeps the reveal key separate.
3. human reviewers explicitly record A/B/tie/defer with reasons.
4. Phase-36 binds reveal-key evidence to the exact blind bundle bytes.
5. Phase-37 can authenticate completed reviewer ledgers with SSHSIG.

The missing capability was a conservative, local bridge from that evidence to an ordinary pairwise preference corpus.

## Implemented

Added:

```
literary-engine blind-review export-preferences \
  <blind-bundle.json> <reveal-key.json> <output.jsonl> \
  <ledger.json> [ledger2.json ...]
```

The exporter:

- reuses the existing bundle/reveal-key/ledger validation path;
- requires completed, non-duplicated reviewer ledgers;
- exports a case only when **all supplied reviewers agree** on Candidate A or all agree on Candidate B;
- excludes tie, defer, mixed A/B disagreement and non-distinguishable candidate text;
- reconstructs the chosen/rejected system IDs only after reveal-key validation;
- writes a DPO-compatible `prompt/chosen/rejected` JSONL record plus bounded metadata;
- does not export reviewer identity, reviewer notes or judgment reasons;
- writes a new artifact without silently overwriting prior evidence;
- performs no training, provider call or network request.

## Why not train from scratch

Training a bespoke language model or adding PyTorch/RL infrastructure to the production engine would currently be unjustified:

- this project needs high-quality Persian literary behavior, not proof that a small Transformer can be trained;
- model training would add substantial compute and dependency cost;
- the project already supports replaceable external/local providers;
- there is no measured evidence that a scratch model would outperform qualified providers;
- literary quality still requires blind human review regardless of training objective.

If future evidence justifies fine-tuning a suitable open model, the exported corpus can be used as one privacy/rights-reviewed input. That future experiment must remain isolated from production until benchmarked and explicitly admitted.

## Privacy and rights boundary

Permanent/CI preference export must use only project-owned or explicitly redistributable evaluation cases. Do not export user manuscripts or private pilot-review text into training datasets.

A valid preference pair is comparative human evidence, not:
- canon;
- automatic provider admission;
- proof of overall literary quality;
- authorization to upload the corpus to a third-party training service.
