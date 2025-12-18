use crate::backend::{commands::RenderCommand, Backend};
use crate::gpu::{
    BufferManager, DeviceManager, PipelineManager, ShaderManager, SimpleVertex, SurfaceManager,
    TextureManager,
};
use anyhow::Result;
use async_trait::async_trait;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use wgpu::{Backends, CommandEncoderDescriptor, Surface};

pub struct WgpuBackend {
    device_mgr: Option<DeviceManager>,
    surface_mgr: Option<SurfaceManager>,
    shader_mgr: Option<ShaderManager>,
    buffer_mgr: Option<BufferManager>,
    texture_mgr: Option<TextureManager>,
    pipeline_mgr: Option<PipelineManager>,

    // State
    scale_factor: f64,

    // Cache for reuse
    vertices: Vec<SimpleVertex>,
    indices: Vec<u32>,
}

impl WgpuBackend {
    pub fn new() -> Self {
        Self {
            device_mgr: None,
            surface_mgr: None,
            shader_mgr: None,
            buffer_mgr: None,
            texture_mgr: None,
            pipeline_mgr: None,
            scale_factor: 1.0,
            vertices: Vec::with_capacity(1024),
            indices: Vec::with_capacity(1536),
        }
    }

    pub async fn init<W>(&mut self, window: &W) -> Result<()>
    where
        W: HasWindowHandle + HasDisplayHandle + Send + Sync,
    {
        println!("=== INITIALIZING WGPU BACKEND ===");

        // 1. Initialize DeviceManager
        let device_mgr = DeviceManager::new(Backends::all()).await?;
        println!("✅ DeviceManager initialized");

        // 2. Create Surface
        // Safety: The surface must live as long as the window.
        // We assume the window outlives the backend.
        let surface = device_mgr.instance().create_surface(window)?;
        let surface: Surface<'static> = unsafe { std::mem::transmute(surface) };

        // 3. Initialize SurfaceManager
        // We use a default size, it will be resized later
        let surface_mgr =
            SurfaceManager::new(surface, device_mgr.device(), device_mgr.adapter(), 800, 600)?;
        println!("✅ SurfaceManager initialized");

        // 4. Initialize ShaderManager
        let shader_mgr = ShaderManager::from_wgsl(
            device_mgr.device(),
            include_str!("../shaders/simple.wgsl"),
            Some("Simple Shader"),
        )?;
        println!("✅ ShaderManager initialized");

        // 5. Initialize BufferManager
        let mut buffer_mgr = BufferManager::new(device_mgr.device());
        println!("✅ BufferManager initialized");

        // 6. Initialize TextureManager
        let texture_mgr = TextureManager::new_with_font(device_mgr.device(), device_mgr.queue());
        println!("✅ TextureManager initialized");

        // 7. Initialize PipelineManager
        let pipeline_mgr = PipelineManager::new(
            device_mgr.device(),
            &shader_mgr,
            &buffer_mgr,
            &texture_mgr,
            surface_mgr.format(),
        )?;
        println!("✅ PipelineManager initialized");

        // Initialize projection matrix
        let width = surface_mgr.width();
        let height = surface_mgr.height();
        let projection =
            glam::Mat4::orthographic_rh(0.0, width as f32, height as f32, 0.0, -1.0, 1.0);
        buffer_mgr.upload_projection(device_mgr.queue(), &projection.to_cols_array_2d());

        self.device_mgr = Some(device_mgr);
        self.surface_mgr = Some(surface_mgr);
        self.shader_mgr = Some(shader_mgr);
        self.buffer_mgr = Some(buffer_mgr);
        self.texture_mgr = Some(texture_mgr);
        self.pipeline_mgr = Some(pipeline_mgr);

        Ok(())
    }
}

