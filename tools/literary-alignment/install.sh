#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "$script_dir/../.." && pwd)"
tool_dir="${PERSIAN_TRANSLATOR_TOOL_DIR:-$repo_root/.tools/bin}"
build_dir="${PERSIAN_TRANSLATOR_BUILD_DIR:-$repo_root/.tools/build/literary-alignment}"
manifest="$script_dir/Cargo.toml"

mkdir -p "$tool_dir" "$build_dir"
cargo build \
  --locked \
  --release \
  --manifest-path "$manifest" \
  --features bge \
  --target-dir "$build_dir"

cp "$build_dir/release/literary-alignment-tool" "$tool_dir/literary-alignment"
chmod +x "$tool_dir/literary-alignment"

cat <<EOF
Installed literary alignment tool:
  $tool_dir/literary-alignment

The installer compiles the already-approved fastembed/BGE-M3 adapter but does not fetch model weights.
Model weights are fetched only when the alignment tool is explicitly invoked.
The tool returns advisory monotonic alignment evidence; it cannot approve, rewrite, or canonize translation text.
EOF
