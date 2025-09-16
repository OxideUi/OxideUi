//! GPU context and surface management

use std::sync::{Arc, OnceLock};
use wgpu::util::DeviceExt;
use bytemuck::cast_slice;
use glam::Mat4;
use wgpu::{Device, Queue, SurfaceConfiguration, TextureFormat};
use winit::window::Window;
use tracing::debug;
// Removed deprecated raw_window_handle imports - not currently used

/// Global GPU cache to avoid recreating device/queue every frame
static GPU_CACHE: OnceLock<(Arc<Device>, Arc<Queue>, TextureFormat)> = OnceLock::new();

/// Global wgpu Instance to ensure all Surfaces and the Device share the same parent
static INSTANCE: OnceLock<wgpu::Instance> = OnceLock::new();

fn global_instance() -> &'static wgpu::Instance {
    INSTANCE.get_or_init(|| {
        wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        })
    })
}

/// Render context holding GPU resources
pub struct RenderContext {
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
    pub surface_format: TextureFormat,
}

impl RenderContext {
    /// Create a new render context
    pub async fn new<'a>(window: &'a Window) -> anyhow::Result<(Self, Surface<'a>)> {
        let instance = global_instance();
        let surface = instance.create_surface(window)?;

        // Initialize or reuse cached device/queue/format
        let (device_arc, queue_arc, cached_format) = if let Some((d, q, f)) = GPU_CACHE.get() {
            (d.clone(), q.clone(), *f)
        } else {
            // First-time initialization: pick adapter compatible with this surface
            let adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::HighPerformance,
                    compatible_surface: Some(&surface),
                    force_fallback_adapter: false,
                })
                .await
                .ok_or_else(|| anyhow::anyhow!("Failed to find suitable GPU adapter"))?;

            let (device, queue) = adapter
                .request_device(
                    &wgpu::DeviceDescriptor {
                        label: Some("OxideUI Device"),
                        required_features: wgpu::Features::empty(),
                        required_limits: if cfg!(target_arch = "wasm32") {
                            wgpu::Limits::downlevel_webgl2_defaults()
                        } else {
                            wgpu::Limits::default()
                        },
                    },
                    None,
                )
                .await?;

            // Choose a surface format compatible with this adapter
            let surface_caps = surface.get_capabilities(&adapter);
            let surface_format = surface_caps
                .formats
                .iter()
                .find(|f| f.is_srgb())
                .copied()
                .unwrap_or(surface_caps.formats[0]);

            let device_arc = Arc::new(device);
            let queue_arc = Arc::new(queue);
            let _ = GPU_CACHE.set((device_arc.clone(), queue_arc.clone(), surface_format));
            (device_arc, queue_arc, surface_format)
        };

        let context = Self {
            device: device_arc,
            queue: queue_arc,
            surface_format: cached_format,
        };

        // Wrap raw surface into our Surface type with an initial config
        let wrapped_surface = Surface {
            surface,
            config: SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: context.surface_format,
                width: 0,
                height: 0,
                present_mode: wgpu::PresentMode::AutoVsync,
                alpha_mode: wgpu::CompositeAlphaMode::Auto,
                view_formats: vec![],
                desired_maximum_frame_latency: 2,
            },
            size: (0, 0),
        };

        Ok((context, wrapped_surface))
    }

    /// Get device limits
    pub fn limits(&self) -> wgpu::Limits {
        self.device.limits()
    }
}

/// Surface wrapper for rendering
pub struct Surface<'a> {
    pub surface: wgpu::Surface<'a>,
    pub config: SurfaceConfiguration,
    size: (u32, u32),
}

impl<'a> Surface<'a> {
    /// Configure the surface
    pub fn configure(&mut self, context: &RenderContext, width: u32, height: u32) {
        self.size = (width, height);
        self.config = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: context.surface_format,
            width,
            height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        self.surface.configure(&context.device, &self.config);
    }

    /// Get current texture
    pub fn get_current_texture(&self) -> Result<wgpu::SurfaceTexture, wgpu::SurfaceError> {
        self.surface.get_current_texture()
    }

    /// Get surface size
    pub fn size(&self) -> (u32, u32) {
        self.size
    }

    /// Resize the surface
    pub fn resize(&mut self, context: &RenderContext, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.configure(context, width, height);
        }
    }
}

