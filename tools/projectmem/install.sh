#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VENV_DIR="${PROJECTMEM_VENV:-${ROOT_DIR}/.venv/projectmem}"
REQUIREMENTS="${ROOT_DIR}/tools/projectmem/requirements.txt"
EXPECTED_VERSION="0.3.3"
UPSTREAM_RELEASE_COMMIT="e8d73137acde6f091ef6196f88ba5eccf6eb0e8a"

pick_python() {
  local candidate
  for candidate in python3.12 python3.11 python3.10; do
    if command -v "${candidate}" >/dev/null 2>&1; then
      printf '%s\n' "${candidate}"
      return 0
    fi
  done

  if command -v python3 >/dev/null 2>&1; then
    if python3 - <<'PY'
import sys
raise SystemExit(0 if (3, 10) <= sys.version_info[:2] <= (3, 12) else 1)
PY
    then
      printf '%s\n' python3
      return 0
    fi
  fi

  return 1
}

PYTHON_BIN="$(pick_python || true)"
if [[ -z "${PYTHON_BIN}" ]]; then
  echo "projectmem requires a supported Python 3.10-3.12 interpreter for this project integration." >&2
  exit 1
fi

"${PYTHON_BIN}" -m venv "${VENV_DIR}"
"${VENV_DIR}/bin/python" -m pip install --disable-pip-version-check --upgrade pip >/dev/null
"${VENV_DIR}/bin/python" -m pip install --disable-pip-version-check -r "${REQUIREMENTS}"

"${VENV_DIR}/bin/python" - <<PY
from importlib.metadata import version
actual = version("projectmem")
expected = "${EXPECTED_VERSION}"
if actual != expected:
    raise SystemExit(f"projectmem version mismatch: expected {expected}, got {actual}")
print(f"projectmem {actual} installed in ${VENV_DIR}")
print("upstream release commit: ${UPSTREAM_RELEASE_COMMIT}")
PY

"${VENV_DIR}/bin/pjm" --help >/dev/null
