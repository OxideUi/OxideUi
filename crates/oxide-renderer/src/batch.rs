//! Render batching system for efficient GPU rendering

use std::collections::HashMap;
use oxide_core::types::{Color, Rect, Transform};
use oxide_core::oxide_text_debug;
use crate::vertex::Vertex;
use crate::text::TextRenderer;

use oxide_core::text::TextAlign;

/// Draw command types
#[derive(Debug, Clone)]
pub enum DrawCommand {
    /// Draw a filled rectangle
    Rect {
        rect: Rect,
        color: Color,
        transform: Transform,
    },
    /// Draw a rectangle with rounded corners
    RoundedRect {
        rect: Rect,
        color: Color,
        radius: f32,
        transform: Transform,
    },
    /// Draw text
    Text {
        text: String,
        position: (f32, f32),
        color: Color,
        font_size: f32,
        letter_spacing: f32,
        align: TextAlign,
    },
    /// Draw a textured quad
    TexturedQuad {
        rect: Rect,
        texture_id: u32,
        uv_rect: Rect,
        color: Color,
        transform: Transform,
    },
    /// Draw a circle
    Circle {
        center: (f32, f32),
        radius: f32,
        color: Color,
        segments: u32,
    },
    /// Draw a line
    Line {
        start: (f32, f32),
        end: (f32, f32),
        color: Color,
        thickness: f32,
    },
}

/// Render batch for collecting draw commands
pub struct RenderBatch {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
    pub commands: Vec<DrawCommand>,
    vertex_count: u16,
    texture_atlas: HashMap<u32, TextureInfo>,
    text_renderer: TextRenderer,
}

/// Texture information for batching
#[derive(Debug, Clone)]
#[allow(dead_code)] // Fields are used for texture management but not in simplified implementation
pub struct TextureInfo {
    pub width: u32,
    pub height: u32,
    pub format: wgpu::TextureFormat,
}

impl RenderBatch {
    /// Create a new render batch
    pub fn new() -> Self {
        Self {
            vertices: Vec::with_capacity(1024),
            indices: Vec::with_capacity(1536),
            commands: Vec::new(),
            vertex_count: 0,
            texture_atlas: HashMap::new(),
            text_renderer: TextRenderer::new(),
        }
    }