#[async_trait]
impl Backend for WgpuBackend {
    fn resize(&mut self, width: u32, height: u32) {
        if let (Some(surface_mgr), Some(device_mgr), Some(buffer_mgr)) = (
            &mut self.surface_mgr,
            &self.device_mgr,
            &mut self.buffer_mgr,
        ) {
            if let Err(e) = surface_mgr.resize(width, height, device_mgr.device()) {
                eprintln!("Failed to resize surface: {}", e);
            }

            // Update projection matrix using logical coordinates
            // This ensures that the UI coordinates (which are logical) map correctly to the physical viewport
            let logical_width = width as f64 / self.scale_factor;
            let logical_height = height as f64 / self.scale_factor;

            let projection = glam::Mat4::orthographic_rh(
                0.0,
                logical_width as f32,
                logical_height as f32,
                0.0,
                -1.0,
                1.0,
            );
            buffer_mgr.upload_projection(device_mgr.queue(), &projection.to_cols_array_2d());
        }
    }

    fn set_scale_factor(&mut self, scale_factor: f64) {
        self.scale_factor = scale_factor;

        // Update projection matrix if initialized
        if let (Some(surface_mgr), Some(device_mgr), Some(buffer_mgr)) =
            (&self.surface_mgr, &self.device_mgr, &mut self.buffer_mgr)
        {
            let width = surface_mgr.width();
            let height = surface_mgr.height();

            let logical_width = width as f64 / self.scale_factor;
            let logical_height = height as f64 / self.scale_factor;

            let projection = glam::Mat4::orthographic_rh(
                0.0,
                logical_width as f32,
                logical_height as f32,
                0.0,
                -1.0,
                1.0,
            );
            buffer_mgr.upload_projection(device_mgr.queue(), &projection.to_cols_array_2d());
        }
    }

    fn begin_frame(&mut self) -> Result<()> {
        if self.surface_mgr.is_none() {
            anyhow::bail!("Backend not initialized");
        }
        Ok(())
    }

    fn end_frame(&mut self) -> Result<()> {
        Ok(())
    }

