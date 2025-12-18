//! Advanced font management system for StratoUI
//!
//! This module provides a comprehensive font management system with:
//! - Font registration and identification via FontId
//! - Glyph atlas for efficient rendering
//! - Font fallback chains for multi-language support
//! - High-performance text rendering with caching

use cosmic_text::{
    Attrs, Buffer, Family, FontSystem as CosmicFontSystem, Metrics, Shaping, SwashCache, Wrap,
};
use dashmap::DashMap;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use strato_core::layout::Size;
use strato_core::types::{Color, Point};
use wgpu::{Device, Sampler, Texture, TextureView};

/// Unique identifier for registered fonts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FontId(u32);

impl FontId {
    pub const DEFAULT: FontId = FontId(0);
    pub const SYSTEM: FontId = FontId(1);
    pub const MONOSPACE: FontId = FontId(2);
}

/// Comprehensive text style configuration
#[derive(Debug, Clone)]
pub struct TextStyle {
    pub font: FontId,
    pub size: f32,
    pub color: Color,
    pub weight: FontWeight,
    pub style: FontStyle,
    pub line_height: f32,
    pub letter_spacing: f32,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            font: FontId::DEFAULT,
            size: 16.0,
            color: Color::BLACK,
            weight: FontWeight::Normal,
            style: FontStyle::Normal,
            line_height: 1.2,
            letter_spacing: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontWeight {
    Thin = 100,
    ExtraLight = 200,
    Light = 300,
    Normal = 400,
    Medium = 500,
    SemiBold = 600,
    Bold = 700,
    ExtraBold = 800,
    Black = 900,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontStyle {
    Normal,
    Italic,
    Oblique,
}

/// Font registration and metadata
#[derive(Debug, Clone)]
pub struct FontInfo {
    pub id: FontId,
    pub family: String,
    pub fallback_chain: Vec<String>,
    pub supports_cjk: bool,
    pub supports_emoji: bool,
}

/// Glyph atlas for efficient GPU rendering
pub struct GlyphAtlas {
    _texture: Texture,
    texture_view: TextureView,
    sampler: Sampler,
    _size: (u32, u32),
    current_x: u32,
    current_y: u32,
    row_height: u32,
    glyph_cache: DashMap<u64, GlyphInfo>,
    needs_update: bool,
}

#[derive(Debug, Clone)]
struct GlyphInfo {
    _texture_coords: (f32, f32, f32, f32), // (u1, v1, u2, v2)
    _size: (u32, u32),
    _offset: (i32, i32),
}

impl GlyphAtlas {
    pub fn new(device: &Device, size: (u32, u32)) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Glyph Atlas"),
            size: wgpu::Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Glyph Atlas Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            _texture: texture,
            texture_view,
            sampler,
            _size: size,
            current_x: 0,
            current_y: 0,
            row_height: 0,
            glyph_cache: DashMap::new(),
            needs_update: false,
        }
    }

    pub fn texture_view(&self) -> &TextureView {
        &self.texture_view
    }

    pub fn sampler(&self) -> &Sampler {
        &self.sampler
    }

    pub fn needs_update(&self) -> bool {
        self.needs_update
    }

    pub fn mark_updated(&mut self) {
        self.needs_update = false;
    }

    #[allow(dead_code)] // Method is used for glyph atlas management but not in simplified implementation
    fn allocate_space(&mut self, width: u32, height: u32) -> Option<(u32, u32)> {
        // Simple row-based allocation
        if self.current_x + width > self._size.0 {
            // Move to next row
            self.current_x = 0;
            self.current_y += self.row_height;
            self.row_height = 0;
        }

        if self.current_y + height > self._size.1 {
            // Atlas is full
            return None;
        }

        let pos = (self.current_x, self.current_y);
        self.current_x += width;
        self.row_height = self.row_height.max(height);

        Some(pos)
    }
}

/// Advanced font management system
/// Creates a safe font system that loads only specific fonts to avoid problematic system fonts
/// This function has been moved to font_config.rs for centralized configuration
///
/// FontSystem implementation with advanced text rendering capabilities

pub struct FontSystem {
    font_system: Arc<RwLock<CosmicFontSystem>>,
    #[allow(dead_code)] // Field is used for glyph caching but not in simplified implementation
    swash_cache: Arc<RwLock<SwashCache>>,
    glyph_atlas: Arc<RwLock<GlyphAtlas>>,
    fonts: HashMap<FontId, FontInfo>,
    next_font_id: u32,
    text_buffers: DashMap<u64, Buffer>,
}

