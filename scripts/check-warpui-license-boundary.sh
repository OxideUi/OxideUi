#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

missing=0
for required in "LICENSES/WARPUI-MIT.txt" "THIRD_PARTY_NOTICES.md"; do
  if [[ ! -f "$required" ]]; then
    echo "missing required attribution file: $required" >&2
    missing=1
  fi
done

if [[ "$missing" -ne 0 ]]; then
  exit 1
fi

forbidden_pattern='warp_util|sum_tree|markdown_parser|warpui_extras|ui_components|crate::app|warpdotdev/warp/app|LICENSE-AGPL'

if grep -RInE "$forbidden_pattern" \
  crates/strato-warpui-core \
  crates/strato-warpui-renderer; then
  echo "forbidden Warp/AGPL boundary reference found in imported WarpUI code" >&2
  exit 1
fi

echo "WarpUI license boundary check passed"
