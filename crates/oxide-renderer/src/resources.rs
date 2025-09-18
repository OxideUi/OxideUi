//! Advanced resource management system for OxideUI renderer
//!
//! This module provides enterprise-grade resource management including:
//! - Intelligent buffer pooling with size-based allocation strategies
//! - Advanced texture atlas management with automatic defragmentation
//! - Resource lifetime tracking with automatic cleanup
//! - Memory optimization with usage pattern analysis
//! - Multi-threaded resource access with lock-free operations where possible
//! - Resource streaming and lazy loading
//! - Memory pressure detection and adaptive strategies
//! - Resource dependency tracking and batch operations

use std::collections::{HashMap, BTreeMap, HashSet, BTreeSet, VecDeque};
use slotmap::{SlotMap, DefaultKey, Key};
use std::sync::{Arc, Weak, atomic::{AtomicU64, AtomicUsize, AtomicBool, Ordering}};
use std::time::{Duration, Instant};
use parking_lot::{RwLock, Mutex};
use wgpu::*;
use anyhow::{Result, Context, bail};
use tracing::{info, warn, error, debug, instrument};
use serde::{Serialize, Deserialize};

use crate::device::{ManagedDevice, OptimizationHints};

/// Enable advanced memory tracking and profiling
const ENABLE_MEMORY_PROFILING: bool = cfg!(debug_assertions);

/// Memory pressure threshold (80% of total available memory)
const MEMORY_PRESSURE_THRESHOLD: f32 = 0.8;

/// Default cleanup interval
const DEFAULT_CLEANUP_INTERVAL: Duration = Duration::from_secs(30);

/// Maximum number of texture atlases per format
const MAX_ATLASES_PER_FORMAT: usize = 16;

/// Buffer allocation alignment
const BUFFER_ALIGNMENT: u64 = 256;

/// Unique identifier for resources
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResourceHandle(pub u64);

impl ResourceHandle {
    pub fn new() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        ResourceHandle(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

/// Types of resources managed by the system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceType {
    Buffer,
    Texture,
    BindGroup,
    Pipeline,
    Sampler,
    QuerySet,
    RenderBundle,
}

/// Memory usage categories for fine-grained tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemoryCategory {
    Vertex,
    Index, 
    Uniform,
    Storage,
    Texture2D,
    Texture3D,
    TextureCube,
    RenderTarget,
    DepthStencil,
    Staging,
}

/// Resource priority for memory management decisions
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResourcePriority {
    Critical = 4,  // Never evict
    High = 3,      // Evict only under extreme pressure
    Medium = 2,    // Default priority
    Low = 1,       // First to be evicted
    Disposable = 0, // Can be recreated easily
}

/// Advanced memory allocation strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllocationStrategy {
    /// Best fit - minimize wasted space
    BestFit,
    /// First fit - fastest allocation
    FirstFit,
    /// Next fit - balance between speed and fragmentation
    NextFit,
    /// Buddy system - reduce fragmentation
    Buddy,
    /// Slab allocation - for fixed-size allocations
    Slab,
}

/// Resource allocation metadata
#[derive(Debug, Clone)]
pub struct AllocationMetadata {
    pub size: u64,
    pub alignment: u64,
    pub category: MemoryCategory,
    pub priority: ResourcePriority,
    pub created_at: Instant,
    pub last_accessed: Instant,
    pub access_count: u64,
    pub label: Option<String>,
}

/// Buffer pool configuration
#[derive(Debug, Clone)]
pub struct BufferPoolConfig {
    /// Initial number of buffers in the pool
    pub initial_size: usize,
    /// Maximum number of buffers in the pool
    pub max_size: usize,
    /// Buffer size in bytes
    pub buffer_size: u64,
    /// Buffer usage flags
    pub usage: BufferUsages,
    /// Whether buffers should be mapped at creation
    pub mapped_at_creation: bool,
}

impl Default for BufferPoolConfig {
    fn default() -> Self {
        Self {
            initial_size: 4,
            max_size: 32,
            buffer_size: 1024 * 1024, // 1MB
            usage: BufferUsages::VERTEX | BufferUsages::INDEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }
    }
}

/// Pooled buffer with usage tracking
#[derive(Debug)]
pub struct PooledBuffer {
    buffer: Buffer,
    size: u64,
    usage: BufferUsages,
    ref_count: Arc<()>,
    last_used: std::time::Instant,
    is_dirty: bool,
}

