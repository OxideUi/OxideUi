//! Application management

use oxide_core::event::Event;
use oxide_widgets::widget::Widget;
use oxide_renderer::gpu::Renderer;
use crate::{Window, WindowBuilder, EventLoop};
use std::collections::HashMap;

/// Application builder
pub struct ApplicationBuilder {
    title: String,
    initial_window: WindowBuilder,
}

impl ApplicationBuilder {
    /// Create a new application builder
    pub fn new() -> Self {
        Self {
            title: "OxideUI Application".to_string(),
            initial_window: WindowBuilder::new(),
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

    /// Build the application
    pub fn build(self) -> Application {
        Application::new(self.title, self.initial_window)
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
    // Renderer is managed by the event loop to avoid lifetime issues
}

impl Application {
    /// Create a new application
    pub fn new(title: impl Into<String>, initial_window: WindowBuilder) -> Self {
        Self {
            title: title.into(),
            windows: HashMap::new(),
            root_widget: None,
            event_loop: Some(EventLoop::new()),
            initial_window: Some(initial_window),
        }
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

    /// Render the application using the provided renderer
    pub fn render(&mut self, renderer: &mut Renderer<'_>) -> anyhow::Result<()> {
        if let Some(root_widget) = self.root_widget.as_mut() {
            let mut batch = oxide_renderer::batch::RenderBatch::new();
            
            // Create a layout for the root widget
            let constraints = oxide_core::layout::Constraints::loose(800.0, 600.0);
            let size = root_widget.layout(constraints);
            let layout = oxide_core::layout::Layout::new(
                glam::Vec2::new(0.0, 0.0),
                size
            );
            
            // Render the root widget
            root_widget.render(&mut batch, layout);
            
            // Render the batch
            renderer.render(&batch)?;
        }
        Ok(())
    }

    /// Run the application
    pub fn run(mut self) -> ! {
        let event_loop = self.event_loop.take().expect("Event loop already taken");
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(window_builder) = self.initial_window.take() {
                event_loop.run_with_window_and_app(window_builder, self)
            } else {
                event_loop.run(move |event| {
                    self.handle_event(event);
                })
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
            Event::Window(oxide_core::event::WindowEvent::Close) => {
                // Handle window close - for now, just log it
                // The application will continue running
                println!("Window close requested");
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
