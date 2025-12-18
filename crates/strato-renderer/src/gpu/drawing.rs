//! Drawing system - integrates all GPU components
//!
//! BLOCCO 7: Drawing System
//! Final integration: converts RenderBatch to GPU draw calls

use super::{
    buffer_mgr::{BufferManager, SimpleVertex},
    device::DeviceManager,
    pipeline_mgr::PipelineManager,
    render_pass_mgr::RenderPassManager,
    shader_mgr::ShaderManager,
    surface::SurfaceManager,
    texture_mgr::TextureManager,
};
use crate::batch::RenderBatch;
use crate::vertex::VertexBuilder;
use std::sync::Arc;
use wgpu::{CommandEncoderDescriptor, IndexFormat};
use winit::window::Window;

/// Complete drawing system
pub struct DrawingSystem {
    device_mgr: DeviceManager,
    surface_mgr: SurfaceManager,
    shader_mgr: ShaderManager,
    buffer_mgr: BufferManager,
    texture_mgr: TextureManager,
    pipeline_mgr: PipelineManager,
    render_pass_mgr: RenderPassManager,
    scale_factor: f32,
}

impl DrawingSystem {
    /// Create new drawing system
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        println!("=== DRAWING SYSTEM INITIALIZATION ===");

        // BLOCCO 1: Device Setup
        let device_mgr = DeviceManager::new(wgpu::Backends::all()).await?;
        println!("✅ DeviceManager initialized");

        // BLOCCO 2: Surface Configuration
        let target = unsafe { wgpu::SurfaceTargetUnsafe::from_window(&*window)? };
        let surface = unsafe { device_mgr.instance().create_surface_unsafe(target)? };
        let size = window.inner_size();

        let surface_mgr = SurfaceManager::new(
            surface,
            device_mgr.device(),
            device_mgr.adapter(),
            size.width,
            size.height,
        )?;
        println!("✅ SurfaceManager initialized");

        // BLOCCO 3: Shader Compilation
        let shader_mgr = ShaderManager::from_wgsl(
            device_mgr.device(),
            include_str!("../shaders/simple.wgsl"),
            Some("Simple Shader"),
        )?;
        println!("✅ ShaderManager initialized");

        // BLOCCO 4: Buffer Management
        let buffer_mgr = BufferManager::new(device_mgr.device());
        println!("✅ BufferManager initialized");

        // BLOCCO 8: Texture Management
        let texture_mgr = TextureManager::new_with_font(device_mgr.device(), device_mgr.queue());
        println!("✅ TextureManager initialized");

        // BLOCCO 5: Pipeline Creation
        let pipeline_mgr = PipelineManager::new(
            device_mgr.device(),
            &shader_mgr,
            &buffer_mgr,
            &texture_mgr,
            surface_mgr.format(),
        )?;
        println!("✅ PipelineManager initialized");

        // BLOCCO 6: Render Pass
        let render_pass_mgr = RenderPassManager::new();
        println!("✅ RenderPassManager initialized");

        println!("====================================");

