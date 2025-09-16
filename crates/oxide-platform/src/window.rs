//! Window management

use oxide_core::{Size, types::Point};
use std::sync::Arc;

/// Window identifier
pub type WindowId = u64;

/// Window handle
pub struct Window {
    pub id: WindowId,
    pub(crate) inner: WindowInner,
}

pub(crate) enum WindowInner {
    #[cfg(not(target_arch = "wasm32"))]
    Desktop(Arc<winit::window::Window>),
    #[cfg(target_arch = "wasm32")]
    Web(web_sys::HtmlCanvasElement),
}

impl Window {
    /// Get window ID
    pub fn id(&self) -> WindowId {
        self.id
    }

    /// Get window size
    pub fn size(&self) -> Size {
        match &self.inner {
            #[cfg(not(target_arch = "wasm32"))]
            WindowInner::Desktop(window) => {
                let size = window.inner_size();
                Size::new(size.width as f32, size.height as f32)
            }
            #[cfg(target_arch = "wasm32")]
            WindowInner::Web(canvas) => {
                Size::new(canvas.width() as f32, canvas.height() as f32)
            }
        }
    }

    /// Set window size
    pub fn set_size(&self, size: Size) {
        match &self.inner {
            #[cfg(not(target_arch = "wasm32"))]
            WindowInner::Desktop(window) => {
                let _ = window.request_inner_size(winit::dpi::LogicalSize::new(
                    size.width,
                    size.height,
                ));
            }
            #[cfg(target_arch = "wasm32")]
            WindowInner::Web(canvas) => {
                canvas.set_width(size.width as u32);
                canvas.set_height(size.height as u32);
            }
        }
    }

    /// Get window position
    pub fn position(&self) -> Point {
        match &self.inner {
            #[cfg(not(target_arch = "wasm32"))]
            WindowInner::Desktop(window) => {
                window.outer_position()
                    .map(|pos| Point::new(pos.x as f32, pos.y as f32))
                    .unwrap_or(Point::zero())
            }
            #[cfg(target_arch = "wasm32")]
            WindowInner::Web(_) => Point::zero(),
        }
    }

    /// Set window title
    pub fn set_title(&self, title: &str) {
        match &self.inner {
            #[cfg(not(target_arch = "wasm32"))]
            WindowInner::Desktop(window) => {
                window.set_title(title);
            }
            #[cfg(target_arch = "wasm32")]
            WindowInner::Web(_) => {
                if let Some(document) = web_sys::window().and_then(|w| w.document()) {
                    document.set_title(title);
                }
            }
        }
    }

    /// Request redraw
    pub fn request_redraw(&self) {
        match &self.inner {
            #[cfg(not(target_arch = "wasm32"))]
            WindowInner::Desktop(window) => {
                window.request_redraw();
            }
            #[cfg(target_arch = "wasm32")]
            WindowInner::Web(_) => {
                // Web platform handles this differently
            }
        }
    }
}

/// Window builder
#[derive(Debug, Clone)]
pub struct WindowBuilder {
    pub title: String,
    pub size: Size,
    pub position: Option<Point>,
    pub resizable: bool,
    pub decorations: bool,
    pub transparent: bool,
    pub always_on_top: bool,
    pub fullscreen: bool,
    pub min_size: Option<Size>,
    pub max_size: Option<Size>,
}

impl WindowBuilder {
    /// Create a new window builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set window title
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Set window size
    pub fn with_size(mut self, width: f32, height: f32) -> Self {
        self.size = Size::new(width, height);
        self
    }

    /// Set window position
    pub fn with_position(mut self, x: f32, y: f32) -> Self {
        self.position = Some(Point::new(x, y));
        self
    }

    /// Set resizable
    pub fn resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    /// Set decorations
    pub fn decorations(mut self, decorations: bool) -> Self {
        self.decorations = decorations;
        self
    }

    /// Set transparent
    pub fn transparent(mut self, transparent: bool) -> Self {
        self.transparent = transparent;
        self
    }

    /// Set always on top
    pub fn always_on_top(mut self, always_on_top: bool) -> Self {
        self.always_on_top = always_on_top;
        self
    }

    /// Set fullscreen
    pub fn fullscreen(mut self, fullscreen: bool) -> Self {
        self.fullscreen = fullscreen;
        self
    }

    /// Set minimum size
    pub fn min_size(mut self, width: f32, height: f32) -> Self {
        self.min_size = Some(Size::new(width, height));
        self
    }

    /// Set maximum size
    pub fn max_size(mut self, width: f32, height: f32) -> Self {
        self.max_size = Some(Size::new(width, height));
        self
    }

    /// Build winit window
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn build_winit(&self, event_loop: &winit::event_loop::EventLoopWindowTarget<crate::event_loop::CustomEvent>) -> Result<winit::window::Window, winit::error::OsError> {
        let mut builder = winit::window::WindowBuilder::new()
            .with_title(&self.title)
            .with_inner_size(winit::dpi::LogicalSize::new(self.size.width, self.size.height))
            .with_resizable(self.resizable)
            .with_decorations(self.decorations)
            .with_transparent(self.transparent)
            .with_window_level(if self.always_on_top { winit::window::WindowLevel::AlwaysOnTop } else { winit::window::WindowLevel::Normal });

        if let Some(pos) = self.position {
            builder = builder.with_position(winit::dpi::LogicalPosition::new(pos.x, pos.y));
        }

        if let Some(min) = self.min_size {
            builder = builder.with_min_inner_size(winit::dpi::LogicalSize::new(min.width, min.height));
        }

        if let Some(max) = self.max_size {
            builder = builder.with_max_inner_size(winit::dpi::LogicalSize::new(max.width, max.height));
        }

        if self.fullscreen {
            builder = builder.with_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
        }

        builder.build(event_loop)
    }
}

impl Default for WindowBuilder {
    fn default() -> Self {
        Self {
            title: "OxideUI Application".to_string(),
            size: Size::new(800.0, 600.0),
            position: None,
            resizable: true,
            decorations: true,
            transparent: false,
            always_on_top: false,
            fullscreen: false,
            min_size: Some(Size::new(200.0, 100.0)),
            max_size: None,
        }
    }
}
