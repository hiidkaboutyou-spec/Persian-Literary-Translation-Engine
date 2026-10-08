#!/usr/bin/env bash
set -euo pipefail

usage() {
  echo "usage: $0 <document.docx> <expected-lines.txt> [artifact-directory]" >&2
  exit 2
}

if [[ $# -lt 2 || $# -gt 3 ]]; then
  usage
fi

document=$1
expected=$2
artifact_dir=${3:-}

if [[ ! -f "$document" || ! -s "$document" ]]; then
  echo "DOCX input is missing or empty: $document" >&2
  exit 1
fi

if [[ ! -f "$expected" || ! -s "$expected" ]]; then
  echo "expected-token file is missing or empty: $expected" >&2
  exit 1
fi

for command in soffice timeout pdfinfo pdftoppm pdftotext python3; do
  if ! command -v "$command" >/dev/null 2>&1; then
    echo "required DOCX render validator command is unavailable: $command" >&2
    exit 1
  fi
done

work_dir=$(mktemp -d)
profile_dir=$(mktemp -d)
cleanup() {
  rm -rf "$work_dir" "$profile_dir"
}
trap cleanup EXIT

profile_uri=$(python3 - "$profile_dir" <<'PY'
import pathlib
import sys

print(pathlib.Path(sys.argv[1]).resolve().as_uri())
PY
)

timeout --signal=TERM --kill-after=10s 60s \
  soffice \
  "-env:UserInstallation=$profile_uri" \
  --headless \
  --nologo \
  --nodefault \
  --nofirststartwizard \
  --convert-to pdf \
  --outdir "$work_dir" \
  "$document"

pdf="$work_dir/$(basename "${document%.docx}.pdf")"
if [[ ! -f "$pdf" || ! -s "$pdf" ]]; then
  echo "LibreOffice did not produce a non-empty PDF: $pdf" >&2
  exit 1
fi

pages=$(pdfinfo "$pdf" | awk -F: '/^Pages:/ { gsub(/[[:space:]]/, "", $2); print $2 }')
if [[ ! "$pages" =~ ^[1-9][0-9]*$ ]]; then
  echo "rendered PDF has no readable positive page count: ${pages:-missing}" >&2
  exit 1
fi

text="$work_dir/rendered.txt"
pdftotext -enc UTF-8 -layout "$pdf" "$text"

python3 - "$expected" "$text" <<'PY'
import pathlib
import sys
import unicodedata

expected_path = pathlib.Path(sys.argv[1])
rendered_path = pathlib.Path(sys.argv[2])
expected = [
    unicodedata.normalize("NFC", line)
    for line in expected_path.read_text(encoding="utf-8").splitlines()
    if line
]
rendered = unicodedata.normalize("NFC", rendered_path.read_text(encoding="utf-8"))

missing = [token for token in expected if token not in rendered]
if missing:
    print("rendered PDF text is missing expected tokens:", file=sys.stderr)
    for token in missing:
        print(f"- {token!r}", file=sys.stderr)
    raise SystemExit(1)
PY

if [[ -n "$artifact_dir" ]]; then
  mkdir -p "$artifact_dir"
  cp "$pdf" "$artifact_dir/rendered.pdf"
  cp "$text" "$artifact_dir/rendered.txt"
  pdftoppm \
    -f 1 \
    -l 1 \
    -singlefile \
    -png \
    -r 144 \
    "$pdf" \
    "$artifact_dir/first-page"

  for artifact in rendered.pdf rendered.txt first-page.png; do
    if [[ ! -f "$artifact_dir/$artifact" || ! -s "$artifact_dir/$artifact" ]]; then
      echo "render artifact is missing or empty: $artifact_dir/$artifact" >&2
      exit 1
    fi
  done
fi

echo "DOCX render smoke passed: pages=$pages"
