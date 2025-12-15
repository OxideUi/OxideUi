//! Render pass management
//!
//! BLOCCO 6: Render Pass
//! Handles render pass setup and execution

use wgpu::{
    CommandEncoder, LoadOp, Operations, RenderPass, RenderPassColorAttachment,
    RenderPassDescriptor, StoreOp, TextureView,
};

/// Manages render pass configuration
pub struct RenderPassManager {
    clear_color: wgpu::Color,
}

impl RenderPassManager {
    /// Create new render pass manager
    pub fn new() -> Self {
        Self {
            clear_color: wgpu::Color {
                r: 0.23,
                g: 0.23,
                b: 0.23,
                a: 1.0,
            },
        }
    }

    /// Set clear color
    pub fn set_clear_color(&mut self, color: wgpu::Color) {
        self.clear_color = color;
    }

    /// Begin render pass
    ///
    /// # Arguments
    /// * `encoder` - Command encoder
    /// * `view` - Target texture view
    pub fn begin<'a>(
        &self,
        encoder: &'a mut CommandEncoder,
        view: &'a TextureView,
    ) -> RenderPass<'a> {
        // TODO: Create render pass descriptor
        // - Color attachment with clear color
        // - LoadOp::Clear, StoreOp::Store
        // - No depth/stencil
        
        encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Main Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(self.clear_color),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        })
    }
}

impl Default for RenderPassManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_clear_color() {
        let render_pass_mgr = RenderPassManager::new();
        
        // Default dark gray
        assert_eq!(render_pass_mgr.clear_color.r, 0.23);
        assert_eq!(render_pass_mgr.clear_color.g, 0.23);
        assert_eq!(render_pass_mgr.clear_color.b, 0.23);
        assert_eq!(render_pass_mgr.clear_color.a, 1.0);
    }

    #[test]
    fn test_set_clear_color() {
        let mut render_pass_mgr = RenderPassManager::new();
        
        let new_color = wgpu::Color {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        };
        
        render_pass_mgr.set_clear_color(new_color);
        
        assert_eq!(render_pass_mgr.clear_color.r, 1.0);
        assert_eq!(render_pass_mgr.clear_color.g, 0.0);
    }
}
