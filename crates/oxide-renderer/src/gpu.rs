//! GPU rendering backend using wgpu for cross-platform support

use std::sync::Arc;
use wgpu::*;
use winit::window::Window;
use oxide_core::types::{Color, Rect, Transform, Size};
use crate::{Backend, RendererConfig, vertex::Vertex, batch::RenderBatch};

/// Main renderer struct
pub struct Renderer<'a> {
    device: Arc<Device>,
    queue: Arc<Queue>,
    surface: Surface<'a>,
    surface_config: SurfaceConfiguration,
    render_pipeline: RenderPipeline,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    uniform_buffer: Buffer,
    bind_group: BindGroup,
    batch: RenderBatch,
    config: RendererConfig,
}

/// Render context for managing GPU resources
pub struct RenderContext {
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
    pub adapter: Adapter,
    pub instance: Instance,
}

impl RenderContext {
    /// Create a new render context
    pub async fn new() -> anyhow::Result<Self> {
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::all(),
            dx12_shader_compiler: Dx12Compiler::Fxc,
            flags: InstanceFlags::default(),
            gles_minor_version: Gles3MinorVersion::Automatic,
        });

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::default(),
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| anyhow::anyhow!("Failed to find suitable adapter"))?;

        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: Some("OxideUI Device"),
                    required_features: Features::empty(),
                    required_limits: Limits::default(),
                },
                None,
            )
            .await?;

        Ok(Self {
            device: Arc::new(device),
            queue: Arc::new(queue),
            adapter,
            instance,
        })
    }
}

impl<'a> Renderer<'a> {
    /// Create a new renderer
    pub async fn new(
        window: &'a Window,
        config: RendererConfig,
    ) -> anyhow::Result<Self> {
        let context = RenderContext::new().await?;
        
        let surface_wrapper = Surface::new(window, &context.instance)?;
        let surface_caps = surface_wrapper.inner.get_capabilities(&context.adapter);
        
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let size = window.inner_size();
        let surface_config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: if config.vsync {
                PresentMode::AutoVsync
            } else {
                PresentMode::AutoNoVsync
            },
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface_wrapper.inner.configure(&context.device, &surface_config);

        // Create shader module
        let shader = context.device.create_shader_module(ShaderModuleDescriptor {
            label: Some("OxideUI Shader"),
            source: ShaderSource::Wgsl(include_str!("shaders/ui.wgsl").into()),
        });

        // Create uniform buffer for projection matrix
        let uniform_buffer = context.device.create_buffer(&BufferDescriptor {
            label: Some("Uniform Buffer"),
            size: 64, // 4x4 matrix
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create bind group layout
        let bind_group_layout = context.device.create_bind_group_layout(&BindGroupLayoutDescriptor {
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

        // Create bind group
        let bind_group = context.device.create_bind_group(&BindGroupDescriptor {
            label: Some("Bind Group"),
            layout: &bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        // Create render pipeline
        let render_pipeline_layout = context.device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = context.device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
                compilation_options: PipelineCompilationOptions::default(),
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(ColorTargetState {
                    format: surface_config.format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
                compilation_options: PipelineCompilationOptions::default(),
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: config.msaa_samples,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        // Create vertex and index buffers
        let vertex_buffer = context.device.create_buffer(&BufferDescriptor {
            label: Some("Vertex Buffer"),
            size: 1024 * 1024, // 1MB
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let index_buffer = context.device.create_buffer(&BufferDescriptor {
            label: Some("Index Buffer"),
            size: 512 * 1024, // 512KB
            usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Ok(Self {
            device: Arc::clone(&context.device),
            queue: Arc::clone(&context.queue),
            surface: surface_wrapper,
            surface_config,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            uniform_buffer,
            bind_group,
            batch: RenderBatch::new(),
            config,
        })
    }

    /// Resize the surface
    pub fn resize(&mut self, new_size: Size) {
        if new_size.width > 0.0 && new_size.height > 0.0 {
            self.surface_config.width = new_size.width as u32;
            self.surface_config.height = new_size.height as u32;
            self.surface.inner.configure(&self.device, &self.surface_config);
            
            // Update projection matrix
            self.update_projection_matrix();
        }
    }

    /// Update projection matrix
    fn update_projection_matrix(&self) {
        let width = self.surface_config.width as f32;
        let height = self.surface_config.height as f32;
        
        // Orthographic projection matrix
        let projection = [
            [2.0 / width, 0.0, 0.0, 0.0],
            [0.0, -2.0 / height, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [-1.0, 1.0, 0.0, 1.0],
        ];

        self.queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&projection),
        );
    }

    /// Get surface format
    pub fn surface_format(&self) -> TextureFormat {
        self.surface_config.format
    }
}

impl<'a> Backend<'a> for Renderer<'a> {
    fn init(&mut self, _surface: &crate::Surface<'a>) -> anyhow::Result<()> {
        self.update_projection_matrix();
        Ok(())
    }

    fn begin_frame(&mut self) -> anyhow::Result<()> {
        self.batch.clear();
        Ok(())
    }

    fn end_frame(&mut self) -> anyhow::Result<()> {
        let output = self.surface.inner.get_current_texture()?;
        let view = output.texture.create_view(&TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.1,
                            b: 0.1,
                            a: 1.0,
                        }),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.bind_group, &[]);
            
            // Render batched commands
            if !self.batch.vertices.is_empty() {
                self.queue.write_buffer(
                    &self.vertex_buffer,
                    0,
                    bytemuck::cast_slice(&self.batch.vertices),
                );
                
                self.queue.write_buffer(
                    &self.index_buffer,
                    0,
                    bytemuck::cast_slice(&self.batch.indices),
                );

                render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint16);
                render_pass.draw_indexed(0..self.batch.indices.len() as u32, 0, 0..1);
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    fn draw_rect(&mut self, rect: Rect, color: Color, transform: Transform) {
        self.batch.add_rect(rect, color, transform);
    }

    fn draw_text(&mut self, text: &str, position: (f32, f32), _color: Color) {
        // Text rendering will be implemented in the text module
        tracing::debug!("Drawing text: {} at {:?}", text, position);
    }

    fn submit(&mut self) -> anyhow::Result<()> {
        // Commands are submitted in end_frame
        Ok(())
    }
}

/// Surface wrapper for cross-platform compatibility
pub struct Surface<'a> {
    pub inner: wgpu::Surface<'a>,
}

impl<'a> Surface<'a> {
    pub fn new(window: &'a Window, instance: &Instance) -> anyhow::Result<Self> {
        let surface = instance.create_surface(window)?;
        Ok(Self { inner: surface })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_render_context_creation() {
        let context = RenderContext::new().await;
        assert!(context.is_ok());
    }
}