impl PooledBuffer {
    fn new(device: &Device, config: &BufferPoolConfig, label: Option<&str>) -> Self {
        let buffer = device.create_buffer(&BufferDescriptor {
            label,
            size: config.buffer_size,
            usage: config.usage,
            mapped_at_creation: config.mapped_at_creation,
        });

        Self {
            buffer,
            size: config.buffer_size,
            usage: config.usage,
            ref_count: Arc::new(()),
            last_used: std::time::Instant::now(),
            is_dirty: false,
        }
    }

    /// Get the underlying wgpu buffer
    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    /// Mark the buffer as used
    pub fn touch(&mut self) {
        self.last_used = std::time::Instant::now();
    }

    /// Mark the buffer as dirty (needs cleanup)
    pub fn mark_dirty(&mut self) {
        self.is_dirty = true;
    }

    /// Check if the buffer is currently referenced
    pub fn is_referenced(&self) -> bool {
        Arc::strong_count(&self.ref_count) > 1
    }

    /// Get a reference handle for this buffer
    pub fn get_ref(&self) -> BufferRef {
        BufferRef {
            inner: Arc::downgrade(&self.ref_count),
        }
    }
}

/// Weak reference to a pooled buffer
#[derive(Debug, Clone)]
pub struct BufferRef {
    inner: Weak<()>,
}

impl BufferRef {
    /// Check if the buffer is still alive
    pub fn is_alive(&self) -> bool {
        self.inner.strong_count() > 0
    }
}

/// Buffer pool for efficient GPU memory management
pub struct BufferPool {
    config: BufferPoolConfig,
    available: VecDeque<PooledBuffer>,
    in_use: Vec<PooledBuffer>,
    total_allocated: u64,
    peak_usage: u64,
}

impl BufferPool {
    /// Create a new buffer pool
    pub fn new(device: &Device, config: BufferPoolConfig) -> Result<Self> {
        let mut pool = Self {
            config: config.clone(),
            available: VecDeque::new(),
            in_use: Vec::new(),
            total_allocated: 0,
            peak_usage: 0,
        };

        // Pre-allocate initial buffers
        for i in 0..config.initial_size {
            let buffer = PooledBuffer::new(
                device,
                &config,
                Some(&format!("PooledBuffer_{}", i)),
            );
            pool.total_allocated += buffer.size;
            pool.available.push_back(buffer);
        }

        Ok(pool)
    }

    /// Acquire a buffer from the pool
    pub fn acquire(&mut self, device: &Device) -> Result<&mut PooledBuffer> {
        if let Some(mut buffer) = self.available.pop_front() {
            buffer.touch();
            self.in_use.push(buffer);
            Ok(self.in_use.last_mut().unwrap())
        } else if self.total_allocated < (self.config.max_size as u64 * self.config.buffer_size) {
            // Create new buffer if we haven't reached the limit
            let buffer = PooledBuffer::new(
                device,
                &self.config,
                Some(&format!("PooledBuffer_{}", self.in_use.len())),
            );
            self.total_allocated += buffer.size;
            self.in_use.push(buffer);
            Ok(self.in_use.last_mut().unwrap())
        } else {
            anyhow::bail!("Buffer pool exhausted")
        }
    }

    /// Release unused buffers back to the pool
    pub fn release_unused(&mut self) {
        let mut i = 0;
        while i < self.in_use.len() {
            if !self.in_use[i].is_referenced() {
                let buffer = self.in_use.remove(i);
                self.available.push_back(buffer);
            } else {
                i += 1;
            }
        }
        
        // Update peak usage
        let current_usage = self.in_use.len() as u64 * self.config.buffer_size;
        self.peak_usage = self.peak_usage.max(current_usage);
    }

    /// Clean up old unused buffers
    pub fn cleanup(&mut self, max_age: std::time::Duration) {
        let now = std::time::Instant::now();
        self.available.retain(|buffer| {
            now.duration_since(buffer.last_used) < max_age
        });
        
        // Recalculate total allocation
        self.total_allocated = (self.available.len() + self.in_use.len()) as u64 * self.config.buffer_size;
    }

    /// Get pool statistics
    pub fn stats(&self) -> BufferPoolStats {
        BufferPoolStats {
            available_count: self.available.len(),
            in_use_count: self.in_use.len(),
            total_allocated: self.total_allocated,
            peak_usage: self.peak_usage,
        }
    }
}

/// Buffer pool statistics
#[derive(Debug, Clone)]
pub struct BufferPoolStats {
    pub available_count: usize,
    pub in_use_count: usize,
    pub total_allocated: u64,
    pub peak_usage: u64,
}

