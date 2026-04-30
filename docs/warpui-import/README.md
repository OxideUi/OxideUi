# WarpUI Import Notes

This directory records the WarpUI import from Warp commit
`c61ad5b8a8c3980148f91c76a4ce17d1bf8105fc`.

The source-side manifest check confirmed that `crates/warpui_core` and
`crates/warpui` explicitly declare `license = "MIT"`, and that Warp's README
says those crates are MIT while the rest of the repository is AGPL v3.

The MIT source has been copied into quarantined Strato crates and rebranded at
the crate boundary for Strato ownership:

| Warp source | Strato path |
| --- | --- |
| `crates/warpui_core` | `crates/strato-ui-core` |
| `crates/warpui` | `crates/strato-ui-renderer` |
| `LICENSE-MIT` | `LICENSES/WARPUI-MIT.txt` |

The imported crates are intentionally excluded from the Cargo workspace for now.
The copied source no longer references the AGPL-inherited local Warp crates
`markdown_parser`, `sum_tree`, or `string-offset`; those integration points now
use clean-room Strato modules:

| Removed Warp dependency | Strato replacement |
| --- | --- |
| `markdown_parser` | `crates/strato-ui-core/src/formatted_text.rs` |
| `sum_tree` | `crates/strato-ui-core/src/linear_index.rs` |
| `string-offset` | `crates/strato-ui-core/src/text_offsets.rs` |

The first cleanup pass already removed direct references to `warp_util`,
`settings_value`, `command`, `asset_cache`, and `virtual-fs` from the active
crate manifests and narrow integration points. Those dependencies were not
copied.

Remaining work before enabling these crates in the workspace:

1. Normalize the remaining inherited `workspace = true` dependency declarations
   to Strato workspace entries or explicit crates.io versions.
2. Separately audit the forked git dependencies used by Strato UI.
3. Complete a third-party crates.io license audit.
4. Add focused tests for the clean-room formatted text, text offset, and linear
   index modules.
5. Remove the Cargo workspace quarantine only after the imported crates compile
   and continue to pass the license-boundary script.
