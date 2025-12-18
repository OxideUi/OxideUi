//! Advanced GPU memory management system
//!
//! This module provides sophisticated memory management including:
//! - Multi-tier memory allocators with different strategies
//! - Dynamic buffer pooling with usage pattern analysis
//! - Memory pressure detection and adaptive allocation
//! - Fragmentation analysis and defragmentation strategies
//! - Memory bandwidth optimization
//! - Resource streaming and prefetching
//! - Memory usage profiling and analytics

use crate::device::{ManagedDevice, OptimizationHints};
use anyhow::{Context, Result};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering as CmpOrdering;
use std::collections::{BTreeMap, BinaryHeap, HashMap};
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc,
};
use std::time::{Duration, Instant};
use strato_core::{logging::LogCategory, strato_debug, strato_error_rate_limited, strato_warn};
use tracing::{debug, info, instrument, warn};
use wgpu::{Buffer, BufferDescriptor, BufferUsages, Device};

/// Memory allocation strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AllocationStrategy {
    /// Best fit - minimize wasted space
    BestFit,
    /// First fit - fastest allocation
    FirstFit,
    /// Buddy system - good for power-of-2 sizes
    Buddy,
    /// Slab allocation - for fixed-size objects
    Slab,
    /// Linear allocation - for temporary resources
    Linear,
    /// Balanced approach - compromise between speed and fragmentation
    Balanced,
}

/// Memory usage pattern classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UsagePattern {
    /// Short-lived resources (< 1 frame)
    Transient,
    /// Frame-persistent resources (1-10 frames)
    FramePersistent,
    /// Long-lived resources (> 10 frames)
    Persistent,
    /// Static resources (never change)
    Static,
    /// Streaming resources (loaded on demand)
    Streaming,
}

/// Memory tier classification based on access patterns
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum MemoryTier {
    /// High-speed GPU memory for frequently accessed data
    HighSpeed = 0,
    /// Standard GPU memory for regular resources
    Standard = 1,
    /// Shared memory for CPU-GPU communication
    Shared = 2,
    /// System memory for rarely accessed data
    System = 3,
}

/// Memory block descriptor
#[derive(Debug)]
pub struct MemoryBlock {
    pub buffer: Arc<Buffer>,
    pub size: u64,
    pub offset: u64,
    pub alignment: u64,
    pub usage: BufferUsages,
    pub tier: MemoryTier,
    pub allocation_time: Instant,
    pub last_access: Instant,
    pub access_count: AtomicU64,
    pub is_mapped: AtomicBool,
}

/// Free memory region
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FreeRegion {
    pub offset: u64,
    pub size: u64,
}

impl PartialOrd for FreeRegion {
    fn partial_cmp(&self, other: &Self) -> Option<CmpOrdering> {
        Some(self.cmp(other))
    }
}

impl Ord for FreeRegion {
    fn cmp(&self, other: &Self) -> CmpOrdering {
        // For best-fit allocation with BinaryHeap (max-heap), invert size comparison
        other
            .size
            .cmp(&self.size)
            .then(self.offset.cmp(&other.offset))
    }
}

/// Memory pool for a specific usage pattern and size range
pub struct MemoryPool {
    pub usage_pattern: UsagePattern,
    pub tier: MemoryTier,
    pub min_block_size: u64,
    pub max_block_size: u64,
    pub allocation_strategy: AllocationStrategy,
    pub blocks: Vec<Arc<MemoryBlock>>,
    pub free_regions: BinaryHeap<FreeRegion>,
    pub allocated_regions: BTreeMap<u64, u64>, // offset -> size
    pub total_size: AtomicU64,
    pub used_size: AtomicU64,
    pub allocation_count: AtomicU64,
    pub deallocation_count: AtomicU64,
    pub fragmentation_ratio: AtomicU64, // Fixed point (x1000)
    pub last_defrag: RwLock<Instant>,
}

impl MemoryPool {
    /// Create a new memory pool
    pub fn new(
        usage_pattern: UsagePattern,
        tier: MemoryTier,
        min_block_size: u64,
        max_block_size: u64,
        allocation_strategy: AllocationStrategy,
    ) -> Self {
        Self {
            usage_pattern,
            tier,
            min_block_size,
            max_block_size,
            allocation_strategy,
            blocks: Vec::new(),
            free_regions: BinaryHeap::new(),
            allocated_regions: BTreeMap::new(),
            total_size: AtomicU64::new(0),
            used_size: AtomicU64::new(0),
            allocation_count: AtomicU64::new(0),
            deallocation_count: AtomicU64::new(0),
            fragmentation_ratio: AtomicU64::new(0),
            last_defrag: RwLock::new(Instant::now()),
        }
    }

