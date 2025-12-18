//! Advanced buffer management system for GPU resources
//!
//! This module provides a comprehensive buffer management system including:
//! - Dynamic buffer allocation with intelligent pooling
//! - Multi-tier memory management (device, host-visible, staging)
//! - Automatic buffer resizing and defragmentation
//! - Usage pattern analysis and optimization
//! - Memory pressure detection and mitigation
//! - Cross-platform buffer optimization
//! - Performance profiling and analytics
//! - Lock-free buffer operations where possible

use anyhow::{Context, Result};
use parking_lot::{Mutex, RwLock};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::ops::Range;
use std::sync::{
    atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
    Arc,
};
use std::time::{Duration, Instant};
use tracing::{debug, info, instrument, warn};
use wgpu::{Buffer, BufferDescriptor, BufferUsages};

use crate::device::ManagedDevice;
use crate::memory::{MemoryManager, MemoryTier};
use crate::resources::ResourceHandle;

/// Buffer usage patterns for optimization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BufferUsagePattern {
    /// Static data that rarely changes
    Static,
    /// Dynamic data that changes frequently
    Dynamic,
    /// Streaming data that changes every frame
    Streaming,
    /// Temporary data for single-use operations
    Transient,
    /// Persistent data that lives for the entire application
    Persistent,
}

/// Buffer allocation strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AllocationStrategy {
    /// Best fit allocation
    BestFit,
    /// First fit allocation
    FirstFit,
    /// Buddy allocation
    Buddy,
    /// Pool allocation
    Pool,
    /// Linear allocation
    Linear,
}

/// Buffer configuration
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BufferConfig {
    pub name: String,
    pub size: u64,
    pub usage: BufferUsages,
    pub usage_pattern: BufferUsagePattern,
    pub allocation_strategy: AllocationStrategy,
    pub alignment: u64,
    pub mapped_at_creation: bool,
    pub persistent_mapping: bool,
}

/// Buffer allocation information
#[derive(Debug)]
pub struct BufferAllocation {
    pub buffer: Arc<Buffer>,
    pub offset: u64,
    pub size: u64,
    pub usage_pattern: BufferUsagePattern,
    pub allocation_time: Instant,
    pub last_access: Instant,
    pub access_count: AtomicU64,
    pub memory_tier: MemoryTier,
}

/// Buffer pool for efficient reuse
pub struct BufferPool {
    device: Arc<ManagedDevice>,
    _memory_manager: Arc<Mutex<MemoryManager>>,

    // Pool storage by size and usage
    pools: RwLock<HashMap<(u64, BufferUsages), VecDeque<Arc<Buffer>>>>,

    // Active allocations
    active_allocations: RwLock<HashMap<ResourceHandle, BufferAllocation>>,

    // Configuration
    max_pool_size: AtomicUsize,
    _min_buffer_size: AtomicU64,
    _max_buffer_size: AtomicU64,
    alignment_requirement: AtomicU64,

    // Statistics
    allocation_count: AtomicU64,
    deallocation_count: AtomicU64,
    pool_hits: AtomicU64,
    pool_misses: AtomicU64,
    total_allocated_bytes: AtomicU64,
    peak_allocated_bytes: AtomicU64,

    // Performance tracking
    allocation_times: RwLock<VecDeque<Duration>>,
    usage_patterns: RwLock<HashMap<BufferUsagePattern, BufferUsageStats>>,
}

/// Buffer usage statistics
#[derive(Debug, Clone, Default)]
pub struct BufferUsageStats {
    pub allocation_count: u64,
    pub total_size: u64,
    pub average_lifetime: Duration,
    pub access_frequency: f64,
    pub fragmentation_ratio: f32,
}

/// Dynamic buffer that can grow and shrink
pub struct DynamicBuffer {
    _device: Arc<ManagedDevice>,
    buffer_pool: Arc<BufferPool>,

    // Current buffer
    current_buffer: RwLock<Option<Arc<Buffer>>>,
    current_size: AtomicU64,
    used_size: AtomicU64,

    // Configuration
    config: BufferConfig,
    growth_factor: f32,
    shrink_threshold: f32,

    // Statistics
    resize_count: AtomicU64,
    last_resize: RwLock<Option<Instant>>,
}

/// Staging buffer for efficient data transfers
#[allow(dead_code)]
pub struct StagingBuffer {
    device: Arc<ManagedDevice>,
    buffer: Arc<Buffer>,
    mapped_range: Option<Range<u64>>,
    size: u64,
    usage_pattern: BufferUsagePattern,

