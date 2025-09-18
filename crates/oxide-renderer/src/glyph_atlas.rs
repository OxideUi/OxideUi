//! Glyph atlas system for efficient text rendering
//!
//! This module provides a texture atlas system that packs glyph bitmaps into larger textures
//! for efficient GPU rendering. It uses cosmic-text for glyph rasterization and manages
//! texture space allocation using a simple bin-packing algorithm.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use cosmic_text::{FontSystem, SwashCache};
use image::{DynamicImage, ImageBuffer, Rgba};
use oxide_core::types::{Color, Point, Size};
use crate::text::Font;

/// Represents a glyph in the atlas
#[derive(Debug, Clone, Copy)]
pub struct GlyphInfo {
    /// UV coordinates in the atlas texture (normalized 0.0-1.0)
    pub uv_rect: (f32, f32, f32, f32), // (u_min, v_min, u_max, v_max)
    /// Size of the glyph in pixels
    pub size: (u32, u32),
    /// Offset for positioning the glyph
    pub bearing: (i32, i32),
    /// How much to advance after this glyph
    pub advance: f32,
}

/// A texture atlas that contains multiple glyphs
pub struct GlyphAtlas {
    /// The atlas texture data (grayscale)
    texture_data: Vec<u8>,
    /// Width and height of the atlas texture
    dimensions: (u32, u32),
    /// Current allocation position (simple top-to-bottom, left-to-right packing)
    current_row_y: u32,
    current_x: u32,
    current_row_height: u32,
    /// Map from (font_id, glyph_id) to glyph info
    glyph_map: HashMap<(u32, u32), GlyphInfo>,
    /// Whether the atlas has been updated and needs to be uploaded to GPU
    dirty: bool,
}

impl GlyphAtlas {
    /// Create a new glyph atlas
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            texture_data: vec![0; (width * height) as usize],
            dimensions: (width, height),
            current_row_y: 0,
            current_x: 0,
            current_row_height: 0,
            glyph_map: HashMap::new(),
            dirty: false,
        }
    }

    /// Add a glyph to the atlas
    pub fn add_glyph(&mut self, font_id: u32, glyph_id: u32, glyph_bitmap: &[u8], size: (u32, u32), bearing: (i32, i32), advance: f32) -> Option<GlyphInfo> {
        let (glyph_width, glyph_height) = size;
        
        // Check if glyph is already in atlas
        if let Some(info) = self.glyph_map.get(&(font_id, glyph_id)) {
            return Some(*info);
        }
        
        // Check if we have space in current row
        if self.current_x + glyph_width > self.dimensions.0 {
            // Move to next row
            self.current_row_y += self.current_row_height;
            self.current_x = 0;
            self.current_row_height = 0;
        }
        
        // Check if we have vertical space
        if self.current_row_y + glyph_height > self.dimensions.1 {
            // Atlas is full
            return None;
        }
        
        // Copy glyph data to atlas
        let atlas_x = self.current_x;
        let atlas_y = self.current_row_y;
        
        for y in 0..glyph_height {
            for x in 0..glyph_width {
                let src_idx = (y * glyph_width + x) as usize;
                let dst_idx = ((atlas_y + y) * self.dimensions.0 + (atlas_x + x)) as usize;
                if src_idx < glyph_bitmap.len() && dst_idx < self.texture_data.len() {
                    self.texture_data[dst_idx] = glyph_bitmap[src_idx];
                }
            }
        }
        
        // Calculate UV coordinates
        let u_min = atlas_x as f32 / self.dimensions.0 as f32;
        let v_min = atlas_y as f32 / self.dimensions.1 as f32;
        let u_max = (atlas_x + glyph_width) as f32 / self.dimensions.0 as f32;
        let v_max = (atlas_y + glyph_height) as f32 / self.dimensions.1 as f32;
        
        let glyph_info = GlyphInfo {
            uv_rect: (u_min, v_min, u_max, v_max),
            size,
            bearing,
            advance,
        };
        
        // Update atlas state
        self.current_x += glyph_width;
        self.current_row_height = self.current_row_height.max(glyph_height);
        self.glyph_map.insert((font_id, glyph_id), glyph_info);
        self.dirty = true;
        
        Some(glyph_info)
    }

    /// Get glyph info if it exists in the atlas
    pub fn get_glyph(&self, font_id: u32, glyph_id: u32) -> Option<GlyphInfo> {
        self.glyph_map.get(&(font_id, glyph_id)).copied()
    }

    /// Get the atlas texture data
    pub fn texture_data(&self) -> &[u8] {
        &self.texture_data
    }

    /// Get atlas dimensions
    pub fn dimensions(&self) -> (u32, u32) {
        self.dimensions
    }

    /// Check if atlas needs to be uploaded to GPU
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Mark atlas as clean (after GPU upload)
    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }

    /// Clear the atlas
    pub fn clear(&mut self) {
        self.texture_data.fill(0);
        self.current_row_y = 0;
        self.current_x = 0;
        self.current_row_height = 0;
        self.glyph_map.clear();
        self.dirty = true;
    }

    /// Get usage statistics
    pub fn get_usage_stats(&self) -> (f32, u32) {
        let used_pixels = self.current_row_y * self.dimensions.0 + self.current_x;
        let total_pixels = self.dimensions.0 * self.dimensions.1;
        let usage_percentage = used_pixels as f32 / total_pixels as f32 * 100.0;
        (usage_percentage, self.glyph_map.len() as u32)
    }
}