    /// Allocate memory from this pool
    pub fn allocate(
        &mut self,
        size: u64,
        alignment: u64,
        device: &Device,
    ) -> Result<Arc<MemoryBlock>> {
        let aligned_size = Self::align_size(size, alignment);

        // Try to find a suitable free region
        if let Some(region) = self.find_free_region(aligned_size, alignment) {
            return self.allocate_from_region(region, aligned_size, alignment, device);
        }

        // Need to create a new block
        self.create_new_block(aligned_size, alignment, device)
    }

    /// Find a suitable free region
    fn find_free_region(&mut self, size: u64, alignment: u64) -> Option<FreeRegion> {
        let mut best_region: Option<FreeRegion> = None;
        let mut temp_regions = Vec::new();

        // Extract regions and find the best fit
        while let Some(region) = self.free_regions.pop() {
            let aligned_offset = Self::align_offset(region.offset, alignment);
            let required_size = aligned_offset - region.offset + size;

            if region.size >= required_size {
                if best_region.is_none() || region.size < best_region.as_ref().unwrap().size {
                    if let Some(prev_best) = best_region.take() {
                        temp_regions.push(prev_best);
                    }
                    best_region = Some(region);
                } else {
                    temp_regions.push(region);
                }
            } else {
                temp_regions.push(region);
            }
        }

        // Put back the regions we didn't use
        for region in temp_regions {
            self.free_regions.push(region);
        }

        best_region
    }

    /// Allocate from an existing free region
    fn allocate_from_region(
        &mut self,
        region: FreeRegion,
        size: u64,
        alignment: u64,
        _device: &Device,
    ) -> Result<Arc<MemoryBlock>> {
        let aligned_offset = Self::align_offset(region.offset, alignment);
        let padding = aligned_offset - region.offset;

        // Create padding region if needed
        if padding > 0 {
            self.free_regions.push(FreeRegion {
                offset: region.offset,
                size: padding,
            });
        }

        // Create remaining region if any
        let remaining_size = region.size - padding - size;
        if remaining_size > 0 {
            self.free_regions.push(FreeRegion {
                offset: aligned_offset + size,
                size: remaining_size,
            });
        }

        // Find the buffer that contains this region
        let buffer = self.find_buffer_for_offset(aligned_offset)?;

        let block = Arc::new(MemoryBlock {
            buffer,
            size,
            offset: aligned_offset,
            alignment,
            usage: self.get_buffer_usage(),
            tier: self.tier,
            allocation_time: Instant::now(),
            last_access: Instant::now(),
            access_count: AtomicU64::new(0),
            is_mapped: AtomicBool::new(false),
        });

        self.allocated_regions.insert(aligned_offset, size);
        self.used_size.fetch_add(size, Ordering::Relaxed);
        self.allocation_count.fetch_add(1, Ordering::Relaxed);

        Ok(block)
    }

    /// Create a new buffer block
    fn create_new_block(
        &mut self,
        size: u64,
        alignment: u64,
        device: &Device,
    ) -> Result<Arc<MemoryBlock>> {
        let block_size = std::cmp::max(size, self.min_block_size);
        let block_size = std::cmp::min(block_size, self.max_block_size);

        let buffer = Arc::new(device.create_buffer(&BufferDescriptor {
            label: Some(&format!(
                "MemoryPool-{:?}-{:?}",
                self.usage_pattern, self.tier
            )),
            size: block_size,
            usage: self.get_buffer_usage(),
            mapped_at_creation: false,
        }));

        let block = Arc::new(MemoryBlock {
            buffer,
            size,
            offset: 0,
            alignment,
            usage: self.get_buffer_usage(),
            tier: self.tier,
            allocation_time: Instant::now(),
            last_access: Instant::now(),
            access_count: AtomicU64::new(0),
            is_mapped: AtomicBool::new(false),
        });

        // Add remaining space to free regions
        if block_size > size {
            self.free_regions.push(FreeRegion {
                offset: size,
                size: block_size - size,
            });
        }

        self.blocks.push(block.clone());
        self.allocated_regions.insert(0, size);
        self.total_size.fetch_add(block_size, Ordering::Relaxed);
        self.used_size.fetch_add(size, Ordering::Relaxed);
        self.allocation_count.fetch_add(1, Ordering::Relaxed);

        Ok(block)
    }

