# WarpUI Import License Audit

Status: **MIT source imported, AGPL-inherited dependencies removed**

This audit was performed to evaluate whether the MIT-licensed WarpUI crates from
`https://github.com/warpdotdev/warp` can be copied into StratoSDK without
bringing AGPL-licensed Warp workspace code into this repository.

## Source And Target

| Item | Value |
| --- | --- |
| Source repository | `https://github.com/warpdotdev/warp` |
| Source commit | `c61ad5b8a8c3980148f91c76a4ce17d1bf8105fc` |
| Target repository | `https://github.com/StratoHQ/StratoSDK` |
| Target branch | `import-warpui-mit-core` |
| Target starting commit | `feff8859bed8d3ef35561c79a9b6f4281381eda3` |
| Audit date | `2026-04-30` |

## Source License Findings

The Warp README at the audited source commit states that Warp's UI framework,
specifically the `warpui_core` and `warpui` crates, is licensed under
`LICENSE-MIT`, and that the rest of the repository is licensed under
`LICENSE-AGPL`.

The Warp root `Cargo.toml` declares:

```toml
[workspace.package]
license = "AGPL-3.0-only"
publish = false
```

The two candidate crates explicitly declare MIT:

```toml
# crates/warpui_core/Cargo.toml
license = "MIT"

# crates/warpui/Cargo.toml
license = "MIT"
```

The hard license gate therefore passed for the crate manifests themselves, but
failed at the dependency boundary described below.

## Copied Paths

The following MIT-scoped paths were copied after the initial hard stop audit.
No AGPL-licensed Warp workspace crate was copied.

| Source path | Target path | Status |
| --- | --- | --- |
| `crates/warpui_core` | `crates/strato-ui-core` | Copied, then pruned to a clean Strato UI seed |
| `crates/warpui` | `crates/strato-ui-renderer` | Copied, then pruned to a clean Strato UI renderer seed |
| `LICENSE-MIT` | `LICENSES/WARPUI-MIT.txt` | Copied |

## Skipped Paths

| Source path | Reason |
| --- | --- |
| `crates/markdown_parser` | Local Warp workspace crate inherits `AGPL-3.0-only`. |
| `crates/sum_tree` | Local Warp workspace crate inherits `AGPL-3.0-only`. |
| `crates/warp_util` | Local Warp workspace crate inherits `AGPL-3.0-only`. |
| `crates/string-offset` | Local Warp workspace crate inherits `AGPL-3.0-only`. |
| `crates/settings_value` | Local Warp workspace crate inherits `AGPL-3.0-only`. |
| `crates/command` | Local Warp workspace crate inherits `AGPL-3.0-only`. |
| `crates/asset_cache` | Local Warp workspace crate inherits `AGPL-3.0-only`. |
| `crates/virtual_fs` | Local Warp workspace crate inherits `AGPL-3.0-only`. |
| `crates/ui_components` | Local Warp workspace crate inherits `AGPL-3.0-only`. |
| `crates/warpui_extras` | Local Warp workspace crate inherits `AGPL-3.0-only`. |

## Local Workspace Dependency Audit

Only `warpui_core` and `warpui` are in the explicitly MIT-permitted import set.
Every other local Warp workspace dependency listed here is unsafe for copying
because its manifest uses `license.workspace = true`, which resolves to the
Warp workspace license `AGPL-3.0-only`.

| Dependency | Used by | Kind / target | Manifest license finding | Status |
| --- | --- | --- | --- | --- |
| `warpui_core` | `warpui` | normal, dev | Explicit `license = "MIT"` | SAFE, but not copied because import stopped |
| `warpui` | `warpui` | dev self-reference | Explicit `license = "MIT"` | SAFE, but not copied because import stopped |
| `markdown_parser` | `warpui_core`, `warpui` | normal | `license.workspace = true` -> `AGPL-3.0-only` | UNSAFE |
| `settings_value` | `warpui_core` | optional normal | `license.workspace = true` -> `AGPL-3.0-only` | UNSAFE |
| `string-offset` | `warpui_core` | normal | `license.workspace = true` -> `AGPL-3.0-only` | UNSAFE |
| `sum_tree` | `warpui_core`, `warpui` | normal | `license.workspace = true` -> `AGPL-3.0-only` | UNSAFE |
| `warp_util` | `warpui_core` | normal | `license.workspace = true` -> `AGPL-3.0-only` | UNSAFE |
| `command` | `warpui_core`, `warpui` | dev and non-macOS normal | `license.workspace = true` -> `AGPL-3.0-only` | UNSAFE |
| `asset_cache` | `warpui` | dev | `license.workspace = true` -> `AGPL-3.0-only` | UNSAFE |
| `virtual-fs` | `warpui` | Linux dev | `license.workspace = true` -> `AGPL-3.0-only` | UNSAFE |

