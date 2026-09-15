#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
TARGET_ROOT="${1:-${ROOT_DIR}}"
VENV_DIR="${PROJECTMEM_VENV:-${ROOT_DIR}/.venv/projectmem}"
PJM="${VENV_DIR}/bin/pjm"

if [[ ! -d "${TARGET_ROOT}" ]]; then
  echo "projectmem target does not exist: ${TARGET_ROOT}" >&2
  exit 1
fi

if [[ ! -x "${PJM}" ]]; then
  bash "${ROOT_DIR}/tools/projectmem/install.sh"
fi

cd "${TARGET_ROOT}"

# Deliberately disable every projectmem behavior that can alter repository
# workflow or run continuously. Project memory is developer-side support only;
# the translation engine must remain fully functional without it.
"${PJM}" init \
  --no-hooks \
  --no-global \
  --no-watch \
  --no-backfill \
  --no-claude-md \
  --no-mcp-config

# projectmem may build .projectmem/structure.json from repository paths. That
# cache is derived/read-only with respect to source code and remains gitignored.
# Its project registry entry is machine-local metadata used for MCP routing.

echo "projectmem initialized in safe mode at ${TARGET_ROOT}/.projectmem"
echo "No git hooks, watcher, history backfill, global-memory inheritance, or AGENTS/CLAUDE bridge was enabled."
