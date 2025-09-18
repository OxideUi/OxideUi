//! Render batching system for efficient GPU rendering

use std::collections::HashMap;
use oxide_core::types::{Color, Rect, Transform, Point};
use crate::vertex::Vertex;
use crate::text::{TextRenderer, Font};
use crate::simple_text::{SimpleTextRenderer, SimpleFont, ColorExt};

/// Draw command types
#[derive(Debug, Clone)]
pub enum DrawCommand {
    /// Draw a filled rectangle
    Rect {
        rect: Rect,
        color: Color,
        transform: Transform,
    },
    /// Draw text
    Text {
        text: String,
        position: (f32, f32),
        color: Color,
        font_size: f32,
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
    commands: Vec<DrawCommand>,
    vertex_count: u16,
    texture_atlas: HashMap<u32, TextureInfo>,
    text_renderer: TextRenderer,
    simple_text_renderer: SimpleTextRenderer,
}

/// Texture information for batching
#[derive(Debug, Clone)]
#[allow(dead_code)] // Fields are used for texture management but not in simplified implementation
struct TextureInfo {
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
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
            simple_text_renderer: SimpleTextRenderer::new(),
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
        self.batch_rect(rect, color, transform);
    }

    /// Add text to the batch
    pub fn add_text(&mut self, text: String, position: (f32, f32), color: Color, font_size: f32) {
        // DEBUG: Always print when add_text is called
        println!("DEBUG: add_text called with text='{}' position=({:.1},{:.1}) color=({:.2},{:.2},{:.2},{:.2}) size={:.1}", 
                text, position.0, position.1, color.r, color.g, color.b, color.a, font_size);
        
        let command = DrawCommand::Text { text: text.clone(), position, color, font_size };
        self.commands.push(command);
        
        // Use the new simple text rendering system
        self.batch_text_simple(&text, position, color, font_size);
    }
    
