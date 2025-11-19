//! Drawing system - integrates all GPU components
//!
//! BLOCCO 7: Drawing System
//! Final integration: converts RenderBatch to GPU draw calls

use crate::batch::RenderBatch;
use super::{
    buffer_mgr::{BufferManager, SimpleVertex},
    device::DeviceManager,
    pipeline_mgr::PipelineManager,
    render_pass_mgr::RenderPassManager,
    shader_mgr::ShaderManager,
    surface::SurfaceManager,
};
use wgpu::{CommandEncoderDescriptor, IndexFormat};
use std::sync::Arc;
use winit::window::Window;

/// Complete drawing system
pub struct DrawingSystem {
    device_mgr: DeviceManager,
    surface_mgr: SurfaceManager,
    shader_mgr: ShaderManager,
    buffer_mgr: BufferManager,
    pipeline_mgr: PipelineManager,
    render_pass_mgr: RenderPassManager,
}

impl DrawingSystem {
    /// Create new drawing system
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        println!("=== DRAWING SYSTEM INITIALIZATION ===");
        
        // BLOCCO 1: Device Setup
        let device_mgr = DeviceManager::new(wgpu::Backends::all()).await?;
        println!("✅ DeviceManager initialized");
        
        // BLOCCO 2: Surface Configuration
        let surface_mgr = SurfaceManager::new(
            window,
            device_mgr.device(),
            device_mgr.adapter(),
            device_mgr.instance(),
        )?;
        println!("✅ SurfaceManager initialized");
        
        // BLOCCO 3: Shader Compilation
        let shader_mgr = ShaderManager::from_wgsl(
            device_mgr.device(),
            include_str!("../shaders/simple.wgsl"),
            Some("Simple Shader"),
        )?;
        println!("✅ ShaderManager initialized");
        
        // BLOCCO 4: Buffer Management
        let buffer_mgr = BufferManager::new(device_mgr.device());
        println!("✅ BufferManager initialized");
        
        // BLOCCO 5: Pipeline Creation
        let pipeline_mgr = PipelineManager::new(
            device_mgr.device(),
            &shader_mgr,
            &buffer_mgr,
            surface_mgr.format(),
        )?;
        println!("✅ PipelineManager initialized");
        
        // BLOCCO 6: Render Pass
        let render_pass_mgr = RenderPassManager::new();
        println!("✅ RenderPassManager initialized");
        
        println!("====================================");
        
        Ok(Self {
            device_mgr,
            surface_mgr,
            shader_mgr,
            buffer_mgr,
            pipeline_mgr,
            render_pass_mgr,
        })
    }

    /// Render a batch
    pub fn render(&mut self, batch: &RenderBatch) -> anyhow::Result<()> {
        // 1. Convert Vertex to SimpleVertex
        let vertices: Vec<SimpleVertex> = batch
            .vertices
            .iter()
            .map(SimpleVertex::from)
            .collect();
        
        // 2. Convert indices to u32
        let indices: Vec<u32> = batch.indices.iter().map(|&i| i as u32).collect();
        
        // 3. Upload vertices and indices to GPU
        self.buffer_mgr.upload_vertices(
            self.device_mgr.device(),
            self.device_mgr.queue(),
            &vertices,
        );
        self.buffer_mgr.upload_indices(
            self.device_mgr.device(),
            self.device_mgr.queue(),
            &indices,
        );
        
        // 4. Upload projection matrix (orthographic for 2D)
        let width = self.surface_mgr.width() as f32;
        let height = self.surface_mgr.height() as f32;
        let projection = create_orthographic_projection(width, height);
        self.buffer_mgr.upload_projection(self.device_mgr.queue(), &projection);
        
        // 5. Get surface texture
        let surface_texture = self.surface_mgr.get_current_texture()?;
        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        
        // 6. Create command encoder
        let mut encoder = self
            .device_mgr
            .device()
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        
        // 7. Begin render pass
        {
            let mut render_pass = self.render_pass_mgr.begin(&mut encoder, &view);
            
            // 8. Set pipeline and bind groups
            render_pass.set_pipeline(self.pipeline_mgr.pipeline());
            render_pass.set_bind_group(0, self.pipeline_mgr.bind_group(), &[]);
            
            // 9. Set vertex/index buffers
            render_pass.set_vertex_buffer(0, self.buffer_mgr.vertex_buffer().slice(..));
            render_pass.set_index_buffer(
                self.buffer_mgr.index_buffer().slice(..),
                IndexFormat::Uint32,
            );
            
            // 10. Draw indexed
            render_pass.draw_indexed(0..indices.len() as u32, 0, 0..1);
        }
        
        // 11. Submit command buffer
        self.device_mgr.queue().submit(std::iter::once(encoder.finish()));
        
        // 12. Present surface
        surface_texture.present();
        
        Ok(())
    }
    
    /// Resize surface
    pub fn resize(&mut self, width: u32, height: u32) -> anyhow::Result<()> {
        self.surface_mgr.resize(width, height, self.device_mgr.device())?;
        
        // Update projection matrix
        let projection = create_orthographic_projection(width as f32, height as f32);
        self.buffer_mgr.upload_projection(self.device_mgr.queue(), &projection);
        
        Ok(())
    }
}

/// Create orthographic projection matrix for 2D rendering
fn create_orthographic_projection(width: f32, height: f32) -> [[f32; 4]; 4] {
    // NDC: x: -1 to 1, y: -1 to 1
    // Screen: x: 0 to width, y: 0 to height
    let left = 0.0;
    let right = width;
    let bottom = height;
    let top = 0.0;
    
    [
        [2.0 / (right - left), 0.0, 0.0, 0.0],
        [0.0, 2.0 / (top - bottom), 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [
            -(right + left) / (right - left),
            -(top + bottom) / (top - bottom),
            0.0,
            1.0,
        ],
    ]
}

/// Convert existing Vertex to SimpleVertex
impl From<&crate::vertex::Vertex> for SimpleVertex {
    fn from(v: &crate::vertex::Vertex) -> Self {
        Self {
            position: v.position,
            color: v.color,
            uv: v.uv,  // Use UV from existing Vertex struct
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_vertex_conversion() {
        let vertex = crate::vertex::Vertex::solid([100.0, 200.0], [1.0, 0.0, 0.0, 1.0]);
        let simple: SimpleVertex = (&vertex).into();
        
        assert_eq!(simple.position, [100.0, 200.0]);
        assert_eq!(simple.color, [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(simple.uv, vertex.uv);
    }
    
    #[test]
    fn test_orthographic_projection() {
        let proj = create_orthographic_projection(800.0, 600.0);
        
        // Top-left corner (0, 0) should map to NDC (-1, 1)
        // Bottom-right (800, 600) should map to NDC (1, -1)
        
        // Check matrix is not identity
        assert_ne!(proj[0][0], 1.0);
        assert_ne!(proj[1][1], 1.0);
    }
}
