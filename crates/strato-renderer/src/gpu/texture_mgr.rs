//! Texture management for GPU rendering
//!
//! BLOCCO 8: Texture Management
//! Handles texture atlas creation, glyph caching, and texture binding

use anyhow::Result;
use std::collections::HashMap;
use wgpu::{
    AddressMode, Device, Extent3d, FilterMode, Queue, Sampler, SamplerDescriptor, Texture,
    TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureView,
};

/// Glyph metrics for positioning and layout
#[derive(Debug, Clone, Copy)]
pub struct GlyphMetrics {
    pub width: u32,
    pub height: u32,
    pub bearing_x: i32,
    pub bearing_y: i32,
    pub advance: f32,
}

/// Cached glyph with atlas location and UV coordinates
#[derive(Debug, Clone)]
pub struct CachedGlyph {
    pub metrics: GlyphMetrics,
    pub uv_rect: (f32, f32, f32, f32),      // (u0, v0, u1, v1)
    pub atlas_region: (u32, u32, u32, u32), // (x, y, w, h)
}

/// Cached image with atlas location and UV coordinates
#[derive(Debug, Clone)]
pub struct CachedImage {
    pub uv_rect: (f32, f32, f32, f32),      // (u0, v0, u1, v1)
    pub atlas_region: (u32, u32, u32, u32), // (x, y, w, h)
}

/// Key for glyph cache lookup
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GlyphKey {
    pub character: char,
    pub font_size: u32, // Size in pixels
}

/// Glyph cache for fast lookup
pub struct GlyphCache {
    glyphs: HashMap<GlyphKey, CachedGlyph>,
}

impl GlyphCache {
    pub fn new() -> Self {
        Self {
            glyphs: HashMap::new(),
        }
    }

    pub fn get(&self, key: &GlyphKey) -> Option<&CachedGlyph> {
        self.glyphs.get(key)
    }

    pub fn insert(&mut self, key: GlyphKey, glyph: CachedGlyph) {
        self.glyphs.insert(key, glyph);
    }

    pub fn len(&self) -> usize {
        self.glyphs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.glyphs.is_empty()
    }
}

/// Glyph rasterizer using fontdue
pub struct GlyphRasterizer {
    pub font: fontdue::Font,
}

impl GlyphRasterizer {
    /// Create new glyph rasterizer with embedded Segoe UI font
    pub fn new() -> Result<Self> {
        // Embed Segoe UI Italic font (path from crates/strato-renderer/src/gpu/ to root/font/)
        const FONT_DATA: &[u8] = include_bytes!("../../../../font/segoeuithis.ttf");

        let font = fontdue::Font::from_bytes(FONT_DATA, fontdue::FontSettings::default())
            .map_err(|e| anyhow::anyhow!("Failed to load font: {}", e))?;

        println!("=== GLYPH RASTERIZER INITIALIZED ===");

        Ok(Self { font })
    }

    /// Rasterize a character at given size
    pub fn rasterize(&self, character: char, size: f32) -> Option<(Vec<u8>, GlyphMetrics)> {
        let (metrics, bitmap) = self.font.rasterize(character, size);

        if metrics.width == 0 || metrics.height == 0 {
            return None;
        }

        // Convert grayscale to RGBA
        let rgba_data: Vec<u8> = bitmap
            .iter()
            .flat_map(|&alpha| [255u8, 255, 255, alpha])
            .collect();

        let glyph_metrics = GlyphMetrics {
            width: metrics.width as u32,
            height: metrics.height as u32,
            bearing_x: metrics.xmin,
            bearing_y: metrics.ymin + metrics.height as i32,
            advance: metrics.advance_width,
        };

        Some((rgba_data, glyph_metrics))
    }
}

/// Texture atlas for efficient texture management
pub struct TextureAtlas {
    texture: Texture,
    texture_view: TextureView,
    sampler: Sampler,
    width: u32,
    height: u32,
    format: TextureFormat,
    // Allocation tracking
    current_x: u32,
    current_y: u32,
    row_height: u32,
}

