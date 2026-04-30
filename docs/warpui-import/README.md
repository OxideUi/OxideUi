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
The copied source still references local Warp workspace crates that inherit
`AGPL-3.0-only` from the Warp workspace, with the primary remaining blockers
currently being `markdown_parser`, `sum_tree`, and `string-offset`.

The first cleanup pass already removed direct references to `warp_util`,
`settings_value`, `command`, `asset_cache`, and `virtual-fs` from the active
crate manifests and narrow integration points. Those dependencies were not
copied.

Future work, if this migration is resumed:

1. Design clean-room replacements for `markdown_parser`, `sum_tree`, and
   `string-offset`.
2. Separately audit the forked git dependencies used by WarpUI.
3. Complete a third-party crates.io license audit.
4. Remove or replace remaining in-source references that still assume Warp-only
   local crates.
5. Import only the MIT source after the dependency boundary can be enforced by
   CI.
6. Remove the Cargo workspace quarantine only after the imported crates pass the
   license-boundary script and compile without AGPL-inherited dependencies.