/// Manager for multiple glyph atlases
pub struct GlyphAtlasManager {
    atlases: Vec<GlyphAtlas>,
    font_system: Arc<Mutex<FontSystem>>,
    swash_cache: Arc<Mutex<SwashCache>>,
    atlas_size: (u32, u32),
}

impl GlyphAtlasManager {
    /// Create a new glyph atlas manager
    pub fn new(atlas_size: (u32, u32)) -> Self {
        let font_system = Arc::new(Mutex::new(FontSystem::new()));
        let swash_cache = Arc::new(Mutex::new(SwashCache::new()));
        
        Self {
            atlases: vec![GlyphAtlas::new(atlas_size.0, atlas_size.1)],
            font_system,
            swash_cache,
            atlas_size,
        }
    }

    /// Get or create a glyph in an atlas
    pub fn get_or_create_glyph(&mut self, font: &Font, character: char) -> Option<(usize, GlyphInfo)> {
        let font_id = self.get_font_id(font);
        let glyph_id = character as u32;

        // Check existing atlases first
        for (atlas_idx, atlas) in self.atlases.iter().enumerate() {
            if let Some(info) = atlas.get_glyph(font_id, glyph_id) {
                return Some((atlas_idx, info));
            }
        }

        // Need to rasterize the glyph
        let glyph_bitmap = self.rasterize_glyph(font, character)?;
        
        // Try to add to existing atlases
        for (atlas_idx, atlas) in self.atlases.iter_mut().enumerate() {
            if let Some(info) = atlas.add_glyph(
                font_id, 
                glyph_id, 
                &glyph_bitmap.data, 
                (glyph_bitmap.width, glyph_bitmap.height),
                glyph_bitmap.bearing,
                glyph_bitmap.advance
            ) {
                return Some((atlas_idx, info));
            }
        }

        // Create new atlas if needed
        let mut new_atlas = GlyphAtlas::new(self.atlas_size.0, self.atlas_size.1);
        if let Some(info) = new_atlas.add_glyph(
            font_id, 
            glyph_id, 
            &glyph_bitmap.data, 
            (glyph_bitmap.width, glyph_bitmap.height),
            glyph_bitmap.bearing,
            glyph_bitmap.advance
        ) {
            let atlas_idx = self.atlases.len();
            self.atlases.push(new_atlas);
            return Some((atlas_idx, info));
        }

        None
    }

    /// Get an atlas by index
    pub fn get_atlas(&self, index: usize) -> Option<&GlyphAtlas> {
        self.atlases.get(index)
    }

    /// Get a mutable atlas by index
    pub fn get_atlas_mut(&mut self, index: usize) -> Option<&mut GlyphAtlas> {
        self.atlases.get_mut(index)
    }

