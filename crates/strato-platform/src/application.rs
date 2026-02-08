//! Application management

use crate::{EventLoop, Window, WindowBuilder};
use std::collections::HashMap;
use strato_core::event::Event;
use strato_widgets::widget::Widget;

/// Application builder
pub struct ApplicationBuilder {
    title: String,
    initial_window: WindowBuilder,
    use_taffy: bool,
}

impl ApplicationBuilder {
    /// Create a new application builder
    pub fn new() -> Self {
        Self {
            title: "StratoUI Application".to_string(),
            initial_window: WindowBuilder::new(),
            use_taffy: false,
        }
    }

    /// Set application title
    pub fn title(mut self, title: impl Into<String>) -> Self {
        let title_string = title.into();
        self.title = title_string.clone();
        self.initial_window = self.initial_window.with_title(title_string);
        self
    }

    /// Set initial window configuration
    pub fn window(mut self, window: WindowBuilder) -> Self {
        self.initial_window = window;
        self
    }

    /// Enable Taffy layout engine
    pub fn with_taffy(mut self, enabled: bool) -> Self {
        self.use_taffy = enabled;
        self
    }

    /// Build the application
    pub fn build(self) -> Application {
        let mut app = Application::new(self.title, self.initial_window);
        if self.use_taffy {
            app.enable_taffy();
        }
        app
    }

    /// Run the application with a root widget
    pub fn run<W: Widget + 'static>(self, root: W) -> ! {
        let mut app = self.build();
        app.set_root(Box::new(root));
        app.run()
    }
}

/// Main application structure
pub struct Application {
    title: String,
    windows: HashMap<u64, Window>,
    root_widget: Option<Box<dyn Widget>>,
    event_loop: Option<EventLoop>,
    initial_window: Option<WindowBuilder>,
    render_batch: Option<strato_renderer::RenderBatch>,
    taffy_manager: Option<strato_core::taffy_layout::TaffyLayoutManager>,
    // Renderer is managed by the event loop to avoid lifetime issues
}

impl Application {
    /// Create a new application
    pub fn new(title: impl Into<String>, initial_window: WindowBuilder) -> Self {
        Self {
            title: title.into(),
            windows: HashMap::new(),
            root_widget: None,
            event_loop: Some(EventLoop::new().expect("Failed to create event loop")),
            initial_window: Some(initial_window),
            render_batch: None,
            taffy_manager: None,
        }
    }

    /// Enable Taffy layout engine
    pub fn enable_taffy(&mut self) {
        self.taffy_manager = Some(strato_core::taffy_layout::TaffyLayoutManager::new());
    }

    /// Set the root widget
    pub fn set_root(&mut self, widget: Box<dyn Widget>) {
        self.root_widget = Some(widget);
    }

    /// Add a window
    pub fn add_window(&mut self, window: Window) {
        self.windows.insert(window.id(), window);
    }

    /// Remove a window
    pub fn remove_window(&mut self, id: u64) -> Option<Window> {
        self.windows.remove(&id)
    }

    /// Get a window by ID
    pub fn window(&self, id: u64) -> Option<&Window> {
        self.windows.get(&id)
    }

    /// Get mutable window by ID
    pub fn window_mut(&mut self, id: u64) -> Option<&mut Window> {
        self.windows.get_mut(&id)
    }