    /// Add text using the simple text rendering system
    pub fn batch_text_simple(&mut self, text: &str, position: (f32, f32), color: Color, font_size: f32) {
        println!("DEBUG: batch_text_simple called with text='{}' at ({:.1}, {:.1}) size {:.1}", 
                text, position.0, position.1, font_size);
        
        // Create simple font
        let font = SimpleFont::system(font_size);
        
        // Get font info for debugging
        let font_info = self.simple_text_renderer.get_font_info();
        println!("DEBUG: Using font system: {}", font_info);
        
        // Render text vertices
        let vertices = self.simple_text_renderer.render_text_simple(
            text,
            &font,
            Point::new(position.0, position.1),
            color,
        );
        
        println!("DEBUG: Generated {} vertices for text '{}'", vertices.len(), text);
        
        // Convert vertices to indices and add them
        let mut indices = Vec::new();
        let base_vertex = self.vertices.len() as u16;
        
        // Each character has 4 vertices, make 2 triangles (6 indices) per character
        for i in (0..vertices.len()).step_by(4) {
            let base = base_vertex + i as u16;
            indices.extend_from_slice(&[
                base, base + 1, base + 2,  // First triangle
                base, base + 2, base + 3,  // Second triangle
            ]);
        }
        
        // Add vertices and indices
        self.vertices.extend(vertices);
        self.indices.extend(indices);
        
        self.vertex_count += self.vertices.len() as u16;
        
        println!("DEBUG: Text batch complete: {} total vertices, {} total indices", 
                self.vertices.len(), self.indices.len());
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

    /// Batch text into vertices and indices (temporary simple implementation for debugging)
    fn batch_text(&mut self, text: &str, position: (f32, f32), color: Color, font_size: f32) {
        use oxide_core::{oxide_trace, logging::LogCategory};
        
        // DEBUG: Always print to console
        println!("DEBUG: batch_text called with text='{}' at ({:.1}, {:.1}) size {:.1} color ({:.2}, {:.2}, {:.2}, {:.2})", 
                text, position.0, position.1, font_size, color.r, color.g, color.b, color.a);
        
        oxide_trace!(LogCategory::Text, "Batching text '{}' at ({:.1}, {:.1}) size {:.1} color ({:.2}, {:.2}, {:.2}, {:.2})", 
                    text, position.0, position.1, font_size, color.r, color.g, color.b, color.a);
        
        // TEMPORARY: Simple rectangle rendering for each character (for debugging)
        let char_width = font_size * 0.6; // Approximate character width
        let char_height = font_size;
        
        println!("DEBUG: Creating {} rectangles for text '{}'", text.chars().count(), text);
        
        for (i, _char) in text.chars().enumerate() {
            let x = position.0 + (i as f32 * char_width);
            let y = position.1;
            
            println!("DEBUG: Character {} at ({:.1}, {:.1}) size ({:.1} x {:.1})", i, x, y, char_width, char_height);
            
            // Create a solid rectangle for each character
            let char_rect = Rect::new(x, y, char_width, char_height);
            let transform = Transform::default();
            
            // Use the provided color for the rectangle
            self.batch_rect(char_rect, color, transform);
        }
        
        println!("DEBUG: Text batch complete: {} vertices, {} indices", self.vertices.len(), self.indices.len());
        oxide_trace!(LogCategory::Text, "Text batch complete: {} vertices, {} indices", 
                    self.vertices.len(), self.indices.len());
    }

    /// Batch a rectangle into vertices and indices
    fn batch_rect(&mut self, rect: Rect, color: Color, transform: Transform) {
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
                tex_coords: [0.0, 0.0], // Solid color - no texture
                color: [color.r, color.g, color.b, color.a],
                flags: 0,
            },
            Vertex {
                position: positions[1],
                tex_coords: [0.0, 0.0], // Solid color - no texture
                color: [color.r, color.g, color.b, color.a],
                flags: 0,
            },
            Vertex {
                position: positions[2],
                tex_coords: [0.0, 0.0], // Solid color - no texture
                color: [color.r, color.g, color.b, color.a],
                flags: 0,
            },
            Vertex {
                position: positions[3],
                tex_coords: [0.0, 0.0], // Solid color - no texture
                color: [color.r, color.g, color.b, color.a],
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
                tex_coords: [u, v],
                color: [color.r, color.g, color.b, color.a],
                flags: 0,
            },
            Vertex {
                position: positions[1],
                tex_coords: [u + uw, v],
                color: [color.r, color.g, color.b, color.a],
                flags: 0,
            },
            Vertex {
                position: positions[2],
                tex_coords: [u + uw, v + vh],
                color: [color.r, color.g, color.b, color.a],
                flags: 0,
            },
            Vertex {
                position: positions[3],
                tex_coords: [u, v + vh],
                color: [color.r, color.g, color.b, color.a],
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
            position: [cx, cy, 0.0],
            tex_coords: [0.5, 0.5],
            color: [color.r, color.g, color.b, color.a],
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
                position: [x, y, 0.0],
                tex_coords: [0.5 + 0.5 * angle.cos(), 0.5 + 0.5 * angle.sin()],
                color: [color.r, color.g, color.b, color.a],
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
                position: [x1 + nx, y1 + ny, 0.0],
                tex_coords: [0.0, 0.0],
                color: [color.r, color.g, color.b, color.a],
                flags: 0,
            },
            Vertex {
                position: [x2 + nx, y2 + ny, 0.0],
                tex_coords: [1.0, 0.0],
                color: [color.r, color.g, color.b, color.a],
                flags: 0,
            },
            Vertex {
                position: [x2 - nx, y2 - ny, 0.0],
                tex_coords: [1.0, 1.0],
                color: [color.r, color.g, color.b, color.a],
                flags: 0,
            },
            Vertex {
                position: [x1 - nx, y1 - ny, 0.0],
                tex_coords: [0.0, 1.0],
                color: [color.r, color.g, color.b, color.a],
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
    fn apply_transform(&self, pos: [f32; 2], transform: Transform) -> [f32; 3] {
        // Transform uses a matrix internally, so we need to transform the point
        let point = oxide_core::types::Point::new(pos[0], pos[1]);
        let transformed = transform.transform_point(point);
        [transformed.x, transformed.y, 0.0]
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
    use oxide_core::types::{Vec2, Color};

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
