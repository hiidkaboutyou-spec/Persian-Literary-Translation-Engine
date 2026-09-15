#!/usr/bin/env bash
set -euo pipefail

VERSION="5.3.0"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
INSTALL_DIR="${EPUBCHECK_HOME:-${REPO_ROOT}/.tools/epubcheck/${VERSION}}"
JAR="${INSTALL_DIR}/epubcheck.jar"

if [[ $# -lt 1 ]]; then
  echo "Usage: $0 <book.epub> [EPUBCheck options...]" >&2
  exit 2
fi

if [[ ! -f "${JAR}" ]]; then
  echo "EPUBCheck ${VERSION} is not installed. Run tools/epubcheck-validator/install.sh first." >&2
  exit 2
fi

if ! command -v java >/dev/null 2>&1; then
  echo "Java is required to run EPUBCheck but was not found on PATH." >&2
  exit 2
fi

exec java -jar "${JAR}" "$@"
