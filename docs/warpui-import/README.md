# WarpUI Import Notes

This directory records the attempted WarpUI import from Warp commit
`c61ad5b8a8c3980148f91c76a4ce17d1bf8105fc`.

No WarpUI source was imported. The source-side manifest check confirmed that
`crates/warpui_core` and `crates/warpui` explicitly declare `license = "MIT"`,
and that Warp's README says those crates are MIT while the rest of the
repository is AGPL v3.

The import stopped because the MIT crates directly depend on local Warp
workspace crates that inherit `AGPL-3.0-only` from the workspace, including
`markdown_parser`, `sum_tree`, `warp_util`, `string-offset`, `settings_value`,
`command`, `asset_cache`, and `virtual-fs`.

Future work, if this migration is resumed:

1. Design clean-room replacements for the required unsafe local APIs.
2. Separately audit the forked git dependencies used by WarpUI.
3. Complete a third-party crates.io license audit.
4. Import only the MIT source after the dependency boundary can be enforced by
   CI.
5. Add attribution and `LICENSES/WARPUI-MIT.txt` only when source is actually
   copied.
