#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
VENV_DIR="${PROJECTMEM_VENV:-${ROOT_DIR}/.venv/projectmem}"
PYTHON_BIN="${VENV_DIR}/bin/python"

if [[ ! -x "${PYTHON_BIN}" ]]; then
  echo "projectmem virtual environment is missing. Run tools/projectmem/install.sh first." >&2
  exit 1
fi

cat <<EOF
[mcp_servers.projectmem]
command = "${PYTHON_BIN}"
args = ["-m", "projectmem.mcp_server"]
EOF
