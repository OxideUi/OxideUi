//! Buffer management for vertices, indices, and uniforms
//!
//! BLOCCO 4: Buffer Management
//! Handles GPU buffer creation, upload, and management

use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Buffer, BufferAddress, BufferUsages, Device, Queue, VertexAttribute, VertexBufferLayout,
    VertexFormat, VertexStepMode,
};
use std::mem;

/// Simple vertex for 2D rendering (position + color + UV)
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SimpleVertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
    pub uv: [f32; 2],
    pub params: [f32; 4],
    pub flags: u32,
}

impl SimpleVertex {
    /// Vertex buffer layout descriptor
    pub fn desc() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: mem::size_of::<SimpleVertex>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &[
                // Position (Location 0)
                VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: VertexFormat::Float32x2,
                },
                // Color (Location 1)
                VertexAttribute {
                    offset: mem::size_of::<[f32; 2]>() as BufferAddress,
                    shader_location: 1,
                    format: VertexFormat::Float32x4,
                },
                // UV (Location 2)
                VertexAttribute {
                    offset: (mem::size_of::<[f32; 2]>() + mem::size_of::<[f32; 4]>()) as BufferAddress,
                    shader_location: 2,
                    format: VertexFormat::Float32x2,
                },
                // Params (Location 3)
                VertexAttribute {
                    offset: (mem::size_of::<[f32; 2]>() * 2 + mem::size_of::<[f32; 4]>()) as BufferAddress,
                    shader_location: 3,
                    format: VertexFormat::Float32x4,
                },
                // Flags (Location 4)
                VertexAttribute {
                    offset: (mem::size_of::<[f32; 2]>() * 2 + mem::size_of::<[f32; 4]>() * 2) as BufferAddress,
                    shader_location: 4,
                    format: VertexFormat::Uint32,
                },
            ],
        }
    }
}

/// Manages GPU buffers
pub struct BufferManager {
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    uniform_buffer: Buffer,
    vertex_capacity: u64,
    index_capacity: u64,
}

impl BufferManager {
    /// Initial capacity for vertex buffer (number of vertices)
    const INITIAL_VERTEX_COUNT: u64 = 1024;
    /// Initial capacity for index buffer (number of indices)
    const INITIAL_INDEX_COUNT: u64 = 1024 * 3;

    /// Create new buffer manager
    ///
    /// # Arguments
    /// * `device` - GPU device
    pub fn new(device: &Device) -> Self {
        // Create initial empty buffers with COPY_DST usage for updates
        
        let vertex_size = Self::INITIAL_VERTEX_COUNT * mem::size_of::<SimpleVertex>() as u64;
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Vertex Buffer"),
            size: vertex_size,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let index_size = Self::INITIAL_INDEX_COUNT * mem::size_of::<u32>() as u64;
        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Index Buffer"),
            size: index_size,
            usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Uniform buffer for projection matrix (4x4 float matrix = 64 bytes)
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Uniform Buffer"),
            size: 64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            vertex_buffer,
            index_buffer,
            uniform_buffer,
            vertex_capacity: vertex_size,
            index_capacity: index_size,
        }
    }

    /// Upload vertices to GPU
    pub fn upload_vertices(&mut self, device: &Device, queue: &Queue, data: &[SimpleVertex]) {
        let required_size = (data.len() * mem::size_of::<SimpleVertex>()) as u64;

        if required_size > self.vertex_capacity {
            // Resize buffer
            self.vertex_capacity = required_size.max(self.vertex_capacity * 2);
            self.vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Vertex Buffer (Resized)"),
                size: self.vertex_capacity,
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }

        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(data));
    }

    /// Upload indices to GPU
    pub fn upload_indices(&mut self, device: &Device, queue: &Queue, data: &[u32]) {
        let required_size = (data.len() * mem::size_of::<u32>()) as u64;

        if required_size > self.index_capacity {
            // Resize buffer
            self.index_capacity = required_size.max(self.index_capacity * 2);
            self.index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Index Buffer (Resized)"),
                size: self.index_capacity,
                usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }

        queue.write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(data));
    }

    /// Upload projection matrix
    pub fn upload_projection(&mut self, queue: &Queue, matrix: &[[f32; 4]; 4]) {
        // Flatten matrix to [f32; 16] for upload
        let data: &[f32; 16] = bytemuck::cast_ref(matrix);
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(data));
    }

    /// Get vertex buffer reference
    pub fn vertex_buffer(&self) -> &Buffer {
        &self.vertex_buffer
    }

    /// Get index buffer reference
    pub fn index_buffer(&self) -> &Buffer {
        &self.index_buffer
    }

    /// Get uniform buffer reference
    pub fn uniform_buffer(&self) -> &Buffer {
        &self.uniform_buffer
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gpu::DeviceManager;
    use wgpu::Backends;

    #[test]
    fn test_vertex_layout() {
        let layout = SimpleVertex::desc();
        assert_eq!(layout.array_stride, mem::size_of::<SimpleVertex>() as u64);
        assert_eq!(layout.step_mode, VertexStepMode::Vertex);
        assert_eq!(layout.attributes.len(), 5);
        // Position
        assert_eq!(layout.attributes[0].format, VertexFormat::Float32x2);
        assert_eq!(layout.attributes[0].offset, 0);
        // Color
        assert_eq!(layout.attributes[1].format, VertexFormat::Float32x4);
        assert_eq!(layout.attributes[1].offset, 8);
        // UV
        assert_eq!(layout.attributes[2].format, VertexFormat::Float32x2);
        assert_eq!(layout.attributes[2].offset, 24);
        // Params
        assert_eq!(layout.attributes[3].format, VertexFormat::Float32x4);
        assert_eq!(layout.attributes[3].offset, 32);
        // Flags
        assert_eq!(layout.attributes[4].format, VertexFormat::Uint32);
        assert_eq!(layout.attributes[4].offset, 48);
    }

    #[tokio::test]
    async fn test_buffer_creation() {
        let dm = DeviceManager::new(Backends::all()).await.unwrap();
        let buffer_mgr = BufferManager::new(dm.device());

        // Check buffers exist
        // Note: We can't easily check internal wgpu state, but we can ensure no panic
        let _v = buffer_mgr.vertex_buffer();
        let _i = buffer_mgr.index_buffer();
        let _u = buffer_mgr.uniform_buffer();
    }

    #[tokio::test]
    async fn test_buffer_upload() {
        let dm = DeviceManager::new(Backends::all()).await.unwrap();
        let mut buffer_mgr = BufferManager::new(dm.device());

        let vertices = vec![
            SimpleVertex { position: [0.0, 0.0], color: [1.0, 0.0, 0.0, 1.0], uv: [0.0, 0.0], params: [0.0; 4], flags: 0 },
            SimpleVertex { position: [1.0, 1.0], color: [0.0, 1.0, 0.0, 1.0], uv: [1.0, 1.0], params: [0.0; 4], flags: 0 },
        ];
        let indices: Vec<u32> = vec![0, 1, 2];
        let projection = [[1.0; 4]; 4];

        // Should not panic
        buffer_mgr.upload_vertices(dm.device(), dm.queue(), &vertices);
        buffer_mgr.upload_indices(dm.device(), dm.queue(), &indices);
        buffer_mgr.upload_projection(dm.queue(), &projection);
    }
}
