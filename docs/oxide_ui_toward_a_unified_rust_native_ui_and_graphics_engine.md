# OxideUI: Toward a Unified Rust‑Native UI and Graphics Engine

**Author**: OxideUI Team (SeregonWar)  
**Status**: Position Paper / Vision Document  
**Version**: 1.0 (Draft)

---

## Abstract

Modern UI frameworks and game engines have converged on similar technical problems—real‑time rendering, layout, input handling, asset management, and tooling—yet they remain fragmented across ecosystems and burdened by legacy abstractions. OxideUI proposes a structural shift: a Rust‑native, UI‑first graphics engine that treats user interfaces not as an afterthought, but as the foundational layer upon which higher‑level rendering, tooling, and interactive systems are built.

This paper outlines the rationale, architectural principles, and evolutionary roadmap behind OxideUI’s transformation from a declarative UI framework into a modular, backend‑agnostic 2D graphics engine with first‑class tooling. The goal is not incremental improvement, but a reset of assumptions around how UI, rendering, and developer experience should be composed in a systems‑level language.

---

## 1. The Problem: Fragmentation and Accidental Complexity

Today’s UI and rendering ecosystems suffer from three systemic issues:

1. **Leaky Abstractions**  
   Frameworks expose low‑level rendering concepts (surfaces, devices, pipelines) directly to application developers, coupling APIs to specific backends and inflating cognitive load.

2. **UI as a Secondary Concern**  
   Most engines evolve bottom‑up (graphics → physics → tools → UI). As a result, UI systems are retrofitted, inconsistent, and rarely engine‑grade.

3. **Tooling Debt**  
   Editors and development tools are often separate products, tightly coupled to engines, and impossible to reuse outside their intended workflows.

These problems are not language‑specific; they are architectural. OxideUI’s thesis is that solving them requires **reversing the traditional order of engine design**.

---

## 2. The Core Thesis: UI‑First Engine Design

OxideUI is built on a simple but radical premise:

> *If a system can render text, rectangles, layout, and handle input correctly and efficiently, it already solves 80% of what a real‑time engine needs.*

Rather than treating UI as a layer on top of a renderer, OxideUI treats **2D rendering and UI composition as the renderer**.

From this perspective:
- Buttons are textured quads with state.
- Layout is deterministic spatial computation.
- Input is hit‑testing over transformed geometry.
- Editors are just applications built with the same primitives as end‑user tools.

This inversion allows the engine to grow **outward**, instead of being patched **upward**.

---

## 3. Why Rust Changes the Equation

Rust is not merely an implementation detail; it is an enabler of architectural clarity.

OxideUI leverages Rust to:
- Enforce strict ownership boundaries between engine layers.
- Encode invariants (state, lifecycle, widget phases) at compile time.
- Provide zero‑cost abstractions where other ecosystems rely on runtime discipline.

Unlike C++ engines, OxideUI does not require convention to maintain safety. Unlike GC‑driven UI frameworks, it does not trade determinism for convenience.

Rust makes it possible to build a **high‑level engine with low‑level guarantees**.

---

## 4. Architectural Revolution: Decoupling Rendering from Backends

A central design goal of OxideUI is **complete detachment from any single graphics API**.

### 4.1 Backend‑Agnostic Rendering Core

The engine defines its own rendering vocabulary:
- Commands (draw, clear, blit)
- Resources (buffers, textures, shaders)
- Pipelines (opaque, 2D, text, UI)

These concepts are expressed in engine‑level terms and never expose backend‑specific types.

### 4.2 Pluggable Backends

Backends (wgpu, Vulkan‑native, Metal, WebGL, or software) are optional modules that translate engine commands into API‑specific calls.

This approach:
- Eliminates vendor lock‑in
- Enables deterministic testing via headless backends
- Allows platform‑specific optimizations without API pollution

wgpu becomes an *implementation detail*, not a dependency of user code.

---

## 5. The Prelude as a Contract, Not a Convenience

OxideUI exposes a unified `prelude` to application developers. This is not syntactic sugar—it is a **stability contract**.

The prelude guarantees:
- Declarative, immutable widget construction
- Backend‑independent rendering semantics
- Forward compatibility across engine evolution

Internally, the engine remains modular and explicit. Externally, it presents a coherent mental model that shields developers from architectural churn.

This separation is critical for long‑term sustainability.

---

## 6. Tooling as a First‑Class Citizen

OxideUI rejects the idea that editors are special applications.

Instead:
- The editor is built *with* OxideUI
- The runtime uses the same renderer, layout engine, and event system
- Tools and applications share identical primitives

This mirrors the success of systems like Qt and Unity, while avoiding their historical coupling and complexity.

The result is a **tool‑grade UI engine** capable of powering IDEs, inspectors, debuggers, and game editors without divergence.

---

## 7. Evolution, Not Reinvention

OxideUI does not aim to replace every engine subsystem immediately.

The roadmap is intentionally staged:

- [x] 1. **Solidify 2D rendering and UI composition**
  - [x] Decouple rendering from specific backends (WGPU)
  - [x] Define abstract `RenderCommand` system
  - [x] Implement initial `WgpuBackend`
- [x] 2. **Establish a stable rendering core**
  - [x] Refactor platform layer to use new Backend API
  - [x] Implement `submit` command processing in WGPU backend
  - [x] Integrate with main UI loop
- [ ] 3. **Enhance UI Widgets and Interaction**
  - [x] Fix UI resizing and scale factor handling
  - [x] Improve Button text alignment and encapsulation
  - [x] Fix Dropdown menu rendering (Z-ordering/Overlay)
  - [x] Fix TextInput functionality (focus, typing, rendering)
  - [x] Implement text clipping and scrolling
- [ ] 4. **Expand into animation, batching, and advanced layout**
- [ ] 5. **Introduce world‑space rendering (2.5D, then 3D)**
- [ ] 6. **Layer physics, scripting, and plugins only when justified**

Each phase builds on proven foundations rather than speculative features.

---

## 8. Impact and Implications

If successful, OxideUI enables:
- A new class of Rust‑native creative tools
- UI frameworks that scale from apps to engines
- Editors that are reusable, portable, and composable
- A shared foundation for UI, tools, and runtime rendering

More importantly, it challenges the assumption that engines must be graphics‑first and UI‑last.

---

## 9. Conclusion: A Controlled Break from the Past

OxideUI is not an experiment in novelty; it is an experiment in **discipline**.

By constraining abstractions, reversing traditional engine priorities, and leveraging Rust’s strengths, OxideUI seeks to simplify a domain that has grown unnecessarily complex.

This is not a promise of quick success. It is a commitment to architectural correctness over convenience, and to long‑term viability over short‑term adoption.

If the UI is solid, the engine will follow.

---

*This document defines direction, not dogma. It will evolve as the engine evolves—but its core thesis should not.*

