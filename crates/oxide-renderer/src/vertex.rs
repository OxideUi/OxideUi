//! Vertex data structures and layouts for wgpu rendering

use wgpu::{VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode, BufferAddress};

/// Vertex data for UI rendering
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
    pub tex_coords: [f32; 2],
    pub flags: u32, // For different rendering modes (solid, textured, etc.)
}

impl Vertex {
    /// Create a new vertex
    pub fn new(position: [f32; 3], color: [f32; 4], tex_coords: [f32; 2]) -> Self {
        Self {
            position,
            color,
            tex_coords,
            flags: 0,
        }
    }

    /// Create a vertex with solid color (no texture)
    pub fn solid(position: [f32; 3], color: [f32; 4]) -> Self {
        Self {
            position,
            color,
            tex_coords: [0.0, 0.0],
            flags: 1, // Solid color flag
        }
    }

    /// Create a vertex with texture coordinates
    pub fn textured(position: [f32; 3], tex_coords: [f32; 2], color: [f32; 4]) -> Self {
        Self {
            position,
            color,
            tex_coords,
            flags: 2, // Textured flag
        }
    }

    /// Get the vertex buffer layout descriptor
    pub fn desc() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &[
                // Position
                VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: VertexFormat::Float32x3,
                },
                // Color
                VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as BufferAddress,
                    shader_location: 1,
                    format: VertexFormat::Float32x4,
                },
                // Texture coordinates
                VertexAttribute {
                    offset: std::mem::size_of::<[f32; 7]>() as BufferAddress,
                    shader_location: 2,
                    format: VertexFormat::Float32x2,
                },
                // Flags
                VertexAttribute {
                    offset: std::mem::size_of::<[f32; 9]>() as BufferAddress,
                    shader_location: 3,
                    format: VertexFormat::Uint32,
                },
            ],
        }
    }
}

/// Text vertex for specialized text rendering
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TextVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub color: [f32; 4],
    pub glyph_index: u32,
}

impl TextVertex {
    /// Create a new text vertex
    pub fn new(position: [f32; 3], tex_coords: [f32; 2], color: [f32; 4], glyph_index: u32) -> Self {
        Self {
            position,
            tex_coords,
            color,
            glyph_index,
        }
    }

    /// Get the vertex buffer layout descriptor for text
    pub fn desc() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<TextVertex>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &[
                // Position
                VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: VertexFormat::Float32x3,
                },
                // Texture coordinates
                VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as BufferAddress,
                    shader_location: 1,
                    format: VertexFormat::Float32x2,
                },
                // Color
                VertexAttribute {
                    offset: std::mem::size_of::<[f32; 5]>() as BufferAddress,
                    shader_location: 2,
                    format: VertexFormat::Float32x4,
                },
                // Glyph index
                VertexAttribute {
                    offset: std::mem::size_of::<[f32; 9]>() as BufferAddress,
                    shader_location: 3,
                    format: VertexFormat::Uint32,
                },
            ],
        }
    }
}

/// Vertex builder for creating common shapes
pub struct VertexBuilder;

impl VertexBuilder {
    /// Create vertices for a rectangle
    pub fn rectangle(
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: [f32; 4],
    ) -> (Vec<Vertex>, Vec<u16>) {
        let vertices = vec![
            Vertex::solid([x, y, 0.0], color),                           // Top-left
            Vertex::solid([x + width, y, 0.0], color),                   // Top-right
            Vertex::solid([x + width, y + height, 0.0], color),          // Bottom-right
            Vertex::solid([x, y + height, 0.0], color),                  // Bottom-left
        ];

        let indices = vec![0, 1, 2, 2, 3, 0];

        (vertices, indices)
    }

    /// Create vertices for a textured rectangle
    pub fn textured_rectangle(
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: [f32; 4],
    ) -> (Vec<Vertex>, Vec<u16>) {
        let vertices = vec![
            Vertex::textured([x, y, 0.0], [0.0, 0.0], color),                           // Top-left
            Vertex::textured([x + width, y, 0.0], [1.0, 0.0], color),                   // Top-right
            Vertex::textured([x + width, y + height, 0.0], [1.0, 1.0], color),          // Bottom-right
            Vertex::textured([x, y + height, 0.0], [0.0, 1.0], color),                  // Bottom-left
        ];

        let indices = vec![0, 1, 2, 2, 3, 0];

        (vertices, indices)
    }

