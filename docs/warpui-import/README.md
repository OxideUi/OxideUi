# WarpUI Import Notes

This directory records the WarpUI import from Warp commit
`c61ad5b8a8c3980148f91c76a4ce17d1bf8105fc`.

The source-side manifest check confirmed that `crates/warpui_core` and
`crates/warpui` explicitly declare `license = "MIT"`, and that Warp's README
says those crates are MIT while the rest of the repository is AGPL v3.

The MIT source has been copied into quarantined Strato crates:

| Warp source | Strato path |
| --- | --- |
| `crates/warpui_core` | `crates/strato-warpui-core` |
| `crates/warpui` | `crates/strato-warpui-renderer` |
| `LICENSE-MIT` | `LICENSES/WARPUI-MIT.txt` |

The imported crates are intentionally excluded from the Cargo workspace for now.
The copied source still references local Warp workspace crates that inherit
`AGPL-3.0-only` from the Warp workspace, including `markdown_parser`,
`sum_tree`, `warp_util`, `string-offset`, `settings_value`, `command`,
`asset_cache`, and `virtual-fs`. Those crates were not copied.

Future work, if this migration is resumed:

1. Design clean-room replacements for the required unsafe local APIs.
2. Separately audit the forked git dependencies used by WarpUI.
3. Complete a third-party crates.io license audit.
4. Import only the MIT source after the dependency boundary can be enforced by
   CI.
5. Remove the Cargo workspace quarantine only after the imported crates pass the
   license-boundary script and compile without AGPL-inherited dependencies.
