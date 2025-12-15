//! Application management

use oxide_core::event::Event;
use oxide_widgets::widget::Widget;
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
    render_batch: Option<oxide_renderer::RenderBatch>,
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


    /// Render the application with a simple approach (no actual GPU rendering)
    pub fn render_simple(&mut self, window_width: f32, window_height: f32) -> anyhow::Result<()> {
        if let Some(root_widget) = self.root_widget.as_mut() {
            let mut batch = oxide_renderer::RenderBatch::new();
            
            // Compute layout constraints using actual window size
            let constraints = oxide_core::layout::Constraints {
                min_width: window_width,
                max_width: window_width,
                min_height: window_height,
                max_height: window_height,
            };
            

            
            // Layout and render the root widget
            let size = root_widget.layout(constraints);
            let layout = oxide_core::layout::Layout::new(
                glam::Vec2::new(0.0, 0.0),
                size
            );
            root_widget.render(&mut batch, layout);
            
            
            tracing::info!("Rendered {} vertices in batch", batch.vertices.len());
            
            // Return the batch for actual rendering
            self.render_batch = Some(batch);
        } else {
            tracing::warn!("No root widget to render");
        }
        Ok(())
    }
    
    /// Get the current render batch
    pub fn get_render_batch(&mut self) -> Option<oxide_renderer::RenderBatch> {
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
            Event::Window(oxide_core::event::WindowEvent::Close) => {
                // Handle window close - for now, just log it
                // The application will continue running
                use oxide_core::{oxide_info, logging::LogCategory};
                oxide_info!(LogCategory::Platform, "Window close requested");
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