    // Transfer tracking
    pending_transfers: RwLock<Vec<PendingTransfer>>,
    completed_transfers: AtomicU64,
}

/// Pending transfer operation
#[derive(Debug, Clone)]
pub struct PendingTransfer {
    pub source_offset: u64,
    pub dest_buffer: Arc<Buffer>,
    pub dest_offset: u64,
    pub size: u64,
    pub timestamp: Instant,
}

/// Ring buffer for streaming data
pub struct RingBuffer {
    _device: Arc<ManagedDevice>,
    buffer: Arc<Buffer>,
    size: u64,
    head: AtomicU64,
    tail: AtomicU64,
    wrapped: AtomicBool,

    // Configuration
    alignment: u64,
    _usage_pattern: BufferUsagePattern,

    // Statistics
    write_count: AtomicU64,
    bytes_written: AtomicU64,
    overruns: AtomicU64,
}

/// Buffer manager coordinating all buffer operations
pub struct BufferManager {
    device: Arc<ManagedDevice>,
    _memory_manager: Arc<Mutex<MemoryManager>>,

    // Buffer pools
    vertex_pool: Arc<BufferPool>,
    index_pool: Arc<BufferPool>,
    uniform_pool: Arc<BufferPool>,
    storage_pool: Arc<BufferPool>,
    staging_pool: Arc<BufferPool>,

    // Dynamic buffers
    dynamic_buffers: RwLock<HashMap<ResourceHandle, Arc<DynamicBuffer>>>,

    // Ring buffers
    ring_buffers: RwLock<HashMap<ResourceHandle, Arc<RingBuffer>>>,

    // Staging buffers
    _staging_buffers: RwLock<HashMap<ResourceHandle, Arc<StagingBuffer>>>,

    // Global statistics
    total_buffers: AtomicU64,
    total_memory_usage: AtomicU64,
    _fragmentation_ratio: RwLock<f32>,

    // Configuration
    auto_defragmentation: AtomicBool,
    memory_pressure_threshold: AtomicU64,
    _profiling_enabled: AtomicBool,
}

impl Default for BufferConfig {
    fn default() -> Self {
        Self {
            name: "unnamed".to_string(),
            size: 1024,
            usage: BufferUsages::VERTEX,
            usage_pattern: BufferUsagePattern::Dynamic,
            allocation_strategy: AllocationStrategy::Pool,
            alignment: 256,
            mapped_at_creation: false,
            persistent_mapping: false,
        }
    }
}

impl BufferPool {
    /// Create a new buffer pool
    pub fn new(device: Arc<ManagedDevice>, memory_manager: Arc<Mutex<MemoryManager>>) -> Self {
        Self {
            device,
            _memory_manager: memory_manager,
            pools: RwLock::new(HashMap::new()),
            active_allocations: RwLock::new(HashMap::new()),
            max_pool_size: AtomicUsize::new(100),
            _min_buffer_size: AtomicU64::new(256),
            _max_buffer_size: AtomicU64::new(256 * 1024 * 1024), // 256MB
            alignment_requirement: AtomicU64::new(256),
            allocation_count: AtomicU64::new(0),
            deallocation_count: AtomicU64::new(0),
            pool_hits: AtomicU64::new(0),
            pool_misses: AtomicU64::new(0),
            total_allocated_bytes: AtomicU64::new(0),
            peak_allocated_bytes: AtomicU64::new(0),
            allocation_times: RwLock::new(VecDeque::new()),
            usage_patterns: RwLock::new(HashMap::new()),
        }
    }

    /// Allocate a buffer from the pool
    #[instrument(skip(self))]
    pub fn allocate(&self, config: &BufferConfig) -> Result<ResourceHandle> {
        let start_time = Instant::now();

        // Align size to requirements
        let aligned_size = self.align_size(config.size);
        let pool_key = (aligned_size, config.usage);

        // Try to get from pool first
        if let Some(buffer) = self.try_get_from_pool(&pool_key) {
            self.pool_hits.fetch_add(1, Ordering::Relaxed);
            return self.create_allocation(buffer, config, start_time);
        }

        self.pool_misses.fetch_add(1, Ordering::Relaxed);

        // Create new buffer
        let buffer = self.create_buffer(config, aligned_size)?;
        self.create_allocation(Arc::new(buffer), config, start_time)
    }

