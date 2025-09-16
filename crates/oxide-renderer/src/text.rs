//! Text rendering with cosmic-text

use cosmic_text::{
    Attrs, Buffer, Family, FontSystem, Metrics, Shaping, SwashCache, Wrap,
};
use image::{DynamicImage, ImageBuffer, Rgba};
use oxide_core::types::{Color, Point};
use oxide_core::layout::Size;
use parking_lot::RwLock;
use std::sync::Arc;
use dashmap::DashMap;

/// Font wrapper
pub struct Font {
    family: Family<'static>,
    size: f32,
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
    fn to_attrs(&self) -> Attrs<'static> {
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
        Self::new("sans-serif", 16.0)
    }
}

/// Glyph cache for efficient text rendering
pub struct GlyphCache {
    cache: SwashCache,
    glyphs: DashMap<u64, CachedGlyph>,
}

/// Cached glyph data
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
    buffers: DashMap<u64, Buffer>,
}

impl TextRenderer {
    /// Create a new text renderer
    pub fn new() -> Self {
        let mut font_system = FontSystem::new();
        
        // Load system fonts
        #[cfg(target_os = "windows")]
        {
            font_system.db_mut().load_system_fonts();
        }
        #[cfg(target_os = "macos")]
        {
            font_system.db_mut().load_system_fonts();
        }
        #[cfg(target_os = "linux")]
        {
            font_system.db_mut().load_system_fonts();
        }

        Self {
            font_system: Arc::new(RwLock::new(font_system)),
            glyph_cache: Arc::new(RwLock::new(GlyphCache::new())),
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
        let mut font_system = self.font_system.write();
        let mut vertices = Vec::new();
        
        // Create buffer for text layout
        let metrics = Metrics::new(font.size, font.size * 1.2);
        let mut buffer = Buffer::new(&mut font_system, metrics);
        buffer.set_text(&mut font_system, text, font.to_attrs(), Shaping::Advanced);
        
        if let Some(width) = max_width {
            buffer.set_wrap(&mut font_system, Wrap::Word);
            buffer.set_size(&mut font_system, Some(width), Some(f32::MAX));
        }
        
        buffer.shape_until_scroll(&mut font_system, false);
        
        // Generate vertices for each glyph
        for run in buffer.layout_runs() {
            for glyph in run.glyphs {
                let physical_glyph = glyph.physical((position.x, position.y), 1.0);
                
                // Create quad vertices for this glyph
                let x = physical_glyph.x;
                let y = physical_glyph.y;
                let w = glyph.w;
                let h = run.line_height;
                
                // Texture coordinates (will be set by glyph cache)
                let u0 = 0.0;
                let v0 = 0.0;
                let u1 = 1.0;
                let v1 = 1.0;
                
                // Create quad (two triangles)
                vertices.extend_from_slice(&[
                    TextVertex::new([x as f32, y as f32], color, [u0, v0]),
                    TextVertex::new([x as f32 + w, y as f32], color, [u1, v0]),
                    TextVertex::new([x as f32 + w, y as f32 + h], color, [u1, v1]),
                    TextVertex::new([x as f32, y as f32 + h], color, [u0, v1]),
                ]);
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
pub struct TextVertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
    pub tex_coords: [f32; 2],
}

impl TextVertex {
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