Additional local crates named in the migration guard list were also inspected:

| Dependency | Manifest license finding | Status |
| --- | --- | --- |
| `ui_components` | `license.workspace = true` -> `AGPL-3.0-only` | UNSAFE |
| `warpui_extras` | `license.workspace = true` -> `AGPL-3.0-only` | UNSAFE |

## Unsafe Reference Evidence

The MIT candidate source currently references unsafe local dependencies in
non-trivial implementation paths:

| Identifier | Example source references | Risk |
| --- | --- | --- |
| `markdown_parser` | formatted text and font/header parsing in `warpui_core` and examples | Requires either a clean-room markdown model/parser or substantial feature removal. |
| `sum_tree` | viewported list and table layout internals | Requires a clean-room tree/cursor data structure with matching behavior. |
| `warp_util` | platform shell family handling | Requires a clean-room replacement for the narrow platform path API. |
| `command` | non-macOS windowing process launch paths and tests | Requires replacement with `std::process::Command` or an existing Strato abstraction. |
| `settings_value` | optional derives and implementations | Feature can likely remain disabled, but must be removed or replaced before import. |
| `string-offset` | normal dependency of `warpui_core` | License is AGPL via workspace inheritance; import cannot depend on it. |

The candidate source is large enough that replacing these safely is not a small
mechanical edit:

| Source path | Approximate size |
| --- | --- |
| `crates/warpui_core` | `3.1M` |
| `crates/warpui` | `19M` |
| Combined file count | `387` files |

## Direct Dependency Inventory

This inventory was produced with `cargo metadata --no-deps --format-version 1`
from the Warp source checkout. Registry and git dependencies were not copied.
Because the import stopped at unsafe local workspace dependencies, third-party
registry crate license validation was not completed and must be done before any
future compileable import.

### `warpui_core`