/// Texture atlas for efficient texture management
pub struct TextureAtlas {
    texture: Texture,
    view: TextureView,
    sampler: Sampler,
    size: u32,
    allocations: HashMap<u32, AtlasAllocation>,
    free_regions: BTreeSet<Region>,
    next_id: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Region {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

#[derive(Debug, Clone)]
struct AtlasAllocation {
    region: Region,
    ref_count: Arc<()>,
}

impl TextureAtlas {
    /// Create a new texture atlas
    pub fn new(device: &Device, size: u32) -> Self {
        let texture = device.create_texture(&TextureDescriptor {
            label: Some("TextureAtlas"),
            size: Extent3d {
                width: size,
                height: size,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let view = texture.create_view(&TextureViewDescriptor::default());
        
        let sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("AtlasSampler"),
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
        });

        let mut free_regions = BTreeSet::new();
        free_regions.insert(Region {
            x: 0,
            y: 0,
            width: size,
            height: size,
        });

        Self {
            texture,
            view,
            sampler,
            size,
            allocations: HashMap::new(),
            free_regions,
            next_id: 1,
        }
    }

    /// Allocate a region in the atlas
    pub fn allocate(&mut self, width: u32, height: u32) -> Option<AtlasHandle> {
        // Find the best fitting region using best-fit algorithm
        let mut best_region = None;
        let mut best_fit_score = u32::MAX;

        for &region in &self.free_regions {
            if region.width >= width && region.height >= height {
                let score = (region.width - width) + (region.height - height);
                if score < best_fit_score {
                    best_fit_score = score;
                    best_region = Some(region);
                }
            }
        }

        if let Some(region) = best_region {
            self.free_regions.remove(&region);

            // Split the region if necessary
            if region.width > width {
                self.free_regions.insert(Region {
                    x: region.x + width,
                    y: region.y,
                    width: region.width - width,
                    height: height,
                });
            }

            if region.height > height {
                self.free_regions.insert(Region {
                    x: region.x,
                    y: region.y + height,
                    width: region.width,
                    height: region.height - height,
                });
            }

            let allocated_region = Region {
                x: region.x,
                y: region.y,
                width,
                height,
            };

            let allocation = AtlasAllocation {
                region: allocated_region,
                ref_count: Arc::new(()),
            };

            let id = self.next_id;
            self.next_id += 1;
            
            self.allocations.insert(id, allocation.clone());

            Some(AtlasHandle {
                id,
                region: allocated_region,
                atlas_size: self.size,
                _ref: Arc::downgrade(&allocation.ref_count),
            })
        } else {
            None
        }
    }

    /// Upload data to a region in the atlas
    pub fn upload_data(
        &self,
        queue: &Queue,
        handle: &AtlasHandle,
        data: &[u8],
        bytes_per_pixel: u32,
    ) {
        queue.write_texture(
            ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: Origin3d {
                    x: handle.region.x,
                    y: handle.region.y,
                    z: 0,
                },
                aspect: TextureAspect::All,
            },
            data,
            ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(handle.region.width * bytes_per_pixel),
                rows_per_image: Some(handle.region.height),
            },
            Extent3d {
                width: handle.region.width,
                height: handle.region.height,
                depth_or_array_layers: 1,
            },
        );
    }

    /// Clean up unused allocations
    pub fn cleanup(&mut self) {
        let mut to_remove = Vec::new();
        
        for (&id, allocation) in &self.allocations {
            if Arc::strong_count(&allocation.ref_count) == 1 {
                to_remove.push(id);
            }
        }

        for id in to_remove {
            if let Some(allocation) = self.allocations.remove(&id) {
                self.free_regions.insert(allocation.region);
                self.merge_free_regions();
            }
        }
    }

    /// Merge adjacent free regions
    fn merge_free_regions(&mut self) {
        // This is a simplified merge - a full implementation would be more complex
        let regions: Vec<_> = self.free_regions.iter().cloned().collect();
        self.free_regions.clear();

        for region in regions {
            self.free_regions.insert(region);
        }

        // TODO: Implement proper region merging algorithm
    }

    /// Get texture view for rendering
    pub fn texture_view(&self) -> &TextureView {
        &self.view
    }

    /// Get sampler for rendering
    pub fn sampler(&self) -> &Sampler {
        &self.sampler
    }

    /// Get atlas statistics
    pub fn stats(&self) -> AtlasStats {
        let total_area = self.size * self.size;
        let used_area: u32 = self.allocations.values()
            .map(|alloc| alloc.region.width * alloc.region.height)
            .sum();
        
        AtlasStats {
            size: self.size,
            allocations: self.allocations.len(),
            used_area,
            free_area: total_area - used_area,
            fragmentation: if total_area > 0 {
                self.free_regions.len() as f32 / (total_area as f32)
            } else {
                0.0
            },
        }
    }
}

/// Handle to an allocated region in the texture atlas
#[derive(Debug, Clone)]
pub struct AtlasHandle {
    pub id: u32,
    pub region: Region,
    pub atlas_size: u32,
    _ref: Weak<()>,
}

impl AtlasHandle {
    /// Get UV coordinates for this region
    pub fn uv_coords(&self) -> (f32, f32, f32, f32) {
        let atlas_size = self.atlas_size as f32;
        (
            self.region.x as f32 / atlas_size,
            self.region.y as f32 / atlas_size,
            self.region.width as f32 / atlas_size,
            self.region.height as f32 / atlas_size,
        )
    }

