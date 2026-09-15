#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "$script_dir/../.." && pwd)"
tool_dir="${PERSIAN_TRANSLATOR_TOOL_DIR:-$repo_root/.tools/bin}"
build_dir="${PERSIAN_TRANSLATOR_BUILD_DIR:-$repo_root/.tools/build/semantic-retrieval}"
manifest="$script_dir/Cargo.toml"

mkdir -p "$tool_dir" "$build_dir"
cargo build \
  --locked \
  --release \
  --manifest-path "$manifest" \
  --features bge \
  --target-dir "$build_dir"

cp "$build_dir/release/semantic-retrieval-tool" "$tool_dir/semantic-retrieval"
chmod +x "$tool_dir/semantic-retrieval"

cat <<EOF
Installed semantic retrieval tool:
  $tool_dir/semantic-retrieval

No embedding/reranker model weights were downloaded by this installer.
To opt in for a translation run, set:
  export LITERARY_ENGINE_SEMANTIC_RETRIEVAL_TOOL="$tool_dir/semantic-retrieval"

Model weights are fetched only when the semantic tool is actually invoked.
If the tool/model fails, the engine falls back to deterministic retrieval.
EOF
