use anyhow::Result;
use strato_renderer::{
    IntegratedRenderer, RendererBuilder, RendererConfig,
    AllocationStrategy, RenderContext,
};
use wgpu::*;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};
use std::sync::Arc;
use tracing::{info, error};

/// Advanced renderer example demonstrating the complete wgpu system
struct AdvancedRendererExample {
    window: Arc<Window>,
    surface: Surface<'static>,
    surface_config: SurfaceConfiguration,
    renderer: IntegratedRenderer,
    
    // Example resources
    vertex_buffer: Option<strato_renderer::ResourceHandle>,
    index_buffer: Option<strato_renderer::ResourceHandle>,
    render_pipeline: Option<RenderPipeline>,
    
    // State
    frame_count: u64,
}

impl AdvancedRendererExample {
    async fn new(window: Arc<Window>) -> Result<Self> {
        info!("Initializing advanced renderer example");
        
        // Create surface
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::PRIMARY,
            ..Default::default()
        });
        
        let surface = instance.create_surface(window.clone())?;
        
        // Create renderer with performance configuration
        let mut renderer = RendererBuilder::new()
            .with_instance(instance)
            .with_surface(&surface)
            .with_profiling(true)
            .with_detailed_profiling(true)
            .with_memory_strategy(AllocationStrategy::Balanced)
            .with_max_memory_pool_size(256 * 1024 * 1024) // 256MB
            .with_preferred_adapter(PowerPreference::HighPerformance)
            .with_validation(cfg!(debug_assertions))
            .build()
            .await?;
        
        // Configure surface
        let adapter = renderer.get_active_adapter()
            .expect("Current adapter not found");

        let size = window.inner_size();
        let surface_config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_capabilities(adapter).formats[0],
            width: size.width,
            height: size.height,
            present_mode: PresentMode::Fifo,
            alpha_mode: CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        
        surface.configure(&renderer.device().device, &surface_config);
        
        // Initialize renderer
        renderer.initialize().await?;
        
        info!("Renderer initialized successfully");
        info!("GPU: {}", renderer.get_device_info().device_name);
        // info!("Backend: {:?}", renderer.get_device_info().backend);
        
        let mut example = Self {
            window,
            surface,
            surface_config,
            renderer,
            vertex_buffer: None,
            index_buffer: None,
            render_pipeline: None,
            frame_count: 0,
        };
        
        // Create example resources
        example.create_resources().await?;
        
        Ok(example)
    }
    
    async fn create_resources(&mut self) -> Result<()> {
        info!("Creating example resources");
        
        // Create vertex buffer
        let vertices: &[f32] = &[
            // Triangle vertices (position + color)
            -0.5, -0.5, 1.0, 0.0, 0.0, // Bottom left - Red
             0.5, -0.5, 0.0, 1.0, 0.0, // Bottom right - Green
             0.0,  0.5, 0.0, 0.0, 1.0, // Top - Blue
        ];
        
        let vertex_buffer = self.renderer.create_buffer(
            (vertices.len() * std::mem::size_of::<f32>()) as u64,
            BufferUsages::VERTEX | BufferUsages::COPY_DST,
        )?;
        
        // Create index buffer
        let indices: &[u16] = &[0, 1, 2];
        let index_buffer = self.renderer.create_buffer(
            (indices.len() * std::mem::size_of::<u16>()) as u64,
            BufferUsages::INDEX | BufferUsages::COPY_DST,
        )?;
        
        // Load shader
        let path = std::path::PathBuf::from("examples/advanced_renderer/shaders/triangle.wgsl");
        let stage = strato_renderer::shader::ShaderStage::Vertex;
        let variant = strato_renderer::shader::ShaderVariant {
            macros: vec![],
            features: vec![],
            optimization_level: 0,
        };
        
        let shader = self.renderer.load_shader(&path, stage, variant)?;
        
        // Create render pipeline
        let render_pipeline_desc = RenderPipelineDescriptor {
            label: Some("Triangle Pipeline"),
            layout: None,
            vertex: VertexState {
                module: &shader.module,
                entry_point: "vs_main",
                buffers: &[VertexBufferLayout {
                    array_stride: 5 * std::mem::size_of::<f32>() as BufferAddress,
                    step_mode: VertexStepMode::Vertex,
                    attributes: &[
                        VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: VertexFormat::Float32x2,
                        },
                        VertexAttribute {
                            offset: 2 * std::mem::size_of::<f32>() as BufferAddress,
                            shader_location: 1,
                            format: VertexFormat::Float32x3,
                        },
                    ],
                }],
                compilation_options: Default::default(),
            },
            fragment: Some(FragmentState {
                module: &shader.module,
                entry_point: "fs_main",
                targets: &[Some(ColorTargetState {
                    format: self.surface_config.format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
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
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        };
        
        let render_pipeline = self.renderer.device().device.create_render_pipeline(&render_pipeline_desc);
        self.render_pipeline = Some(render_pipeline);
        
        self.vertex_buffer = Some(vertex_buffer);
        self.index_buffer = Some(index_buffer);
        
        info!("Resources created successfully");
        Ok(())
    }
    
    fn render(&mut self) -> Result<()> {
        // Resolve resources first to ensure they live long enough for the render pass
        let vertex_buffer_res = if let Some(handle) = self.vertex_buffer {
            self.renderer.get_buffer(handle)
        } else {
            None
        };
        
        let index_buffer_res = if let Some(handle) = self.index_buffer {
            self.renderer.get_buffer(handle)
        } else {
            None
        };

        // Begin frame
        let mut render_context = self.renderer.begin_frame()?;
        
        // Get surface texture
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&TextureViewDescriptor::default());
        
        // Begin render pass
        let mut render_pass = render_context.begin_render_pass(&RenderPassDescriptor {
            label: Some("Main Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });
        
        // Draw triangle
        if let (Some(pipeline), Some(vb), Some(ib)) = (
            &self.render_pipeline,
            &vertex_buffer_res,
            &index_buffer_res,
        ) {
            render_pass.set_pipeline(pipeline);
            render_pass.set_vertex_buffer(0, vb.slice(..));
            render_pass.set_index_buffer(ib.slice(..), IndexFormat::Uint16);
            render_pass.draw_indexed(0..3, 0, 0..1);
        }
        
        drop(render_pass);
        render_context.end_render_pass();
        
        // End frame
        self.renderer.end_frame(render_context)?;
        
        // Present
        output.present();
        
        self.frame_count += 1;
        
        // Print stats every 60 frames
        if self.frame_count % 60 == 0 {
            let stats = self.renderer.get_stats();
            info!("Frame {}: {:.2}ms avg, {}MB memory, {} resources", 
                stats.frame_count,
                stats.average_frame_time * 1000.0,
                stats.memory_usage / (1024 * 1024),
                stats.active_resources
            );
            
            if let Some(report) = self.renderer.get_performance_report() {
                info!("Performance report: {:#?}", report);
            }
        }
        
        Ok(())
    }
    
    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) -> Result<()> {
        if new_size.width > 0 && new_size.height > 0 {
            self.surface_config.width = new_size.width;
            self.surface_config.height = new_size.height;
            self.surface.configure(&self.renderer.device().device, &self.surface_config);
            self.renderer.resize((new_size.width, new_size.height))?;
        }
        Ok(())
    }
}

async fn run() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    info!("Starting advanced renderer example");
    
    // Create event loop and window
    let event_loop = EventLoop::new()?;
    let window = Arc::new(
        WindowBuilder::new()
            .with_title("Advanced wgpu Renderer Example")
            .with_inner_size(winit::dpi::LogicalSize::new(800, 600))
            .build(&event_loop)?
    );
    
    // Create example
    let mut example = AdvancedRendererExample::new(window.clone()).await?;
    
    info!("Starting event loop");
    
    event_loop.run(move |event, elwt| {
        elwt.set_control_flow(ControlFlow::Poll);
        
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => match event {
                WindowEvent::CloseRequested => {
                    info!("Close requested");
                    elwt.exit();
                }
                WindowEvent::Resized(physical_size) => {
                    if let Err(e) = example.resize(*physical_size) {
                        error!("Resize error: {}", e);
                    }
                }
                WindowEvent::RedrawRequested => {
                    if let Err(e) = example.render() {
                        error!("Render error: {}", e);
                    }
                }
                _ => {}
            },
            Event::AboutToWait => {
                window.request_redraw();
            }
            _ => {}
        }
    })?;
    
    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        error!("Application error: {}", e);
        std::process::exit(1);
    }
}