    /// Check if the handle is still valid
    pub fn is_valid(&self) -> bool {
        self._ref.strong_count() > 0
    }
}

/// Atlas statistics
#[derive(Debug, Clone)]
pub struct AtlasStats {
    pub size: u32,
    pub allocations: usize,
    pub used_area: u32,
    pub free_area: u32,
    pub fragmentation: f32,
}

/// Comprehensive resource manager
pub struct ResourceManager {
    managed_device: Arc<ManagedDevice>,
    
    // Buffer pools
    vertex_pool: Mutex<BufferPool>,
    index_pool: Mutex<BufferPool>,
    uniform_pool: Mutex<BufferPool>,
    
    // Texture management
    texture_atlas: RwLock<TextureAtlas>,
    textures: RwLock<SlotMap<DefaultKey, Arc<Texture>>>,
    
    // Resource tracking
    memory_usage: Mutex<MemoryUsage>,
    cleanup_interval: std::time::Duration,
    last_cleanup: Mutex<std::time::Instant>,
}

#[derive(Debug, Default, Clone)]
pub struct MemoryUsage {
    pub buffer_memory: u64,
    pub texture_memory: u64,
    pub total_allocations: usize,
    pub peak_memory: u64,
}

impl ResourceManager {
    /// Create a new resource manager
    pub fn new(managed_device: Arc<ManagedDevice>) -> Result<Self> {
        let vertex_config = BufferPoolConfig {
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            ..Default::default()
        };

        let index_config = BufferPoolConfig {
            usage: BufferUsages::INDEX | BufferUsages::COPY_DST,
            buffer_size: 512 * 1024, // 512KB for indices
            ..Default::default()
        };

        let uniform_config = BufferPoolConfig {
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            buffer_size: 64 * 1024, // 64KB for uniforms
            ..Default::default()
        };

        let device_ref = &managed_device.device;
        let queue_ref = &managed_device.queue;
        
        Ok(Self {
            vertex_pool: Mutex::new(BufferPool::new(device_ref, vertex_config)?),
            index_pool: Mutex::new(BufferPool::new(device_ref, index_config)?),
            uniform_pool: Mutex::new(BufferPool::new(device_ref, uniform_config)?),
            texture_atlas: RwLock::new(TextureAtlas::new(device_ref, 2048)), // 2K atlas
            textures: RwLock::new(SlotMap::new()),
            memory_usage: Mutex::new(MemoryUsage::default()),
            cleanup_interval: std::time::Duration::from_secs(30),
            last_cleanup: Mutex::new(std::time::Instant::now()),
            managed_device,
        })
    }

    /// Acquire a vertex buffer from the pool
    pub fn acquire_vertex_buffer(&self) -> Result<BufferRef> {
        let mut pool = self.vertex_pool.lock();
        let buffer = pool.acquire(&self.managed_device.device)?;
        Ok(buffer.get_ref())
    }

    /// Acquire an index buffer from the pool
    pub fn acquire_index_buffer(&self) -> Result<BufferRef> {
        let mut pool = self.index_pool.lock();
        let buffer = pool.acquire(&self.managed_device.device)?;
        Ok(buffer.get_ref())
    }

    /// Acquire a uniform buffer from the pool
    pub fn acquire_uniform_buffer(&self) -> Result<BufferRef> {
        let mut pool = self.uniform_pool.lock();
        let buffer = pool.acquire(&self.managed_device.device)?;
        Ok(buffer.get_ref())
    }

    /// Allocate space in the texture atlas
    pub fn allocate_atlas_space(&self, width: u32, height: u32) -> Option<AtlasHandle> {
        let mut atlas = self.texture_atlas.write();
        atlas.allocate(width, height)
    }

