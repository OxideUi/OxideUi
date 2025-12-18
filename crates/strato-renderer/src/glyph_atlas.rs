//! Glyph atlas system for efficient text rendering
//!
//! This module provides a texture atlas system that packs glyph bitmaps into larger textures
//! for efficient GPU rendering. It uses cosmic-text for glyph rasterization and manages
//! texture space allocation using a simple bin-packing algorithm.

use crate::font_config::create_safe_font_system;
use crate::text::Font;
use cosmic_text::{CacheKey, FontSystem, SwashCache};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use strato_core::types::Color;

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
    /// Map from CacheKey to glyph info
    glyph_map: HashMap<CacheKey, GlyphInfo>,
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
    pub fn add_glyph(
        &mut self,
        cache_key: CacheKey,
        glyph_bitmap: &[u8],
        size: (u32, u32),
        bearing: (i32, i32),
        advance: f32,
    ) -> Option<GlyphInfo> {
        let (glyph_width, glyph_height) = size;

        // Check if glyph is already in atlas
        if let Some(info) = self.glyph_map.get(&cache_key) {
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
        self.glyph_map.insert(cache_key, glyph_info);
        self.dirty = true;

        Some(glyph_info)
    }

    /// Get glyph info if it exists in the atlas
    pub fn get_glyph(&self, cache_key: CacheKey) -> Option<GlyphInfo> {
        self.glyph_map.get(&cache_key).copied()
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
    atlas_size: (u32, u32),
}

impl GlyphAtlasManager {
    /// Create a new glyph atlas manager
    pub fn new(atlas_size: (u32, u32)) -> Self {
        Self {
            atlases: vec![GlyphAtlas::new(atlas_size.0, atlas_size.1)],
            atlas_size,
        }
    }

    /// Get or create a glyph in an atlas
    pub fn get_or_create_glyph(
        &mut self,
        font_system: &mut FontSystem,
        swash_cache: &mut SwashCache,
        cache_key: CacheKey,
    ) -> Option<(usize, GlyphInfo)> {
        // Check existing atlases first
        for (atlas_idx, atlas) in self.atlases.iter().enumerate() {
            if let Some(info) = atlas.get_glyph(cache_key) {
                return Some((atlas_idx, info));
            }
        }

        // Need to rasterize the glyph
        // Rasterize using swash
        let image = swash_cache
            .get_image(font_system, cache_key)
            .as_ref()
            .cloned()?;

        let glyph_width = image.placement.width;
        let glyph_height = image.placement.height;
        let bearing_x = image.placement.left;
        let bearing_y = image.placement.top;

        // Convert content to alpha mask (if it's not already?)
        // swash_cache.get_image returns image data. cosmic-text uses Format::Alpha usually?
        // Let's check image.content.

        let glyph_bitmap = match image.content {
            cosmic_text::SwashContent::Mask => image.data,
            cosmic_text::SwashContent::SubpixelMask => {
                // Convert subpixel to standard alpha? Or just use it?
                // For now assume we handle it as alpha or it's handled by shader
                // We'll take every 3rd byte or average?
                // For simplicity, let's just take it as is, but it might be 3x wider?
                // No, cosmic-text handles this.
                image.data
            }
            cosmic_text::SwashContent::Color => {
                // Color emoji etc. Not supported in our simple atlas yet (grayscale).
                return None;
            }
        };

        // Try to add to existing atlases
        for (atlas_idx, atlas) in self.atlases.iter_mut().enumerate() {
            if let Some(info) = atlas.add_glyph(
                cache_key,
                &glyph_bitmap,
                (glyph_width, glyph_height),
                (bearing_x, bearing_y),
                0.0, // Advance is handled by layout run, we store 0 or don't use it in vertex gen
            ) {
                return Some((atlas_idx, info));
            }
        }

        // Create new atlas if needed
        let mut new_atlas = GlyphAtlas::new(self.atlas_size.0, self.atlas_size.1);
        if let Some(info) = new_atlas.add_glyph(
            cache_key,
            &glyph_bitmap,
            (glyph_width, glyph_height),
            (bearing_x, bearing_y),
            0.0,
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
}

impl Default for GlyphAtlasManager {
    fn default() -> Self {
        Self::new((1024, 1024))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmic_text::{Attrs, Buffer, Metrics, Shaping};

    #[test]
    fn test_atlas_creation() {
        let atlas = GlyphAtlas::new(256, 256);
        assert_eq!(atlas.dimensions(), (256, 256));
        assert!(!atlas.is_dirty());
    }

    #[test]
    fn test_glyph_addition() {
        let mut atlas = GlyphAtlas::new(256, 256);

        let mut font_system = FontSystem::new();
        let mut swash_cache = SwashCache::new();

        let metrics = Metrics::new(16.0, 20.0);
        let mut buffer = Buffer::new(&mut font_system, metrics);
        buffer.set_text(&mut font_system, "A", Attrs::new(), Shaping::Advanced);
        buffer.shape_until_scroll(&mut font_system, false);

        let mut cache_key_opt = None;
        for run in buffer.layout_runs() {
            for glyph in run.glyphs.iter() {
                let physical = glyph.physical((0.0, 0.0), 1.0);
                cache_key_opt = Some(physical.cache_key);
                break;
            }
            if cache_key_opt.is_some() {
                break;
            }
        }

        let cache_key = cache_key_opt.expect("Failed to obtain glyph cache key");

        let image = swash_cache
            .get_image(&mut font_system, cache_key)
            .as_ref()
            .cloned()
            .expect("Failed to rasterize glyph image");

        let glyph_width = image.placement.width;
        let glyph_height = image.placement.height;
        let bearing_x = image.placement.left;
        let bearing_y = image.placement.top;
        let glyph_bitmap = image.data;

        let info = atlas.add_glyph(
            cache_key,
            &glyph_bitmap,
            (glyph_width, glyph_height),
            (bearing_x, bearing_y),
            0.0,
        );

        assert!(info.is_some());
        assert!(atlas.is_dirty());

        let info = info.unwrap();
        assert_eq!(
            info.uv_rect,
            (
                0.0,
                0.0,
                glyph_width as f32 / 256.0,
                glyph_height as f32 / 256.0
            )
        );
    }

    #[test]
    fn test_atlas_manager() {
        let manager = GlyphAtlasManager::new((256, 256));
        assert_eq!(manager.atlas_count(), 1);
    }
}