    /// Clear the batch
    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
        self.commands.clear();
        self.vertex_count = 0;
    }

    /// Get the number of draw commands in the batch
    pub fn command_count(&self) -> usize {
        self.commands.len()
    }

    /// Add a rectangle to the batch
    pub fn add_rect(&mut self, rect: Rect, color: Color, transform: Transform) {
        let command = DrawCommand::Rect { rect, color, transform };
        self.commands.push(command);
        // Self::batch_rect is not used by drawing.rs, so we don't need to call it if we use commands
        // But for compatibility with other renderers (if any), we might keep it.
        // However, drawing.rs ignores batch.vertices!
        // So calling batch_rect here is wasteful if drawing.rs is the only consumer and it ignores vertices.
        // But I will enable batch.vertices in drawing.rs soon.
        // For now, just push command.
    }

    /// Add a rounded rectangle to the batch
    pub fn add_rounded_rect(&mut self, rect: Rect, color: Color, radius: f32, transform: Transform) {
        let command = DrawCommand::RoundedRect { rect, color, radius, transform };
        self.commands.push(command);
    }

    /// Add text to the batch
    pub fn add_text(&mut self, text: String, position: (f32, f32), color: Color, font_size: f32, letter_spacing: f32) {
        self.add_text_aligned(text, position, color, font_size, letter_spacing, TextAlign::Left);
    }

    /// Add aligned text to the batch
    pub fn add_text_aligned(&mut self, text: String, position: (f32, f32), color: Color, font_size: f32, letter_spacing: f32, align: TextAlign) {
        let command = DrawCommand::Text {
            text: text.clone(),
            position,
            color,
            font_size,
            letter_spacing,
            align,
        };
        self.commands.push(command);
    }


    /// Add a textured quad to the batch
    pub fn add_textured_quad(
        &mut self,
        rect: Rect,
        texture_id: u32,
        uv_rect: Rect,
        color: Color,
        transform: Transform,
    ) {
        let command = DrawCommand::TexturedQuad {
            rect,
            texture_id,
            uv_rect,
            color,
            transform,
        };
        self.commands.push(command);
        self.batch_textured_quad(rect, uv_rect, color, transform);
    }

    /// Add a circle to the batch
    pub fn add_circle(&mut self, center: (f32, f32), radius: f32, color: Color, segments: u32) {
        let command = DrawCommand::Circle { center, radius, color, segments };
        self.commands.push(command);
        self.batch_circle(center, radius, color, segments);
    }

    /// Add a line to the batch
    pub fn add_line(&mut self, start: (f32, f32), end: (f32, f32), color: Color, thickness: f32) {
        let command = DrawCommand::Line { start, end, color, thickness };
        self.commands.push(command);
        self.batch_line(start, end, color, thickness);
    }

    /// Add raw vertices and indices to the batch
    pub fn add_vertices(&mut self, vertices: &[Vertex], indices: &[u16]) {
        let vertex_offset = self.vertices.len() as u16;
        
        // Add vertices
        self.vertices.extend_from_slice(vertices);
        
        // Add indices with offset
        for &index in indices {
            self.indices.push(vertex_offset + index);
        }
        
        self.vertex_count += vertices.len() as u16;
    }
    
    /// Batch text with real GPU glyph rendering (requires TextureManager access)
    /// 
    /// This is the full implementation that renders actual glyphs from the font atlas
    #[allow(dead_code)]
    fn batch_text_gpu(
        &mut self,
        texture_mgr: &mut crate::gpu::TextureManager,
        queue: &wgpu::Queue,
        text: &str,
        position: (f32, f32),
        color: Color,
        font_size: u32,
    ) {
        let (mut x, y) = position;
        let color_arr = [color.r, color.g, color.b, color.a];
        
        for ch in text.chars() {
            if let Some(glyph) = texture_mgr.get_or_cache_glyph(queue, ch, font_size) {
                // Calculate glyph position with bearing
                let glyph_x = x + glyph.metrics.bearing_x as f32;
                let glyph_y = y - glyph.metrics.bearing_y as f32;
                
                // Glyph dimensions
                let w = glyph.metrics.width as f32;
                let h = glyph.metrics.height as f32;
                
                // UV coordinates from atlas
                let (u0, v0, u1, v1) = glyph.uv_rect;
                
                // Create textured quad for this glyph
                let base_idx = self.vertex_count;
                
                // Add 4 vertices for the quad
                self.vertices.push(Vertex::textured([glyph_x, glyph_y], [u0, v0], color_arr));
                self.vertices.push(Vertex::textured([glyph_x + w, glyph_y], [u1, v0], color_arr));
                self.vertices.push(Vertex::textured([glyph_x + w, glyph_y + h], [u1, v1], color_arr));
                self.vertices.push(Vertex::textured([glyph_x, glyph_y + h], [u0, v1], color_arr));
                
                // Add 2 triangles (6 indices)
                self.indices.push(base_idx);
                self.indices.push(base_idx + 1);
                self.indices.push(base_idx + 2);
                
                self.indices.push(base_idx);
                self.indices.push(base_idx + 2);
                self.indices.push(base_idx + 3);
                
                self.vertex_count += 4;
                
                // Advance to next character position
                x += glyph.metrics.advance;
            } else {
                // Fallback: advance by half font size if glyph unavailable
                x += font_size as f32 * 0.5;
            }
        }
    }

    /// Batch a rectangle into vertices and indices
    fn batch_rect(&mut self, rect: Rect, color: Color, transform: Transform) {
        // Store command for deferred rendering (if needed)
        self.commands.push(DrawCommand::Rect {
            rect,
            color,
            transform,
        });

        let (x, y, w, h) = (rect.x, rect.y, rect.width, rect.height);
        
        // Apply transform to vertices
        let positions = [
            self.apply_transform([x, y], transform),
            self.apply_transform([x + w, y], transform),
            self.apply_transform([x + w, y + h], transform),
            self.apply_transform([x, y + h], transform),
        ];

        // Create vertices with all UV coords at (0,0) for solid color rendering
        let vertices = [
            Vertex {
                position: positions[0],
                uv: [0.0, 0.0], // Solid color - no texture
                color: [color.r, color.g, color.b, color.a],
                params: [0.0, 0.0, 0.0, 0.0],
                flags: 0,
            },
            Vertex {
                position: positions[1],
                uv: [0.0, 0.0], // Solid color - no texture
                color: [color.r, color.g, color.b, color.a],
                params: [0.0, 0.0, 0.0, 0.0],
                flags: 0,
            },
            Vertex {
                position: positions[2],
                uv: [0.0, 0.0], // Solid color - no texture
                color: [color.r, color.g, color.b, color.a],
                params: [0.0, 0.0, 0.0, 0.0],
                flags: 0,
            },
            Vertex {
                position: positions[3],
                uv: [0.0, 0.0], // Solid color - no texture
                color: [color.r, color.g, color.b, color.a],
                params: [0.0, 0.0, 0.0, 0.0],
                flags: 0,
            },
        ];

        // Add vertices
        self.vertices.extend_from_slice(&vertices);

        // Add indices for two triangles
        let base = self.vertex_count;
        self.indices.extend_from_slice(&[
            base, base + 1, base + 2,
            base, base + 2, base + 3,
        ]);

        self.vertex_count += 4;
    }

    /// Batch a textured quad
    fn batch_textured_quad(&mut self, rect: Rect, uv_rect: Rect, color: Color, transform: Transform) {
        let (x, y, w, h) = (rect.x, rect.y, rect.width, rect.height);
        let (u, v, uw, vh) = (uv_rect.x, uv_rect.y, uv_rect.width, uv_rect.height);
        
        // Apply transform to vertices
        let positions = [
            self.apply_transform([x, y], transform),
            self.apply_transform([x + w, y], transform),
            self.apply_transform([x + w, y + h], transform),
            self.apply_transform([x, y + h], transform),
        ];

        // Create vertices with UV coordinates
        let vertices = [
            Vertex {
                position: positions[0],
                uv: [u, v],
                color: [color.r, color.g, color.b, color.a],
                params: [0.0, 0.0, 0.0, 0.0],
                flags: 0,
            },
            Vertex {
                position: positions[1],
                uv: [u + uw, v],
                color: [color.r, color.g, color.b, color.a],
                params: [0.0, 0.0, 0.0, 0.0],
                flags: 0,
            },
            Vertex {
                position: positions[2],
                uv: [u + uw, v + vh],
                color: [color.r, color.g, color.b, color.a],
                params: [0.0, 0.0, 0.0, 0.0],
                flags: 0,
            },
            Vertex {
                position: positions[3],
                uv: [u, v + vh],
                color: [color.r, color.g, color.b, color.a],
                params: [0.0, 0.0, 0.0, 0.0],
                flags: 0,
            },
        ];

        self.vertices.extend_from_slice(&vertices);

        let base = self.vertex_count;
        self.indices.extend_from_slice(&[
            base, base + 1, base + 2,
            base, base + 2, base + 3,
        ]);

        self.vertex_count += 4;
    }

    /// Batch a circle
    fn batch_circle(&mut self, center: (f32, f32), radius: f32, color: Color, segments: u32) {
        let (cx, cy) = center;
        
        // Center vertex
        self.vertices.push(Vertex {
            position: [cx, cy],
            uv: [0.5, 0.5],
            color: [color.r, color.g, color.b, color.a],
            params: [0.0, 0.0, 0.0, 0.0],
            flags: 0,
        });

        let center_index = self.vertex_count;
        self.vertex_count += 1;

        // Generate circle vertices
        for i in 0..=segments {
            let angle = (i as f32 / segments as f32) * 2.0 * std::f32::consts::PI;
            let x = cx + radius * angle.cos();
            let y = cy + radius * angle.sin();
            
            self.vertices.push(Vertex {
                position: [x, y],
                uv: [0.5 + 0.5 * angle.cos(), 0.5 + 0.5 * angle.sin()],
                color: [color.r, color.g, color.b, color.a],
                params: [0.0, 0.0, 0.0, 0.0],
                flags: 0,
            });

            if i > 0 {
                self.indices.extend_from_slice(&[
                    center_index,
                    self.vertex_count - 1,
                    self.vertex_count,
                ]);
            }

            self.vertex_count += 1;
        }
    }

    /// Batch a line as a rectangle
    fn batch_line(&mut self, start: (f32, f32), end: (f32, f32), color: Color, thickness: f32) {
        let (x1, y1) = start;
        let (x2, y2) = end;
        
        // Calculate line direction and perpendicular
        let dx = x2 - x1;
        let dy = y2 - y1;
        let length = (dx * dx + dy * dy).sqrt();
        
        if length == 0.0 {
            return;
        }
        
        let nx = -dy / length * thickness * 0.5;
        let ny = dx / length * thickness * 0.5;

        // Create line vertices
        let vertices = [
            Vertex {
                position: [x1 + nx, y1 + ny],
                uv: [0.0, 0.0],
                color: [color.r, color.g, color.b, color.a],
                params: [0.0, 0.0, 0.0, 0.0],
                flags: 0,
            },
            Vertex {
                position: [x2 + nx, y2 + ny],
                uv: [1.0, 0.0],
                color: [color.r, color.g, color.b, color.a],
                params: [0.0, 0.0, 0.0, 0.0],
                flags: 0,
            },
            Vertex {
                position: [x2 - nx, y2 - ny],
                uv: [1.0, 1.0],
                color: [color.r, color.g, color.b, color.a],
                params: [0.0, 0.0, 0.0, 0.0],
                flags: 0,
            },
            Vertex {
                position: [x1 - nx, y1 - ny],
                uv: [0.0, 1.0],
                color: [color.r, color.g, color.b, color.a],
                params: [0.0, 0.0, 0.0, 0.0],
                flags: 0,
            },
        ];

        self.vertices.extend_from_slice(&vertices);

        let base = self.vertex_count;
        self.indices.extend_from_slice(&[
            base, base + 1, base + 2,
            base, base + 2, base + 3,
        ]);

        self.vertex_count += 4;
    }

    /// Apply transform to a position
    fn apply_transform(&self, pos: [f32; 2], transform: Transform) -> [f32; 2] {
        // Transform uses a matrix internally, so we need to transform the point
        let point = oxide_core::types::Point::new(pos[0], pos[1]);
        let transformed = transform.transform_point(point);
        [transformed.x, transformed.y]
    }

    /// Get the number of draw calls
    pub fn draw_call_count(&self) -> usize {
        self.commands.len()
    }

    /// Get the number of vertices
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    /// Get the number of triangles
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    /// Register a texture in the atlas
    pub fn register_texture(&mut self, id: u32, width: u32, height: u32, format: wgpu::TextureFormat) {
        self.texture_atlas.insert(id, TextureInfo { width, height, format });
    }

    /// Get texture info
    pub fn get_texture_info(&self, id: u32) -> Option<&TextureInfo> {
        self.texture_atlas.get(&id)
    }
}

