#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VENV_DIR="${COMET_VENV:-$ROOT_DIR/.venv/comet}"

if [[ -n "${COMET_PYTHON:-}" ]]; then
  PYTHON="$COMET_PYTHON"
elif command -v python3.11 >/dev/null 2>&1; then
  PYTHON="python3.11"
elif command -v python3.10 >/dev/null 2>&1; then
  PYTHON="python3.10"
elif command -v python3 >/dev/null 2>&1; then
  PYTHON="python3"
else
  echo "Python 3 is required to install the optional COMET evaluator." >&2
  exit 1
fi

"$PYTHON" -m venv "$VENV_DIR"
"$VENV_DIR/bin/python" -m pip install --upgrade pip
"$VENV_DIR/bin/python" -m pip install -r "$ROOT_DIR/tools/comet-evaluator/requirements.txt"
"$VENV_DIR/bin/python" -m py_compile "$ROOT_DIR/tools/comet-evaluator/evaluate.py"

cat <<EOF
COMET evaluator installed in:
  $VENV_DIR

Python executable for the Rust CometSidecar:
  $VENV_DIR/bin/python

Evaluator script:
  $ROOT_DIR/tools/comet-evaluator/evaluate.py

Model checkpoints are intentionally not downloaded by this installer.
Choose a COMET/XCOMET model explicitly and accept any model-specific license
requirements on Hugging Face before first use.
EOF