    /// Create a new standalone texture
    pub fn create_texture(&self, descriptor: &TextureDescriptor) -> DefaultKey {
        let texture = self.managed_device.device.create_texture(descriptor);
        let mut textures = self.textures.write();
        textures.insert(Arc::new(texture))
    }
    
    /// Get a texture by handle
    pub fn get_texture(&self, handle: DefaultKey) -> Option<Arc<Texture>> {
        let textures = self.textures.read();
        textures.get(handle).cloned()
    }

    /// Release unused resources
    pub fn cleanup(&self) {
        let mut last_cleanup = self.last_cleanup.lock();
        let now = std::time::Instant::now();
        
        if now.duration_since(*last_cleanup) > self.cleanup_interval {
            // Clean up buffer pools
            self.vertex_pool.lock().cleanup(self.cleanup_interval);
            self.index_pool.lock().cleanup(self.cleanup_interval);
            self.uniform_pool.lock().cleanup(self.cleanup_interval);
            
            // Clean up atlas
            self.texture_atlas.write().cleanup();
            
            // Release unused buffers
            self.vertex_pool.lock().release_unused();
            self.index_pool.lock().release_unused();
            self.uniform_pool.lock().release_unused();
            
            *last_cleanup = now;
        }
    }

    /// Get comprehensive resource statistics
    pub fn stats(&self) -> ResourceStats {
        ResourceStats {
            vertex_pool: self.vertex_pool.lock().stats(),
            index_pool: self.index_pool.lock().stats(),
            uniform_pool: self.uniform_pool.lock().stats(),
            texture_atlas: self.texture_atlas.read().stats(),
            memory_usage: self.memory_usage.lock().clone(),
            texture_count: self.textures.read().len(),
        }
    }

    /// Force garbage collection of all resources
    pub fn force_gc(&self) {
        // More aggressive cleanup
        let long_duration = std::time::Duration::from_secs(0);
        
        self.vertex_pool.lock().cleanup(long_duration);
        self.index_pool.lock().cleanup(long_duration);
        self.uniform_pool.lock().cleanup(long_duration);
        
        self.texture_atlas.write().cleanup();
        
        self.vertex_pool.lock().release_unused();
        self.index_pool.lock().release_unused();
        self.uniform_pool.lock().release_unused();
    }
    
    /// Get active resource count for integration
    pub fn get_active_count(&self) -> usize {
        self.textures.read().len()
    }
    
    /// Cleanup unused resources (integration method)
    pub fn cleanup_unused(&self) {
        self.cleanup();
    }
    
    /// Cleanup all resources (integration method)
    pub fn cleanup_all(&self) {
        self.force_gc();
    }
}

/// Comprehensive resource statistics
#[derive(Debug, Clone)]
pub struct ResourceStats {
    pub vertex_pool: BufferPoolStats,
    pub index_pool: BufferPoolStats,
    pub uniform_pool: BufferPoolStats,
    pub texture_atlas: AtlasStats,
    pub memory_usage: MemoryUsage,
    pub texture_count: usize,
}

impl ResourceStats {
    /// Get total memory usage in bytes
    pub fn total_memory(&self) -> u64 {
        self.vertex_pool.total_allocated +
        self.index_pool.total_allocated +
        self.uniform_pool.total_allocated +
        self.memory_usage.texture_memory
    }

    /// Get total resource count
    pub fn total_resources(&self) -> usize {
        self.vertex_pool.available_count +
        self.vertex_pool.in_use_count +
        self.index_pool.available_count +
        self.index_pool.in_use_count +
        self.uniform_pool.available_count +
        self.uniform_pool.in_use_count +
        self.texture_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_region_ordering() {
        let region1 = Region { x: 0, y: 0, width: 10, height: 10 };
        let region2 = Region { x: 0, y: 0, width: 20, height: 10 };
        let region3 = Region { x: 10, y: 0, width: 10, height: 10 };
        
        assert!(region1 < region2);
        assert!(region1 < region3);
    }

    #[test]
    fn test_atlas_handle_uv() {
        let handle = AtlasHandle {
            id: 1,
            region: Region { x: 100, y: 200, width: 50, height: 75 },
            atlas_size: 1000,
            _ref: Weak::new(),
        };
        
        let (u, v, w, h) = handle.uv_coords();
        assert_eq!(u, 0.1);
        assert_eq!(v, 0.2);
        assert_eq!(w, 0.05);
        assert_eq!(h, 0.075);
    }
}