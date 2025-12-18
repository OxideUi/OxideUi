# SwiftUI Parity Tracker for StratoSDK

This document inventories StratoSDK widgets against their closest SwiftUI counterparts and highlights gaps to help us prioritize API parity.

## Current Coverage

| SwiftUI component | StratoSDK equivalent | Notes on parity |
| --- | --- | --- |
| `Text` | `Text` | Size/color APIs already mirror SwiftUI's basic modifiers. |
| `Image` | `Image` | Supports fit/format options; add rendering modes to match `resizable()` / `aspectRatio()`. |
| `Button` | `Button` | Now exposes hover/press/focus/disabled visuals and accessibility labels. |
| `Slider` | `Slider` | Dragging, hover/press states, semantics for value exposure. |
| `ProgressView` | `ProgressBar` | Determinate progress matches `ProgressView(value:)`; indeterminate animation still basic. |
| `Toggle` | `Checkbox` | Provides boolean selection; radio buttons cover grouped selection similar to `Picker` with `segmented`. |
| `TextField` | `TextInput` | Basic text entry available; needs parity for secure entry and prompt styles. |
| `ScrollView` | `ScrollView` | Vertical/horizontal scrolling already supported. |
| `HStack` / `VStack` | `Row` / `Column` | Flex-based layout covers SwiftUI stacks. |
| `ZStack` | `Stack` | Overlapping layout via `Stack` matches SwiftUI. |
| `List` | `ScrollView` + `Column` | No dedicated list yet; composition required. |

## Missing or Partial Components

| SwiftUI component | Status in StratoSDK | Proposed parity direction |
| --- | --- | --- |
| `Toggle` (switch style) | Missing dedicated switch UI | Introduce `Switch` widget with `on_change`, `is_on`, and `.tint(Color)` akin to SwiftUI. |
| `Stepper` | Not available | Add `Stepper` with `value`/`in`/`step` bindings; reuse slider semantics for accessibility. |
| `Picker` (menus/wheels) | Only dropdown covered | Extend `Dropdown` into a general `Picker` supporting wheel style for desktop/mobile parity. |
| `DatePicker` / `ColorPicker` | Missing | Provide specialized pickers with localized formatting and signal outputs. |
| `NavigationStack` / `TabView` | Not present | Plan container widgets for navigation stacks and tab bars to mirror SwiftUI routing. |
| `Alert` / `Sheet` | Absent | Modal primitives with async confirmation callbacks should mirror SwiftUI's `.alert` and `.sheet` modifiers. |
| `ToggleStyle` / `ButtonStyle` protocols | Style structs only | Introduce trait-based styling hooks so widgets accept custom style conformances similarly to SwiftUI. |
| `GeometryReader` | Partial via layout engine | Expose explicit `GeometryReader`-like widget to pass layout info into children. |

## Parity Guidelines

1. **Naming alignment**: Prefer SwiftUI naming when semantics match (e.g., `Stepper`, `Toggle`, `NavigationStack`).
2. **Modifier parity**: Map common modifiers (`.disabled`, `.focused`, `.animation`, `.tint`) to builder methods on widgets.
3. **Accessibility**: Every interactive control should expose role, label, hint, and value/toggled state just like SwiftUI's accessibility modifiers.
4. **Stateful visuals**: Hover/press/focus/disabled should be animated by default to reduce custom boilerplate and mirror SwiftUI's built-in behaviors.
5. **Composable fallbacks**: When a dedicated widget is missing, document the composition path (e.g., `ScrollView` + `Column` as a `List` stand-in) until a native widget ships.
