#[cfg(winit)]
pub mod winit;

pub use strato_ui_core::windowing::*;
#[cfg(target_os = "linux")]
pub use winit::WindowingSystem;
