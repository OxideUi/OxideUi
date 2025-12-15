# StratoSDK 🦀

[![License: MIT/Apache-2.0](https://img.shields.io/badge/License-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.80%2B-orange.svg)](https://www.rust-lang.org)
[![Platform](https://img.shields.io/badge/Platform-Windows%20%7C%20macOS%20%7C%20Linux%20%7C%20Web-green.svg)](https://github.com/StratoSDK/strato-ui)
[![GitHub Sponsors](https://img.shields.io/github/sponsors/seregonwar?label=GitHub%20Sponsors&style=flat&logo=GitHub)](https://github.com/sponsors/seregonwar)


**StratoSDK** is a next-generation, lightweight, secure, and reactive UI framework written in pure Rust. It combines the declarative programming model of Flutter/React with Rust's performance and safety guarantees, offering GPU-accelerated rendering for desktop applications and WebAssembly support for web deployment.

![Logo StratoSDK](https://github.com/StratoSDK/StratoSDK/raw/main/logos/StratoSDK-org.jpg)

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

StratoSDK follows a modular architecture with clear separation of concerns:

```
strato-ui/
├── strato-core/       # Core functionality (state, events, layout)
├── strato-renderer/   # GPU rendering backend
├── strato-widgets/    # UI component library
├── strato-platform/   # Platform abstraction layer
└── strato-macros/     # Procedural macros for better DX
```

## Quick Start

### Prerequisites

- Rust 1.80+ (stable)
- Platform-specific requirements:
  - **Windows**: MSVC or MinGW
  - **macOS**: Xcode Command Line Tools
  - **Linux**: `build-essential`, `libxkbcommon-dev`

### Installation

Add StratoSDK to your `Cargo.toml`:

```toml
[dependencies]
strato-core = "0.1.0"
strato-widgets = "0.1.0"
strato-platform = "0.1.0"
```

### Hello World Example

```rust
use strato_widgets::prelude::*;
use strato_platform::ApplicationBuilder;

fn main() {
    ApplicationBuilder::new()
        .title("Hello StratoSDK")
        .run(build_ui())
}

fn build_ui() -> impl Widget {
    Container::new()
        .padding(20.0)
        .child(
            Column::new()
                .spacing(10.0)
                .children(vec![
                    Box::new(Text::new("Hello, StratoSDK!")),
                    Box::new(Button::new("Click Me")
                        .on_click(|| println!("Clicked!"))),
                ])
        )
}
```

## 📚 Documentation

### Core Concepts

#### Widgets
Widgets are the building blocks of StratoSDK applications. Every UI element is a widget with properties, state, and rendering logic.

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
StratoSDK uses a Flexbox-based layout system for arranging widgets:

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
use strato_core::state::Signal;

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
git clone https://github.com/StratoSDK/strato-ui.git
cd strato-ui

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
wasm-pack build --target web crates/strato-platform

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
| `strato-core` | Core functionality | State management, events, layout engine |
| `strato-renderer` | GPU rendering | wgpu backend, texture atlas, text rendering |
| `strato-widgets` | Widget library | Declarative widgets, theming, builders |
| `strato-platform` | Platform layer | Window management, event loop, WASM support |
| `strato-macros` | Procedural macros | Derive macros, DSL support |

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

StratoSDK is dual-licensed under either:

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

- **Website**: [StratoSDK.dev](https://StratoSDK.dev)
- **GitHub**: [github.com/StratoSDK/strato-ui](https://github.com/StratoSDK/strato-ui)
- **Discord**: [Join our community](https://discord.gg/StratoSDK)
- **Twitter**: [@StratoHQ](https://twitter.com/StratoSDK)

## Examples

Check out our [examples](examples/) directory for more complete applications:

- [Hello World](examples/hello_world/) - Basic application structure
<img width="373" height="516" alt="Screenshot 2025-12-15 alle 22 55 19" src="https://github.com/user-attachments/assets/800d2741-f65b-4023-b3eb-917a117f4d23" />

- [Counter](examples/counter/) - State management example
<img width="462" height="394" alt="Screenshot 2025-12-15 alle 22 48 41" src="https://github.com/user-attachments/assets/c9803155-a17e-4f42-8876-e65b8a87a2bd" />

- [Modern Dashboard](examples/modern_dashboard/) - **New!** Comprehensive example featuring:
  - Modular architecture (Views, Components)
  - Multi-page navigation (Dashboard, Analytics, Users, Settings)
  - Modern UI with responsive layout (Flexbox)
  - Theming system
  - Simulated backend integration
  <img width="1552" height="987" alt="Screenshot 2025-12-15 alle 22 47 22" src="https://github.com/user-attachments/assets/3b8f3b3b-6191-4ad1-aee2-aacfc6d15efb" />

- [Todo App](examples/todo_app/) - Full CRUD application (coming soon)
- [Calculator](examples/calculator/) - Complex layout example
<img width="432" height="624" alt="Screenshot 2025-12-15 alle 22 46 42" src="https://github.com/user-attachments/assets/687f46d5-4d7b-49b6-93cd-9fc965b0c13f" />


---

Built with ❤️ in Rust by the StratoSDK Team