    fn submit(&mut self, commands: &[RenderCommand]) -> Result<()> {
        let device_mgr = self
            .device_mgr
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("DeviceManager not initialized"))?;
        let texture_mgr = self
            .texture_mgr
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("TextureManager not initialized"))?;
        let pipeline_mgr = self
            .pipeline_mgr
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("PipelineManager not initialized"))?;
        let buffer_mgr = self
            .buffer_mgr
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("BufferManager not initialized"))?;
        let surface_mgr = self
            .surface_mgr
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("SurfaceManager not initialized"))?;

        // 1. Clear buffers
        self.vertices.clear();
        self.indices.clear();

        let mut vertex_count = 0;
        let mut current_index_start = 0;
        let mut current_index_count = 0;
        let mut batches: Vec<DrawBatch> = Vec::new();
        let mut scissor_stack: Vec<[u32; 4]> = Vec::new();

        let get_current_scissor =
            |stack: &[[u32; 4]]| -> Option<[u32; 4]> { stack.last().cloned() };

        // 2. Process commands
        for cmd in commands {
            match cmd {
                RenderCommand::PushClip(rect) => {
                    if current_index_count > 0 {
                        batches.push(DrawBatch {
                            index_start: current_index_start,
                            index_count: current_index_count,
                            scissor: get_current_scissor(&scissor_stack),
                        });
                        current_index_start += current_index_count;
                        current_index_count = 0;
                    }
                    // Calculate scissor (simplified reuse from existing code)
                    let scale = self.scale_factor;
                    let x = (rect.x as f64 * scale).round() as i32;
                    let y = (rect.y as f64 * scale).round() as i32;
                    let w = (rect.width as f64 * scale).round() as i32;
                    let h = (rect.height as f64 * scale).round() as i32;
                    let surface_w = surface_mgr.width() as i32;
                    let surface_h = surface_mgr.height() as i32;
                    let min_x = x.max(0);
                    let min_y = y.max(0);
                    let max_x = (x + w).min(surface_w).max(min_x);
                    let max_y = (y + h).min(surface_h).max(min_y);
                    let mut new_rect = [
                        min_x as u32,
                        min_y as u32,
                        (max_x - min_x) as u32,
                        (max_y - min_y) as u32,
                    ];
                    if let Some(parent) = scissor_stack.last() {
                        let px = parent[0];
                        let py = parent[1];
                        let pw = parent[2];
                        let ph = parent[3];
                        let ix = new_rect[0].max(px);
                        let iy = new_rect[1].max(py);
                        let iw = (new_rect[0] + new_rect[2]).min(px + pw).saturating_sub(ix);
                        let ih = (new_rect[1] + new_rect[3]).min(py + ph).saturating_sub(iy);
                        new_rect = [ix, iy, iw, ih];
                    }
                    scissor_stack.push(new_rect);
                }
                RenderCommand::PopClip => {
                    if current_index_count > 0 {
                        batches.push(DrawBatch {
                            index_start: current_index_start,
                            index_count: current_index_count,
                            scissor: get_current_scissor(&scissor_stack),
                        });
                        current_index_start += current_index_count;
                        current_index_count = 0;
                    }
                    scissor_stack.pop();
                }
                RenderCommand::DrawRect {
                    rect,
                    color,
                    transform,
                } => {
                    // Implementation as before...
                    let (x, y, w, h) = (rect.x, rect.y, rect.width, rect.height);
                    let transform = transform.unwrap_or(strato_core::types::Transform::identity());
                    let apply_transform = |p: [f32; 2]| -> [f32; 2] {
                        let point = strato_core::types::Point::new(p[0], p[1]);
                        let transformed = transform.transform_point(point);
                        [transformed.x, transformed.y]
                    };
                    let p0 = apply_transform([x, y]);
                    let p1 = apply_transform([x + w, y]);
                    let p2 = apply_transform([x + w, y + h]);
                    let p3 = apply_transform([x, y + h]);
                    let color_arr = [color.r, color.g, color.b, color.a];

                    self.vertices
                        .push(SimpleVertex::from(&crate::vertex::Vertex::solid(
                            p0, color_arr,
                        )));
                    self.vertices
                        .push(SimpleVertex::from(&crate::vertex::Vertex::solid(
                            p1, color_arr,
                        )));
                    self.vertices
                        .push(SimpleVertex::from(&crate::vertex::Vertex::solid(
                            p2, color_arr,
                        )));
                    self.vertices
                        .push(SimpleVertex::from(&crate::vertex::Vertex::solid(
                            p3, color_arr,
                        )));

                    self.indices.push(vertex_count);
                    self.indices.push(vertex_count + 1);
                    self.indices.push(vertex_count + 2);
                    self.indices.push(vertex_count);
                    self.indices.push(vertex_count + 2);
                    self.indices.push(vertex_count + 3);
                    vertex_count += 4;
                    current_index_count += 6;
                }
                RenderCommand::DrawText {
                    text,
                    position,
                    color,
                    font_size,
                    align,
                } => {
                    // Copied implementation from before...
                    let (x_orig, y) = *position;
                    let color_arr = [color.r, color.g, color.b, color.a];
                    let font_size = *font_size;
                    let align = *align;
                    let text_width = if align != strato_core::text::TextAlign::Left {
                        let mut width = 0.0;
                        for ch in text.chars() {
                            if let Some(glyph) = texture_mgr.get_or_cache_glyph(
                                device_mgr.queue(),
                                ch,
                                font_size as u32,
                            ) {
                                width += glyph.metrics.advance;
                            } else if ch == ' ' {
                                width += font_size * 0.3;
                            }
                        }
                        width
                    } else {
                        0.0
                    };
                    let mut x = x_orig;
                    if align == strato_core::text::TextAlign::Center {
                        x -= text_width / 2.0;
                    } else if align == strato_core::text::TextAlign::Right {
                        x -= text_width;
                    }

                    for ch in text.chars() {
                        if let Some(glyph) =
                            texture_mgr.get_or_cache_glyph(device_mgr.queue(), ch, font_size as u32)
                        {
                            let (gx, gy, w, h) = (
                                x + glyph.metrics.bearing_x as f32,
                                y + font_size - glyph.metrics.bearing_y as f32,
                                glyph.metrics.width as f32,
                                glyph.metrics.height as f32,
                            );
                            let (u0, v0, u1, v1) = glyph.uv_rect;
                            let p0 = [gx, gy];
                            let p1 = [gx + w, gy];
                            let p2 = [gx + w, gy + h];
                            let p3 = [gx, gy + h];
                            self.vertices.push(SimpleVertex {
                                position: p0,
                                color: color_arr,
                                uv: [u0, v0],
                                params: [0.0; 4],
                                flags: 1,
                            });
                            self.vertices.push(SimpleVertex {
                                position: p1,
                                color: color_arr,
                                uv: [u1, v0],
                                params: [0.0; 4],
                                flags: 1,
                            });
                            self.vertices.push(SimpleVertex {
                                position: p2,
                                color: color_arr,
                                uv: [u1, v1],
                                params: [0.0; 4],
                                flags: 1,
                            });
                            self.vertices.push(SimpleVertex {
                                position: p3,
                                color: color_arr,
                                uv: [u0, v1],
                                params: [0.0; 4],
                                flags: 1,
                            });
                            self.indices.push(vertex_count);
                            self.indices.push(vertex_count + 1);
                            self.indices.push(vertex_count + 2);
                            self.indices.push(vertex_count);
                            self.indices.push(vertex_count + 2);
                            self.indices.push(vertex_count + 3);
                            vertex_count += 4;
                            current_index_count += 6;
                            x += glyph.metrics.advance;
                        } else if ch == ' ' {
                            x += font_size * 0.3;
                        }
                    }
                }
                _ => {}
            }
        }

        // Push final batch
        if current_index_count > 0 {
            batches.push(DrawBatch {
                index_start: current_index_start,
                index_count: current_index_count,
                scissor: get_current_scissor(&scissor_stack),
            });
        }

        Self::flush_and_render(
            batches,
            device_mgr,
            surface_mgr,
            buffer_mgr,
            pipeline_mgr,
            &self.vertices,
            &self.indices,
        )
    }

    fn submit_batch(&mut self, batch: &crate::batch::RenderBatch) -> Result<()> {
        let device_mgr = self
            .device_mgr
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("DeviceManager not initialized"))?;
        let texture_mgr = self
            .texture_mgr
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("TextureManager not initialized"))?;
        let pipeline_mgr = self
            .pipeline_mgr
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("PipelineManager not initialized"))?;
        let buffer_mgr = self
            .buffer_mgr
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("BufferManager not initialized"))?;
        let surface_mgr = self
            .surface_mgr
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("SurfaceManager not initialized"))?;

        // 1. Clear buffers
        self.vertices.clear();
        self.indices.clear();

        // 2. Pre-populate vertices from batch
        // We convert them to SimpleVertex
        self.vertices.reserve(batch.vertices.len());
        for v in &batch.vertices {
            self.vertices.push(SimpleVertex::from(v));
        }

        // 3. Process commands
        let mut batches: Vec<DrawBatch> = Vec::new();
        let mut current_index_start = 0;
        let mut current_index_count = 0;
        let mut scissor_stack: Vec<[u32; 4]> = Vec::new();
        let mut vertex_count = self.vertices.len() as u32; // Offset for new vertices (text)

        let get_current_scissor =
            |stack: &[[u32; 4]]| -> Option<[u32; 4]> { stack.last().cloned() };

        // Combine commands and overlay_commands for processing
        let all_commands = batch.commands.iter().chain(batch.overlay_commands.iter());

        for cmd in all_commands {
            use crate::batch::DrawCommand;
            match cmd {
                DrawCommand::PushClip(rect) => {
                    if current_index_count > 0 {
                        batches.push(DrawBatch {
                            index_start: current_index_start,
                            index_count: current_index_count,
                            scissor: get_current_scissor(&scissor_stack),
                        });
                        current_index_start += current_index_count;
                        current_index_count = 0;
                    }
                    // Calculate scissor
                    let scale = self.scale_factor;
                    let x = (rect.x as f64 * scale).round() as i32;
                    let y = (rect.y as f64 * scale).round() as i32;
                    let w = (rect.width as f64 * scale).round() as i32;
                    let h = (rect.height as f64 * scale).round() as i32;
                    let surface_w = surface_mgr.width() as i32;
                    let surface_h = surface_mgr.height() as i32;
                    let min_x = x.max(0);
                    let min_y = y.max(0);
                    let max_x = (x + w).min(surface_w).max(min_x);
                    let max_y = (y + h).min(surface_h).max(min_y);
                    let mut new_rect = [
                        min_x as u32,
                        min_y as u32,
                        (max_x - min_x) as u32,
                        (max_y - min_y) as u32,
                    ];
                    if let Some(parent) = scissor_stack.last() {
                        let px = parent[0];
                        let py = parent[1];
                        let pw = parent[2];
                        let ph = parent[3];
                        let ix = new_rect[0].max(px);
                        let iy = new_rect[1].max(py);
                        let iw = (new_rect[0] + new_rect[2]).min(px + pw).saturating_sub(ix);
                        let ih = (new_rect[1] + new_rect[3]).min(py + ph).saturating_sub(iy);
                        new_rect = [ix, iy, iw, ih];
                    }
                    scissor_stack.push(new_rect);
                }
                DrawCommand::PopClip => {
                    if current_index_count > 0 {
                        batches.push(DrawBatch {
                            index_start: current_index_start,
                            index_count: current_index_count,
                            scissor: get_current_scissor(&scissor_stack),
                        });
                        current_index_start += current_index_count;
                        current_index_count = 0;
                    }
                    scissor_stack.pop();
                }
                DrawCommand::Rect { index_range, .. }
                | DrawCommand::TexturedQuad { index_range, .. }
                | DrawCommand::Circle { index_range, .. }
                | DrawCommand::Line { index_range, .. } => {
                    // Use pre-batched indices
                    // We need to copy indices from batch.indices[index_range] to self.indices
                    // self.vertices already contains batch vertices at offset 0
                    // batch.indices are 0-based relative to batch vertices
                    // So we can use them directly (just cast to u32)
                    for i in index_range.clone() {
                        if (i as usize) < batch.indices.len() {
                            self.indices.push(batch.indices[i as usize] as u32);
                            current_index_count += 1;
                        }
                    }
                }
                DrawCommand::Text {
                    text,
                    position,
                    color,
                    font_size,
                    align,
                    ..
                } => {
                    // Generate text vertices/indices immediate mode style
                    // Appending to self.vertices, so indices start at 'vertex_count'
                    let (x_orig, y) = *position;
                    let color_arr = [color.r, color.g, color.b, color.a];
                    let font_size = *font_size;
                    let align = *align;
                    let text_width = if align != strato_core::text::TextAlign::Left {
                        let mut width = 0.0;
                        for ch in text.chars() {
                            if let Some(glyph) = texture_mgr.get_or_cache_glyph(
                                device_mgr.queue(),
                                ch,
                                font_size as u32,
                            ) {
                                width += glyph.metrics.advance;
                            } else if ch == ' ' {
                                width += font_size * 0.3;
                            }
                        }
                        width
                    } else {
                        0.0
                    };
                    let mut x = x_orig;
                    if align == strato_core::text::TextAlign::Center {
                        x -= text_width / 2.0;
                    } else if align == strato_core::text::TextAlign::Right {
                        x -= text_width;
                    }

                    for ch in text.chars() {
                        if let Some(glyph) =
                            texture_mgr.get_or_cache_glyph(device_mgr.queue(), ch, font_size as u32)
                        {
                            let (gx, gy, w, h) = (
                                x + glyph.metrics.bearing_x as f32,
                                y + font_size - glyph.metrics.bearing_y as f32,
                                glyph.metrics.width as f32,
                                glyph.metrics.height as f32,
                            );
                            let (u0, v0, u1, v1) = glyph.uv_rect;
                            let p0 = [gx, gy];
                            let p1 = [gx + w, gy];
                            let p2 = [gx + w, gy + h];
                            let p3 = [gx, gy + h];
                            self.vertices.push(SimpleVertex {
                                position: p0,
                                color: color_arr,
                                uv: [u0, v0],
                                params: [0.0; 4],
                                flags: 1,
                            });
                            self.vertices.push(SimpleVertex {
                                position: p1,
                                color: color_arr,
                                uv: [u1, v0],
                                params: [0.0; 4],
                                flags: 1,
                            });
                            self.vertices.push(SimpleVertex {
                                position: p2,
                                color: color_arr,
                                uv: [u1, v1],
                                params: [0.0; 4],
                                flags: 1,
                            });
                            self.vertices.push(SimpleVertex {
                                position: p3,
                                color: color_arr,
                                uv: [u0, v1],
                                params: [0.0; 4],
                                flags: 1,
                            });
                            self.indices.push(vertex_count);
                            self.indices.push(vertex_count + 1);
                            self.indices.push(vertex_count + 2);
                            self.indices.push(vertex_count);
                            self.indices.push(vertex_count + 2);
                            self.indices.push(vertex_count + 3);
                            vertex_count += 4;
                            current_index_count += 6;
                            x += glyph.metrics.advance;
                        } else if ch == ' ' {
                            x += font_size * 0.3;
                        }
                    }
                }
                _ => {}
            }
        }

        if current_index_count > 0 {
            batches.push(DrawBatch {
                index_start: current_index_start,
                index_count: current_index_count,
                scissor: get_current_scissor(&scissor_stack),
            });
        }

        Self::flush_and_render(
            batches,
            device_mgr,
            surface_mgr,
            buffer_mgr,
            pipeline_mgr,
            &self.vertices,
            &self.indices,
        )
    }
}