    /// Deallocate a memory block
    pub fn deallocate(&mut self, block: &MemoryBlock) {
        if let Some(size) = self.allocated_regions.remove(&block.offset) {
            self.free_regions.push(FreeRegion {
                offset: block.offset,
                size,
            });

            self.used_size.fetch_sub(size, Ordering::Relaxed);
            self.deallocation_count.fetch_add(1, Ordering::Relaxed);

            // Coalesce adjacent free regions
            self.coalesce_free_regions();
        }
    }

    /// Coalesce adjacent free regions to reduce fragmentation
    fn coalesce_free_regions(&mut self) {
        let mut regions: Vec<_> = self.free_regions.drain().collect();
        regions.sort_by_key(|r| r.offset);

        let mut coalesced = Vec::new();
        let mut current: Option<FreeRegion> = None;

        for region in regions {
            match current.take() {
                None => current = Some(region),
                Some(mut prev) => {
                    if prev.offset + prev.size == region.offset {
                        // Adjacent regions, merge them
                        prev.size += region.size;
                        current = Some(prev);
                    } else {
                        // Not adjacent, keep previous and start new
                        coalesced.push(prev);
                        current = Some(region);
                    }
                }
            }
        }

        if let Some(last) = current {
            coalesced.push(last);
        }

        self.free_regions = coalesced.into_iter().collect();
    }

    /// Calculate fragmentation ratio
    pub fn calculate_fragmentation(&self) -> f32 {
        let total = self.total_size.load(Ordering::Relaxed);
        if total == 0 {
            return 0.0;
        }

        let _used = self.used_size.load(Ordering::Relaxed);
        let free_regions = self.free_regions.len();

        // Fragmentation increases with number of free regions and decreases with usage
        let fragmentation = (free_regions as f32 * 100.0) / (total as f32 / 1024.0);
        fragmentation.min(100.0)
    }

    /// Get buffer usage flags for this pool
    fn get_buffer_usage(&self) -> BufferUsages {
        match self.usage_pattern {
            UsagePattern::Transient => {
                BufferUsages::VERTEX | BufferUsages::INDEX | BufferUsages::COPY_DST
            }
            UsagePattern::FramePersistent => {
                BufferUsages::UNIFORM | BufferUsages::STORAGE | BufferUsages::COPY_DST
            }
            UsagePattern::Persistent => {
                BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC
            }
            UsagePattern::Static => BufferUsages::VERTEX | BufferUsages::INDEX,
            UsagePattern::Streaming => BufferUsages::COPY_DST | BufferUsages::COPY_SRC,
        }
    }

    /// Find buffer that contains the given offset
    fn find_buffer_for_offset(&self, _offset: u64) -> Result<Arc<Buffer>> {
        // This is a simplified implementation
        // In practice, you'd need to track which buffer contains which offset range
        self.blocks
            .first()
            .map(|block| block.buffer.clone())
            .context("No buffer available for offset")
    }

    /// Align size to the given alignment
    fn align_size(size: u64, alignment: u64) -> u64 {
        (size + alignment - 1) & !(alignment - 1)
    }

    /// Align offset to the given alignment
    fn align_offset(offset: u64, alignment: u64) -> u64 {
        (offset + alignment - 1) & !(alignment - 1)
    }
}

/// Advanced memory manager with multiple allocation strategies
pub struct MemoryManager {
    device: Arc<ManagedDevice>,
    pools: HashMap<(UsagePattern, MemoryTier), MemoryPool>,
    allocation_stats: RwLock<AllocationStats>,
    memory_pressure_threshold: AtomicU64,
    auto_defrag_enabled: AtomicBool,
    last_cleanup: RwLock<Instant>,
    optimization_hints: OptimizationHints,
}

/// Memory allocation statistics
#[derive(Debug, Clone, Default)]
pub struct AllocationStats {
    pub total_allocated: u64,
    pub total_freed: u64,
    pub peak_usage: u64,
    pub current_usage: u64,
    pub allocation_count: u64,
    pub deallocation_count: u64,
    pub failed_allocations: u64,
    pub defragmentation_count: u64,
    pub average_fragmentation: f32,
}

impl MemoryManager {
    /// Create a new memory manager
    pub fn new(device: Arc<ManagedDevice>) -> Self {
        let optimization_hints = device.optimization_hints.clone();

        let mut pools = HashMap::new();

        // Create pools for different usage patterns and tiers
        for &pattern in &[
            UsagePattern::Transient,
            UsagePattern::FramePersistent,
            UsagePattern::Persistent,
            UsagePattern::Static,
            UsagePattern::Streaming,
        ] {
            for &tier in &[
                MemoryTier::HighSpeed,
                MemoryTier::Standard,
                MemoryTier::Shared,
            ] {
                let (min_size, max_size, strategy) =
                    Self::get_pool_config(pattern, tier, &optimization_hints);

                pools.insert(
                    (pattern, tier),
                    MemoryPool::new(pattern, tier, min_size, max_size, strategy),
                );
            }
        }

        Self {
            device,
            pools,
            allocation_stats: RwLock::new(AllocationStats::default()),
            memory_pressure_threshold: AtomicU64::new(1024 * 1024 * 1024), // 1GB
            auto_defrag_enabled: AtomicBool::new(true),
            last_cleanup: RwLock::new(Instant::now()),
            optimization_hints,
        }
    }