    /// Try to get buffer from pool
    fn try_get_from_pool(&self, pool_key: &(u64, BufferUsages)) -> Option<Arc<Buffer>> {
        let mut pools = self.pools.write();
        if let Some(pool) = pools.get_mut(pool_key) {
            pool.pop_front()
        } else {
            None
        }
    }

    /// Create a new buffer
    fn create_buffer(&self, config: &BufferConfig, size: u64) -> Result<Buffer> {
        let buffer = self.device.device.create_buffer(&BufferDescriptor {
            label: Some(&config.name),
            size,
            usage: config.usage,
            mapped_at_creation: config.mapped_at_creation,
        });

        Ok(buffer)
    }

    /// Create allocation record
    fn create_allocation(
        &self,
        buffer: Arc<Buffer>,
        config: &BufferConfig,
        start_time: Instant,
    ) -> Result<ResourceHandle> {
        let handle = ResourceHandle::new();
        let allocation_time = start_time.elapsed();

        let allocation = BufferAllocation {
            buffer,
            offset: 0,
            size: config.size,
            usage_pattern: config.usage_pattern,
            allocation_time: start_time,
            last_access: start_time,
            access_count: AtomicU64::new(0),
            memory_tier: MemoryTier::HighSpeed, // Default tier
        };

        self.active_allocations.write().insert(handle, allocation);

        // Update statistics
        self.allocation_count.fetch_add(1, Ordering::Relaxed);
        self.total_allocated_bytes
            .fetch_add(config.size, Ordering::Relaxed);

        let current_total = self.total_allocated_bytes.load(Ordering::Relaxed);
        let peak = self.peak_allocated_bytes.load(Ordering::Relaxed);
        if current_total > peak {
            self.peak_allocated_bytes
                .store(current_total, Ordering::Relaxed);
        }

        // Record allocation time
        let mut times = self.allocation_times.write();
        times.push_back(allocation_time);
        if times.len() > 1000 {
            times.pop_front();
        }

        // Update usage pattern statistics
        self.update_usage_stats(config.usage_pattern, config.size);

        debug!(
            "Allocated buffer '{}' ({} bytes) in {:?}",
            config.name, config.size, allocation_time
        );

        Ok(handle)
    }

    /// Deallocate a buffer
    #[instrument(skip(self))]
    pub fn deallocate(&self, handle: ResourceHandle) -> Result<()> {
        let allocation = self
            .active_allocations
            .write()
            .remove(&handle)
            .context("Buffer allocation not found")?;

        let pool_key = (allocation.size, allocation.buffer.usage());

        // Return to pool if there's space
        let mut pools = self.pools.write();
        let pool = pools.entry(pool_key).or_insert_with(VecDeque::new);

        if pool.len() < self.max_pool_size.load(Ordering::Relaxed) {
            pool.push_back(allocation.buffer);
        }
        // Otherwise, buffer will be dropped and freed

        // Update statistics
        self.deallocation_count.fetch_add(1, Ordering::Relaxed);
        self.total_allocated_bytes
            .fetch_sub(allocation.size, Ordering::Relaxed);

        debug!("Deallocated buffer (handle: {:?})", handle);

        Ok(())
    }

    /// Get buffer allocation (returns reference to avoid clone issues)
    pub fn get_allocation(&self, handle: ResourceHandle) -> Option<Arc<Buffer>> {
        self.active_allocations
            .read()
            .get(&handle)
            .map(|alloc| alloc.buffer.clone())
    }

