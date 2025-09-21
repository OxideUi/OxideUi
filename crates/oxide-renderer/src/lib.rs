//! OxideUI Renderer
//!
//! A high-performance, GPU-accelerated 2D renderer built on wgpu.
//! Designed for modern UI frameworks with a focus on performance and flexibility.
//!
//! ## Features
//! - Advanced GPU device management with multi-adapter support
//! - Intelligent resource management with automatic pooling and deframmentation
//! - Sophisticated memory management with multi-tier allocation
//! - Dynamic shader compilation with hot-reload support
//! - Modular pipeline system with render graph optimization
//! - Efficient buffer management with lock-free operations
//! - Comprehensive performance profiling and monitoring
//! - Enterprise-grade error handling and recovery

pub mod batch;
pub mod buffer;
pub mod device;
pub mod font_config;
pub mod font_system;
pub mod glyph_atlas;
pub mod gpu;
pub mod memory;
pub mod pipeline;
pub mod profiler;
pub mod resources;
pub mod shader;
pub mod text;
pub mod texture;
pub mod vertex;

pub mod integration;

// Re-export commonly used types
pub use batch::RenderBatch;
pub use buffer::{BufferManager, DynamicBuffer, BufferPool};
pub use device::{ManagedDevice, DeviceManager, AdapterInfo};
pub use integration::{IntegratedRenderer, RendererBuilder, RenderContext, RenderStats};
pub use memory::{MemoryManager, MemoryPool, AllocationStrategy};
pub use pipeline::{PipelineManager, RenderGraph, RenderNode};
pub use profiler::{Profiler, PerformanceReport, FrameStats};
pub use resources::{ResourceManager, ResourceHandle, ResourceType};
pub use shader::{ShaderManager, ShaderSource, CompiledShader};



use oxide_core::types::{Color, Rect, Transform};
use wgpu::Surface;

/// Rendering backend trait
pub trait Backend: Send + Sync {
    /// Initialize the backend
    fn init(&mut self, surface: &Surface) -> anyhow::Result<()>;
    
    /// Begin a new frame
    fn begin_frame(&mut self) -> anyhow::Result<()>;
    
    /// End the current frame
    fn end_frame(&mut self) -> anyhow::Result<()>;
    
    /// Draw a rectangle
    fn draw_rect(&mut self, rect: Rect, color: Color, transform: Transform);
    
    /// Draw text
    fn draw_text(&mut self, text: &str, position: (f32, f32), color: Color);
    
    /// Set the background color
    fn set_background_color(&mut self, color: Color);
    
    /// Submit draw commands
    fn submit(&mut self) -> anyhow::Result<()>;
}

/// Renderer configuration
#[derive(Debug, Clone)]
pub struct RendererConfig {
    /// Enable MSAA
    pub msaa_samples: u32,
    /// Enable vsync
    pub vsync: bool,
    /// Maximum texture atlas size
    pub max_texture_size: u32,
    /// Enable GPU validation (debug mode)
    pub validation: bool,
}

impl Default for RendererConfig {
    fn default() -> Self {
        Self {
            msaa_samples: 1,
            vsync: true,
            max_texture_size: 4096,
            validation: cfg!(debug_assertions),
        }
    }
}

/// Initialize the renderer
pub fn init(config: RendererConfig) -> anyhow::Result<()> {
    tracing::info!("Initializing OxideUI Renderer with config: {:?}", config);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = RendererConfig::default();
        assert_eq!(config.msaa_samples, 4);
        assert!(config.vsync);
    }
}