    /// Get pool configuration for usage pattern and tier
    fn get_pool_config(
        pattern: UsagePattern,
        tier: MemoryTier,
        hints: &OptimizationHints,
    ) -> (u64, u64, AllocationStrategy) {
        let base_alignment = hints.preferred_buffer_alignment as u64;

        match (pattern, tier) {
            (UsagePattern::Transient, _) => (
                4 * 1024,         // 4KB min
                16 * 1024 * 1024, // 16MB max
                AllocationStrategy::Linear,
            ),
            (UsagePattern::FramePersistent, MemoryTier::HighSpeed) => (
                64 * 1024,        // 64KB min
                64 * 1024 * 1024, // 64MB max
                AllocationStrategy::BestFit,
            ),
            (UsagePattern::Persistent, _) => (
                1024 * 1024,       // 1MB min
                256 * 1024 * 1024, // 256MB max
                AllocationStrategy::Buddy,
            ),
            (UsagePattern::Static, _) => (
                base_alignment,     // Alignment min
                1024 * 1024 * 1024, // 1GB max
                AllocationStrategy::FirstFit,
            ),
            (UsagePattern::Streaming, _) => (
                1024 * 1024,       // 1MB min
                128 * 1024 * 1024, // 128MB max
                AllocationStrategy::Slab,
            ),
            _ => (
                base_alignment,
                64 * 1024 * 1024,
                AllocationStrategy::BestFit,
            ),
        }
    }

    /// Allocate memory with specific usage pattern
    #[instrument(skip(self))]
    pub fn allocate(
        &mut self,
        size: u64,
        alignment: u64,
        usage_pattern: UsagePattern,
        tier: MemoryTier,
    ) -> Result<Arc<MemoryBlock>> {
        let pool_key = (usage_pattern, tier);

        // Get pool reference and try allocation
        let allocation_result = {
            let pool = self
                .pools
                .get_mut(&pool_key)
                .context("Pool not found for usage pattern and tier")?;
            pool.allocate(size, alignment, &self.device.device)
        };

        match allocation_result {
            Ok(block) => {
                let mut stats = self.allocation_stats.write();
                stats.total_allocated += size;
                stats.current_usage += size;
                stats.allocation_count += 1;
                stats.peak_usage = stats.peak_usage.max(stats.current_usage);

                Ok(block)
            }
            Err(e) => {
                let mut stats = self.allocation_stats.write();
                stats.failed_allocations += 1;

                // Try memory pressure relief
                if self.auto_defrag_enabled.load(Ordering::Relaxed) {
                    drop(stats);
                    self.relieve_memory_pressure()?;

                    // Retry allocation
                    let pool = self
                        .pools
                        .get_mut(&pool_key)
                        .context("Pool not found for usage pattern and tier")?;
                    pool.allocate(size, alignment, &self.device.device)
                } else {
                    Err(e)
                }
            }
        }
    }

    /// Deallocate memory block
    pub fn deallocate(&mut self, block: Arc<MemoryBlock>) {
        let key = (self.classify_usage_pattern(&block), block.tier);

        if let Some(pool) = self.pools.get_mut(&key) {
            let size = block.size;
            pool.deallocate(&block);

            let mut stats = self.allocation_stats.write();
            stats.total_freed += size;
            stats.current_usage = stats.current_usage.saturating_sub(size);
            stats.deallocation_count += 1;
        }
    }

    /// Classify usage pattern from block characteristics
    fn classify_usage_pattern(&self, block: &MemoryBlock) -> UsagePattern {
        let age = block.allocation_time.elapsed();
        let access_count = block.access_count.load(Ordering::Relaxed);

        if age < Duration::from_millis(16) {
            UsagePattern::Transient
        } else if age < Duration::from_millis(160) && access_count > 10 {
            UsagePattern::FramePersistent
        } else if access_count == 0 {
            UsagePattern::Static
        } else {
            UsagePattern::Persistent
        }
    }