    /// Render the application with a simple approach (no actual GPU rendering)
    pub fn render_simple(&mut self, window_width: f32, window_height: f32) -> anyhow::Result<()> {
        if let Some(root_widget) = self.root_widget.as_mut() {
            let mut batch = strato_renderer::RenderBatch::new();

            // Compute layout constraints using actual window size
            let constraints = strato_core::layout::Constraints {
                min_width: window_width,
                max_width: window_width,
                min_height: window_height,
                max_height: window_height,
            };

            // Layout and render the root widget
            if let Some(taffy_manager) = &mut self.taffy_manager {
                 // Try to use Taffy layout
                 // We need to check if the root widget supports Taffy
                 if let Some(taffy_root) = root_widget.as_taffy() {
                     let size = strato_core::taffy::geometry::Size {
                         width: window_width,
                         height: window_height,
                     };
                     
                     match taffy_manager.compute(taffy_root, size) {
                         Ok(computed_layout) => {
                             // Render using Taffy draw commands
                             // We need to map draw commands to render batch
                             // For now, Taffy doesn't have a direct "render to batch" utility that matches the recursive render() pattern perfectly
                             // because render() expects a mutable batch and recursive calls.
                             // But ComputedLayout gives us a flat list of commands with viewports.
                             // However, the *rendering* logic (drawing rects, text) is inside `Widget::render`.
                             // `Widget::render` expects a `Layout` object.
                             
                             // So we iterate through draw commands, find the widget (by NodeId?? No, we don't have a map from NodeId to Widget reference readily available here unless we traverse).
                             
                             // Wait, TaffyLayoutManager::compute returns ComputedLayout which has NodeIds.
                             // But to call render() on widgets, we need reference to the actual widgets.
                             // Taffy doesn't store widget references.
                             
                             // Alternative: Pass the ComputedLayout TO the recursive render?
                             // OR: Just use the root_widget.render() but with the size calculated by Taffy?
                             
                             // Use Case 1: Root is a TaffyWidget (e.g. Column). 
                             // taffy_manager.compute() returns the layout for the whole tree.
                             // But we need to invoke render() on the tree.
                             
                             // In the legacy system:
                             // root.layout(constraints) -> determines size and positions children internally.
                             // root.render(batch, layout) -> renders self and calls children.render().
                             
                             // In Taffy system:
                             // taffy_manager.compute() -> calculates all positions.
                             // BUT `root_widget.render()` still follows legacy pattern: it receives a Layout (pos, size) and renders.
                             // *However*, legacy `render` usually assumes it already knows children positions (stored in the widget state during layout()).
                             // My Taffy implementation separates layout state from widget state.
                             
                             // Implementation detail: `TaffyWidget` has `render`? 
                             // No, `TaffyWidget` only has `build_layout`.
                             // `Widget` has `render`.
                             
                             // PROPER SOLUTION:
                             // 1. Compute layout with Taffy.
                             // 2. We need to "apply" the layout to the widgets so they know where they are?
                             //    Or pass the Taffy layout map to the render function?
                             //    The `ComputedLayout` contains `DrawCommand`s which have `NodeId` and `ValidatedRect`.
                             //    It doesn't link back to Widget instances easily.
                             
                             //    Actually, `TaffyLayoutManager` builds the tree from the widgets.
                             //    The widgets generally don't store their Taffy NodeId (unless we added it? `BaseWidget` has `id: WidgetId`).
                             
                             //    This reveals a gap in my Taffy integration plan vs `strato-platform` integration.
                             //    If I use Taffy, `root_widget.layout()` is NOT called. So `root_widget` doesn't update its internal layout state.
                             //    If `root_widget.render()` relies on that state, it will render at (0,0) or wrong size.
                             
                             //    Legacy `Column::render`:
                             //    `let child_layout = self.children_layouts[i];`
                             //    It uses cached layout from `layout()`.
                             
                             //    So, Taffy layout needs to either:
                             //    A) Update the widget's internal layout state (requires mutable access to widget tree + mapping Taffy Nodes to Widgets).
                             //    B) Be passed down during render. `root.render(batch, layout, &taffy_map)`.
                             
                             //    Option B requires changing `Widget::render` signature, which is a breaking change for ALL widgets. I want to avoid that if possible, or do it carefully.
                             //    Option A is hard because Taffy NodeId != WidgetId.
                             
                             //    Wait, I implemented `TaffyLayoutManager`.
                             //    How did I intend to render?
                             //    In `task.md`: "Implement TaffyWidget...".
                             //    In `walkthrough.md`: "Render using Taffy draw commands... TaffyLayoutManager::compute... for cmd in layout.draw_commands() { // Render widget at cmd.viewport }".
                             //    BUT `DrawCommand` only has `NodeId`. It doesn't have the Widget.
                             //    So I can't call `widget.render()`.
                             
                             //    I need a way to look up the Widget from the NodeId or traversal order.
                             
                             //    Crude Fix for `taffy_demo` window:
                             //    In `taffy_demo`, I construct the tree manually.
                             
                             //    For `Application` integration:
                             //    I can't easily map NodeId -> Widget without a map.
                             //    `TaffyLayoutManager` doesn't keep a map.
                             
                             //    Maybe I should fallback to legacy for now in `render_simple` and NOT use Taffy in `Application` yet, 
                             //    BUT `taffy_demo` needs to see something.
                             
                             //    If I want `taffy_demo` to work, I should implement the render loop IN `taffy_demo` manually, 
                             //    where I hold both the widget tree and the layout manager.
                             //    `taffy_demo` constructs the tree.
                             //    It can traverse it and render.
                             
                             //    For `Application`, support is blocked by "How to render Taffy layout without widget mapping".
                             
                             //    Let's revert `Application` changes regarding Taffy for now? 
                             //    OR keep `use_taffy` but strictly for "If you provide a Taffy-ready root, we expect... something?"
                             
                             //    Actually, look at `crates/strato-widgets/src/layout.rs`. 
                             //    Does `Column` implement `TaffyWidget`? Yes.
                             //    Does it implement `render` using Taffy? No.
                             
                             //    So `taffy_demo` CANNOT simply plug into `Application` expecting magic.
                             
                             //    The best path for `taffy_demo` windowing is to write a CUSTOM render loop in `taffy_demo` using `winit` and `NonNull` raw pointers or `Rc/RefCell` to map widgets?
                             //    Actually, if I traverse the widget tree in standard order (DFS), and Taffy builds in DFS...
                             //    Taffy NodeIds are sequential?
                             //    If I traverse the widget tree and query Taffy layout by index/order...
                             
                             //    Let's stick to the user request: "modifichiamo e riadattiamo tutti gli example".
                             //    I should fix `taffy_demo` first.
                             //    I will modify `taffy_demo/src/main.rs` to create a window using `winit` directly (copying from `hello_world` but swapping internal logic).
                             //    AND defining the render loop there.
                             
                             //    So I should undo changes to `Application.rs`? Or leave them as "infrastructure for later"?
                             //    Leaving them is fine, but `enable_taffy` won't work yet.
                             //    I'll remove the `if let Some(taffy_manager)` block I was about to add.
                             
                             //    Let's ABORT the `render_simple` replacement call.
                             //    I will KEEP the `taffy_manager` field and builder methods (they are harmless), 
                             //    but I won't use them in `render_simple` yet.
                             
                             tracing::warn!("Taffy layout enabled but rendering path not fully implemented in Application");
                             // Fallback to legacy
                             let size = root_widget.layout(constraints);
                             let layout = strato_core::layout::Layout::new(glam::Vec2::new(0.0, 0.0), size);
                             root_widget.render(&mut batch, layout);
                         }
                         Err(e) => {
                             tracing::error!("Taffy layout failed: {}", e);
                         }
                     }
                 } else {
                     // Root doesn't support Taffy
                     let size = root_widget.layout(constraints);
                     let layout = strato_core::layout::Layout::new(glam::Vec2::new(0.0, 0.0), size);
                     root_widget.render(&mut batch, layout);
                 }
            } else {
                // Legacy
                let size = root_widget.layout(constraints);
                let layout = strato_core::layout::Layout::new(glam::Vec2::new(0.0, 0.0), size);
                root_widget.render(&mut batch, layout);
            }

            tracing::info!("Rendered {} vertices in batch", batch.vertices.len());

            // Return the batch for actual rendering
            self.render_batch = Some(batch);
        } else {
            tracing::warn!("No root widget to render");
        }
        Ok(())
    }

