use anyhow::Result;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use self::commands::RenderCommand;

pub mod commands;
pub mod wgpu;

pub use wgpu::WgpuBackend;

/// Trait that all rendering backends must implement.
/// This decouples the engine from specific graphics APIs (wgpu, vulkan, etc.).
use async_trait::async_trait;

#[async_trait]
pub trait Backend: Send + Sync {
    /// Resize the backend surface
    fn resize(&mut self, width: u32, height: u32);

    /// Set the scale factor (DPI)
    fn set_scale_factor(&mut self, scale_factor: f64);

    /// Begin a new frame
    fn begin_frame(&mut self) -> Result<()>;

    /// End the current frame and present
    fn end_frame(&mut self) -> Result<()>;

    /// Submit a list of render commands to be executed
    fn submit(&mut self, commands: &[RenderCommand]) -> Result<()>;

    /// Submit a render batch for execution (optimized path)
    fn submit_batch(&mut self, _batch: &crate::batch::RenderBatch) -> Result<()> {
         // Default implementation falls back to submit if possible, or errors?
         // Since RenderBatch contains DrawCommands which are not exactly RenderCommands (DrawCommand vs RenderCommand),
         // we can't easily fallback without conversion logic.
         // Let's make it mandatory or return logic error.
         Err(anyhow::anyhow!("submit_batch not implemented for this backend"))
    }
}