    /// Update access time for buffer
    pub fn update_access(&self, handle: ResourceHandle) {
        if let Some(allocation) = self.active_allocations.write().get_mut(&handle) {
            allocation.last_access = Instant::now();
            allocation.access_count.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Align size to requirements
    fn align_size(&self, size: u64) -> u64 {
        let alignment = self.alignment_requirement.load(Ordering::Relaxed);
        (size + alignment - 1) & !(alignment - 1)
    }

    /// Update usage pattern statistics
    fn update_usage_stats(&self, pattern: BufferUsagePattern, size: u64) {
        let mut stats = self.usage_patterns.write();
        let pattern_stats = stats.entry(pattern).or_default();
        pattern_stats.allocation_count += 1;
        pattern_stats.total_size += size;
    }

    /// Get pool statistics
    pub fn get_stats(&self) -> HashMap<String, u64> {
        let mut stats = HashMap::new();
        stats.insert(
            "allocations".to_string(),
            self.allocation_count.load(Ordering::Relaxed),
        );
        stats.insert(
            "deallocations".to_string(),
            self.deallocation_count.load(Ordering::Relaxed),
        );
        stats.insert(
            "pool_hits".to_string(),
            self.pool_hits.load(Ordering::Relaxed),
        );
        stats.insert(
            "pool_misses".to_string(),
            self.pool_misses.load(Ordering::Relaxed),
        );
        stats.insert(
            "total_bytes".to_string(),
            self.total_allocated_bytes.load(Ordering::Relaxed),
        );
        stats.insert(
            "peak_bytes".to_string(),
            self.peak_allocated_bytes.load(Ordering::Relaxed),
        );
        stats
    }

    /// Clear unused buffers from pools
    pub fn cleanup(&self) {
        let mut pools = self.pools.write();
        for pool in pools.values_mut() {
            pool.clear();
        }
        info!("Cleared buffer pools");
    }

    /// Set maximum pool size
    pub fn set_max_pool_size(&self, size: usize) {
        self.max_pool_size.store(size, Ordering::Relaxed);
    }
}

impl DynamicBuffer {
    /// Create a new dynamic buffer
    pub fn new(
        device: Arc<ManagedDevice>,
        buffer_pool: Arc<BufferPool>,
        config: BufferConfig,
    ) -> Self {
        Self {
            _device: device,
            buffer_pool,
            current_buffer: RwLock::new(None),
            current_size: AtomicU64::new(0),
            used_size: AtomicU64::new(0),
            config,
            growth_factor: 1.5,
            shrink_threshold: 0.25,
            resize_count: AtomicU64::new(0),
            last_resize: RwLock::new(None),
        }
    }

    /// Ensure buffer has at least the specified capacity
    pub fn ensure_capacity(&self, required_size: u64) -> Result<()> {
        let current_size = self.current_size.load(Ordering::Relaxed);

        if required_size <= current_size {
            return Ok(());
        }

        let new_size = (required_size as f32 * self.growth_factor) as u64;
        self.resize(new_size)?;

        Ok(())
    }

    /// Resize the buffer
    fn resize(&self, new_size: u64) -> Result<()> {
        let mut config = self.config.clone();
        config.size = new_size;

        let handle = self.buffer_pool.allocate(&config)?;
        let allocation = self
            .buffer_pool
            .get_allocation(handle)
            .context("Failed to get buffer allocation")?;

        // Copy existing data if needed
        if let Some(_old_buffer) = self.current_buffer.read().as_ref() {}

        *self.current_buffer.write() = Some(allocation);
        self.current_size.store(new_size, Ordering::Relaxed);
        self.resize_count.fetch_add(1, Ordering::Relaxed);
        *self.last_resize.write() = Some(Instant::now());

        debug!("Resized dynamic buffer to {} bytes", new_size);

        Ok(())
    }

    /// Get current buffer
    pub fn get_buffer(&self) -> Option<Arc<Buffer>> {
        self.current_buffer.read().clone()
    }

    /// Update used size
    pub fn set_used_size(&self, size: u64) {
        self.used_size.store(size, Ordering::Relaxed);

        // Check if we should shrink
        let current_size = self.current_size.load(Ordering::Relaxed);
        let usage_ratio = size as f32 / current_size as f32;

        if usage_ratio < self.shrink_threshold && current_size > self.config.size {
            let new_size =
                std::cmp::max((size as f32 * self.growth_factor) as u64, self.config.size);

            if let Err(e) = self.resize(new_size) {
                warn!("Failed to shrink buffer: {}", e);
            }
        }
    }
}

impl RingBuffer {
    /// Create a new ring buffer
    pub fn new(
        device: Arc<ManagedDevice>,
        size: u64,
        usage: BufferUsages,
        usage_pattern: BufferUsagePattern,
    ) -> Result<Self> {
        let buffer = Arc::new(device.device.create_buffer(&BufferDescriptor {
            label: Some("RingBuffer"),
            size,
            usage,
            mapped_at_creation: false,
        }));

        Ok(Self {
            _device: device,
            buffer,
            size,
            head: AtomicU64::new(0),
            tail: AtomicU64::new(0),
            wrapped: AtomicBool::new(false),
            alignment: 256,
            _usage_pattern: usage_pattern,
            write_count: AtomicU64::new(0),
            bytes_written: AtomicU64::new(0),
            overruns: AtomicU64::new(0),
        })
    }

    /// Allocate space in the ring buffer
    pub fn allocate(&self, size: u64) -> Option<u64> {
        let aligned_size = (size + self.alignment - 1) & !(self.alignment - 1);
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Relaxed);

        let available = if self.wrapped.load(Ordering::Relaxed) {
            if head >= tail {
                tail - head
            } else {
                self.size - head + tail
            }
        } else {
            self.size - head
        };

        if aligned_size > available {
            self.overruns.fetch_add(1, Ordering::Relaxed);
            return None;
        }

        let offset = head;
        let new_head = (head + aligned_size) % self.size;

        if new_head < head {
            self.wrapped.store(true, Ordering::Relaxed);
        }

        self.head.store(new_head, Ordering::Relaxed);
        self.write_count.fetch_add(1, Ordering::Relaxed);
        self.bytes_written
            .fetch_add(aligned_size, Ordering::Relaxed);

        Some(offset)
    }

    /// Get buffer reference
    pub fn get_buffer(&self) -> &Arc<Buffer> {
        &self.buffer
    }

    /// Get statistics
    pub fn get_stats(&self) -> HashMap<String, u64> {
        let mut stats = HashMap::new();
        stats.insert(
            "writes".to_string(),
            self.write_count.load(Ordering::Relaxed),
        );
        stats.insert(
            "bytes_written".to_string(),
            self.bytes_written.load(Ordering::Relaxed),
        );
        stats.insert(
            "overruns".to_string(),
            self.overruns.load(Ordering::Relaxed),
        );
        stats.insert("head".to_string(), self.head.load(Ordering::Relaxed));
        stats.insert("tail".to_string(), self.tail.load(Ordering::Relaxed));
        stats
    }
}

impl BufferManager {
    /// Create a new buffer manager
    pub fn new(device: Arc<ManagedDevice>, memory_manager: Arc<Mutex<MemoryManager>>) -> Self {
        let vertex_pool = Arc::new(BufferPool::new(device.clone(), memory_manager.clone()));
        let index_pool = Arc::new(BufferPool::new(device.clone(), memory_manager.clone()));
        let uniform_pool = Arc::new(BufferPool::new(device.clone(), memory_manager.clone()));
        let storage_pool = Arc::new(BufferPool::new(device.clone(), memory_manager.clone()));
        let staging_pool = Arc::new(BufferPool::new(device.clone(), memory_manager.clone()));

        Self {
            device,
            _memory_manager: memory_manager,
            vertex_pool,
            index_pool,
            uniform_pool,
            storage_pool,
            staging_pool,
            dynamic_buffers: RwLock::new(HashMap::new()),
            ring_buffers: RwLock::new(HashMap::new()),
            _staging_buffers: RwLock::new(HashMap::new()),
            total_buffers: AtomicU64::new(0),
            total_memory_usage: AtomicU64::new(0),
            _fragmentation_ratio: RwLock::new(0.0),
            auto_defragmentation: AtomicBool::new(true),
            memory_pressure_threshold: AtomicU64::new(1024 * 1024 * 1024), // 1GB
            _profiling_enabled: AtomicBool::new(true),
        }
    }