impl FontSystem {
    pub fn new(device: &Device) -> Self {
        let font_system = crate::font_config::create_safe_font_system();

        // Register default fonts
        let mut fonts = HashMap::new();

        // Default system font with platform-specific fallbacks
        #[cfg(target_os = "windows")]
        let default_font_chain = vec![
            "Segoe UI".to_string(),
            "Segoe UI Emoji".to_string(),
            "Tahoma".to_string(),
            "Arial".to_string(),
            "sans-serif".to_string(),
        ];

        #[cfg(target_os = "macos")]
        let default_font_chain = vec![
            "SF Pro Display".to_string(),
            "Apple Color Emoji".to_string(),
            "Helvetica Neue".to_string(),
            "Arial".to_string(),
            "sans-serif".to_string(),
        ];

        #[cfg(target_os = "linux")]
        let default_font_chain = vec![
            "Ubuntu".to_string(),
            "Noto Color Emoji".to_string(),
            "DejaVu Sans".to_string(),
            "Liberation Sans".to_string(),
            "Arial".to_string(),
            "sans-serif".to_string(),
        ];

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        let default_font_chain = vec!["Arial".to_string(), "sans-serif".to_string()];

        fonts.insert(
            FontId::DEFAULT,
            FontInfo {
                id: FontId::DEFAULT,
                family: default_font_chain[0].clone(),
                fallback_chain: default_font_chain.clone(),
                supports_cjk: false,
                supports_emoji: true,
            },
        );

        // System font with CJK support
        #[cfg(target_os = "windows")]
        let system_font_chain = vec![
            "Segoe UI".to_string(),
            "Segoe UI Emoji".to_string(),
            "Microsoft YaHei".to_string(),
            "SimSun".to_string(),
            "Tahoma".to_string(),
            "Arial".to_string(),
            "sans-serif".to_string(),
        ];

        #[cfg(target_os = "macos")]
        let system_font_chain = vec![
            "SF Pro Display".to_string(),
            "Apple Color Emoji".to_string(),
            "Hiragino Sans".to_string(),
            "PingFang SC".to_string(),
            "Helvetica Neue".to_string(),
            "Arial".to_string(),
            "sans-serif".to_string(),
        ];

        #[cfg(target_os = "linux")]
        let system_font_chain = vec![
            "Ubuntu".to_string(),
            "Noto Color Emoji".to_string(),
            "Noto Sans CJK".to_string(),
            "DejaVu Sans".to_string(),
            "Liberation Sans".to_string(),
            "Arial".to_string(),
            "sans-serif".to_string(),
        ];

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        let system_font_chain = vec!["Arial".to_string(), "sans-serif".to_string()];

        fonts.insert(
            FontId::SYSTEM,
            FontInfo {
                id: FontId::SYSTEM,
                family: system_font_chain[0].clone(),
                fallback_chain: system_font_chain,
                supports_cjk: true,
                supports_emoji: true,
            },
        );

        // Monospace font
        fonts.insert(
            FontId::MONOSPACE,
            FontInfo {
                id: FontId::MONOSPACE,
                family: "monospace".to_string(),
                fallback_chain: vec![
                    "Consolas".to_string(),
                    "Monaco".to_string(),
                    "Courier New".to_string(),
                    "monospace".to_string(),
                ],
                supports_cjk: false,
                supports_emoji: false,
            },
        );

        Self {
            font_system: Arc::new(RwLock::new(font_system)),
            swash_cache: Arc::new(RwLock::new(SwashCache::new())),
            glyph_atlas: Arc::new(RwLock::new(GlyphAtlas::new(device, (1024, 1024)))),
            fonts,
            next_font_id: 3,
            text_buffers: DashMap::new(),
        }
    }

    /// Register a new font family
    pub fn register_font(&mut self, family: String, fallback_chain: Vec<String>) -> FontId {
        let id = FontId(self.next_font_id);
        self.next_font_id += 1;

        let font_info = FontInfo {
            id,
            family,
            fallback_chain,
            supports_cjk: false, // Could be detected automatically
            supports_emoji: false,
        };

        self.fonts.insert(id, font_info);
        id
    }

    /// Get font information
    pub fn get_font_info(&self, font_id: FontId) -> Option<&FontInfo> {
        self.fonts.get(&font_id)
    }

