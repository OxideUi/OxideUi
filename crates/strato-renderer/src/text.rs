//! Text rendering with cosmic-text

use cosmic_text::{
    Attrs, Buffer, Family, FontSystem, Metrics, Shaping,
    Weight, Wrap, SwashCache, CacheKey,
};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use parking_lot::RwLock;
use dashmap::DashMap;
use image::{DynamicImage, ImageBuffer, Rgba};
use crate::glyph_atlas::{GlyphAtlasManager};
use crate::vertex::{Vertex, TextVertex};
use strato_core::types::{Color, Point, Size};

/// Font wrapper
pub struct Font {
    family: Family<'static>,
    pub size: f32,
    weight: u16,
    italic: bool,
}

impl Font {
    /// Create a new font
    pub fn new(family: &str, size: f32) -> Self {
        Self {
            // Family::Name requires a 'static str, so leak the provided family name.
            // This is acceptable for long-lived font definitions.
            family: Family::Name(Box::leak(family.to_string().into_boxed_str())),
            size,
            weight: 400,
            italic: false,
        }
    }

    /// Set font weight
    pub fn with_weight(mut self, weight: u16) -> Self {
        self.weight = weight;
        self
    }

    /// Set italic style
    pub fn with_italic(mut self, italic: bool) -> Self {
        self.italic = italic;
        self
    }

    /// Convert to cosmic-text attributes
    pub fn to_attrs(&self) -> Attrs<'static> {
        Attrs::new()
            .family(self.family.clone())
            .weight(cosmic_text::Weight(self.weight))
            .style(if self.italic {
                cosmic_text::Style::Italic
            } else {
                cosmic_text::Style::Normal
            })
    }
}

impl Default for Font {
    fn default() -> Self {
        // Use platform-specific default fonts
        #[cfg(target_os = "windows")]
        let default_family = "Segoe UI";
        
        #[cfg(target_os = "macos")]
        let default_family = "San Francisco";
        
        #[cfg(target_os = "linux")]
        let default_family = "Ubuntu";
        
        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        let default_family = "sans-serif";
        
        Self::new(default_family, 16.0)
    }
}

/// Glyph cache for efficient text rendering
pub struct GlyphCache {
    #[allow(dead_code)] // Field is used for glyph caching but not in simplified implementation
    cache: SwashCache,
    glyphs: DashMap<u64, CachedGlyph>,
}

/// Cached glyph data
#[allow(dead_code)] // Fields are used for glyph rendering but not in simplified implementation
struct CachedGlyph {
    texture_coords: (f32, f32, f32, f32),
    size: (u32, u32),
    offset: (i32, i32),
}

impl GlyphCache {
    /// Create a new glyph cache
    pub fn new() -> Self {
        Self {
            cache: SwashCache::new(),
            glyphs: DashMap::new(),
        }
    }

    /// Clear the cache
    pub fn clear(&mut self) {
        self.glyphs.clear();
    }
}

/// Text renderer
pub struct TextRenderer {
    font_system: Arc<RwLock<FontSystem>>,
    glyph_cache: Arc<RwLock<GlyphCache>>,
    glyph_atlas_manager: Arc<RwLock<GlyphAtlasManager>>,
    buffers: DashMap<u64, Buffer>,
}

impl TextRenderer {
    /// Create a new text renderer
    pub fn new() -> Self {
        let font_system = crate::font_config::create_safe_font_system();

        Self {
            font_system: Arc::new(RwLock::new(font_system)),
            glyph_cache: Arc::new(RwLock::new(GlyphCache::new())),
            glyph_atlas_manager: Arc::new(RwLock::new(GlyphAtlasManager::new((1024, 1024)))),
            buffers: DashMap::new(),
        }
    }

    /// Render text to vertices for GPU rendering
    pub fn render_text(
        &self,
        text: &str,
        font: &Font,
        position: Point,
        color: Color,
        max_width: Option<f32>,
    ) -> Vec<TextVertex> {
        let mut vertices = Vec::new();
        let mut font_system = self.font_system.write();
        let mut glyph_cache = self.glyph_cache.write();
        let mut glyph_atlas = self.glyph_atlas_manager.write();
        
        // Create buffer for layout
        let metrics = Metrics::new(font.size, font.size * 1.2);
        let mut buffer = Buffer::new(&mut font_system, metrics);
        buffer.set_text(&mut font_system, text, font.to_attrs(), Shaping::Advanced);
        
        if let Some(width) = max_width {
            buffer.set_wrap(&mut font_system, Wrap::Word);
            buffer.set_size(&mut font_system, Some(width), Some(f32::MAX));
        } else {
            buffer.set_size(&mut font_system, None, Some(f32::MAX));
        }
        
        buffer.shape_until_scroll(&mut font_system, false);
        
        let start_x = position.x;
        let start_y = position.y;
        
        // Iterate over layout runs
        for run in buffer.layout_runs() {
            for glyph in run.glyphs.iter() {
                let physical_glyph = glyph.physical((start_x, start_y + run.line_y), 1.0);
                
                // Get texture coordinates from atlas
                if let Some((_atlas_index, glyph_info)) = glyph_atlas.get_or_create_glyph(
                    &mut font_system,
                    &mut glyph_cache.cache,
                    physical_glyph.cache_key
                ) {
                    let glyph_x = physical_glyph.x as f32;
                    let glyph_y = physical_glyph.y as f32;
                    let glyph_w = glyph_info.size.0 as f32;
                    let glyph_h = glyph_info.size.1 as f32;
                    
                    let (u0, v0, u1, v1) = glyph_info.uv_rect;
                    
                    // Create quad vertices for this glyph
                    vertices.extend_from_slice(&[
                        TextVertex::new([glyph_x, glyph_y, 0.0], [u0, v0], [color.r, color.g, color.b, color.a], 0),
                        TextVertex::new([glyph_x + glyph_w, glyph_y, 0.0], [u1, v0], [color.r, color.g, color.b, color.a], 0),
                        TextVertex::new([glyph_x + glyph_w, glyph_y + glyph_h, 0.0], [u1, v1], [color.r, color.g, color.b, color.a], 0),
                        TextVertex::new([glyph_x, glyph_y + glyph_h, 0.0], [u0, v1], [color.r, color.g, color.b, color.a], 0),
                    ]);
                }
            }
        }
        
        vertices
    }