impl Default for RenderBatch {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxide_core::types::Color;
    use glam::Vec2;

    #[test]
    fn test_batch_rect() {
        let mut batch = RenderBatch::new();
        let rect = Rect::new(10.0, 20.0, 100.0, 50.0);
        let color = Color::rgba(1.0, 0.0, 0.0, 1.0);
        let transform = Transform::default();

        batch.add_rect(rect, color, transform);

        assert_eq!(batch.vertex_count(), 4);
        assert_eq!(batch.triangle_count(), 2);
        assert_eq!(batch.draw_call_count(), 1);
    }

    #[test]
    fn test_batch_circle() {
        let mut batch = RenderBatch::new();
        let center = (50.0, 50.0);
        let radius = 25.0;
        let color = Color::rgba(0.0, 1.0, 0.0, 1.0);
        let segments = 16;

        batch.add_circle(center, radius, color, segments);

        assert_eq!(batch.vertex_count(), segments as usize + 2); // center + perimeter + closing
        assert_eq!(batch.draw_call_count(), 1);
    }

    #[test]
    fn test_clear_batch() {
        let mut batch = RenderBatch::new();
        let rect = Rect::new(0.0, 0.0, 10.0, 10.0);
        let color = Color::WHITE;
        let transform = Transform::default();

        batch.add_rect(rect, color, transform);
        assert!(!batch.vertices.is_empty());

        batch.clear();
        assert!(batch.vertices.is_empty());
        assert!(batch.indices.is_empty());
        assert_eq!(batch.draw_call_count(), 0);
    }
}