        Ok(Self {
            device_mgr,
            surface_mgr,
            shader_mgr,
            buffer_mgr,
            texture_mgr,
            pipeline_mgr,
            render_pass_mgr,
            scale_factor: 1.0,
        })
    }

    /// Set the DPI scale factor
    pub fn set_scale_factor(&mut self, scale_factor: f32) {
        self.scale_factor = scale_factor;
    }

    /// Render a batch
    pub fn render(&mut self, batch: &RenderBatch) -> anyhow::Result<()> {
        // 1. Process batch commands to generate vertices (including text)
        let mut vertices: Vec<SimpleVertex> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();
        let mut vertex_count = 0;

        // Clipping state
        struct GPUDrawBatch {
            index_start: u32,
            index_count: u32,
            scissor: Option<[u32; 4]>,
        }
        let mut batches: Vec<GPUDrawBatch> = Vec::new();
        let mut current_index_start = 0;
        let mut current_index_count = 0;
        let mut scissor_stack: Vec<[u32; 4]> = Vec::new();

        let get_current_scissor =
            |stack: &[[u32; 4]]| -> Option<[u32; 4]> { stack.last().cloned() };

        // Note: We ignore batch.vertices here because we regenerate everything from commands
        // to ensure correct Z-ordering and support interleaved clipping.

        for command in &batch.commands {
            match command {
                crate::batch::DrawCommand::PushClip(rect) => {
                    // Finish current batch if needed
                    if current_index_count > 0 {
                        batches.push(GPUDrawBatch {
                            index_start: current_index_start,
                            index_count: current_index_count,
                            scissor: get_current_scissor(&scissor_stack),
                        });
                        current_index_start += current_index_count;
                        current_index_count = 0;
                    }

                    // Calculate new scissor rect
                    let scale = self.scale_factor;
                    let x = (rect.x as f32 * scale).round() as i32;
                    let y = (rect.y as f32 * scale).round() as i32;
                    let w = (rect.width as f32 * scale).round() as i32;
                    let h = (rect.height as f32 * scale).round() as i32;

                    let surface_w = self.surface_mgr.width() as i32;
                    let surface_h = self.surface_mgr.height() as i32;

                    // Intersect with surface bounds
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

                    // Intersect with current scissor
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
                crate::batch::DrawCommand::PopClip => {
                    // Finish current batch if needed
                    if current_index_count > 0 {
                        batches.push(GPUDrawBatch {
                            index_start: current_index_start,
                            index_count: current_index_count,
                            scissor: get_current_scissor(&scissor_stack),
                        });
                        current_index_start += current_index_count;
                        current_index_count = 0;
                    }
                    scissor_stack.pop();
                }
                crate::batch::DrawCommand::RoundedRect {
                    rect,
                    color,
                    radius,
                    transform,
                } => {
                    let color_arr = [color.r, color.g, color.b, color.a];
                    let (v_list, i_list) = VertexBuilder::rounded_rectangle(
                        rect.x,
                        rect.y,
                        rect.width,
                        rect.height,
                        *radius,
                        color_arr,
                        8,
                    );

                    let added_count = v_list.len() as u32;
                    let index_count = i_list.len() as u32;

                    for v in v_list {
                        let mut sv = SimpleVertex::from(&v);
                        // Apply transform
                        let p = strato_core::types::Point::new(sv.position[0], sv.position[1]);
                        let transformed = transform.transform_point(p);
                        sv.position = [transformed.x, transformed.y];
                        vertices.push(sv);
                    }

                    for i in i_list {
                        indices.push((i as u32) + vertex_count);
                    }
                    vertex_count += added_count;
                    current_index_count += index_count;
                }
                crate::batch::DrawCommand::Rect {
                    rect,
                    color,
                    transform,
                    ..
                } => {
                    let (x, y, w, h) = (rect.x, rect.y, rect.width, rect.height);

                    // Apply transform using strato_core::Transform method
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

                    // Solid color vertices (uv = 0,0)
                    vertices.push(SimpleVertex::from(&crate::vertex::Vertex::solid(
                        p0, color_arr,
                    )));
                    vertices.push(SimpleVertex::from(&crate::vertex::Vertex::solid(
                        p1, color_arr,
                    )));
                    vertices.push(SimpleVertex::from(&crate::vertex::Vertex::solid(
                        p2, color_arr,
                    )));
                    vertices.push(SimpleVertex::from(&crate::vertex::Vertex::solid(
                        p3, color_arr,
                    )));

                    indices.push(vertex_count);
                    indices.push(vertex_count + 1);
                    indices.push(vertex_count + 2);
                    indices.push(vertex_count);
                    indices.push(vertex_count + 2);
                    indices.push(vertex_count + 3);

                    vertex_count += 4;
                    current_index_count += 6;
                }
                crate::batch::DrawCommand::Text {
                    text,
                    position,
                    color,
                    font_size,
                    letter_spacing,
                    align,
                } => {
                    let (mut x, y) = *position;
                    let color_arr = [color.r, color.g, color.b, color.a];
                    let font_size_val = *font_size;
                    let spacing_val = *letter_spacing;

                    // Use scale factor for high-resolution text rasterization
                    let scale = self.scale_factor;
                    let physical_font_size = (font_size_val * scale).round() as u32;

                    // Handle alignment
                    if *align != strato_core::text::TextAlign::Left {
                        let mut width = 0.0;
                        for ch in text.chars() {
                            if let Some(glyph) = self.texture_mgr.get_or_cache_glyph(
                                self.device_mgr.queue(),
                                ch,
                                physical_font_size,
                            ) {
                                // Scale metrics back to logical coordinates for layout
                                let advance = glyph.metrics.advance / scale;
                                width += advance + spacing_val;
                            } else if ch == ' ' {
                                width += font_size_val * 0.3 + spacing_val;
                            }
                        }

                        match align {
                            strato_core::text::TextAlign::Center => x -= width / 2.0,
                            strato_core::text::TextAlign::Right => x -= width,
                            _ => {} // Justify not implemented yet
                        }
                    }

                    let ascent = if let Some(metrics) =
                        self.texture_mgr.get_line_metrics(physical_font_size as f32)
                    {
                        metrics.ascent / scale
                    } else {
                        font_size_val * 0.8 // Fallback approximation
                    };

                    let baseline = y + ascent;

                    for ch in text.chars() {
                        if let Some(glyph) = self.texture_mgr.get_or_cache_glyph(
                            self.device_mgr.queue(),
                            ch,
                            physical_font_size,
                        ) {
                            // Scale metrics back to logical coordinates for rendering
                            let bearing_x = glyph.metrics.bearing_x as f32 / scale;
                            let bearing_y = glyph.metrics.bearing_y as f32 / scale;
                            let w = glyph.metrics.width as f32 / scale;
                            let h = glyph.metrics.height as f32 / scale;
                            let advance = glyph.metrics.advance / scale;

                            let glyph_x = (x + bearing_x).round();
                            let glyph_y = (baseline - bearing_y).round();

                            let (u0, v0, u1, v1) = glyph.uv_rect;

                            vertices.push(SimpleVertex::from(&crate::vertex::Vertex::textured(
                                [glyph_x, glyph_y],
                                [u0, v0],
                                color_arr,
                            )));
                            vertices.push(SimpleVertex::from(&crate::vertex::Vertex::textured(
                                [glyph_x + w, glyph_y],
                                [u1, v0],
                                color_arr,
                            )));
                            vertices.push(SimpleVertex::from(&crate::vertex::Vertex::textured(
                                [glyph_x + w, glyph_y + h],
                                [u1, v1],
                                color_arr,
                            )));
                            vertices.push(SimpleVertex::from(&crate::vertex::Vertex::textured(
                                [glyph_x, glyph_y + h],
                                [u0, v1],
                                color_arr,
                            )));

                            indices.push(vertex_count);
                            indices.push(vertex_count + 1);
                            indices.push(vertex_count + 2);
                            indices.push(vertex_count);
                            indices.push(vertex_count + 2);
                            indices.push(vertex_count + 3);

                            vertex_count += 4;
                            current_index_count += 6;

                            x += advance + spacing_val;
                        } else {
                            if ch == ' ' {
                                x += font_size_val * 0.3 + spacing_val;
                            }
                        }
                    }
                }
                crate::batch::DrawCommand::Image {
                    id,
                    data,
                    width,
                    height,
                    rect,
                    color,
                } => {
                    if let Some(image) = self.texture_mgr.get_or_upload_image(
                        self.device_mgr.queue(),
                        *id,
                        data,
                        *width,
                        *height,
                    ) {
                        let (x, y, w, h) = (rect.x, rect.y, rect.width, rect.height);
                        let (u0, v0, u1, v1) = image.uv_rect;
                        let color_arr = [color.r, color.g, color.b, color.a];

                        vertices.push(SimpleVertex::from(&crate::vertex::Vertex::textured(
                            [x, y],
                            [u0, v0],
                            color_arr,
                        )));
                        vertices.push(SimpleVertex::from(&crate::vertex::Vertex::textured(
                            [x + w, y],
                            [u1, v0],
                            color_arr,
                        )));
                        vertices.push(SimpleVertex::from(&crate::vertex::Vertex::textured(
                            [x + w, y + h],
                            [u1, v1],
                            color_arr,
                        )));
                        vertices.push(SimpleVertex::from(&crate::vertex::Vertex::textured(
                            [x, y + h],
                            [u0, v1],
                            color_arr,
                        )));

                        indices.push(vertex_count);
                        indices.push(vertex_count + 1);
                        indices.push(vertex_count + 2);
                        indices.push(vertex_count);
                        indices.push(vertex_count + 2);
                        indices.push(vertex_count + 3);

                        vertex_count += 4;
                        current_index_count += 6;
                    }
                }
                crate::batch::DrawCommand::TexturedQuad {
                    rect,
                    texture_id: _,
                    uv_rect,
                    color,
                    transform,
                    ..
                } => {
                    let (x, y, w, h) = (rect.x, rect.y, rect.width, rect.height);
                    let (u, v, uw, vh) = (uv_rect.x, uv_rect.y, uv_rect.width, uv_rect.height);
                    let color_arr = [color.r, color.g, color.b, color.a];

                    let apply_transform = |p: [f32; 2]| -> [f32; 2] {
                        let point = strato_core::types::Point::new(p[0], p[1]);
                        let transformed = transform.transform_point(point);
                        [transformed.x, transformed.y]
                    };

                    let p0 = apply_transform([x, y]);
                    let p1 = apply_transform([x + w, y]);
                    let p2 = apply_transform([x + w, y + h]);
                    let p3 = apply_transform([x, y + h]);

                    let uv0 = [u, v];
                    let uv1 = [u + uw, v];
                    let uv2 = [u + uw, v + vh];
                    let uv3 = [u, v + vh];

                    vertices.push(SimpleVertex::from(&crate::vertex::Vertex::textured(
                        p0, uv0, color_arr,
                    )));
                    vertices.push(SimpleVertex::from(&crate::vertex::Vertex::textured(
                        p1, uv1, color_arr,
                    )));
                    vertices.push(SimpleVertex::from(&crate::vertex::Vertex::textured(
                        p2, uv2, color_arr,
                    )));
                    vertices.push(SimpleVertex::from(&crate::vertex::Vertex::textured(
                        p3, uv3, color_arr,
                    )));

                    indices.push(vertex_count);
                    indices.push(vertex_count + 1);
                    indices.push(vertex_count + 2);
                    indices.push(vertex_count);
                    indices.push(vertex_count + 2);
                    indices.push(vertex_count + 3);

                    vertex_count += 4;
                    current_index_count += 6;
                }
                crate::batch::DrawCommand::Circle {
                    center,
                    radius,
                    color,
                    segments,
                    ..
                } => {
                    let (cx, cy) = *center;
                    let radius = *radius;
                    let color_arr = [color.r, color.g, color.b, color.a];
                    let segments = *segments;

                    // Center vertex
                    vertices.push(SimpleVertex::from(&crate::vertex::Vertex {
                        position: [cx, cy],
                        uv: [0.5, 0.5],
                        color: color_arr,
                        params: [0.0, 0.0, 0.0, 0.0],
                        flags: 0,
                    }));

                    let center_index = vertex_count;
                    vertex_count += 1;

                    for i in 0..=segments {
                        let angle = (i as f32 / segments as f32) * 2.0 * std::f32::consts::PI;
                        let x = cx + radius * angle.cos();
                        let y = cy + radius * angle.sin();

                        vertices.push(SimpleVertex::from(&crate::vertex::Vertex {
                            position: [x, y],
                            uv: [0.5 + 0.5 * angle.cos(), 0.5 + 0.5 * angle.sin()],
                            color: color_arr,
                            params: [0.0, 0.0, 0.0, 0.0],
                            flags: 0,
                        }));

                        if i > 0 {
                            indices.push(center_index);
                            indices.push(vertex_count - 1);
                            indices.push(vertex_count);
                            current_index_count += 3;
                        }

                        vertex_count += 1;
                    }
                }
                crate::batch::DrawCommand::Line {
                    start,
                    end,
                    color,
                    thickness,
                    ..
                } => {
                    let (x1, y1) = *start;
                    let (x2, y2) = *end;
                    let thickness = *thickness;
                    let color_arr = [color.r, color.g, color.b, color.a];

                    let dx = x2 - x1;
                    let dy = y2 - y1;
                    let length = (dx * dx + dy * dy).sqrt();

                    if length > 0.0 {
                        let nx = -dy / length * thickness * 0.5;
                        let ny = dx / length * thickness * 0.5;

                        let p0 = [x1 + nx, y1 + ny];
                        let p1 = [x2 + nx, y2 + ny];
                        let p2 = [x2 - nx, y2 - ny];
                        let p3 = [x1 - nx, y1 - ny];

                        vertices.push(SimpleVertex::from(&crate::vertex::Vertex::solid(
                            p0, color_arr,
                        )));
                        vertices.push(SimpleVertex::from(&crate::vertex::Vertex::solid(
                            p1, color_arr,
                        )));
                        vertices.push(SimpleVertex::from(&crate::vertex::Vertex::solid(
                            p2, color_arr,
                        )));
                        vertices.push(SimpleVertex::from(&crate::vertex::Vertex::solid(
                            p3, color_arr,
                        )));

                        indices.push(vertex_count);
                        indices.push(vertex_count + 1);
                        indices.push(vertex_count + 2);
                        indices.push(vertex_count);
                        indices.push(vertex_count + 2);
                        indices.push(vertex_count + 3);

                        vertex_count += 4;
                        current_index_count += 6;
                    }
                }
            }
        }

        // Push final batch
        if current_index_count > 0 {
            batches.push(GPUDrawBatch {
                index_start: current_index_start,
                index_count: current_index_count,
                scissor: get_current_scissor(&scissor_stack),
            });
        }

        // 3. Upload vertices and indices to GPU
        self.buffer_mgr.upload_vertices(
            self.device_mgr.device(),
            self.device_mgr.queue(),
            &vertices,
        );
        self.buffer_mgr
            .upload_indices(self.device_mgr.device(), self.device_mgr.queue(), &indices);

        // 4. Upload projection matrix (orthographic for 2D)
        // Use logical size for projection to handle DPI scaling correctly
        let width = self.surface_mgr.width() as f32;
        let height = self.surface_mgr.height() as f32;

        // Adjust projection for DPI scale factor
        // If scale_factor is 2.0 (Retina), physical width is 2x logical width.
        // We want to use logical coordinates (e.g. 0..400) which map to physical pixels (0..800).
        // So we project 0..width/scale to -1..1.
        let logical_width = width / self.scale_factor;
        let logical_height = height / self.scale_factor;

        let projection = create_orthographic_projection(logical_width, logical_height);
        self.buffer_mgr
            .upload_projection(self.device_mgr.queue(), &projection);

        // 5. Get surface texture
        let surface_texture = self.surface_mgr.get_current_texture()?;
        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // 6. Create command encoder
        let mut encoder =
            self.device_mgr
                .device()
                .create_command_encoder(&CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        // 7. Begin render pass
        {
            let mut render_pass = self.render_pass_mgr.begin(&mut encoder, &view);

            // 8. Set pipeline and bind groups
            render_pass.set_pipeline(self.pipeline_mgr.pipeline());
            render_pass.set_bind_group(0, self.pipeline_mgr.bind_group(), &[]);

            // 9. Set vertex/index buffers
            render_pass.set_vertex_buffer(0, self.buffer_mgr.vertex_buffer().slice(..));
            render_pass.set_index_buffer(
                self.buffer_mgr.index_buffer().slice(..),
                IndexFormat::Uint32,
            );

            // 10. Draw indexed
            for batch in batches {
                if batch.index_count == 0 {
                    continue;
                }

                // Apply scissor
                if let Some(scissor) = batch.scissor {
                    if scissor[2] == 0 || scissor[3] == 0 {
                        continue;
                    }
                    render_pass.set_scissor_rect(scissor[0], scissor[1], scissor[2], scissor[3]);
                } else {
                    render_pass.set_scissor_rect(
                        0,
                        0,
                        self.surface_mgr.width(),
                        self.surface_mgr.height(),
                    );
                }

                render_pass.draw_indexed(
                    batch.index_start..batch.index_start + batch.index_count,
                    0,
                    0..1,
                );
            }
        }

        // 11. Submit command buffer
        self.device_mgr
            .queue()
            .submit(std::iter::once(encoder.finish()));

        // 12. Present surface
        surface_texture.present();

        Ok(())
    }

    /// Resize surface
    pub fn resize(&mut self, width: u32, height: u32) -> anyhow::Result<()> {
        self.surface_mgr
            .resize(width, height, self.device_mgr.device())?;

        // Update projection matrix
        // Use logical size for projection to match render() behavior
        let logical_width = (width as f32) / self.scale_factor;
        let logical_height = (height as f32) / self.scale_factor;

        let projection = create_orthographic_projection(logical_width, logical_height);
        self.buffer_mgr
            .upload_projection(self.device_mgr.queue(), &projection);

        Ok(())
    }
}

/// Create orthographic projection matrix for 2D rendering
fn create_orthographic_projection(width: f32, height: f32) -> [[f32; 4]; 4] {
    // NDC: x: -1 to 1, y: -1 to 1
    // Screen: x: 0 to width, y: 0 to height
    let left = 0.0;
    let right = width;
    let bottom = height;
    let top = 0.0;

    [
        [2.0 / (right - left), 0.0, 0.0, 0.0],
        [0.0, 2.0 / (top - bottom), 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [
            -(right + left) / (right - left),
            -(top + bottom) / (top - bottom),
            0.0,
            1.0,
        ],
    ]
}

/// Convert existing Vertex to SimpleVertex
impl From<&crate::vertex::Vertex> for SimpleVertex {
    fn from(v: &crate::vertex::Vertex) -> Self {
        Self {
            position: v.position,
            color: v.color,
            uv: v.uv, // Use UV from existing Vertex struct
            params: v.params,
            flags: v.flags,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vertex_conversion() {
        let vertex = crate::vertex::Vertex::solid([100.0, 200.0], [1.0, 0.0, 0.0, 1.0]);
        let simple: SimpleVertex = (&vertex).into();

        assert_eq!(simple.position, [100.0, 200.0]);
        assert_eq!(simple.color, [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(simple.uv, vertex.uv);
    }

    #[test]
    fn test_orthographic_projection() {
        let proj = create_orthographic_projection(800.0, 600.0);

        // Top-left corner (0, 0) should map to NDC (-1, 1)
        // Bottom-right (800, 600) should map to NDC (1, -1)

        // Check matrix is not identity
        assert_ne!(proj[0][0], 1.0);
        assert_ne!(proj[1][1], 1.0);
    }
}
