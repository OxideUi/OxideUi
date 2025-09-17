//! Render pipeline management for wgpu

use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, Device, PipelineLayout,
    PipelineLayoutDescriptor, RenderPipeline, RenderPipelineDescriptor, ShaderStages,
    TextureSampleType, TextureViewDimension, VertexState, FragmentState, ColorTargetState,
    BlendState, ColorWrites, PrimitiveState, MultisampleState, VertexBufferLayout,
    VertexAttribute, VertexFormat, BufferAddress, VertexStepMode,
};
use crate::vertex::Vertex;

/// Uniform data for the UI shader
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct UIUniforms {
    pub view_proj: [[f32; 4]; 4],
    pub screen_size: [f32; 2],
    pub time: f32,
    pub _padding: f32,
}

impl UIUniforms {
    pub fn new(width: f32, height: f32, time: f32) -> Self {
        // Create orthographic projection matrix
        let view_proj = Self::orthographic_projection(0.0, width, height, 0.0, -1.0, 1.0);
        
        Self {
            view_proj,
            screen_size: [width, height],
            time,
            _padding: 0.0,
        }
    }

    fn orthographic_projection(
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        near: f32,
        far: f32,
    ) -> [[f32; 4]; 4] {
        let width = right - left;
        let height = top - bottom;
        let depth = far - near;

        [
            [2.0 / width, 0.0, 0.0, 0.0],
            [0.0, -2.0 / height, 0.0, 0.0],
            [0.0, 0.0, -1.0 / depth, 0.0],
            [-(right + left) / width, -(top + bottom) / height, -near / depth, 1.0],
        ]
    }
}

/// Render pipeline for UI rendering
pub struct UIPipeline {
    pub pipeline: RenderPipeline,
    pub bind_group_layout: BindGroupLayout,
    pub uniform_buffer: Buffer,
    pub bind_group: BindGroup,
}

impl UIPipeline {
    /// Create a new UI render pipeline
    pub fn new(device: &Device, surface_format: wgpu::TextureFormat) -> Self {
        // Load shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("UI Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/ui.wgsl").into()),
        });

        // Create bind group layout
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("UI Bind Group Layout"),
            entries: &[
                // Uniforms
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Texture
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                // Sampler
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        // Create pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("UI Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create render pipeline
        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("UI Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(ColorTargetState {
                    format: surface_format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        // Create uniform buffer
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("UI Uniform Buffer"),
            size: std::mem::size_of::<UIUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create default texture (1x1 white pixel)
        let default_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Default Texture"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let default_texture_view = default_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Create sampler
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("UI Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        // Create bind group
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("UI Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&default_texture_view),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        Self {
            pipeline,
            bind_group_layout,
            uniform_buffer,
            bind_group,
        }
    }

    /// Update uniforms
    pub fn update_uniforms(&self, queue: &wgpu::Queue, uniforms: &UIUniforms) {
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[*uniforms]));
    }

    /// Create a bind group with a custom texture
    pub fn create_bind_group_with_texture(
        &self,
        device: &Device,
        texture_view: &wgpu::TextureView,
        sampler: &wgpu::Sampler,
    ) -> BindGroup {
        device.create_bind_group(&BindGroupDescriptor {
            label: Some("UI Bind Group with Texture"),
            layout: &self.bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: self.uniform_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(texture_view),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ],
        })
    }
}

/// Text rendering pipeline
pub struct TextPipeline {
    pub pipeline: RenderPipeline,
    pub bind_group_layout: BindGroupLayout,
}

impl TextPipeline {
    /// Create a new text render pipeline
    pub fn new(device: &Device, surface_format: wgpu::TextureFormat) -> Self {
        // For now, use the same shader as UI
        // In a real implementation, you'd have a specialized text shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Text Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/ui.wgsl").into()),
        });

        // Create bind group layout (same as UI for now)
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Text Bind Group Layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Text Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Text Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(ColorTargetState {
                    format: surface_format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        Self {
            pipeline,
            bind_group_layout,
        }
    }
}

/// Pipeline manager for handling multiple render pipelines
pub struct PipelineManager {
    pub ui_pipeline: UIPipeline,
    pub text_pipeline: TextPipeline,
}

impl PipelineManager {
    /// Create a new pipeline manager
    pub fn new(device: &Device, surface_format: wgpu::TextureFormat) -> Self {
        let ui_pipeline = UIPipeline::new(device, surface_format);
        let text_pipeline = TextPipeline::new(device, surface_format);

        Self {
            ui_pipeline,
            text_pipeline,
        }
    }

    /// Update all pipeline uniforms
    pub fn update_uniforms(&self, queue: &wgpu::Queue, uniforms: &UIUniforms) {
        self.ui_pipeline.update_uniforms(queue, uniforms);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_uniforms_creation() {
        let uniforms = UIUniforms::new(800.0, 600.0, 1.0);
        assert_eq!(uniforms.screen_size, [800.0, 600.0]);
        assert_eq!(uniforms.time, 1.0);
    }

    #[test]
    fn test_orthographic_projection() {
        let proj = UIUniforms::orthographic_projection(0.0, 800.0, 600.0, 0.0, -1.0, 1.0);
        
        // Check that it's a valid orthographic projection matrix
        assert_eq!(proj[0][0], 2.0 / 800.0); // X scale
        assert_eq!(proj[1][1], -2.0 / 600.0); // Y scale (flipped for screen coordinates)
        assert_eq!(proj[3][3], 1.0); // W component
    }
}