    /// Measure text dimensions
    pub fn measure_text(&self, text: &str, font: &Font, max_width: Option<f32>) -> Size {
        let mut font_system = self.font_system.write();
        
        // Create buffer for measurement
        let metrics = Metrics::new(font.size, font.size * 1.2);
        let mut buffer = Buffer::new(&mut font_system, metrics);
        buffer.set_text(&mut font_system, text, font.to_attrs(), Shaping::Advanced);
        
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

    /// Hash text and font for caching
    #[allow(dead_code)] // Used for text caching but not in simplified implementation
    fn hash_text(text: &str, font: &Font) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        font.size.to_bits().hash(&mut hasher);
        font.weight.hash(&mut hasher);
        font.italic.hash(&mut hasher);
        hasher.finish()
    }

    /// Clear all caches
    pub fn clear_cache(&self) {
        self.buffers.clear();
        self.glyph_cache.write().clear();
    }

    /// Rasterize text to an image using cosmic-text
    pub fn rasterize_text_image(
        &self,
        text: &str,
        font: &Font,
        max_width: Option<f32>,
    ) -> DynamicImage {
        let mut font_system = self.font_system.write();
        let mut swash_cache = SwashCache::new();
        
        // Create buffer for text layout
        let metrics = Metrics::new(font.size, font.size * 1.2);
        let mut buffer = Buffer::new(&mut font_system, metrics);
        buffer.set_text(&mut font_system, text, font.to_attrs(), Shaping::Advanced);
        
        if let Some(width) = max_width {
            buffer.set_wrap(&mut font_system, cosmic_text::Wrap::Word);
            buffer.set_size(&mut font_system, Some(width), Some(f32::MAX));
        }
        
        buffer.shape_until_scroll(&mut font_system, false);
        
        // Calculate dimensions
        let size = self.measure_text(text, font, max_width);
        let width = size.width.ceil().max(1.0) as u32;
        let height = size.height.ceil().max(1.0) as u32;
        
        // Create pixel buffer
        let mut img = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(width, height);
        
        // Render each glyph
        for run in buffer.layout_runs() {
            for glyph in run.glyphs {
                let physical_glyph = glyph.physical((0.0, 0.0), 1.0);
                
                swash_cache.with_pixels(
                    &mut font_system,
                    physical_glyph.cache_key,
                    cosmic_text::Color::rgba(255, 255, 255, 255),
                    |x, y, color| {
                        let px = (physical_glyph.x as i32 + x) as u32;
                        let py = (physical_glyph.y as i32 + y) as u32;
                        
                        if px < width && py < height {
                            let pixel = img.get_pixel_mut(px, py);
                            pixel[0] = color.r();
                            pixel[1] = color.g();
                            pixel[2] = color.b();
                            pixel[3] = color.a();
                        }
                    },
                );
            }
        }
        
        DynamicImage::ImageRgba8(img)
    }
}

impl Default for TextRenderer {
    fn default() -> Self {
        Self::new()
    }
}

/// Text vertex for GPU rendering
#[repr(C)]
#[derive(Debug, Clone)]
pub struct LocalTextVertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
    pub tex_coords: [f32; 2],
}

impl LocalTextVertex {
    pub fn new(position: [f32; 2], color: Color, tex_coords: [f32; 2]) -> Self {
        Self {
            position,
            color: color.to_array(),
            tex_coords,
        }
    }
}

/// Text layout options
#[derive(Debug, Clone)]
pub struct TextLayout {
    pub align: TextAlign,
    pub wrap: TextWrap,
    pub line_spacing: f32,
    pub letter_spacing: f32,
}

/// Text alignment
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAlign {
    Left,
    Center,
    Right,
    Justify,
}

/// Text wrapping mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextWrap {
    None,
    Word,
    Character,
}

impl Default for TextLayout {
    fn default() -> Self {
        Self {
            align: TextAlign::Left,
            wrap: TextWrap::Word,
            line_spacing: 1.2,
            letter_spacing: 0.0,
        }
    }
}
