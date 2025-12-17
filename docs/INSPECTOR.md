# StratoSDK Inspector Overlay

The inspector provides an in-app panel that visualizes the widget hierarchy, recent state snapshots, layout boxes, and frame timelines so you can debug rendering behavior without leaving the running app.

## Toggling the overlay
- Press **Ctrl+Shift+I** (configurable when constructing `InspectorOverlay`) to toggle the panel at runtime.
- You can also enable/disable programmatically with `strato_core::inspector::inspector().set_enabled(bool)`.

## Development vs. production defaults
- In **debug/development builds** (`cfg!(debug_assertions)`), the inspector is enabled by default. Instrumentation hooks capture layout and state changes automatically.
- In **release/production builds**, disable the inspector to remove overhead:
  - Call `inspector().configure(InspectorConfig { enabled: false, ..Default::default() })` during startup, or
  - Wrap your root widget in `InspectorOverlay` only for debug builds (`#[cfg(debug_assertions)]`).

## Adding the overlay to your UI
```rust
use strato_widgets::InspectorOverlay;

let app = build_app_ui();
let instrumented = InspectorOverlay::new(app);
```

The overlay renders a compact panel with:
- **Component hierarchy**: per-widget IDs and depth.
- **State snapshots**: the latest serialized signal updates.
- **Layout boxes**: highlighted rectangles overlaying the current frame.
- **Performance timeline**: per-frame CPU/GPU timings from the renderer profiler.

No additional plumbing is required—the overlay pulls data from the shared inspector instrumentation in `strato-core` and `strato-renderer`.
