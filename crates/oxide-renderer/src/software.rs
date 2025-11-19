//! Software renderer using softbuffer for debugging
//! This is a simple CPU-based renderer that draws directly to a framebuffer

use crate::batch::RenderBatch;
use crate::vertex::Vertex;
use softbuffer::{Context, Surface};
use std::num::NonZeroU32;
use std::sync::Arc;
use winit::window::Window;

pub struct SoftwareRenderer {
    context: Context<Arc<Window>>,
    surface: Surface<Arc<Window>, Arc<Window>>,
    width: u32,
    height: u32,
}

impl SoftwareRenderer {
    /// Create a new software renderer
    pub fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let size = window.inner_size();
        let context = Context::new(window.clone())
            .map_err(|e| anyhow::anyhow!("Failed to create softbuffer context: {}", e))?;
        let mut surface = Surface::new(&context, window)
            .map_err(|e| anyhow::anyhow!("Failed to create softbuffer surface: {}", e))?;
        
        surface.resize(
            NonZeroU32::new(size.width).unwrap(),
            NonZeroU32::new(size.height).unwrap(),
        ).map_err(|e| anyhow::anyhow!("Failed to resize softbuffer: {}", e))?;
        
        println!("=== SOFTWARE RENDERER INITIALIZED ===");
        println!("Window size: {}x{}", size.width, size.height);
        println!("====================================");
        
        Ok(Self {
            context,
            surface,
            width: size.width,
            height: size.height,
        })
    }
    
    /// Resize the renderer
    pub fn resize(&mut self, width: u32, height: u32) -> anyhow::Result<()> {
        if width > 0 && height > 0 {
            self.width = width;
            self.height = height;
            self.surface.resize(
                NonZeroU32::new(width).unwrap(),
                NonZeroU32::new(height).unwrap(),
            ).map_err(|e| anyhow::anyhow!("Failed to resize softbuffer: {}", e))?;
            println!("Software renderer resized to: {}x{}", width, height);
        }
        Ok(())
    }
    
    /// Render a batch
    pub fn render(&mut self, batch: &RenderBatch) -> anyhow::Result<()> {
        // Copy these before mut borrowing buffer
        let width = self.width;
        let height = self.height;
        
        let mut buffer = self.surface.buffer_mut()
            .map_err(|e| anyhow::anyhow!("Failed to get softbuffer: {}", e))?;
        
        // Clear to dark gray background
        let clear_color = 0xFF_3A_3A_3A; // Dark gray (ARGB)
        for pixel in buffer.iter_mut() {
            *pixel = clear_color;
        }
        
        println!("=== SOFTWARE RENDER ===");
        println!("Rendering {} triangles", batch.triangle_count());
        
        // Draw each triangle from the batch
        for i in (0..batch.indices.len()).step_by(3) {
            let idx0 = batch.indices[i] as usize;
            let idx1 = batch.indices[i + 1] as usize;
            let idx2 = batch.indices[i + 2] as usize;
            
            if idx0 < batch.vertices.len() && idx1 < batch.vertices.len() && idx2 < batch.vertices.len() {
                let v0 = &batch.vertices[idx0];
                let v1 = &batch.vertices[idx1];
                let v2 = &batch.vertices[idx2];
                
                // Draw filled triangle
                Self::draw_triangle(&mut buffer, v0, v1, v2, width, height);
            }
        }
        
        println!("======================");
        
        buffer.present()
            .map_err(|e| anyhow::anyhow!("Failed to present softbuffer: {}", e))?;
        Ok(())
    }
    
    /// Draw a filled triangle (simple scanline algorithm)
    fn draw_triangle(buffer: &mut [u32], v0: &Vertex, v1: &Vertex, v2: &Vertex, width: u32, height: u32) {
        // Convert vertex color to ARGB u32
        let color = Self::color_to_argb(&v0.color);
        
        // Get triangle vertices positions
        let mut vertices = [
            (v0.position[0] as i32, v0.position[1] as i32),
            (v1.position[0] as i32, v1.position[1] as i32),
            (v2.position[0] as i32, v2.position[1] as i32),
        ];
        
        // Sort vertices by y coordinate
        vertices.sort_by_key(|v| v.1);
        let (x0, y0) = vertices[0];
        let (x1, y1) = vertices[1];
        let (x2, y2) = vertices[2];
        
        // Draw the triangle using scanlines
        Self::fill_triangle_scanline(buffer, x0, y0, x1, y1, x2, y2, color, width, height);
    }
    
    /// Fill triangle using scanline algorithm
    fn fill_triangle_scanline(buffer: &mut [u32], x0: i32, y0: i32, x1: i32, y1: i32, x2: i32, y2: i32, color: u32, width: u32, height: u32) {
        // Draw each horizontal scanline
        for y in y0.max(0)..=y2.min(height as i32 - 1) {
            // Find intersections with triangle edges
            let mut x_intersections = Vec::new();
            
            // Check intersection with edge 0-1
            if y >= y0 && y <= y1 && y1 != y0 {
                let t = (y - y0) as f32 / (y1 - y0) as f32;
                let x = x0 as f32 + t * (x1 - x0) as f32;
                x_intersections.push(x as i32);
            }
            
            // Check intersection with edge 1-2
            if y >= y1 && y <= y2 && y2 != y1 {
                let t = (y - y1) as f32 / (y2 - y1) as f32;
                let x = x1 as f32 + t * (x2 - x1) as f32;
                x_intersections.push(x as i32);
            }
            
            // Check intersection with edge 0-2
            if y >= y0 && y <= y2 && y2 != y0 {
                let t = (y - y0) as f32 / (y2 - y0) as f32;
                let x = x0 as f32 + t * (x2 - x0) as f32;
                x_intersections.push(x as i32);
            }
            
            // Draw horizontal line between intersections
            if x_intersections.len() >= 2 {
                x_intersections.sort();
                let x_start = x_intersections[0].max(0);
                let x_end = x_intersections[x_intersections.len() - 1].min(width as i32 - 1);
                
                for x in x_start..=x_end {
                    let index = (y as u32 * width + x as u32) as usize;
                    if index < buffer.len() {
                        buffer[index] = color;
                    }
                }
            }
        }
    }
    
    /// Convert RGBA float color to ARGB u32
    fn color_to_argb(color: &[f32; 4]) -> u32 {
        let r = (color[0] * 255.0).min(255.0).max(0.0) as u32;
        let g = (color[1] * 255.0).min(255.0).max(0.0) as u32;
        let b = (color[2] * 255.0).min(255.0).max(0.0) as u32;
        let a = (color[3] * 255.0).min(255.0).max(0.0) as u32;
        
        (a << 24) | (r << 16) | (g << 8) | b
    }
}
