#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "$script_dir/../.." && pwd)"
venv_dir="${PERSIAN_TRANSLATOR_HAZM_VENV:-$repo_root/.venv/hazm}"

python_bin=""
for candidate in python3.13 python3.12; do
  if command -v "$candidate" >/dev/null 2>&1; then
    python_bin="$candidate"
    break
  fi
done

if [[ -z "$python_bin" ]]; then
  echo "Hazm 0.12.1 requires Python 3.12 or 3.13." >&2
  exit 2
fi

"$python_bin" -m venv "$venv_dir"
"$venv_dir/bin/python" -m pip install --upgrade pip
"$venv_dir/bin/python" -m pip install --requirement "$script_dir/requirements.txt"

cat <<EOF
Installed Hazm diagnostics in:
  $venv_dir

Diagnostic executable:
  $venv_dir/bin/python $script_dir/diagnose.py

No pretrained Hazm model is downloaded by this installer or by diagnose.py.
The sidecar reports advisory surface evidence only and never rewrites translation text.
EOF