| Dependency | Class | Kind / target | Status |
| --- | --- | --- | --- |
| `anyhow` | crates.io | normal / all | External, not copied; future third-party license audit required |
| `async-broadcast` | crates.io | normal / all | External, not copied; future third-party license audit required |
| `async-channel` | crates.io | normal / all | External, not copied; future third-party license audit required |
| `async-executor` | crates.io | normal / all | External, not copied; future third-party license audit required |
| `async-fs` | crates.io | normal / all | External, not copied; future third-party license audit required |
| `async-task` | crates.io | normal / all | External, not copied; future third-party license audit required |
| `bounded-vec-deque` | crates.io | normal / all | External, not copied; future third-party license audit required |
| `bytes` | crates.io | normal / all | External, not copied; future third-party license audit required |
| `cfg-if` | crates.io | normal / all | External, not copied; future third-party license audit required |
| `chrono` | crates.io | normal / all | External, not copied; future third-party license audit required |
| `dashmap` | crates.io | normal / all | External, not copied; future third-party license audit required |
| `derivative` | crates.io | normal / all | External, not copied; future third-party license audit required |
| `derive_more` | crates.io | normal / all | External, not copied; future third-party license audit required |
| `dirs` | crates.io | normal / all | External, not copied; future third-party license audit required |
| `enum-iterator` | crates.io | normal / all | External, not copied; future third-party license audit required |
| `float-cmp` | crates.io | normal / all | External, not copied; future third-party license audit required |
| `futures` | crates.io | normal / all | External, not copied; future third-party license audit required |
| `futures-lite` | crates.io | normal / all | External, not copied; future third-party license audit required |
| `futures-util` | crates.io | normal / all | External, not copied; future third-party license audit required |
| `image` | crates.io | normal / all | External, not copied; future third-party license audit required |
| `infer` | crates.io | normal / all | External, not copied; future third-party license audit required |
| `instant` | crates.io | normal / all | External, not copied; future third-party license audit required |
| `itertools` | crates.io | normal / all | External, not copied; future third-party license audit required |
| `lazy_static` | crates.io | normal / all | External, not copied; future third-party license audit required |
| `log` | crates.io | normal / all | External, not copied; future third-party license audit required |
| `markdown_parser` | local Warp workspace | normal / all | UNSAFE: AGPL via workspace license inheritance |
| `minimp4` | crates.io | optional normal / all | External, not copied; future third-party license audit required |
| `num-derive` | crates.io | normal / all | External, not copied; future third-party license audit required |
| `num-traits` | crates.io | normal / all | External, not copied; future third-party license audit required |
| `num_cpus` | crates.io | normal / all | External, not copied; future third-party license audit required |
| `openh264` | crates.io | optional normal / all | External, not copied; future third-party license audit required |
| `ordered-float` | crates.io | normal / all | External, not copied; future third-party license audit required |
| `parking_lot` | crates.io | normal / all | External, not copied; future third-party license audit required |
| `pathfinder_color` | crates.io | normal / all | External, not copied; future third-party license audit required |
| `pathfinder_geometry` | crates.io | normal / all | External, not copied; future third-party license audit required |
| `rand` | crates.io | normal / all | External, not copied; future third-party license audit required |
| `rangemap` | crates.io | normal / all | External, not copied; future third-party license audit required |
| `raw-window-handle` | crates.io | normal / all | External, not copied; future third-party license audit required |
| `resvg` | crates.io | normal / all | External, not copied; future third-party license audit required |
| `rstar` | crates.io | normal / all | External, not copied; future third-party license audit required |
| `rustc-hash` | crates.io | normal / all | External, not copied; future third-party license audit required |
| `schemars` | crates.io | optional normal / all | External, not copied; future third-party license audit required |
| `serde` | crates.io | normal / all | External, not copied; future third-party license audit required |
| `serde_json` | crates.io | normal / all | External, not copied; future third-party license audit required |
| `settings_value` | local Warp workspace | optional normal / all | UNSAFE: AGPL via workspace license inheritance |
| `similar` | crates.io | normal / all | External, not copied; future third-party license audit required |
| `smallvec` | crates.io | normal / all | External, not copied; future third-party license audit required |
| `string-offset` | local Warp workspace | normal / all | UNSAFE: AGPL via workspace license inheritance |
| `strum` | crates.io | normal / all | External, not copied; future third-party license audit required |
| `strum_macros` | crates.io | normal / all | External, not copied; future third-party license audit required |
| `sum_tree` | local Warp workspace | normal / all | UNSAFE: AGPL via workspace license inheritance |
| `tempfile` | crates.io | normal / all | External, not copied; future third-party license audit required |
| `thiserror` | crates.io | normal / all | External, not copied; future third-party license audit required |
| `titlecase` | crates.io | normal / all | External, not copied; future third-party license audit required |
| `tokio` | crates.io | normal / all | External, not copied; future third-party license audit required |
| `trait-set` | crates.io | normal / all | External, not copied; future third-party license audit required |
| `vec1` | crates.io | normal / all | External, not copied; future third-party license audit required |
| `warp_util` | local Warp workspace | normal / all | UNSAFE: AGPL via workspace license inheritance |
| `command` | local Warp workspace | dev / all | UNSAFE: AGPL via workspace license inheritance |
| `concat-idents` | crates.io | dev / all | External, not copied; future third-party license audit required |
| `ctor` | crates.io | dev / all | External, not copied; future third-party license audit required |
| `rust-embed` | crates.io | dev / all | External, not copied; future third-party license audit required |
| `simplelog` | crates.io | dev / all | External, not copied; future third-party license audit required |
| `cfg_aliases` | crates.io | build / all | External, not copied; future third-party license audit required |
| `arboard` | crates.io | normal / Linux or Windows | External, not copied; future third-party license audit required |
| `async-io` | crates.io | normal / non-wasm | External, not copied; future third-party license audit required |
| `ctrlc` | crates.io | normal / non-wasm | External, not copied; future third-party license audit required |
| `font-kit` | git | normal / non-wasm | External git dependency; unresolved until separately audited |
| `gloo` | crates.io | normal / wasm | External, not copied; future third-party license audit required |
| `wasm-bindgen-futures` | crates.io | normal / wasm | External, not copied; future third-party license audit required |
| `woothee` | crates.io | normal / wasm | External, not copied; future third-party license audit required |

### `warpui`

