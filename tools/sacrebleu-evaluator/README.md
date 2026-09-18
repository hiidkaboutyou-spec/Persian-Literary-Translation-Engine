# SacreBLEU evaluator — Phase 21 optional evidence

This isolated sidecar provides reproducible **chrF2++** reference-overlap evidence
for the literary benchmark. It is intentionally not a literary-quality judge.

Pinned package:

- `sacrebleu==2.6.0`
- Apache-2.0
- Python >= 3.9

The project uses chrF2++ because Persian literary references permit lexical and
word-order variation, while character n-gram evidence is less tokenizer-sensitive
than BLEU. Even chrF2++ remains advisory: a natural, faithful literary translation
may differ substantially from one reference.

Install with:

```bash
bash tools/sacrebleu-evaluator/install.sh
```

The tool reads JSON from stdin and writes JSON to stdout. It never downloads test
sets. Normal build/translation/review/export behavior does not depend on it.