/// Enhanced renderer with performance optimizations
pub struct Renderer<'a> {
    context: Arc<RenderContext>,
    surface: Surface<'a>,
    depth_texture: Option<wgpu::TextureView>,
    // Rendering pipeline
    pipeline: crate::pipeline::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    // Textures
    white_texture: wgpu::Texture,
    white_view: wgpu::TextureView,
    sampler: wgpu::Sampler,
    // Text rendering
    text_renderer: crate::text::TextRenderer,
    // Performance optimizations
    frame_stats: FrameStats,
    vertex_buffer_pool: Vec<wgpu::Buffer>,
    index_buffer_pool: Vec<wgpu::Buffer>,
    current_frame: u64,
}

/// Frame performance statistics
#[derive(Debug, Default)]
pub struct FrameStats {
    pub frame_time: f32,
    pub draw_calls: u32,
    pub vertices_rendered: u32,
    pub triangles_rendered: u32,
    pub texture_switches: u32,
}

impl FrameStats {
    pub fn reset(&mut self) {
        *self = Default::default();
    }
    
    pub fn fps(&self) -> f32 {
        if self.frame_time > 0.0 {
            1000.0 / self.frame_time
        } else {
            0.0
        }
    }
}

impl<'a> Renderer<'a> {
    /// Create a new renderer
    pub async fn new(window: &'a Window) -> anyhow::Result<Self> {
        let (context, mut surface) = RenderContext::new(window).await?;
        let size = window.inner_size();
        surface.configure(&context, size.width, size.height);

        let device = &context.device;
        let queue = &context.queue;

        // Build shader/pipeline once
        let shader = crate::pipeline::ShaderModule::default(device);
        let pipeline = crate::pipeline::RenderPipeline::new(device, &shader, context.surface_format);

        // Create uniform buffer once (transform + projection = 2 * mat4 = 32 f32s)
        let mut uniforms_data = [0.0f32; 32];
        // Start with identity transform and an initial projection for current size
        let transform = glam::Mat4::IDENTITY.to_cols_array();
        let (w, h) = (size.width.max(1) as f32, size.height.max(1) as f32);
        let projection = glam::Mat4::orthographic_rh(0.0, w, h, 0.0, -1.0, 1.0).to_cols_array();
        uniforms_data[..16].copy_from_slice(&transform);
        uniforms_data[16..].copy_from_slice(&projection);
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&uniforms_data),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create a persistent 1x1 white texture and sampler
        let white_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("White Texture"),
            size: wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let white_view = white_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let white_pixel: [u8; 4] = [255, 255, 255, 255];
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &white_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &white_pixel,
            wgpu::ImageDataLayout { offset: 0, bytes_per_row: Some(4), rows_per_image: Some(1) },
            wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
        );
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Default Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        // Create bind group once (buffer is updated per frame)
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Main Bind Group"),
            layout: pipeline.bind_group_layout(),
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: uniform_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&white_view) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(&sampler) },
            ],
        });

        Ok(Self {
            context: Arc::new(context),
            surface,
            depth_texture: None,
            pipeline,
            uniform_buffer,
            bind_group,
            white_texture,
            white_view,
            sampler,
            text_renderer: crate::text::TextRenderer::new(),
            frame_stats: FrameStats::default(),
            vertex_buffer_pool: Vec::new(),
            index_buffer_pool: Vec::new(),
            current_frame: 0,
        })
    }

    /// Begin a new frame with performance tracking
    pub fn begin_frame(&mut self) -> anyhow::Result<wgpu::SurfaceTexture> {
        use std::time::Instant;
        
        // Reset frame stats
        self.frame_stats.reset();
        self.current_frame += 1;
        
        let start_time = Instant::now();
        let frame = self.surface.get_current_texture()?;
        
        // Track frame start time for performance metrics
        self.frame_stats.frame_time = start_time.elapsed().as_millis() as f32;
        
        Ok(frame)
    }

    /// End frame and present with performance tracking
    pub fn end_frame(&mut self, frame: wgpu::SurfaceTexture) {
        use std::time::Instant;
        
        let start_time = Instant::now();
        frame.present();
        
        // Update frame time with presentation time
        self.frame_stats.frame_time += start_time.elapsed().as_millis() as f32;
        
        // Log performance stats every 60 frames
        if self.current_frame % 60 == 0 {
            tracing::debug!(
                "Frame {}: {:.2}ms ({:.1} FPS), {} draw calls, {} vertices",
                self.current_frame,
                self.frame_stats.frame_time,
                self.frame_stats.fps(),
                self.frame_stats.draw_calls,
                self.frame_stats.vertices_rendered
            );
        }
    }

    /// Resize the renderer
    pub fn resize(&mut self, width: u32, height: u32) {
        self.surface.resize(&self.context, width, height);
        self.depth_texture = None; // Recreate depth texture on next frame
    }

    /// Get render context
    pub fn context(&self) -> &RenderContext {
        &self.context
    }

    /// Render a batch of draw commands with optimizations
    pub fn render(&mut self, batch: &crate::batch::RenderBatch) -> anyhow::Result<()> {
        use std::time::Instant;
        let render_start = Instant::now();
        
        let frame = self.begin_frame()?;
        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Upload geometry with buffer pooling first
        let vertices = batch.vertices();
        let indices = batch.indices();
        
        // Update performance stats
        self.frame_stats.vertices_rendered = vertices.len() as u32;
        self.frame_stats.triangles_rendered = (indices.len() / 3) as u32;

        let vertex_buffer = self.get_or_create_vertex_buffer(vertices);
        let index_buffer = self.get_or_create_index_buffer(indices);

        // Now get device and queue references after the mutable borrows are done
        let device = &self.context.device;
        let queue = &self.context.queue;

        // Update uniforms (identity transform + orthographic projection for current surface)
        let (width, height) = self.surface.size();
        let transform = glam::Mat4::IDENTITY.to_cols_array();
        // Origin at top-left, Y-down. bottom=height, top=0 flips Y.
        let projection = glam::Mat4::orthographic_rh(0.0, width as f32, height as f32, 0.0, -1.0, 1.0)
            .to_cols_array();
        let mut uniforms_data = [0.0f32; 32];
        uniforms_data[..16].copy_from_slice(&transform);
        uniforms_data[16..].copy_from_slice(&projection);
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&uniforms_data));

        // Create render pass
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.95,
                            g: 0.95,
                            b: 0.95,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(self.pipeline.pipeline());
            render_pass.set_bind_group(0, &self.bind_group, &[]);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..indices.len() as u32, 0, 0..1);
            
            self.frame_stats.draw_calls += 1;
        }

        queue.submit(std::iter::once(encoder.finish()));
        self.end_frame(frame);
        
        // Update total frame time
        self.frame_stats.frame_time = render_start.elapsed().as_millis() as f32;
        
        Ok(())
    }
    
    /// Get or create a vertex buffer from the pool
    fn get_or_create_vertex_buffer(&mut self, vertices: &[crate::vertex::Vertex]) -> wgpu::Buffer {
        let required_size = (vertices.len() * std::mem::size_of::<crate::vertex::Vertex>()) as u64;
        
        // Try to reuse a buffer from the pool
        for (i, buffer) in self.vertex_buffer_pool.iter().enumerate() {
            if buffer.size() >= required_size {
                let buffer = self.vertex_buffer_pool.swap_remove(i);
                self.context.queue.write_buffer(&buffer, 0, bytemuck::cast_slice(vertices));
                return buffer;
            }
        }
        
        // Create new buffer if none suitable found
        self.context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        })
    }
    
    /// Get or create an index buffer from the pool
    fn get_or_create_index_buffer(&mut self, indices: &[u32]) -> wgpu::Buffer {
        let required_size = (indices.len() * std::mem::size_of::<u32>()) as u64;
        
        // Try to reuse a buffer from the pool
        for (i, buffer) in self.index_buffer_pool.iter().enumerate() {
            if buffer.size() >= required_size {
                let buffer = self.index_buffer_pool.swap_remove(i);
                self.context.queue.write_buffer(&buffer, 0, bytemuck::cast_slice(indices));
                return buffer;
            }
        }
        
        // Create new buffer if none suitable found
        self.context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        })
    }
    
    /// Return buffers to the pool for reuse
    pub fn return_buffers_to_pool(&mut self, vertex_buffer: wgpu::Buffer, index_buffer: wgpu::Buffer) {
        // Limit pool size to prevent memory bloat
        if self.vertex_buffer_pool.len() < 10 {
            self.vertex_buffer_pool.push(vertex_buffer);
        }
        if self.index_buffer_pool.len() < 10 {
            self.index_buffer_pool.push(index_buffer);
        }
    }
    
    /// Get current frame statistics
    pub fn frame_stats(&self) -> &FrameStats {
        &self.frame_stats
    }

    /// Create depth texture when needed
    fn create_depth_texture(&mut self) -> wgpu::TextureView {
        let (width, height) = self.surface.size();
        let depth_texture = self.context.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        depth_texture.create_view(&wgpu::TextureViewDescriptor::default())
    }
}