| Dependency | Class | Kind / target | Status |
| --- | --- | --- | --- |
| `anyhow` | crates.io | normal / all | External, not copied; future third-party license audit required |
| `async-task` | crates.io | normal / all | External, not copied; future third-party license audit required |
| `bytemuck` | crates.io | optional normal / all | External, not copied; future third-party license audit required |
| `cfg-if` | crates.io | normal / all | External, not copied; future third-party license audit required |
| `dashmap` | crates.io | normal / all | External, not copied; future third-party license audit required |
| `derive_more` | crates.io | normal / all | External, not copied; future third-party license audit required |
| `futures` | crates.io | normal / all | External, not copied; future third-party license audit required |
| `futures-util` | crates.io | normal / all | External, not copied; future third-party license audit required |
| `instant` | crates.io | normal / all | External, not copied; future third-party license audit required |
| `itertools` | crates.io | normal / all | External, not copied; future third-party license audit required |
| `lazy_static` | crates.io | normal / all | External, not copied; future third-party license audit required |
| `log` | crates.io | normal / all | External, not copied; future third-party license audit required |
| `markdown_parser` | local Warp workspace | normal / all | UNSAFE: AGPL via workspace license inheritance |
| `num-traits` | crates.io | normal / all | External, not copied; future third-party license audit required |
| `ordered-float` | crates.io | normal / all | External, not copied; future third-party license audit required |
| `parking_lot` | crates.io | normal / all | External, not copied; future third-party license audit required |
| `pathfinder_color` | crates.io | normal / all | External, not copied; future third-party license audit required |
| `pathfinder_geometry` | crates.io | normal / all | External, not copied; future third-party license audit required |
| `sum_tree` | local Warp workspace | normal / all | UNSAFE: AGPL via workspace license inheritance |
| `takecell` | crates.io | normal / all | External, not copied; future third-party license audit required |
| `thiserror` | crates.io | normal / all | External, not copied; future third-party license audit required |
| `vec1` | crates.io | normal / all | External, not copied; future third-party license audit required |
| `version-compare` | crates.io | optional normal / all | External, not copied; future third-party license audit required |
| `warpui_core` | local Warp workspace | normal / all | SAFE MIT candidate, but import stopped |
| `wgpu` | crates.io | optional normal / all | External, not copied; future third-party license audit required |
| `asset_cache` | local Warp workspace | dev / all | UNSAFE: AGPL via workspace license inheritance |
| `env_logger` | crates.io | dev / all | External, not copied; future third-party license audit required |
| `futures-timer` | crates.io | dev / all | External, not copied; future third-party license audit required |
| `image` | crates.io | dev / all | External, not copied; future third-party license audit required |
| `rust-embed` | crates.io | dev / all | External, not copied; future third-party license audit required |
| `warpui` | local Warp workspace | dev / all | SAFE MIT candidate, but import stopped |
| `bindgen` | crates.io | build / all | External, not copied; future third-party license audit required |
| `cc` | crates.io | build / all | External, not copied; future third-party license audit required |
| `cfg_aliases` | crates.io | build / all | External, not copied; future third-party license audit required |
| `arboard` | crates.io | normal / Linux or Windows | External, not copied; future third-party license audit required |
| `native-dialog` | crates.io | normal / Linux or Windows | External, not copied; future third-party license audit required |
| `open` | crates.io | normal / Linux or Windows | External, not copied; future third-party license audit required |
| `global-hotkey` | crates.io | normal / non-macOS and non-wasm | External, not copied; future third-party license audit required |
| `async-io` | crates.io | normal / non-wasm | External, not copied; future third-party license audit required |
| `ctrlc` | crates.io | normal / non-wasm | External, not copied; future third-party license audit required |
| `font-kit` | git | normal / non-wasm, Linux, macOS | External git dependency; unresolved until separately audited |
| `bimap` | crates.io | normal / non-macOS | External, not copied; future third-party license audit required |
| `command` | local Warp workspace | normal / non-macOS | UNSAFE: AGPL via workspace license inheritance |
| `cosmic-text` | git | normal / non-macOS | External git dependency; unresolved until separately audited |
| `derivative` | crates.io | normal / non-macOS | External, not copied; future third-party license audit required |
| `fontdb` | crates.io | normal / non-macOS | External, not copied; future third-party license audit required |
| `memmap2` | crates.io | normal / non-macOS | External, not copied; future third-party license audit required |
| `owned_ttf_parser` | crates.io | normal / non-macOS | External, not copied; future third-party license audit required |
| `resvg` | crates.io | normal / non-macOS | External, not copied; future third-party license audit required |
| `serde` | crates.io | normal / non-macOS | External, not copied; future third-party license audit required |
| `version-compare` | crates.io | normal / non-macOS | External, not copied; future third-party license audit required |
| `winit` | git | normal / non-macOS | External git dependency; unresolved until separately audited |
| `gloo` | crates.io | normal / wasm | External, not copied; future third-party license audit required |
| `js-sys` | crates.io | normal / wasm | External, not copied; future third-party license audit required |
| `wasm-bindgen` | crates.io | normal / wasm | External, not copied; future third-party license audit required |
| `wasm-bindgen-futures` | crates.io | normal / wasm | External, not copied; future third-party license audit required |
| `web-sys` | crates.io | normal / wasm | External, not copied; future third-party license audit required |
| `blocking` | crates.io | normal / Linux | External, not copied; future third-party license audit required |
| `dirs` | crates.io | normal / Linux | External, not copied; future third-party license audit required |
| `fontconfig` | crates.io | normal / Linux | External, not copied; future third-party license audit required |
| `futures-lite` | crates.io | normal / Linux | External, not copied; future third-party license audit required |
| `nix` | crates.io | normal / Linux | External, not copied; future third-party license audit required |
| `notify-rust` | crates.io | normal / Linux | External, not copied; future third-party license audit required |
| `tini` | crates.io | normal / Linux | External, not copied; future third-party license audit required |
| `urlencoding` | crates.io | normal / Linux | External, not copied; future third-party license audit required |
| `x11-dl` | crates.io | normal / Linux | External, not copied; future third-party license audit required |
| `x11rb` | crates.io | normal / Linux | External, not copied; future third-party license audit required |
| `zbus` | crates.io | normal / Linux | External, not copied; future third-party license audit required |
| `virtual-fs` | local Warp workspace | dev / Linux | UNSAFE: AGPL via workspace license inheritance |
| `block` | crates.io | normal / macOS | External, not copied; future third-party license audit required |
| `chrono` | crates.io | normal / macOS | External, not copied; future third-party license audit required |
| `cocoa` | crates.io | normal / macOS | External, not copied; future third-party license audit required |
| `core-foundation` | crates.io | normal / macOS | External, not copied; future third-party license audit required |
| `core-graphics` | crates.io | normal / macOS | External, not copied; future third-party license audit required |
| `core-text` | crates.io | normal / macOS | External, not copied; future third-party license audit required |
| `dispatch` | crates.io | normal / macOS | External, not copied; future third-party license audit required |
| `foreign-types` | crates.io | normal / macOS | External, not copied; future third-party license audit required |
| `libc` | crates.io | normal / macOS | External, not copied; future third-party license audit required |
| `metal` | crates.io | normal / macOS | External, not copied; future third-party license audit required |
| `objc` | crates.io | normal / macOS | External, not copied; future third-party license audit required |
| `rand` | crates.io | dev / macOS | External, not copied; future third-party license audit required |
| `dwrote` | git | normal / Windows | External git dependency; unresolved until separately audited |
| `tauri-winrt-notification` | crates.io | normal / Windows | External, not copied; future third-party license audit required |
| `windows` | crates.io | normal / Windows | External, not copied; future third-party license audit required |
| `windows-core` | crates.io | normal / Windows | External, not copied; future third-party license audit required |
| `windows-version` | crates.io | normal / Windows | External, not copied; future third-party license audit required |
| `winreg` | crates.io | normal / Windows | External, not copied; future third-party license audit required |

## Blockers

1. `warpui_core` and `warpui` cannot currently compile or be safely imported
   without addressing direct unsafe local Warp workspace dependencies.
2. The unsafe local dependencies inherit `AGPL-3.0-only` from the Warp workspace.
3. Replacing `sum_tree` and `markdown_parser` requires meaningful clean-room
   design and validation. This is not a narrow compatibility shim.
4. Git dependencies under the `warpdotdev` organization, including forked
   `font-kit`, `cosmic-text`, `winit`, and `dwrote-rs`, require separate license
   provenance review before any future use.
5. A full third-party registry dependency license audit was not completed
   because the local AGPL boundary is already a hard stop.

## Conclusion

The repository remains MIT/commercial-license compatible on the basis of this
audit because only the two explicitly MIT WarpUI crates and Warp's MIT license
text were copied, and all references to AGPL-inherited local Warp workspace
dependencies were removed from the active Strato UI crates. Future functionality
must be rebuilt through clean-room implementations, existing StratoSDK modules,
or audited external crates.
