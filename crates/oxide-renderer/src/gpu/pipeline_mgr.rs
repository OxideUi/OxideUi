//! Render pipeline management
//!
//! BLOCCO 5: Pipeline Creation
//! Handles render pipeline, bind groups, and pipeline state

use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, BlendState, BufferBindingType, ColorTargetState,
    ColorWrites, Device, Face, FragmentState, FrontFace, MultisampleState, PipelineLayoutDescriptor,
    PolygonMode, PrimitiveState, PrimitiveTopology, RenderPipeline, RenderPipelineDescriptor,
    ShaderStages, TextureFormat, VertexState,
};

use super::{buffer_mgr::{BufferManager, SimpleVertex}, shader_mgr::ShaderManager};

/// Manages render pipeline and bind groups
pub struct PipelineManager {
    bind_group_layout: BindGroupLayout,
    bind_group: BindGroup,
    render_pipeline: RenderPipeline,
}

impl PipelineManager {
    /// Create new pipeline manager
    ///
    /// # Arguments
    /// * `device` - GPU device
    /// * `shader` - Compiled shader module
    /// * `buffer_mgr` - Buffer manager (for uniform binding)
    /// * `surface_format` - Surface texture format
    pub fn new(
        device: &Device,
        shader: &ShaderManager,
        buffer_mgr: &BufferManager,
        surface_format: TextureFormat,
    ) -> anyhow::Result<Self> {
        println!("=== PIPELINE CREATION ===");
        
        // Create bind group layout for uniform buffer
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Bind Group Layout"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        // Create bind group with actual uniform buffer
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Bind Group"),
            layout: &bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: buffer_mgr.uniform_buffer().as_entire_binding(),
            }],
        });

        // Create pipeline layout
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create render pipeline
        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: shader.module(),
                entry_point: shader.vertex_entry(),
                buffers: &[SimpleVertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(FragmentState {
                module: shader.module(),
                entry_point: shader.fragment_entry(),
                targets: &[Some(ColorTargetState {
                    format: surface_format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: None, // No culling for debugging
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
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

        println!("Bind group layout: created");
        println!("Bind group: created");
        println!("Render pipeline: created");
        println!("Surface format: {:?}", surface_format);
        println!("Blend mode: ALPHA_BLENDING");
        println!("=========================");

        Ok(Self {
            bind_group_layout,
            bind_group,
            render_pipeline,
        })
    }

    /// Get bind group reference
    pub fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }

    /// Get render pipeline reference
    pub fn pipeline(&self) -> &RenderPipeline {
        &self.render_pipeline
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gpu::{DeviceManager, SurfaceManager};
    use wgpu::Backends;
    use winit::dpi::PhysicalSize;
    use winit::event_loop::EventLoop;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_pipeline_creation() {
        let dm = DeviceManager::new(Backends::all()).await.unwrap();
        let shader = ShaderManager::from_wgsl(
            dm.device(),
            include_str!("../shaders/simple.wgsl"),
            Some("Test Shader"),
        )
        .unwrap();
        let buffer_mgr = BufferManager::new(dm.device());

        // Use a common format for testing
        let format = TextureFormat::Bgra8UnormSrgb;

        let pipeline_mgr = PipelineManager::new(dm.device(), &shader, &buffer_mgr, format);

        assert!(pipeline_mgr.is_ok());
    }

    #[tokio::test]
    async fn test_bind_group_setup() {
        let dm = DeviceManager::new(Backends::all()).await.unwrap();
        let shader = ShaderManager::from_wgsl(
            dm.device(),
            include_str!("../shaders/simple.wgsl"),
            Some("Test Shader"),
        )
        .unwrap();
        let buffer_mgr = BufferManager::new(dm.device());

        let format = TextureFormat::Bgra8UnormSrgb;
        let pipeline_mgr = PipelineManager::new(dm.device(), &shader, &buffer_mgr, format).unwrap();

        // Verify bind group exists
        let _bg = pipeline_mgr.bind_group();
        let _pipeline = pipeline_mgr.pipeline();
    }

    #[test]
    fn test_blend_state_configuration() {
        // Verify blend state is correct (ALPHA_BLENDING)
        let blend = BlendState::ALPHA_BLENDING;
        
        // ALPHA_BLENDING should have:
        // - color: src_alpha * src + (1 - src_alpha) * dst
        // - alpha: 1 * src + (1 - src_alpha) * dst
        assert_eq!(blend.color.src_factor, wgpu::BlendFactor::SrcAlpha);
        assert_eq!(blend.color.dst_factor, wgpu::BlendFactor::OneMinusSrcAlpha);
        assert_eq!(blend.alpha.src_factor, wgpu::BlendFactor::One);
        assert_eq!(blend.alpha.dst_factor, wgpu::BlendFactor::OneMinusSrcAlpha);
    }
}