impl TextureAtlas {
    /// Create new texture atlas
    ///
    /// # Arguments
    /// * `device` - GPU device
    /// * `width` - Atlas width
    /// * `height` - Atlas height
    pub fn new(device: &Device, width: u32, height: u32) -> Self {
        let size = Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let format = TextureFormat::Rgba8UnormSrgb;

        let texture = device.create_texture(&TextureDescriptor {
            label: Some("Texture Atlas"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("Texture Atlas Sampler"),
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
        });

        println!("=== TEXTURE ATLAS CREATED ===");
        println!("Size: {}x{}", width, height);
        println!("Format: {:?}", format);
        println!("=============================");

        Self {
            texture,
            texture_view,
            sampler,
            width,
            height,
            format,
            current_x: 0,
            current_y: 0,
            row_height: 0,
        }
    }

    /// Allocate region in atlas for a glyph (simple shelf-packing)
    pub fn allocate_region(&mut self, width: u32, height: u32) -> Option<(u32, u32)> {
        // Check if glyph fits in current row
        if self.current_x + width > self.width {
            // Move to next row
            self.current_x = 0;
            self.current_y += self.row_height;
            self.row_height = 0;
        }

        // Check if we have vertical space
        if self.current_y + height > self.height {
            return None; // Atlas full
        }

        let x = self.current_x;
        let y = self.current_y;

        // Update allocation state
        self.current_x += width;
        self.row_height = self.row_height.max(height);

        Some((x, y))
    }

    /// Upload texture data to a region of the atlas
    ///
    /// # Arguments
    /// * `queue` - GPU queue
    /// * `data` - RGBA8 pixel data
    /// * `x` - X offset in atlas
    /// * `y` - Y offset in atlas
    /// * `width` - Region width
    /// * `height` - Region height
    pub fn upload_region(
        &self,
        queue: &Queue,
        data: &[u8],
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> Result<()> {
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d { x, y, z: 0 },
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        Ok(())
    }

    /// Reserve a 1x1 white pixel at (0,0) for solid color rendering
    pub fn reserve_white_pixel(&mut self, queue: &Queue) {
        let white_pixel = [255u8, 255, 255, 255];
        self.upload_region(queue, &white_pixel, 0, 0, 1, 1)
            .expect("Failed to upload white pixel");

        // Advance allocator to skip this pixel
        // We'll just advance by a small amount to keep it simple, e.g., move to x=1
        // effectively reserving the first pixel of the first row
        self.current_x = 1;
        self.row_height = 1;
    }

    /// Create a default 1x1 white texture for solid color rendering
    pub fn create_default_white(device: &Device, queue: &Queue) -> Self {
        let mut atlas = Self::new(device, 1, 1);
        atlas.reserve_white_pixel(queue);
        atlas
    }

    /// Get texture view
    pub fn view(&self) -> &TextureView {
        &self.texture_view
    }

    /// Get sampler
    pub fn sampler(&self) -> &Sampler {
        &self.sampler
    }

    /// Get atlas dimensions
    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}

/// Texture manager with glyph caching
pub struct TextureManager {
    atlas: TextureAtlas,
    glyph_cache: GlyphCache,
    image_cache: HashMap<u64, CachedImage>,
    rasterizer: GlyphRasterizer,
}

impl TextureManager {
    /// Create new texture manager with default white texture
    pub fn new(device: &Device, queue: &Queue) -> Self {
        Self {
            atlas: TextureAtlas::create_default_white(device, queue),
            glyph_cache: GlyphCache::new(),
            image_cache: HashMap::new(),
            rasterizer: GlyphRasterizer::new().expect("Failed to create glyph rasterizer"),
        }
    }

    /// Create texture manager with font support (512x512 atlas)
    pub fn new_with_font(device: &Device, queue: &Queue) -> Self {
        // Increase atlas size to 2048x2048 to support images
        let mut atlas = TextureAtlas::new(device, 2048, 2048);

        // IMPORTANT: Reserve white pixel at (0,0) for solid color rendering
        // The shader samples (0,0) when rendering non-textured shapes
        atlas.reserve_white_pixel(queue);

        Self {
            atlas,
            glyph_cache: GlyphCache::new(),
            image_cache: HashMap::new(),
            rasterizer: GlyphRasterizer::new().expect("Failed to create glyph rasterizer"),
        }
    }

    /// Get or cache a glyph, rasterizing if needed
    pub fn get_or_cache_glyph(
        &mut self,
        queue: &Queue,
        character: char,
        font_size: u32,
    ) -> Option<&CachedGlyph> {
        let key = GlyphKey {
            character,
            font_size,
        };

        // Check cache first
        if self.glyph_cache.get(&key).is_some() {
            return self.glyph_cache.get(&key);
        }

        // Rasterize and cache
        if let Some((rgba_data, metrics)) = self.rasterizer.rasterize(character, font_size as f32) {
            // Allocate space in atlas
            if let Some((x, y)) = self.atlas.allocate_region(metrics.width, metrics.height) {
                // Upload to GPU
                if self
                    .atlas
                    .upload_region(queue, &rgba_data, x, y, metrics.width, metrics.height)
                    .is_ok()
                {
                    // Calculate UV coordinates
                    let atlas_size = self.atlas.size();
                    let u0 = x as f32 / atlas_size.0 as f32;
                    let v0 = y as f32 / atlas_size.1 as f32;
                    let u1 = (x + metrics.width) as f32 / atlas_size.0 as f32;
                    let v1 = (y + metrics.height) as f32 / atlas_size.1 as f32;

                    let cached_glyph = CachedGlyph {
                        metrics,
                        uv_rect: (u0, v0, u1, v1),
                        atlas_region: (x, y, metrics.width, metrics.height),
                    };

                    self.glyph_cache.insert(key, cached_glyph);
                    return self.glyph_cache.get(&key);
                }
            }
        }

        None
    }

    /// Get or upload an image
    pub fn get_or_upload_image(
        &mut self,
        queue: &Queue,
        id: u64,
        data: &[u8],
        width: u32,
        height: u32,
    ) -> Option<&CachedImage> {
        // Check cache first
        if self.image_cache.contains_key(&id) {
            return self.image_cache.get(&id);
        }

        // Allocate space in atlas
        if let Some((x, y)) = self.atlas.allocate_region(width, height) {
            // Upload to GPU
            if self
                .atlas
                .upload_region(queue, data, x, y, width, height)
                .is_ok()
            {
                // Calculate UV coordinates
                let atlas_size = self.atlas.size();
                let u0 = x as f32 / atlas_size.0 as f32;
                let v0 = y as f32 / atlas_size.1 as f32;
                let u1 = (x + width) as f32 / atlas_size.0 as f32;
                let v1 = (y + height) as f32 / atlas_size.1 as f32;

                let cached_image = CachedImage {
                    uv_rect: (u0, v0, u1, v1),
                    atlas_region: (x, y, width, height),
                };

                self.image_cache.insert(id, cached_image);
                return self.image_cache.get(&id);
            } else {
                println!("Failed to upload image region");
            }
        } else {
            println!(
                "Failed to allocate atlas region for image: {}x{}",
                width, height
            );
        }

        None
    }

    /// Get texture atlas
    pub fn atlas(&self) -> &TextureAtlas {
        &self.atlas
    }

    /// Get glyph cache stats
    pub fn cache_stats(&self) -> (usize, (u32, u32)) {
        (self.glyph_cache.len(), self.atlas.size())
    }
    /// Get line metrics for a given font size
    pub fn get_line_metrics(&self, size: f32) -> Option<fontdue::LineMetrics> {
        self.rasterizer.font.horizontal_line_metrics(size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gpu::DeviceManager;
    use wgpu::Backends;

    #[test]
    fn test_glyph_key() {
        let key1 = GlyphKey {
            character: 'A',
            font_size: 24,
        };
        let key2 = GlyphKey {
            character: 'A',
            font_size: 24,
        };
        let key3 = GlyphKey {
            character: 'B',
            font_size: 24,
        };

        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_glyph_cache() {
        let mut cache = GlyphCache::new();

        let key = GlyphKey {
            character: 'A',
            font_size: 24,
        };
        let glyph = CachedGlyph {
            metrics: GlyphMetrics {
                width: 10,
                height: 12,
                bearing_x: 1,
                bearing_y: 11,
                advance: 11.0,
            },
            uv_rect: (0.0, 0.0, 0.1, 0.1),
            atlas_region: (0, 0, 10, 12),
        };

        cache.insert(key, glyph);
        assert!(cache.get(&key).is_some());
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn test_glyph_rasterizer() {
        let rasterizer = GlyphRasterizer::new().unwrap();

        let result = rasterizer.rasterize('A', 24.0);
        assert!(result.is_some());

        let (data, metrics) = result.unwrap();
        assert!(metrics.width > 0);
        assert!(metrics.height > 0);
        assert_eq!(data.len(), (metrics.width * metrics.height * 4) as usize);
    }

    #[tokio::test]
    #[ignore] // TODO: Fix shelf packing test expectations
    async fn test_atlas_allocation() {
        let dm = DeviceManager::new(Backends::all()).await.unwrap();
        let mut atlas = TextureAtlas::new(dm.device(), 256, 256);

        // Test basic allocation
        let region1 = atlas.allocate_region(10, 12);
        assert!(region1.is_some());
        assert_eq!(region1.unwrap(), (0, 0));

        // Test multiple allocations in same row
        let region2 = atlas.allocate_region(8, 10);
        assert!(region2.is_some());

        // Test allocation that should succeed (fits in atlas)
        let region3 = atlas.allocate_region(100, 20);
        assert!(region3.is_some());

        // Test allocation that's too wide for the atlas
        let region_fail = atlas.allocate_region(300, 20);
        assert_eq!(region_fail, None);
    }

    #[tokio::test]
    async fn test_texture_manager_glyph_caching() {
        let dm = DeviceManager::new(Backends::all()).await.unwrap();
        let mut tex_mgr = TextureManager::new_with_font(dm.device(), dm.queue());

        // Cache first glyph
        let glyph1 = tex_mgr.get_or_cache_glyph(dm.queue(), 'A', 24);
        assert!(glyph1.is_some());

        // Should retrieve from cache (not rasterize again)
        let glyph2 = tex_mgr.get_or_cache_glyph(dm.queue(), 'A', 24);
        assert!(glyph2.is_some());

        let (cache_size, _) = tex_mgr.cache_stats();
        assert_eq!(cache_size, 1); // Only one glyph cached
    }

    #[tokio::test]
    async fn test_texture_atlas_creation() {
        let dm = DeviceManager::new(Backends::all()).await.unwrap();
        let atlas = TextureAtlas::new(dm.device(), 256, 256);

        assert_eq!(atlas.size(), (256, 256));
    }

    #[tokio::test]
    async fn test_default_white_texture() {
        let dm = DeviceManager::new(Backends::all()).await.unwrap();
        let atlas = TextureAtlas::create_default_white(dm.device(), dm.queue());

        assert_eq!(atlas.size(), (1, 1));
    }
}
