//! Vertex data structures and buffers

use bytemuck::{Pod, Zeroable};
use oxide_core::types::Color;

/// Vertex data for GPU rendering
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
    pub tex_coords: [f32; 2],
}

impl Vertex {
    /// Create a new vertex
    pub fn new(position: [f32; 3], color: Color, tex_coords: [f32; 2]) -> Self {
        Self {
            position,
            color: color.to_array(),
            tex_coords,
        }
    }

    /// Vertex buffer layout descriptor
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 7]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

/// Vertex buffer wrapper
pub struct VertexBuffer {
    buffer: wgpu::Buffer,
    count: u32,
    capacity: u32,
}

impl VertexBuffer {
    /// Create a new vertex buffer
    pub fn new(device: &wgpu::Device, capacity: u32) -> Self {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Vertex Buffer"),
            size: (capacity * std::mem::size_of::<Vertex>() as u32) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            buffer,
            count: 0,
            capacity,
        }
    }

    /// Update buffer data
    pub fn update(&mut self, queue: &wgpu::Queue, vertices: &[Vertex]) {
        if vertices.len() > self.capacity as usize {
            panic!("Vertex buffer overflow");
        }
        
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(vertices));
        self.count = vertices.len() as u32;
    }

    /// Get buffer reference
    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    /// Get vertex count
    pub fn count(&self) -> u32 {
        self.count
    }
}

/// Index buffer wrapper
pub struct IndexBuffer {
    buffer: wgpu::Buffer,
    count: u32,
    capacity: u32,
}

impl IndexBuffer {
    /// Create a new index buffer
    pub fn new(device: &wgpu::Device, capacity: u32) -> Self {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Index Buffer"),
            size: (capacity * std::mem::size_of::<u32>() as u32) as u64,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            buffer,
            count: 0,
            capacity,
        }
    }

    /// Update buffer data
    pub fn update(&mut self, queue: &wgpu::Queue, indices: &[u32]) {
        if indices.len() > self.capacity as usize {
            panic!("Index buffer overflow");
        }
        
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(indices));
        self.count = indices.len() as u32;
    }

    /// Get buffer reference
    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }

    /// Get index count
    pub fn count(&self) -> u32 {
        self.count
    }
}
