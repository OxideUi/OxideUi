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

forbidden_manifest_pattern='markdown_parser|sum_tree|warp_util|settings_value|command|asset_cache|virtual-fs|virtual_fs|ui_components|warpui_extras|string-offset'
forbidden_source_pattern='markdown_parser|sum_tree|warp_util|string_offset|warpdotdev/warp/app|LICENSE-AGPL'

if grep -RInE "$forbidden_manifest_pattern" \
  crates/strato-ui-core/Cargo.toml \
  crates/strato-ui-renderer/Cargo.toml; then
  echo "forbidden Warp/AGPL dependency reference found in Strato UI manifests" >&2
  exit 1
fi

if grep -RInE "$forbidden_source_pattern" \
  crates/strato-ui-core/src \
  crates/strato-ui-renderer/src \
  crates/strato-ui-renderer/examples; then
  echo "forbidden Warp/AGPL code reference found in imported Strato UI source" >&2
  exit 1
fi

echo "Strato UI license boundary check passed"