    /// Create text attributes for rendering
    fn create_attrs<'a>(&'a self, style: &'a TextStyle) -> Attrs<'a> {
        let font_info = self
            .fonts
            .get(&style.font)
            .unwrap_or_else(|| &self.fonts[&FontId::DEFAULT]);

        Attrs::new()
            .family(Family::Name(&font_info.family))
            .weight(cosmic_text::Weight(style.weight as u16))
            .style(match style.style {
                FontStyle::Normal => cosmic_text::Style::Normal,
                FontStyle::Italic => cosmic_text::Style::Italic,
                FontStyle::Oblique => cosmic_text::Style::Oblique,
            })
    }

    /// Measure text dimensions
    pub fn measure_text(&self, text: &str, style: &TextStyle, max_width: Option<f32>) -> Size {
        let mut font_system = self.font_system.write();

        let metrics = Metrics::new(style.size, style.size * style.line_height);
        let mut buffer = Buffer::new(&mut font_system, metrics);

        let attrs = self.create_attrs(style);
        buffer.set_text(&mut font_system, text, attrs, Shaping::Advanced);

        if let Some(width) = max_width {
            buffer.set_wrap(&mut font_system, Wrap::Word);
            buffer.set_size(&mut font_system, Some(width), Some(f32::MAX));
        }

        buffer.shape_until_scroll(&mut font_system, false);

        let mut max_width: f32 = 0.0;
        let mut total_height = 0.0;

        for run in buffer.layout_runs() {
            let line_width = run.glyphs.iter().map(|g| g.w).sum::<f32>();
            max_width = max_width.max(line_width);
            total_height += run.line_height;
        }

        Size::new(max_width, total_height)
    }

    /// Render text and return vertex data
    pub fn render_text(
        &self,
        text: &str,
        style: &TextStyle,
        position: Point,
        max_width: Option<f32>,
    ) -> Vec<TextVertex> {
        let mut vertices = Vec::new();
        let mut font_system = self.font_system.write();

        let metrics = Metrics::new(style.size, style.size * style.line_height);
        let mut buffer = Buffer::new(&mut font_system, metrics);

        let attrs = self.create_attrs(style);
        buffer.set_text(&mut font_system, text, attrs, Shaping::Advanced);

        if let Some(width) = max_width {
            buffer.set_wrap(&mut font_system, Wrap::Word);
            buffer.set_size(&mut font_system, Some(width), Some(f32::MAX));
        }

        buffer.shape_until_scroll(&mut font_system, false);

        // Process glyphs and add to atlas if needed
        for run in buffer.layout_runs() {
            for glyph in run.glyphs {
                let physical_glyph = glyph.physical((position.x, position.y), 1.0);
                // Add glyph to atlas and create vertex
                let tex_coords = self.ensure_glyph_in_atlas(&physical_glyph);

                vertices.push(TextVertex {
                    position: [physical_glyph.x as f32, physical_glyph.y as f32],
                    color: [style.color.r, style.color.g, style.color.b, style.color.a],
                    tex_coords,
                });
            }
        }

        vertices
    }

    fn ensure_glyph_in_atlas(&self, _glyph: &cosmic_text::PhysicalGlyph) -> [f32; 2] {
        [0.0, 0.0] // Placeholder texture coordinates
    }

    /// Clear all caches
    pub fn clear_cache(&self) {
        self.text_buffers.clear();
        let mut atlas = self.glyph_atlas.write();
        atlas.glyph_cache.clear();
        atlas.current_x = 0;
        atlas.current_y = 0;
        atlas.row_height = 0;
        atlas.needs_update = true;
    }
}

/// Vertex data for text rendering
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TextVertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
    pub tex_coords: [f32; 2],
}

unsafe impl bytemuck::Pod for TextVertex {}
unsafe impl bytemuck::Zeroable for TextVertex {}

/// High-level text renderer interface
pub struct TextRenderer {
    font_system: FontSystem,
}

impl TextRenderer {
    pub fn new(device: &Device) -> Self {
        Self {
            font_system: FontSystem::new(device),
        }
    }

    /// Draw text with the specified style
    pub fn draw_text(&self, text: &str, style: &TextStyle, position: Point) -> Vec<TextVertex> {
        self.font_system.render_text(text, style, position, None)
    }

    /// Measure text dimensions
    pub fn measure_text(&self, text: &str, style: &TextStyle, max_width: Option<f32>) -> Size {
        self.font_system.measure_text(text, style, max_width)
    }

    /// Register a new font
    pub fn register_font(&mut self, family: String, fallback_chain: Vec<String>) -> FontId {
        self.font_system.register_font(family, fallback_chain)
    }

    /// Get the glyph atlas for GPU binding
    pub fn glyph_atlas(&self) -> &Arc<RwLock<GlyphAtlas>> {
        &self.font_system.glyph_atlas
    }
}