    /// Relieve memory pressure through cleanup and defragmentation
    #[instrument(skip(self))]
    pub fn relieve_memory_pressure(&mut self) -> Result<()> {
        info!("Relieving memory pressure...");

        // Cleanup unused resources
        self.cleanup_unused_resources();

        // Defragment pools with high fragmentation
        let pools_to_defrag: Vec<_> = self
            .pools
            .values_mut()
            .filter_map(|pool| {
                let fragmentation = pool.calculate_fragmentation();
                if fragmentation > 50.0 {
                    Some(pool as *mut MemoryPool)
                } else {
                    None
                }
            })
            .collect();

        for pool_ptr in pools_to_defrag {
            unsafe {
                self.defragment_pool(&mut *pool_ptr)?;
            }
        }

        let mut stats = self.allocation_stats.write();
        stats.defragmentation_count += 1;

        Ok(())
    }

    /// Cleanup unused resources
    fn cleanup_unused_resources(&mut self) {
        let now = Instant::now();
        let cleanup_threshold = Duration::from_secs(30);

        for pool in self.pools.values_mut() {
            pool.blocks.retain(|block| {
                let age = now.duration_since(block.last_access);
                let access_count = block.access_count.load(Ordering::Relaxed);

                // Keep blocks that are recently accessed or frequently used
                age < cleanup_threshold || access_count > 100
            });
        }

        *self.last_cleanup.write() = now;
    }

    /// Defragment a memory pool
    fn defragment_pool(&mut self, pool: &mut MemoryPool) -> Result<()> {
        debug!(
            "Defragmenting pool: {:?}-{:?}",
            pool.usage_pattern, pool.tier
        );

        // Coalesce free regions
        pool.coalesce_free_regions();

        // Update fragmentation ratio
        let fragmentation = pool.calculate_fragmentation();
        pool.fragmentation_ratio
            .store((fragmentation * 1000.0) as u64, Ordering::Relaxed);

        *pool.last_defrag.write() = Instant::now();

        Ok(())
    }

    /// Get memory statistics
    pub fn get_stats(&self) -> AllocationStats {
        let mut stats = self.allocation_stats.read().clone();

        // Calculate average fragmentation across all pools
        let total_fragmentation: f32 = self
            .pools
            .values()
            .map(|pool| pool.calculate_fragmentation())
            .sum();

        stats.average_fragmentation = if self.pools.is_empty() {
            0.0
        } else {
            total_fragmentation / self.pools.len() as f32
        };

        stats
    }

    /// Check if memory pressure relief is needed
    pub fn needs_pressure_relief(&self) -> bool {
        let stats = self.allocation_stats.read();
        let threshold = self.memory_pressure_threshold.load(Ordering::Relaxed);

        stats.current_usage > threshold || stats.average_fragmentation > 75.0
    }

    /// Set memory pressure threshold
    pub fn set_pressure_threshold(&self, threshold: u64) {
        self.memory_pressure_threshold
            .store(threshold, Ordering::Relaxed);
    }

    /// Enable or disable automatic defragmentation
    pub fn set_auto_defrag(&self, enabled: bool) {
        self.auto_defrag_enabled.store(enabled, Ordering::Relaxed);
    }

    /// Get total allocated memory (integration method)
    pub fn get_total_allocated(&self) -> u64 {
        let stats = self.get_stats();
        stats.total_allocated
    }

    /// Defragment memory (integration method)
    pub fn defragment(&mut self) -> Result<()> {
        self.relieve_memory_pressure()
    }

    /// Cleanup memory (integration method)
    pub fn cleanup(&mut self) -> Result<()> {
        self.relieve_memory_pressure()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_free_region_ordering() {
        let mut regions = BinaryHeap::new();

        regions.push(FreeRegion {
            offset: 100,
            size: 50,
        });
        regions.push(FreeRegion {
            offset: 0,
            size: 100,
        });
        regions.push(FreeRegion {
            offset: 200,
            size: 25,
        });

        // Should pop smallest size first (best fit)
        assert_eq!(regions.pop().unwrap().size, 25);
        assert_eq!(regions.pop().unwrap().size, 50);
        assert_eq!(regions.pop().unwrap().size, 100);
    }

    #[test]
    fn test_alignment() {
        assert_eq!(MemoryPool::align_size(100, 256), 256);
        assert_eq!(MemoryPool::align_size(256, 256), 256);
        assert_eq!(MemoryPool::align_size(257, 256), 512);

        assert_eq!(MemoryPool::align_offset(100, 256), 256);
        assert_eq!(MemoryPool::align_offset(256, 256), 256);
        assert_eq!(MemoryPool::align_offset(257, 256), 512);
    }
}
