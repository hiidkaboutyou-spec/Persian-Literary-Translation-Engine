#!/usr/bin/env bash
set -euo pipefail

VERSION="5.3.0"
ARCHIVE="epubcheck-${VERSION}.zip"
URL="https://github.com/w3c/epubcheck/releases/download/v${VERSION}/${ARCHIVE}"
SHA256="6c07e68584b2e2ce2f89fe06e1246dfead3eb36b46b340e7d93524f29dcff6c5"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
INSTALL_DIR="${EPUBCHECK_HOME:-${REPO_ROOT}/.tools/epubcheck/${VERSION}}"

for command in curl unzip; do
  if ! command -v "${command}" >/dev/null 2>&1; then
    echo "Required command not found: ${command}" >&2
    exit 1
  fi
done

if [[ -f "${INSTALL_DIR}/epubcheck.jar" ]]; then
  echo "EPUBCheck ${VERSION} is already installed at ${INSTALL_DIR}"
  exit 0
fi

tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT
archive_path="${tmp_dir}/${ARCHIVE}"

curl --fail --location --proto '=https' --tlsv1.2 "${URL}" --output "${archive_path}"

if command -v sha256sum >/dev/null 2>&1; then
  echo "${SHA256}  ${archive_path}" | sha256sum --check --status
elif command -v shasum >/dev/null 2>&1; then
  actual_sha="$(shasum -a 256 "${archive_path}" | awk '{print $1}')"
  if [[ "${actual_sha}" != "${SHA256}" ]]; then
    echo "EPUBCheck checksum mismatch: expected ${SHA256}, got ${actual_sha}" >&2
    exit 1
  fi
else
  echo "Cannot verify EPUBCheck download: sha256sum or shasum is required" >&2
  exit 1
fi

extract_dir="${tmp_dir}/extract"
mkdir -p "${extract_dir}"
unzip -q "${archive_path}" -d "${extract_dir}"

jar_path="$(find "${extract_dir}" -type f -name 'epubcheck.jar' -print -quit)"
if [[ -z "${jar_path}" ]]; then
  echo "Downloaded EPUBCheck archive does not contain epubcheck.jar" >&2
  exit 1
fi

source_dir="$(dirname "${jar_path}")"
mkdir -p "$(dirname "${INSTALL_DIR}")"
rm -rf "${INSTALL_DIR}"
mv "${source_dir}" "${INSTALL_DIR}"

if [[ ! -f "${INSTALL_DIR}/epubcheck.jar" ]]; then
  echo "EPUBCheck installation did not produce ${INSTALL_DIR}/epubcheck.jar" >&2
  exit 1
fi

echo "Installed EPUBCheck ${VERSION} at ${INSTALL_DIR}"
if ! command -v java >/dev/null 2>&1; then
  echo "Note: Java is required to run EPUBCheck and was not found on PATH." >&2
fi