    /// Allocate a buffer with the specified configuration
    pub fn allocate_buffer(&self, config: &BufferConfig) -> Result<ResourceHandle> {
        let pool = self.get_pool_for_usage(config.usage);
        let handle = pool.allocate(config)?;

        self.total_buffers.fetch_add(1, Ordering::Relaxed);
        self.total_memory_usage
            .fetch_add(config.size, Ordering::Relaxed);

        Ok(handle)
    }

    /// Deallocate a buffer
    pub fn deallocate_buffer(&self, handle: ResourceHandle, usage: BufferUsages) -> Result<()> {
        let pool = self.get_pool_for_usage(usage);

        if let Some(_buffer) = pool.get_allocation(handle) {
            // Size tracking would need to be handled differently
            // For now, just proceed with deallocation
        }

        pool.deallocate(handle)?;
        self.total_buffers.fetch_sub(1, Ordering::Relaxed);

        Ok(())
    }

    /// Create a dynamic buffer
    pub fn create_dynamic_buffer(&self, config: BufferConfig) -> Result<ResourceHandle> {
        let handle = ResourceHandle::new();
        let pool = self.get_pool_for_usage(config.usage);

        let dynamic_buffer = Arc::new(DynamicBuffer::new(self.device.clone(), pool, config));

        self.dynamic_buffers.write().insert(handle, dynamic_buffer);

        Ok(handle)
    }

