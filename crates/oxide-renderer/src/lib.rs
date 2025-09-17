//! GPU-accelerated renderer for OxideUI framework
//!
//! This crate provides the rendering backend using wgpu for cross-platform GPU acceleration.

pub mod gpu;
pub mod pipeline;
pub mod text;
pub mod texture;
pub mod batch;
pub mod vertex;
pub mod font_system;

pub use gpu::{Renderer, RenderContext, Surface};
pub use pipeline::{UIPipeline, TextPipeline, PipelineManager, UIUniforms};
pub use text::{TextRenderer, Font, GlyphCache};
pub use texture::{Texture, TextureAtlas};
pub use batch::{DrawCommand, RenderBatch};
pub use vertex::{Vertex, TextVertex, VertexBuilder};
pub use font_system::*;

use oxide_core::types::{Color, Rect, Transform};

/// Rendering backend trait
pub trait Backend<'a>: Send + Sync {
    /// Initialize the backend
    fn init(&mut self, surface: &Surface<'a>) -> anyhow::Result<()>;
    
    /// Begin a new frame
    fn begin_frame(&mut self) -> anyhow::Result<()>;
    
    /// End the current frame
    fn end_frame(&mut self) -> anyhow::Result<()>;
    
    /// Draw a rectangle
    fn draw_rect(&mut self, rect: Rect, color: Color, transform: Transform);
    
    /// Draw text
    fn draw_text(&mut self, text: &str, position: (f32, f32), color: Color);
    
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
