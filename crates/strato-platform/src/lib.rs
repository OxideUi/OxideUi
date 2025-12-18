//! Platform abstraction layer for StratoUI framework
//!
//! Provides cross-platform window management and event handling.

pub mod application;
pub mod event_loop;
pub mod window;

#[cfg(not(target_arch = "wasm32"))]
pub mod desktop;

#[cfg(target_arch = "wasm32")]
pub mod web;

pub use application::{Application, ApplicationBuilder};
pub use event_loop::{EventLoop, EventLoopProxy};
pub use window::{Window, WindowBuilder, WindowId};

use strato_core::event::Event;

/// Platform-specific error type
#[derive(Debug, thiserror::Error)]
pub enum PlatformError {
    #[error("Window creation failed: {0}")]
    WindowCreation(String),

    #[error("Event loop error: {0}")]
    EventLoop(String),

    #[error("Platform not supported")]
    Unsupported,

    #[error("WebAssembly error: {0}")]
    #[cfg(target_arch = "wasm32")]
    Wasm(String),
}

/// Platform trait for OS-specific implementations
pub trait Platform {
    /// Initialize the platform
    fn init() -> Result<Self, PlatformError>
    where
        Self: Sized;

    /// Create a window
    fn create_window(&mut self, builder: WindowBuilder) -> Result<Window, PlatformError>;

    /// Run the event loop
    fn run_event_loop(
        &mut self,
        callback: Box<dyn FnMut(Event) + 'static>,
    ) -> Result<(), PlatformError>;

    /// Request a redraw
    fn request_redraw(&self, window_id: WindowId);

    /// Exit the application
    fn exit(&mut self);
}

/// Get the current platform implementation
pub fn current_platform() -> Box<dyn Platform> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        Box::new(desktop::DesktopPlatform::new())
    }

    #[cfg(target_arch = "wasm32")]
    {
        Box::new(web::WebPlatform::new())
    }
}

pub mod init;

/// Initialize the platform layer
pub fn init() -> Result<(), PlatformError> {
    tracing::info!("StratoUI Platform initialized");

    #[cfg(target_arch = "wasm32")]
    {
        console_error_panic_hook::set_once();
    }

    Ok(())
}
