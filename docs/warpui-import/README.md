# WarpUI Import Notes

This directory records the Strato UI import derived from the MIT-scoped WarpUI
crates copied from Warp commit `c61ad5b8a8c3980148f91c76a4ce17d1bf8105fc`.
A follow-up source license re-check against Warp commit
`e9ff9324ea14641326ae62f49e6ba0d39f6954b2` confirmed the same boundary:
`warpui_core` and `warpui` are MIT, while the rest of Warp remains AGPL v3.

The source-side manifest check confirmed that `crates/warpui_core` and
`crates/warpui` explicitly declare `license = "MIT"`, and that Warp's README
says those crates are MIT while the rest of the repository is AGPL v3.

The MIT source has been copied into Strato crates and rebranded at the crate
boundary for Strato ownership:

| Warp source | Strato path |
| --- | --- |
| `crates/warpui_core` | `crates/strato-ui-core` |
| `crates/warpui` | `crates/strato-ui-renderer` |
| `LICENSE-MIT` | `LICENSES/WARPUI-MIT.txt` |

The imported crates are now workspace members. The copied source no longer
references the AGPL-inherited local Warp crates `markdown_parser`, `sum_tree`,
or `string-offset`; those integration points now use clean-room Strato modules:

| Removed Warp dependency | Strato replacement |
| --- | --- |
| `markdown_parser` | `crates/strato-ui-core/src/formatted_text.rs` |
| `sum_tree` | `crates/strato-ui-core/src/linear_index.rs` |
| `string-offset` | `crates/strato-ui-core/src/text_offsets.rs` |

The cleanup pass also removed direct references to `warp_util`,
`settings_value`, `command`, `asset_cache`, and `virtual-fs` from the active
crate manifests and narrow integration points. Those dependencies were not
copied.

Dependency adaptations made for Strato:

| Warp dependency shape | Strato resolution |
| --- | --- |
| Warp workspace local crates with AGPL inheritance | Excluded and replaced only by clean-room Strato modules where needed |
| Warp git `cosmic-text` fork | Replaced with crates.io `cosmic-text` |
| Warp git `dwrote-rs` fork | Replaced with crates.io `dwrote` |
| Warp git `font-kit` fork | Replaced with crates.io `font-kit`; Warp-only helper APIs were removed or adapted |

Known gaps and future cleanup:

1. The macOS renderer build script writes a placeholder `shaders.metallib` when
   Apple's Metal Toolchain is missing, so `cargo check` can run in developer
   environments. Runtime Metal validation still requires installing the Metal
   Toolchain.
2. Color emoji detection from Warp's `font-kit` fork is disabled until Strato
   owns a clean implementation against public crates.
3. A full third-party dependency license report should be generated before
   publishing Strato UI crates, although no AGPL Warp dependency is copied or
   linked from the imported code.