impl WgpuBackend {
    fn flush_and_render(
        batches: Vec<DrawBatch>,
        device_mgr: &DeviceManager,
        surface_mgr: &mut SurfaceManager,
        buffer_mgr: &mut BufferManager,
        pipeline_mgr: &PipelineManager,
        vertices: &[SimpleVertex],
        indices: &[u32],
    ) -> Result<()> {
        // 3. Update buffers
        buffer_mgr.upload_vertices(device_mgr.device(), device_mgr.queue(), vertices);
        buffer_mgr.upload_indices(device_mgr.device(), device_mgr.queue(), indices);

        // 4. Render Pass
        let output = surface_mgr.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = device_mgr
            .device()
            .create_command_encoder(&CommandEncoderDescriptor {
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
                            r: 0.1,
                            g: 0.1,
                            b: 0.1,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            if !indices.is_empty() {
                render_pass.set_pipeline(pipeline_mgr.pipeline());
                render_pass.set_bind_group(0, pipeline_mgr.bind_group(), &[]);
                render_pass.set_vertex_buffer(0, buffer_mgr.vertex_buffer().slice(..));
                render_pass.set_index_buffer(
                    buffer_mgr.index_buffer().slice(..),
                    wgpu::IndexFormat::Uint32,
                );

                for batch in batches {
                    if batch.index_count == 0 {
                        continue;
                    }
                    if let Some(scissor) = batch.scissor {
                        if scissor[2] == 0 || scissor[3] == 0 {
                            continue;
                        }
                        render_pass
                            .set_scissor_rect(scissor[0], scissor[1], scissor[2], scissor[3]);
                    } else {
                        render_pass.set_scissor_rect(
                            0,
                            0,
                            surface_mgr.width(),
                            surface_mgr.height(),
                        );
                    }
                    render_pass.draw_indexed(
                        batch.index_start..batch.index_start + batch.index_count,
                        0,
                        0..1,
                    );
                }
            }
        }
        device_mgr.queue().submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }
}

struct DrawBatch {
    index_start: u32,
    index_count: u32,
    scissor: Option<[u32; 4]>,
}
