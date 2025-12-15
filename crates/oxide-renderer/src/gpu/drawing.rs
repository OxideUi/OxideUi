//! Drawing system - integrates all GPU components
//!
//! BLOCCO 7: Drawing System
//! Final integration: converts RenderBatch to GPU draw calls

use crate::batch::RenderBatch;
use crate::vertex::VertexBuilder;
use super::{
    buffer_mgr::{BufferManager, SimpleVertex},
    device::DeviceManager,
    pipeline_mgr::PipelineManager,
    render_pass_mgr::RenderPassManager,
    shader_mgr::ShaderManager,
    surface::SurfaceManager,
    texture_mgr::TextureManager,
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
    texture_mgr: TextureManager,
    pipeline_mgr: PipelineManager,
    render_pass_mgr: RenderPassManager,
    scale_factor: f32,
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
        
        // BLOCCO 8: Texture Management  
        let texture_mgr = TextureManager::new_with_font(device_mgr.device(), device_mgr.queue());
        println!("✅ TextureManager initialized");
        
        // BLOCCO 5: Pipeline Creation
        let pipeline_mgr = PipelineManager::new(
            device_mgr.device(),
            &shader_mgr,
            &buffer_mgr,
            &texture_mgr,
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
            texture_mgr,
            pipeline_mgr,
            render_pass_mgr,
            scale_factor: 1.0,
        })
    }

    /// Set the DPI scale factor
    pub fn set_scale_factor(&mut self, scale_factor: f32) {
        self.scale_factor = scale_factor;
    }

    /// Render a batch
    pub fn render(&mut self, batch: &RenderBatch) -> anyhow::Result<()> {
        // 1. Process batch commands to generate vertices (including text)
        // We rebuild the vertex buffer here to handle text rendering which needs TextureManager
        let mut vertices: Vec<SimpleVertex> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();
        let mut vertex_count = 0;

        // Process direct vertices from batch
        for v in &batch.vertices {
            vertices.push(SimpleVertex::from(v));
        }
        for i in &batch.indices {
            indices.push((*i as u32) + vertex_count);
        }
        vertex_count += batch.vertices.len() as u32;

        for command in &batch.commands {
            match command {
                crate::batch::DrawCommand::RoundedRect { rect, color, radius, transform } => {
                    let color_arr = [color.r, color.g, color.b, color.a];
                    let (v_list, i_list) = VertexBuilder::rounded_rectangle(
                        rect.x, rect.y, rect.width, rect.height, *radius, color_arr, 8
                    );
                    
                    let added_count = v_list.len() as u32;
                    for v in v_list {
                         let mut sv = SimpleVertex::from(&v);
                         // Apply transform
                         let p = oxide_core::types::Point::new(sv.position[0], sv.position[1]);
                         let transformed = transform.transform_point(p);
                         sv.position = [transformed.x, transformed.y];
                         vertices.push(sv);
                    }
                    
                    for i in i_list {
                        indices.push((i as u32) + vertex_count);
                    }
                    vertex_count += added_count;
                }
                crate::batch::DrawCommand::Rect { rect, color, transform } => {
                    // Re-implement rect batching logic here or reuse helper
                    // For simplicity, we duplicate the logic for now to ensure correct ordering
                    let (x, y, w, h) = (rect.x, rect.y, rect.width, rect.height);
                    
                    // Apply transform using oxide_core::Transform method
                    let apply_transform = |p: [f32; 2]| -> [f32; 2] {
                        let point = oxide_core::types::Point::new(p[0], p[1]);
                        let transformed = transform.transform_point(point);
                        [transformed.x, transformed.y]
                    };

                    let p0 = apply_transform([x, y]);
                    let p1 = apply_transform([x + w, y]);
                    let p2 = apply_transform([x + w, y + h]);
                    let p3 = apply_transform([x, y + h]);

                    let color_arr = [color.r, color.g, color.b, color.a];
                    
                    // Solid color vertices (uv = 0,0)
                    vertices.push(SimpleVertex::from(&crate::vertex::Vertex::solid(p0, color_arr)));
                    vertices.push(SimpleVertex::from(&crate::vertex::Vertex::solid(p1, color_arr)));
                    vertices.push(SimpleVertex::from(&crate::vertex::Vertex::solid(p2, color_arr)));
                    vertices.push(SimpleVertex::from(&crate::vertex::Vertex::solid(p3, color_arr)));

                    indices.push(vertex_count);
                    indices.push(vertex_count + 1);
                    indices.push(vertex_count + 2);
                    indices.push(vertex_count);
                    indices.push(vertex_count + 2);
                    indices.push(vertex_count + 3);

                    vertex_count += 4;
                }
                crate::batch::DrawCommand::Text { text, position, color, font_size, letter_spacing, align } => {
                    let (mut x, y) = *position;
                    let color_arr = [color.r, color.g, color.b, color.a];
                    // font_size is &f32, dereference it for use as f32
                    let font_size_val = *font_size; 
                    let spacing_val = *letter_spacing;
                    
                    // Handle alignment
                    if *align != oxide_core::text::TextAlign::Left {
                        let mut width = 0.0;
                        for ch in text.chars() {
                             if let Some(glyph) = self.texture_mgr.get_or_cache_glyph(self.device_mgr.queue(), ch, font_size_val as u32) {
                                 width += glyph.metrics.advance + spacing_val;
                             } else if ch == ' ' {
                                 width += font_size_val * 0.3 + spacing_val;
                             }
                        }
                        
                        match align {
                            oxide_core::text::TextAlign::Center => x -= width / 2.0,
                            oxide_core::text::TextAlign::Right => x -= width,
                            _ => {} // Justify not implemented yet
                        }
                    }
                    
                    // Calculate baseline from top-left y
                    // The layout engine passes top-left coordinates, but font rendering works relative to baseline.
                    // We need to add the ascent to get the baseline Y.
                    // Calculate this ONCE before the loop to avoid borrow checker issues and improve performance.
                    let ascent = if let Some(metrics) = self.texture_mgr.get_line_metrics(font_size_val) {
                        metrics.ascent
                    } else {
                        font_size_val * 0.8 // Fallback approximation
                    };
                    
                    let baseline = y + ascent;

                    for ch in text.chars() {
                        if let Some(glyph) = self.texture_mgr.get_or_cache_glyph(
                            self.device_mgr.queue(),
                            ch, 
                            font_size_val as u32
                        ) {
                            let glyph_x = (x + glyph.metrics.bearing_x as f32).round();
                            
                            // Calculate glyph position relative to baseline
                            // In screen coordinates (Y down), to go "up" to the top of the glyph from baseline,
                            // we subtract the bearing_y (which is distance from baseline to top).
                            let glyph_y = (baseline - glyph.metrics.bearing_y as f32).round();
                            
                            let w = glyph.metrics.width as f32;
                            let h = glyph.metrics.height as f32;
                            let (u0, v0, u1, v1) = glyph.uv_rect;

                            // Textured quad for glyph
                            vertices.push(SimpleVertex::from(&crate::vertex::Vertex::textured(
                                [glyph_x, glyph_y], 
                                [u0, v0],
                                color_arr
                            )));
                            vertices.push(SimpleVertex::from(&crate::vertex::Vertex::textured(
                                [glyph_x + w, glyph_y], 
                                [u1, v0],
                                color_arr
                            )));
                            vertices.push(SimpleVertex::from(&crate::vertex::Vertex::textured(
                                [glyph_x + w, glyph_y + h], 
                                [u1, v1],
                                color_arr
                            )));
                            vertices.push(SimpleVertex::from(&crate::vertex::Vertex::textured(
                                [glyph_x, glyph_y + h], 
                                [u0, v1],
                                color_arr
                            )));

                            indices.push(vertex_count);
                            indices.push(vertex_count + 1);
                            indices.push(vertex_count + 2);
                            indices.push(vertex_count);
                            indices.push(vertex_count + 2);
                            indices.push(vertex_count + 3);

                            vertex_count += 4;
                            
                            // Advance cursor
                            x += glyph.metrics.advance + spacing_val;
                        } else {
                            // Space or unknown char
                            if ch == ' ' {
                                x += font_size_val * 0.3 + spacing_val;
                            }
                        }
                    }
                }
                _ => {} // Handle other commands if needed
            }
        }
        
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
        // Use logical size for projection to handle DPI scaling correctly
        let width = self.surface_mgr.width() as f32;
        let height = self.surface_mgr.height() as f32;
        
        // Adjust projection for DPI scale factor
        // If scale_factor is 2.0 (Retina), physical width is 2x logical width.
        // We want to use logical coordinates (e.g. 0..400) which map to physical pixels (0..800).
        // So we project 0..width/scale to -1..1.
        let logical_width = width / self.scale_factor;
        let logical_height = height / self.scale_factor;
        
        let projection = create_orthographic_projection(logical_width, logical_height);
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
            params: v.params,
            flags: v.flags,
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
