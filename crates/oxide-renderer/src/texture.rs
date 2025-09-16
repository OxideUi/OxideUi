//! Texture management and atlas

use std::sync::Arc;
use dashmap::DashMap;
use image::{DynamicImage, ImageBuffer, Rgba};

/// Texture wrapper
pub struct Texture {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    sampler: wgpu::Sampler,
    size: (u32, u32),
}

impl Texture {
    /// Create a new texture
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, image: &DynamicImage) -> Self {
        let rgba = image.to_rgba8();
        let size = (image.width(), image.height());
        
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Texture"),
            size: wgpu::Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * size.0),
                rows_per_image: Some(size.1),
            },
            wgpu::Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            texture,
            view,
            sampler,
            size,
        }
    }

    /// Create a white texture (for untextured rendering)
    pub fn white(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let white_pixel = ImageBuffer::<Rgba<u8>, _>::from_raw(1, 1, vec![255u8; 4]).unwrap();
        let image = DynamicImage::ImageRgba8(white_pixel);
        Self::new(device, queue, &image)
    }

    /// Get texture view
    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    /// Get sampler
    pub fn sampler(&self) -> &wgpu::Sampler {
        &self.sampler
    }

    /// Get size
    pub fn size(&self) -> (u32, u32) {
        self.size
    }
}

/// Texture atlas for efficient texture management
pub struct TextureAtlas {
    texture: Arc<Texture>,
    regions: DashMap<String, AtlasRegion>,
    next_position: parking_lot::Mutex<(u32, u32)>,
    row_height: parking_lot::Mutex<u32>,
    size: (u32, u32),
}

/// Region within a texture atlas
#[derive(Debug, Clone, Copy)]
pub struct AtlasRegion {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub tex_coords: TexCoords,
}

/// Texture coordinates
#[derive(Debug, Clone, Copy)]
pub struct TexCoords {
    pub min_u: f32,
    pub min_v: f32,
    pub max_u: f32,
    pub max_v: f32,
}

impl TextureAtlas {
    /// Create a new texture atlas
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, size: (u32, u32)) -> Self {
        let empty_image = ImageBuffer::<Rgba<u8>, _>::from_fn(size.0, size.1, |_, _| {
            Rgba([0, 0, 0, 0])
        });
        let image = DynamicImage::ImageRgba8(empty_image);
        let texture = Arc::new(Texture::new(device, queue, &image));

        Self {
            texture,
            regions: DashMap::new(),
            next_position: parking_lot::Mutex::new((0, 0)),
            row_height: parking_lot::Mutex::new(0),
            size,
        }
    }

    /// Add an image to the atlas
    pub fn add_image(
        &self,
        queue: &wgpu::Queue,
        name: String,
        image: &DynamicImage,
    ) -> Option<AtlasRegion> {
        let img_size = (image.width(), image.height());
        
        let mut next_pos = self.next_position.lock();
        let mut row_height = self.row_height.lock();

        // Check if we need to move to next row
        if next_pos.0 + img_size.0 > self.size.0 {
            next_pos.0 = 0;
            next_pos.1 += *row_height;
            *row_height = 0;
        }

        // Check if image fits in atlas
        if next_pos.1 + img_size.1 > self.size.1 {
            return None; // Atlas is full
        }

        let region = AtlasRegion {
            x: next_pos.0,
            y: next_pos.1,
            width: img_size.0,
            height: img_size.1,
            tex_coords: TexCoords {
                min_u: next_pos.0 as f32 / self.size.0 as f32,
                min_v: next_pos.1 as f32 / self.size.1 as f32,
                max_u: (next_pos.0 + img_size.0) as f32 / self.size.0 as f32,
                max_v: (next_pos.1 + img_size.1) as f32 / self.size.1 as f32,
            },
        };

        // Write image data to texture
        let rgba = image.to_rgba8();
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.texture.texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: region.x,
                    y: region.y,
                    z: 0,
                },
                aspect: wgpu::TextureAspect::All,
            },
            &rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * img_size.0),
                rows_per_image: Some(img_size.1),
            },
            wgpu::Extent3d {
                width: img_size.0,
                height: img_size.1,
                depth_or_array_layers: 1,
            },
        );

        // Update position for next image
        next_pos.0 += img_size.0;
        *row_height = (*row_height).max(img_size.1);

        self.regions.insert(name, region);
        Some(region)
    }

    /// Get a region by name
    pub fn get_region(&self, name: &str) -> Option<AtlasRegion> {
        self.regions.get(name).map(|r| *r)
    }

    /// Get the atlas texture
    pub fn texture(&self) -> &Texture {
        &self.texture
    }

    /// Clear the atlas
    pub fn clear(&self) {
        self.regions.clear();
        *self.next_position.lock() = (0, 0);
        *self.row_height.lock() = 0;
    }
}