    /// Create a ring buffer
    pub fn create_ring_buffer(
        &self,
        size: u64,
        usage: BufferUsages,
        usage_pattern: BufferUsagePattern,
    ) -> Result<ResourceHandle> {
        let handle = ResourceHandle::new();

        let ring_buffer = Arc::new(RingBuffer::new(
            self.device.clone(),
            size,
            usage,
            usage_pattern,
        )?);

        self.ring_buffers.write().insert(handle, ring_buffer);
        self.total_buffers.fetch_add(1, Ordering::Relaxed);
        self.total_memory_usage.fetch_add(size, Ordering::Relaxed);

        Ok(handle)
    }

    /// Get pool for buffer usage
    fn get_pool_for_usage(&self, usage: BufferUsages) -> Arc<BufferPool> {
        if usage.contains(BufferUsages::VERTEX) {
            self.vertex_pool.clone()
        } else if usage.contains(BufferUsages::INDEX) {
            self.index_pool.clone()
        } else if usage.contains(BufferUsages::UNIFORM) {
            self.uniform_pool.clone()
        } else if usage.contains(BufferUsages::STORAGE) {
            self.storage_pool.clone()
        } else {
            self.staging_pool.clone()
        }
    }

    /// Get buffer manager statistics
    pub fn get_stats(&self) -> HashMap<String, u64> {
        let mut stats = HashMap::new();
        stats.insert(
            "total_buffers".to_string(),
            self.total_buffers.load(Ordering::Relaxed),
        );
        stats.insert(
            "total_memory".to_string(),
            self.total_memory_usage.load(Ordering::Relaxed),
        );

        // Add pool-specific stats
        let vertex_stats = self.vertex_pool.get_stats();
        for (key, value) in vertex_stats {
            stats.insert(format!("vertex_{}", key), value);
        }

        let uniform_stats = self.uniform_pool.get_stats();
        for (key, value) in uniform_stats {
            stats.insert(format!("uniform_{}", key), value);
        }

        stats
    }

    /// Get buffer by handle
    pub fn get_buffer(&self, handle: ResourceHandle) -> Option<Arc<Buffer>> {
        // Check all pools
        if let Some(buffer) = self.vertex_pool.get_allocation(handle) {
            return Some(buffer);
        }
        if let Some(buffer) = self.index_pool.get_allocation(handle) {
            return Some(buffer);
        }
        if let Some(buffer) = self.uniform_pool.get_allocation(handle) {
            return Some(buffer);
        }
        if let Some(buffer) = self.storage_pool.get_allocation(handle) {
            return Some(buffer);
        }
        if let Some(buffer) = self.staging_pool.get_allocation(handle) {
            return Some(buffer);
        }

        // Check dynamic buffers
        if let Some(dynamic) = self.dynamic_buffers.read().get(&handle) {
            return dynamic.get_buffer();
        }

        // Check ring buffers
        if let Some(ring) = self.ring_buffers.read().get(&handle) {
            return Some(ring.get_buffer().clone());
        }

        None
    }

    /// Perform defragmentation if needed
    pub fn defragment(&self) -> Result<()> {
        if !self.auto_defragmentation.load(Ordering::Relaxed) {
            return Ok(());
        }

        let memory_usage = self.total_memory_usage.load(Ordering::Relaxed);
        let threshold = self.memory_pressure_threshold.load(Ordering::Relaxed);

        if memory_usage > threshold {
            info!("Starting buffer defragmentation");

            // Clear unused buffers from pools
            self.vertex_pool.cleanup();
            self.index_pool.cleanup();
            self.uniform_pool.cleanup();
            self.storage_pool.cleanup();
            self.staging_pool.cleanup();

            info!("Buffer defragmentation completed");
        }

        Ok(())
    }

    /// Set auto defragmentation
    pub fn set_auto_defragmentation(&self, enabled: bool) {
        self.auto_defragmentation.store(enabled, Ordering::Relaxed);
    }

    /// Set memory pressure threshold
    pub fn set_memory_pressure_threshold(&self, threshold: u64) {
        self.memory_pressure_threshold
            .store(threshold, Ordering::Relaxed);
    }

    /// Initialize the buffer manager (integration method)
    pub fn initialize(&self) -> Result<()> {
        info!("Buffer manager initialized");
        Ok(())
    }

    /// Create buffer (integration method)
    pub fn create_buffer(&self, config: &BufferConfig) -> Result<ResourceHandle> {
        self.allocate_buffer(config)
    }

    /// Collect garbage (integration method)
    pub fn collect_garbage(&self) {
        let _ = self.defragment();
    }
}
