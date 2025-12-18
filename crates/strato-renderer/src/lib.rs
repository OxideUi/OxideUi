//! StratoUI Renderer
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
pub mod gpu; // Modular GPU pipeline
pub mod memory;
pub mod pipeline;
pub mod profiler;
pub mod resources;
pub mod shader;
pub mod text;
pub mod texture;
pub mod vertex;

pub mod backend;

pub mod integration;

// Re-export commonly used types
pub use backend::commands::RenderCommand;
pub use backend::Backend;
pub use batch::RenderBatch;
pub use buffer::{BufferManager, BufferPool, DynamicBuffer};
pub use device::{AdapterInfo, DeviceManager, ManagedDevice};
pub use integration::{IntegratedRenderer, RenderContext, RenderStats, RendererBuilder};
pub use memory::{AllocationStrategy, MemoryManager, MemoryPool};
pub use pipeline::{PipelineManager, RenderGraph, RenderNode};
pub use profiler::{FrameStats, PerformanceReport, Profiler};
pub use resources::{ResourceHandle, ResourceManager, ResourceType};
pub use shader::{CompiledShader, ShaderManager, ShaderSource};

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
            msaa_samples: 4,
            vsync: true,
            max_texture_size: 4096,
            validation: cfg!(debug_assertions),
        }
    }
}

/// Initialize the renderer
pub fn init(config: RendererConfig) -> anyhow::Result<()> {
    tracing::info!("Initializing StratoUI Renderer with config: {:?}", config);
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
