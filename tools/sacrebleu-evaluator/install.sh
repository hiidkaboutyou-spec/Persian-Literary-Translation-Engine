#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VENV_DIR="${SACREBLEU_VENV:-$ROOT_DIR/.venv/sacrebleu}"

if [[ -n "${SACREBLEU_PYTHON:-}" ]]; then
  PYTHON="$SACREBLEU_PYTHON"
elif command -v python3.13 >/dev/null 2>&1; then
  PYTHON="python3.13"
elif command -v python3.12 >/dev/null 2>&1; then
  PYTHON="python3.12"
elif command -v python3.11 >/dev/null 2>&1; then
  PYTHON="python3.11"
elif command -v python3 >/dev/null 2>&1; then
  PYTHON="python3"
else
  echo "Python 3.9+ is required for the optional SacreBLEU evaluator." >&2
  exit 1
fi

"$PYTHON" -m venv "$VENV_DIR"
"$VENV_DIR/bin/python" -m pip install --upgrade pip
"$VENV_DIR/bin/python" -m pip install -r "$ROOT_DIR/tools/sacrebleu-evaluator/requirements.txt"
"$VENV_DIR/bin/python" -m pip check
"$VENV_DIR/bin/python" -m py_compile "$ROOT_DIR/tools/sacrebleu-evaluator/evaluate.py"

VERSION="$("$VENV_DIR/bin/python" - <<'PY'
import sacrebleu
print(sacrebleu.__version__)
PY
)"
if [[ "$VERSION" != "2.6.0" ]]; then
  echo "unexpected SacreBLEU version: $VERSION" >&2
  exit 1
fi

cat <<EOF
SacreBLEU evaluator installed in:
  $VENV_DIR

This optional tool computes chrF2++ evidence only.
It does not download external corpora and is not a runtime requirement.
EOF
