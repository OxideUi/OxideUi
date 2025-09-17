# OxideUI 🦀

[![License: MIT/Apache-2.0](https://img.shields.io/badge/License-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.80%2B-orange.svg)](https://www.rust-lang.org)
[![Platform](https://img.shields.io/badge/Platform-Windows%20%7C%20macOS%20%7C%20Linux%20%7C%20Web-lightgrey.svg)](https://github.com/oxideui/oxide-ui)
[![GitHub Sponsors](https://img.shields.io/github/sponsors/seregonwar?label=GitHub%20Sponsors&style=flat&logo=GitHub)](https://github.com/sponsors/seregonwar)


**OxideUI** is a next-generation, lightweight, secure, and reactive UI framework written in pure Rust. It combines the declarative programming model of Flutter/React with Rust's performance and safety guarantees, offering GPU-accelerated rendering for desktop applications and WebAssembly support for web deployment.
> ⚠️ **Important Notice**
>
> We are currently considering a name change for this framework.  
> The current name (`OxideUI`) is temporary and may change in future releases.  
> If you have creative ideas or suggestions for a new name, feel free to share them by opening an **issue** or 
> contacting us directly.

![Logo OxideUi](https://github.com/OxideUi/OxideUi/raw/main/logos/OxideUi-org.jpg)

## Features

- 🚀 **Pure Rust Implementation** - Zero unsafe code, leveraging Rust's safety guarantees
- 🎨 **Declarative API** - Intuitive widget-based UI construction similar to Flutter/React
- ⚡ **GPU Acceleration** - Hardware-accelerated rendering via wgpu (Vulkan/Metal/DirectX/WebGL)
- 🌐 **Cross-Platform** - Native support for Windows, macOS, Linux, and WebAssembly
- 🔄 **Reactive State Management** - Built-in signals and reactive primitives
- 📦 **Lightweight** - Minimal dependencies and small binary size
- 🎭 **Theming System** - Comprehensive theming with dark/light mode support
- 🔥 **Hot Reload** - Fast development iteration (development mode)

## Architecture

OxideUI follows a modular architecture with clear separation of concerns:

```
oxide-ui/
├── oxide-core/       # Core functionality (state, events, layout)
├── oxide-renderer/   # GPU rendering backend
├── oxide-widgets/    # UI component library
├── oxide-platform/   # Platform abstraction layer
└── oxide-macros/     # Procedural macros for better DX
```

## Quick Start

### Prerequisites

- Rust 1.80+ (stable)
- Platform-specific requirements:
  - **Windows**: MSVC or MinGW
  - **macOS**: Xcode Command Line Tools
  - **Linux**: `build-essential`, `libxkbcommon-dev`

### Installation

Add OxideUI to your `Cargo.toml`:

```toml
[dependencies]
oxide-core = "0.1.0"
oxide-widgets = "0.1.0"
oxide-platform = "0.1.0"
```

### Hello World Example

```rust
use oxide_widgets::prelude::*;
use oxide_platform::ApplicationBuilder;

fn main() {
    ApplicationBuilder::new()
        .title("Hello OxideUI")
        .run(build_ui())
}

fn build_ui() -> impl Widget {
    Container::new()
        .padding(20.0)
        .child(
            Column::new()
                .spacing(10.0)
                .children(vec![
                    Box::new(Text::new("Hello, OxideUI!")),
                    Box::new(Button::new("Click Me")
                        .on_click(|| println!("Clicked!"))),
                ])
        )
}
```

## 📚 Documentation

### Core Concepts

#### Widgets
Widgets are the building blocks of OxideUI applications. Every UI element is a widget with properties, state, and rendering logic.

```rust
// Creating widgets
let button = Button::new("Submit")
    .style(ButtonStyle::primary())
    .on_click(handle_submit);

let input = TextInput::new()
    .placeholder("Enter your name...")
    .on_change(|text| println!("Text: {}", text));
```

#### Layout System
OxideUI uses a Flexbox-based layout system for arranging widgets:

```rust
Row::new()
    .spacing(10.0)
    .main_axis_alignment(MainAxisAlignment::SpaceBetween)
    .children(vec![...])

Column::new()
    .cross_axis_alignment(CrossAxisAlignment::Center)
    .children(vec![...])
```

#### State Management
Reactive state management with signals:

```rust
use oxide_core::state::Signal;

let count = Signal::new(0);

// Subscribe to changes
count.subscribe(Box::new(|value| {
    println!("Count changed to: {}", value);
}));

// Update state
count.set(42);
```

#### Theming
Comprehensive theming support:

```rust
let theme = Theme::dark();
ThemeProvider::new(theme)
    .child(your_app_widget)
```

## 🛠️ Development

### Building from Source

```bash
# Clone the repository
git clone https://github.com/oxideui/oxide-ui.git
cd oxide-ui

# Build all crates
cargo build --workspace

# Run tests
cargo test --workspace

# Run examples
cargo run --example hello_world
cargo run --example counter
```

### WebAssembly Build

```bash
# Install wasm-pack
cargo install wasm-pack

# Build for web
wasm-pack build --target web crates/oxide-platform

# Serve with a local server
python -m http.server 8000
```

## Roadmap

### Phase 1: Foundation ✅
- [x] Core architecture
- [x] Basic state management
- [x] Event system
- [x] Layout engine

### Phase 2: Rendering ✅
- [x] wgpu integration
- [x] Basic shape rendering
- [x] Text rendering
- [x] Texture management

### Phase 3: Widgets ✅
- [x] Core widget system
- [x] Basic widgets (Button, Text, Container)
- [x] Layout widgets (Row, Column, Stack)
- [x] Input widgets (TextInput)

### Phase 4: Platform 
- [x] Desktop support (Windows, macOS, Linux)
- [x] WebAssembly support
- [ ] Mobile support (future)

### Phase 5: Polish 
- [ ] Animation system
- [ ] Advanced widgets
- [ ] Visual designer
- [ ] Plugin system

## Architecture Details

### Multi-Crate Structure

| Crate | Description | Key Features |
|-------|-------------|--------------|
| `oxide-core` | Core functionality | State management, events, layout engine |
| `oxide-renderer` | GPU rendering | wgpu backend, texture atlas, text rendering |
| `oxide-widgets` | Widget library | Declarative widgets, theming, builders |
| `oxide-platform` | Platform layer | Window management, event loop, WASM support |
| `oxide-macros` | Procedural macros | Derive macros, DSL support |

### Performance Targets

- **Startup Time**: < 100ms
- **Frame Time**: < 16.67ms (60 FPS minimum)
- **Memory Usage**: < 50MB base
- **WASM Size**: < 500KB compressed
- **Layout Time**: < 1ms per 1000 widgets

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Development Setup

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

OxideUI is dual-licensed under either:

- AGPL v3 License ([LICENSE-AGPLv3](LICENSE.md))
- Commercial license ([LICENSE-COMMERCIAL](COMMERCIAL.md))

at your option.

## Acknowledgments

- **wgpu** team for the excellent GPU abstraction
- **winit** for cross-platform windowing
- **lyon** for 2D graphics algorithms
- **cosmic-text** for text rendering
- The Rust community for amazing tooling and support

## Contact - (not yet available)

- **Website**: [oxideui.dev](https://oxideui.dev)
- **GitHub**: [github.com/oxideui/oxide-ui](https://github.com/oxideui/oxide-ui)
- **Discord**: [Join our community](https://discord.gg/oxideui)
- **Twitter**: [@oxideui](https://twitter.com/oxideui)

## Examples

Check out our [examples](examples/) directory for more complete applications:

- [Hello World](examples/hello_world/) - Basic application structure
- [Counter](examples/counter/) - State management example
- [Todo App](examples/todo_app/) - Full CRUD application (coming soon)
- [Calculator](examples/calculator/) - Complex layout example (coming soon)

---

Built with ❤️ in Rust by the OxideUI Team