    /// Create vertices for a circle (approximated with triangles)
    pub fn circle(
        center_x: f32,
        center_y: f32,
        radius: f32,
        color: [f32; 4],
        segments: u32,
    ) -> (Vec<Vertex>, Vec<u16>) {
        let mut vertices = Vec::with_capacity((segments + 1) as usize);
        let mut indices = Vec::with_capacity((segments * 3) as usize);

        // Center vertex
        vertices.push(Vertex::solid([center_x, center_y, 0.0], color));

        // Create vertices around the circle
        for i in 0..segments {
            let angle = (i as f32) * 2.0 * std::f32::consts::PI / (segments as f32);
            let x = center_x + radius * angle.cos();
            let y = center_y + radius * angle.sin();
            vertices.push(Vertex::solid([x, y, 0.0], color));
        }

        // Create triangles
        for i in 0..segments {
            let next = if i == segments - 1 { 1 } else { i + 2 };
            indices.extend_from_slice(&[0, (i + 1) as u16, next as u16]);
        }

        (vertices, indices)
    }

    /// Create vertices for a rounded rectangle
    pub fn rounded_rectangle(
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        radius: f32,
        color: [f32; 4],
        corner_segments: u32,
    ) -> (Vec<Vertex>, Vec<u16>) {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        // Create the main rectangle (without corners)
        let inner_x = x + radius;
        let inner_y = y + radius;
        let inner_width = width - 2.0 * radius;
        let inner_height = height - 2.0 * radius;

        // Center rectangle
        if inner_width > 0.0 && inner_height > 0.0 {
            let (rect_verts, rect_indices) = Self::rectangle(inner_x, inner_y, inner_width, inner_height, color);
            let offset = vertices.len() as u16;
            vertices.extend(rect_verts);
            indices.extend(rect_indices.iter().map(|&i| i + offset));
        }

        // Add rounded corners
        let corners = [
            (inner_x, inner_y),                           // Top-left
            (inner_x + inner_width, inner_y),             // Top-right
            (inner_x + inner_width, inner_y + inner_height), // Bottom-right
            (inner_x, inner_y + inner_height),            // Bottom-left
        ];

        for (i, &(cx, cy)) in corners.iter().enumerate() {
            let start_angle = (i as f32) * std::f32::consts::PI / 2.0;
            let (corner_verts, corner_indices) = Self::circle_sector(
                cx, cy, radius, color, corner_segments, start_angle, std::f32::consts::PI / 2.0
            );
            let offset = vertices.len() as u16;
            vertices.extend(corner_verts);
            indices.extend(corner_indices.iter().map(|&i| i + offset));
        }

        (vertices, indices)
    }

    /// Create vertices for a rounded rectangle outline (border)
    pub fn rounded_rectangle_outline(
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        radius: f32,
        color: [f32; 4],
        thickness: f32,
        corner_segments: u32,
    ) -> (Vec<Vertex>, Vec<u16>) {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        // Create the four border lines
        let half_thickness = thickness / 2.0;
        
        // Top line
        let (top_verts, top_indices) = Self::line(
            x + radius, y - half_thickness,
            x + width - radius, y - half_thickness,
            thickness, color
        );
        vertices.extend(top_verts);
        indices.extend(top_indices);

        // Right line
        let offset = vertices.len() as u16;
        let (right_verts, right_indices) = Self::line(
            x + width + half_thickness, y + radius,
            x + width + half_thickness, y + height - radius,
            thickness, color
        );
        vertices.extend(right_verts);
        indices.extend(right_indices.iter().map(|&i| i + offset));

        // Bottom line
        let offset = vertices.len() as u16;
        let (bottom_verts, bottom_indices) = Self::line(
            x + width - radius, y + height + half_thickness,
            x + radius, y + height + half_thickness,
            thickness, color
        );
        vertices.extend(bottom_verts);
        indices.extend(bottom_indices.iter().map(|&i| i + offset));

        // Left line
        let offset = vertices.len() as u16;
        let (left_verts, left_indices) = Self::line(
            x - half_thickness, y + height - radius,
            x - half_thickness, y + radius,
            thickness, color
        );
        vertices.extend(left_verts);
        indices.extend(left_indices.iter().map(|&i| i + offset));

        // Add rounded corners (outline arcs)
        let corners = [
            (x + radius, y + radius),                           // Top-left
            (x + width - radius, y + radius),                   // Top-right
            (x + width - radius, y + height - radius),          // Bottom-right
            (x + radius, y + height - radius),                  // Bottom-left
        ];

        for (i, &(cx, cy)) in corners.iter().enumerate() {
            let start_angle = (i as f32) * std::f32::consts::PI / 2.0 + std::f32::consts::PI;
            
            // Create arc outline using multiple line segments
            for j in 0..corner_segments {
                let angle1 = start_angle + (j as f32) * (std::f32::consts::PI / 2.0) / (corner_segments as f32);
                let angle2 = start_angle + ((j + 1) as f32) * (std::f32::consts::PI / 2.0) / (corner_segments as f32);
                
                let x1 = cx + radius * angle1.cos();
                let y1 = cy + radius * angle1.sin();
                let x2 = cx + radius * angle2.cos();
                let y2 = cy + radius * angle2.sin();
                
                let offset = vertices.len() as u16;
                let (arc_verts, arc_indices) = Self::line(x1, y1, x2, y2, thickness, color);
                vertices.extend(arc_verts);
                indices.extend(arc_indices.iter().map(|&i| i + offset));
            }
        }

        (vertices, indices)
    }