    /// Get the current render batch
    pub fn get_render_batch(&mut self) -> Option<strato_renderer::RenderBatch> {
        self.render_batch.take()
    }

    /// Run the application
    pub fn run(mut self) -> ! {
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(window_builder) = self.initial_window.take() {
                if let Some(event_loop) = self.event_loop.take() {
                    match event_loop.run_with_window_and_app(window_builder, self, move |_event| {
                        // Event handling is now done inside the event loop
                    }) {
                        Ok(_) => std::process::exit(0),
                        Err(e) => {
                            eprintln!("Event loop error: {}", e);
                            std::process::exit(1);
                        }
                    }
                } else {
                    eprintln!("No event loop available");
                    std::process::exit(1);
                }
            } else {
                if let Some(event_loop) = self.event_loop.take() {
                    event_loop.run(move |_event| {
                        // Handle event
                    });
                } else {
                    eprintln!("No event loop available");
                    std::process::exit(1);
                }
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            panic!("WebAssembly platform not fully implemented");
        }
    }

    /// Handle an event
    pub fn handle_event(&mut self, event: Event) {
        // Dispatch event to root widget
        if let Some(widget) = &mut self.root_widget {
            widget.handle_event(&event);
        }

        // Handle application-level events
        match event {
            Event::Window(strato_core::event::WindowEvent::Close) => {
                // Handle window close - for now, just log it
                // The application will continue running
                use strato_core::{logging::LogCategory, strato_info};
                strato_info!(LogCategory::Platform, "Window close requested");
            }
            _ => {}
        }
    }
}

impl Default for ApplicationBuilder {
    fn default() -> Self {
        Self::new()
    }
}