    /// Get the number of atlases
    pub fn atlas_count(&self) -> usize {
        self.atlases.len()
    }

    /// Simple font ID generation (in a real system, this would be more sophisticated)
    fn get_font_id(&self, font: &Font) -> u32 {
        // For now, just use a hash of the font family and size
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        // Note: this is a simplified approach - in reality we'd need to access the font family
        font.size.to_bits().hash(&mut hasher);
        hasher.finish() as u32
    }

    /// Rasterize a glyph using cosmic-text
    fn rasterize_glyph(&self, font: &Font, character: char) -> Option<RasterizedGlyph> {
        use cosmic_text::{
            Attrs, Buffer, Family, Metrics, Shaping,
        };
        use oxide_core::{oxide_trace, logging::LogCategory};
        
        oxide_trace!(LogCategory::Text, "Rasterizing glyph '{}' with font size {}", character, font.size);
        
        let mut font_system = self.font_system.lock().unwrap();
        let mut swash_cache = self.swash_cache.lock().unwrap();
        
        // Create buffer for this single character
        let metrics = Metrics::new(font.size, font.size * 1.2);
        let mut buffer = Buffer::new(&mut font_system, metrics);
        
        let attrs = font.to_attrs();
        let text = character.to_string();
        
        buffer.set_text(&mut font_system, &text, attrs, Shaping::Advanced);
        buffer.shape_until_scroll(&mut font_system, false);
        
        // Get the first (and only) glyph
        for run in buffer.layout_runs() {
            if let Some(glyph) = run.glyphs.first() {
                let physical_glyph = glyph.physical((0.0, 0.0), 1.0);
                
                // Try to get glyph bitmap from swash cache
                let mut glyph_data: Vec<u8> = Vec::new();
                let (width, height) = (glyph.w as u32, run.line_height as u32);
                
                // Initialize with zeros
                glyph_data.resize((width * height) as usize, 0);
                
                // Use swash to render the glyph
                swash_cache.with_pixels(
                    &mut font_system,
                    physical_glyph.cache_key,
                    cosmic_text::Color::rgba(255, 255, 255, 255),
                    |x, y, color| {
                        if x < width as i32 && y < height as i32 {
                            let idx = (y * width as i32 + x) as usize;
                            if idx < glyph_data.len() {
                                glyph_data[idx] = color.a();
                            }
                        }
                    },
                );
                
                oxide_trace!(LogCategory::Text, "Rasterized glyph '{}': {}x{} pixels", character, width, height);
                
                return Some(RasterizedGlyph {
                    data: glyph_data,
                    width,
                    height,
                    bearing: (physical_glyph.x as i32, physical_glyph.y as i32),
                    advance: glyph.w,
                });
            }
        }
        
        oxide_trace!(LogCategory::Text, "Failed to rasterize glyph '{}'", character);
        None
    }
}

/// A rasterized glyph bitmap
struct RasterizedGlyph {
    data: Vec<u8>,
    width: u32,
    height: u32,
    bearing: (i32, i32),
    advance: f32,
}

impl Default for GlyphAtlasManager {
    fn default() -> Self {
        Self::new((1024, 1024))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atlas_creation() {
        let atlas = GlyphAtlas::new(256, 256);
        assert_eq!(atlas.dimensions(), (256, 256));
        assert!(!atlas.is_dirty());
    }

    #[test]
    fn test_glyph_addition() {
        let mut atlas = GlyphAtlas::new(256, 256);
        let bitmap = vec![255u8; 16 * 16]; // 16x16 white square
        
        let info = atlas.add_glyph(1, 65, &bitmap, (16, 16), (0, 0), 16.0);
        assert!(info.is_some());
        assert!(atlas.is_dirty());
        
        // Check UV coordinates
        let info = info.unwrap();
        assert_eq!(info.uv_rect, (0.0, 0.0, 16.0/256.0, 16.0/256.0));
    }

    #[test]
    fn test_atlas_manager() {
        let manager = GlyphAtlasManager::new((256, 256));
        assert_eq!(manager.atlas_count(), 1);
    }
}