    /// Create vertices for a circle sector
    fn circle_sector(
        center_x: f32,
        center_y: f32,
        radius: f32,
        color: [f32; 4],
        segments: u32,
        start_angle: f32,
        angle_span: f32,
    ) -> (Vec<Vertex>, Vec<u16>) {
        let mut vertices = Vec::with_capacity((segments + 1) as usize);
        let mut indices = Vec::with_capacity((segments * 3) as usize);

        // Center vertex
        vertices.push(Vertex::solid([center_x, center_y, 0.0], color));

        // Create vertices around the sector
        for i in 0..=segments {
            let angle = start_angle + (i as f32) * angle_span / (segments as f32);
            let x = center_x + radius * angle.cos();
            let y = center_y + radius * angle.sin();
            vertices.push(Vertex::solid([x, y, 0.0], color));
        }

        // Create triangles
        for i in 0..segments {
            indices.extend_from_slice(&[0, (i + 1) as u16, (i + 2) as u16]);
        }

        (vertices, indices)
    }

    /// Create vertices for a line with thickness
    pub fn line(
        start_x: f32,
        start_y: f32,
        end_x: f32,
        end_y: f32,
        thickness: f32,
        color: [f32; 4],
    ) -> (Vec<Vertex>, Vec<u16>) {
        let dx = end_x - start_x;
        let dy = end_y - start_y;
        let length = (dx * dx + dy * dy).sqrt();
        
        if length == 0.0 {
            return (Vec::new(), Vec::new());
        }

        // Normalize and get perpendicular vector
        let nx = -dy / length;
        let ny = dx / length;
        
        let half_thickness = thickness * 0.5;
        
        let vertices = vec![
            Vertex::solid([start_x + nx * half_thickness, start_y + ny * half_thickness, 0.0], color),
            Vertex::solid([start_x - nx * half_thickness, start_y - ny * half_thickness, 0.0], color),
            Vertex::solid([end_x - nx * half_thickness, end_y - ny * half_thickness, 0.0], color),
            Vertex::solid([end_x + nx * half_thickness, end_y + ny * half_thickness, 0.0], color),
        ];

        let indices = vec![0, 1, 2, 2, 3, 0];

        (vertices, indices)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vertex_creation() {
        let vertex = Vertex::new([1.0, 2.0, 3.0], [1.0, 0.0, 0.0, 1.0], [0.5, 0.5]);
        assert_eq!(vertex.position, [1.0, 2.0, 3.0]);
        assert_eq!(vertex.color, [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(vertex.tex_coords, [0.5, 0.5]);
    }

    #[test]
    fn test_vertex_solid() {
        let vertex = Vertex::solid([0.0, 0.0, 0.0], [1.0, 1.0, 1.0, 1.0]);
        assert_eq!(vertex.flags, 1);
        assert_eq!(vertex.tex_coords, [0.0, 0.0]);
    }

    #[test]
    fn test_vertex_textured() {
        let vertex = Vertex::textured([0.0, 0.0, 0.0], [1.0, 1.0], [1.0, 1.0, 1.0, 1.0]);
        assert_eq!(vertex.flags, 2);
        assert_eq!(vertex.tex_coords, [1.0, 1.0]);
    }

    #[test]
    fn test_rectangle_builder() {
        let (vertices, indices) = VertexBuilder::rectangle(0.0, 0.0, 100.0, 50.0, [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(vertices.len(), 4);
        assert_eq!(indices.len(), 6);
        assert_eq!(vertices[0].position, [0.0, 0.0, 0.0]);
        assert_eq!(vertices[2].position, [100.0, 50.0, 0.0]);
    }

    #[test]
    fn test_circle_builder() {
        let (vertices, indices) = VertexBuilder::circle(50.0, 50.0, 25.0, [0.0, 1.0, 0.0, 1.0], 8);
        assert_eq!(vertices.len(), 9); // Center + 8 segments
        assert_eq!(indices.len(), 24); // 8 triangles * 3 indices
    }
}
