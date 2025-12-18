//! Advanced GPU device management system
//!
//! This module provides GPU device management including:
//! - Multi-adapter support with intelligent selection
//! - Hardware capability detection and optimization
//! - Automatic fallback mechanisms for compatibility
//! - Device loss recovery and hot-swapping
//! - Power management and thermal monitoring
//! - Vendor-specific optimizations (NVIDIA, AMD, Intel, Apple)

use anyhow::{bail, Result};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc,
};
use std::time::{Duration, Instant};
use strato_core::{logging::LogCategory, strato_debug, strato_error_rate_limited, strato_warn};
use tracing::{debug, info, instrument, warn};
use wgpu::{
    Adapter, Backends, Device, DeviceDescriptor, DeviceType, Dx12Compiler, Features,
    Gles3MinorVersion, Instance, InstanceDescriptor, InstanceFlags, Limits, PowerPreference, Queue,
    RequestAdapterOptions, RequestDeviceError, Surface, SurfaceConfiguration,
};

/// GPU vendor identification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GpuVendor {
    Nvidia,
    Amd,
    Intel,
    Apple,
    Qualcomm,
    ARM,
    Unknown,
}

impl From<u32> for GpuVendor {
    fn from(vendor_id: u32) -> Self {
        match vendor_id {
            0x10DE => GpuVendor::Nvidia,
            0x1002 | 0x1022 => GpuVendor::Amd,
            0x8086 => GpuVendor::Intel,
            0x106B => GpuVendor::Apple,
            0x5143 => GpuVendor::Qualcomm,
            0x13B5 => GpuVendor::ARM,
            _ => GpuVendor::Unknown,
        }
    }
}

/// GPU performance tier classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PerformanceTier {
    Integrated = 0,
    Entry = 1,
    Mainstream = 2,
    HighEnd = 3,
    Enthusiast = 4,
    Professional = 5,
}

/// AdapterInfo re-export for public API
pub use wgpu::AdapterInfo;

/// Comprehensive GPU capabilities and characteristics
#[derive(Debug, Clone)]
pub struct GpuCapabilities {
    pub vendor: GpuVendor,
    pub device_name: String,
    pub device_id: u32,
    pub vendor_id: u32,
    pub performance_tier: PerformanceTier,
    pub memory_size: u64,
    pub memory_bandwidth: Option<u64>,
    pub compute_units: Option<u32>,
    pub base_clock: Option<u32>,
    pub boost_clock: Option<u32>,
    pub supports_ray_tracing: bool,
    pub supports_mesh_shaders: bool,
    pub supports_variable_rate_shading: bool,
    pub max_texture_size: u32,
    pub max_texture_array_layers: u32,
    pub max_bind_groups: u32,
    pub max_dynamic_uniform_buffers: u32,
    pub max_storage_buffers: u32,
    pub max_sampled_textures: u32,
    pub max_samplers: u32,
    pub max_storage_textures: u32,
    pub max_vertex_buffers: u32,
    pub max_vertex_attributes: u32,
    pub max_push_constant_size: u32,
    pub timestamp_period: f32,
    pub supported_features: Features,
    pub limits: Limits,
}

impl GpuCapabilities {
    /// Create capabilities from adapter info and limits
    pub fn from_adapter(adapter: &Adapter) -> Self {
        let info = adapter.get_info();
        let limits = adapter.limits();
        let features = adapter.features();

        let vendor = GpuVendor::from(info.vendor);
        let performance_tier = Self::classify_performance_tier(&info, &limits);

        Self {
            vendor,
            device_name: info.name.clone(),
            device_id: info.device,
            vendor_id: info.vendor,
            performance_tier,
            memory_size: Self::estimate_memory_size(&info, &limits),
            memory_bandwidth: Self::estimate_memory_bandwidth(&info, &limits),
            compute_units: Self::estimate_compute_units(&info),
            base_clock: None, // Would need vendor-specific APIs
            boost_clock: None,
            supports_ray_tracing: features.contains(Features::RAY_TRACING_ACCELERATION_STRUCTURE),
            supports_mesh_shaders: false, // Features::EXPERIMENTAL_FEATURES removed in newer wgpu
            supports_variable_rate_shading: false, // Not exposed in wgpu yet
            max_texture_size: limits.max_texture_dimension_2d,
            max_texture_array_layers: limits.max_texture_array_layers,
            max_bind_groups: limits.max_bind_groups,
            max_dynamic_uniform_buffers: limits.max_dynamic_uniform_buffers_per_pipeline_layout,
            max_storage_buffers: limits.max_storage_buffers_per_shader_stage,
            max_sampled_textures: limits.max_sampled_textures_per_shader_stage,
            max_samplers: limits.max_samplers_per_shader_stage,
            max_storage_textures: limits.max_storage_textures_per_shader_stage,
            max_vertex_buffers: limits.max_vertex_buffers,
            max_vertex_attributes: limits.max_vertex_attributes,
            max_push_constant_size: limits.max_push_constant_size,
            timestamp_period: 1.0, // timestamp_period field removed, using default
            supported_features: features,
            limits,
        }
    }

    fn classify_performance_tier(info: &AdapterInfo, limits: &Limits) -> PerformanceTier {
        let memory_score = (limits.max_buffer_size / (1024 * 1024 * 1024)) as u32; // GB
        let compute_score =
            limits.max_compute_workgroup_size_x * limits.max_compute_workgroup_size_y;

        match info.device_type {
            DeviceType::DiscreteGpu => {
                if memory_score >= 16 && compute_score >= 1024 * 1024 {
                    PerformanceTier::Professional
                } else if memory_score >= 8 && compute_score >= 512 * 512 {
                    PerformanceTier::Enthusiast
                } else if memory_score >= 4 {
                    PerformanceTier::HighEnd
                } else {
                    PerformanceTier::Mainstream
                }
            }
            DeviceType::IntegratedGpu => {
                if memory_score >= 4 {
                    PerformanceTier::Mainstream
                } else {
                    PerformanceTier::Integrated
                }
            }
            DeviceType::VirtualGpu => PerformanceTier::Entry,
            DeviceType::Cpu => PerformanceTier::Entry,
            DeviceType::Other => PerformanceTier::Entry,
        }
    }

    fn estimate_memory_size(info: &AdapterInfo, limits: &Limits) -> u64 {
        // Rough estimation based on buffer limits and device type
        match info.device_type {
            DeviceType::DiscreteGpu => {
                let buffer_limit = limits.max_buffer_size;
                // Discrete GPUs typically have dedicated VRAM
                std::cmp::min(buffer_limit, 32 * 1024 * 1024 * 1024) // Cap at 32GB
            }
            DeviceType::IntegratedGpu => {
                // Integrated GPUs share system memory
                std::cmp::min(limits.max_buffer_size, 8 * 1024 * 1024 * 1024) // Cap at 8GB
            }
            _ => limits.max_buffer_size,
        }
    }

    fn estimate_memory_bandwidth(_info: &AdapterInfo, _limits: &Limits) -> Option<u64> {
        // Would need vendor-specific APIs or lookup tables
        None
    }

    fn estimate_compute_units(info: &AdapterInfo) -> Option<u32> {
        // Would need vendor-specific detection
        match GpuVendor::from(info.vendor) {
            GpuVendor::Nvidia => {
                // Could parse device name for SM count
                None
            }
            GpuVendor::Amd => {
                // Could parse device name for CU count
                None
            }
            _ => None,
        }
    }

    /// Get vendor-specific optimization hints
    pub fn get_optimization_hints(&self) -> OptimizationHints {
        match self.vendor {
            GpuVendor::Nvidia => OptimizationHints {
                preferred_workgroup_size: (32, 1, 1), // Warp size
                prefers_texture_arrays: true,
                supports_async_compute: true,
                memory_coalescing_alignment: 128,
                preferred_buffer_alignment: 256,
                supports_fast_math: true,
            },
            GpuVendor::Amd => OptimizationHints {
                preferred_workgroup_size: (64, 1, 1), // Wavefront size
                prefers_texture_arrays: true,
                supports_async_compute: true,
                memory_coalescing_alignment: 256,
                preferred_buffer_alignment: 256,
                supports_fast_math: true,
            },
            GpuVendor::Intel => OptimizationHints {
                preferred_workgroup_size: (16, 1, 1), // EU thread group
                prefers_texture_arrays: false,
                supports_async_compute: false,
                memory_coalescing_alignment: 64,
                preferred_buffer_alignment: 64,
                supports_fast_math: false,
            },
            GpuVendor::Apple => OptimizationHints {
                preferred_workgroup_size: (32, 1, 1), // SIMD group size
                prefers_texture_arrays: true,
                supports_async_compute: true,
                memory_coalescing_alignment: 16,
                preferred_buffer_alignment: 16,
                supports_fast_math: true,
            },
            _ => OptimizationHints::default(),
        }
    }
}

/// Vendor-specific optimization hints
#[derive(Debug, Clone)]
pub struct OptimizationHints {
    pub preferred_workgroup_size: (u32, u32, u32),
    pub prefers_texture_arrays: bool,
    pub supports_async_compute: bool,
    pub memory_coalescing_alignment: u32,
    pub preferred_buffer_alignment: u32,
    pub supports_fast_math: bool,
}

impl Default for OptimizationHints {
    fn default() -> Self {
        Self {
            preferred_workgroup_size: (64, 1, 1),
            prefers_texture_arrays: true,
            supports_async_compute: false,
            memory_coalescing_alignment: 128,
            preferred_buffer_alignment: 256,
            supports_fast_math: false,
        }
    }
}

/// Device selection criteria for automatic adapter selection
#[derive(Debug, Clone)]
pub struct DeviceSelectionCriteria {
    pub prefer_discrete_gpu: bool,
    pub min_memory_size: u64,
    pub required_features: Features,
    pub preferred_vendor: Option<GpuVendor>,
    pub min_performance_tier: PerformanceTier,
    pub require_timestamp_queries: bool,
    pub require_pipeline_statistics: bool,
}

impl Default for DeviceSelectionCriteria {
    fn default() -> Self {
        Self {
            prefer_discrete_gpu: true,
            min_memory_size: 1024 * 1024 * 1024, // 1GB
            required_features: Features::empty(),
            preferred_vendor: None,
            min_performance_tier: PerformanceTier::Integrated,
            require_timestamp_queries: false,
            require_pipeline_statistics: false,
        }
    }
}

/// Device health monitoring
#[derive(Debug)]
pub struct DeviceHealth {
    pub is_lost: AtomicBool,
    pub last_error: RwLock<Option<String>>,
    pub error_count: AtomicU64,
    pub last_successful_operation: RwLock<Instant>,
    pub temperature: RwLock<Option<f32>>,
    pub power_usage: RwLock<Option<f32>>,
    pub memory_usage: RwLock<Option<f32>>,
}

impl Default for DeviceHealth {
    fn default() -> Self {
        Self {
            is_lost: AtomicBool::new(false),
            last_error: RwLock::new(None),
            error_count: AtomicU64::new(0),
            last_successful_operation: RwLock::new(Instant::now()),
            temperature: RwLock::new(None),
            power_usage: RwLock::new(None),
            memory_usage: RwLock::new(None),
        }
    }
}

/// Advanced GPU device manager
#[derive(Debug)]
pub struct DeviceManager {
    instance: Instance,
    adapters: Vec<(Adapter, GpuCapabilities)>,
    active_device: RwLock<Option<Arc<ManagedDevice>>>,
    active_adapter_index: RwLock<Option<usize>>,
    device_health: Arc<DeviceHealth>,
    selection_criteria: RwLock<DeviceSelectionCriteria>,
    fallback_chain: RwLock<Vec<usize>>, // Indices into adapters
    monitoring_enabled: AtomicBool,
}

/// Managed device wrapper with additional metadata
#[derive(Debug)]
pub struct ManagedDevice {
    pub device: Device,
    pub queue: Queue,
    pub capabilities: GpuCapabilities,
    pub optimization_hints: OptimizationHints,
    pub creation_time: Instant,
    pub health: Arc<DeviceHealth>,
}

impl DeviceManager {
    /// Get all available adapters
    pub fn adapters(&self) -> &[(Adapter, GpuCapabilities)] {
        &self.adapters
    }

    /// Create a new device manager
    #[instrument(skip(instance, surface))]
    pub async fn new(instance: Option<Instance>, surface: Option<&Surface<'_>>) -> Result<Self> {
        let instance = instance.unwrap_or_else(|| {
            Instance::new(InstanceDescriptor {
                backends: Backends::all(),
                flags: InstanceFlags::default(),
                dx12_shader_compiler: Dx12Compiler::Fxc,
                gles_minor_version: Gles3MinorVersion::Automatic,
            })
        });

        info!("Enumerating GPU adapters...");
        let adapters = Self::enumerate_adapters(&instance, surface).await?;

        if adapters.is_empty() {
            bail!("No compatible GPU adapters found");
        }

        info!("Found {} compatible GPU adapter(s)", adapters.len());
        for (i, (_, caps)) in adapters.iter().enumerate() {
            info!(
                "  [{}] {} ({:?}, {:?})",
                i, caps.device_name, caps.vendor, caps.performance_tier
            );
        }

        let fallback_chain = Self::create_fallback_chain(&adapters);

        Ok(Self {
            instance,
            adapters,
            active_device: RwLock::new(None),
            active_adapter_index: RwLock::new(None),
            device_health: Arc::new(DeviceHealth::default()),
            selection_criteria: RwLock::new(DeviceSelectionCriteria::default()),
            fallback_chain: RwLock::new(fallback_chain),
            monitoring_enabled: AtomicBool::new(true),
        })
    }

    /// Enumerate and analyze all available adapters
    async fn enumerate_adapters(
        instance: &Instance,
        surface: Option<&Surface<'_>>,
    ) -> Result<Vec<(Adapter, GpuCapabilities)>> {
        let mut adapters = Vec::new();

        // Try all power preferences to find all adapters
        for power_pref in [PowerPreference::HighPerformance, PowerPreference::LowPower] {
            if let Some(adapter) = instance
                .request_adapter(&RequestAdapterOptions {
                    power_preference: power_pref,
                    compatible_surface: surface,
                    force_fallback_adapter: false,
                })
                .await
            {
                let capabilities = GpuCapabilities::from_adapter(&adapter);

                // Check if we already have this adapter
                if !adapters
                    .iter()
                    .any(|(_, caps): &(Adapter, GpuCapabilities)| {
                        caps.device_id == capabilities.device_id
                            && caps.vendor_id == capabilities.vendor_id
                    })
                {
                    adapters.push((adapter, capabilities));
                }
            }
        }

        // Also try fallback adapter
        if let Some(adapter) = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::default(),
                compatible_surface: surface,
                force_fallback_adapter: true,
            })
            .await
        {
            let capabilities = GpuCapabilities::from_adapter(&adapter);

            if !adapters.iter().any(|(_, caps)| {
                caps.device_id == capabilities.device_id && caps.vendor_id == capabilities.vendor_id
            }) {
                adapters.push((adapter, capabilities));
            }
        }

        Ok(adapters)
    }

    /// Create fallback chain ordered by preference
    fn create_fallback_chain(adapters: &[(Adapter, GpuCapabilities)]) -> Vec<usize> {
        let mut indices: Vec<usize> = (0..adapters.len()).collect();

        // Sort by performance tier (descending), then by memory size (descending)
        indices.sort_by(|&a, &b| {
            let caps_a = &adapters[a].1;
            let caps_b = &adapters[b].1;

            caps_b
                .performance_tier
                .cmp(&caps_a.performance_tier)
                .then(caps_b.memory_size.cmp(&caps_a.memory_size))
        });

        indices
    }

    /// Initialize device with automatic selection
    #[instrument]
    pub async fn initialize_device(&self) -> Result<Arc<ManagedDevice>> {
        let criteria = self.selection_criteria.read().clone();
        self.initialize_device_with_criteria(criteria).await
    }

    /// Initialize device with specific criteria
    #[instrument]
    pub async fn initialize_device_with_criteria(
        &self,
        criteria: DeviceSelectionCriteria,
    ) -> Result<Arc<ManagedDevice>> {
        let fallback_chain = self.fallback_chain.read().clone();

        for &adapter_idx in &fallback_chain {
            let (adapter, capabilities) = &self.adapters[adapter_idx];

            if !Self::meets_criteria(capabilities, &criteria) {
                debug!(
                    "Adapter {} doesn't meet criteria, skipping",
                    capabilities.device_name
                );
                continue;
            }

            match self.create_device(adapter, capabilities, &criteria).await {
                Ok(device) => {
                    info!(
                        "Successfully initialized device: {}",
                        capabilities.device_name
                    );
                    let managed_device = Arc::new(device);
                    *self.active_device.write() = Some(managed_device.clone());
                    *self.active_adapter_index.write() = Some(adapter_idx);
                    return Ok(managed_device);
                }
                Err(e) => {
                    warn!(
                        "Failed to create device {}: {}",
                        capabilities.device_name, e
                    );
                    continue;
                }
            }
        }

        bail!("Failed to initialize any compatible device");
    }

    /// Check if capabilities meet selection criteria
    fn meets_criteria(capabilities: &GpuCapabilities, criteria: &DeviceSelectionCriteria) -> bool {
        if capabilities.memory_size < criteria.min_memory_size {
            return false;
        }

        if capabilities.performance_tier < criteria.min_performance_tier {
            return false;
        }

        if !capabilities
            .supported_features
            .contains(criteria.required_features)
        {
            return false;
        }

        if let Some(preferred_vendor) = criteria.preferred_vendor {
            if capabilities.vendor != preferred_vendor {
                return false;
            }
        }

        if criteria.require_timestamp_queries
            && !capabilities
                .supported_features
                .contains(Features::TIMESTAMP_QUERY)
        {
            return false;
        }

        if criteria.require_pipeline_statistics
            && !capabilities
                .supported_features
                .contains(Features::PIPELINE_STATISTICS_QUERY)
        {
            return false;
        }

        true
    }

    /// Create managed device from adapter
    async fn create_device(
        &self,
        adapter: &Adapter,
        capabilities: &GpuCapabilities,
        criteria: &DeviceSelectionCriteria,
    ) -> Result<ManagedDevice> {
        info!("Creating device for adapter: {}", capabilities.device_name);

        let mut required_features = criteria.required_features;

        // Enable timestamp queries if requested
        if criteria.require_timestamp_queries {
            required_features |= Features::TIMESTAMP_QUERY;
        }

        // Enable pipeline statistics if requested
        if criteria.require_pipeline_statistics {
            required_features |= Features::PIPELINE_STATISTICS_QUERY;
        }

        let required_limits = Limits::default();

        // Set up error callback for Vulkan validation errors
        let device_descriptor = DeviceDescriptor {
            label: Some(&format!("StratoUI Device - {}", capabilities.device_name)),
            required_features,
            required_limits: required_limits.clone(),
        };

        match adapter.request_device(&device_descriptor, None).await {
            Ok((device, queue)) => {
                // Set up error callback to handle Vulkan validation errors with rate limiting
                device.on_uncaptured_error(Box::new(|error| {
                    match error {
                        wgpu::Error::Validation { description, .. } => {
                            // Rate limit Vulkan validation errors, especially VUID-vkQueueSubmit
                            if description.contains("VUID-vkQueueSubmit")
                                || description.contains("pSignalSemaphores")
                            {
                                strato_error_rate_limited!(
                                    LogCategory::Vulkan,
                                    "Vulkan validation warning (known WGPU issue): {}",
                                    description
                                );
                            } else {
                                strato_error_rate_limited!(
                                    LogCategory::Vulkan,
                                    "Vulkan validation error: {}",
                                    description
                                );
                            }
                        }
                        wgpu::Error::OutOfMemory { .. } => {
                            strato_error_rate_limited!(
                                LogCategory::Vulkan,
                                "GPU out of memory: {}",
                                error
                            );
                        }
                        _ => {
                            strato_warn!(LogCategory::Vulkan, "GPU error: {}", error);
                        }
                    }
                }));

                let health = Arc::new(DeviceHealth::default());
                health
                    .last_successful_operation
                    .write()
                    .clone_from(&Instant::now());

                let optimization_hints = capabilities.get_optimization_hints();

                strato_debug!(
                    LogCategory::Renderer,
                    "Successfully created device '{}' with {} MB memory",
                    capabilities.device_name,
                    capabilities.memory_size / (1024 * 1024)
                );

                Ok(ManagedDevice {
                    device,
                    queue,
                    capabilities: capabilities.clone(),
                    optimization_hints,
                    creation_time: Instant::now(),
                    health,
                })
            }
            Err(e) => {
                strato_error_rate_limited!(
                    LogCategory::Vulkan,
                    "Failed to create device for adapter '{}': {}",
                    capabilities.device_name,
                    e
                );

                // All device creation errors are treated the same way
                bail!("Failed to create device: {}", e);
            }
        }
    }

    /// Get current active device
    pub fn get_device(&self) -> Option<Arc<ManagedDevice>> {
        self.active_device.read().clone()
    }

    /// Get current active adapter
    pub fn get_active_adapter(&self) -> Option<&Adapter> {
        self.active_adapter_index
            .read()
            .map(|idx| &self.adapters[idx].0)
    }

    /// Check device health and attempt recovery if needed
    #[instrument]
    pub async fn check_device_health(&self) -> Result<()> {
        if let Some(device) = self.get_device() {
            if device.health.is_lost.load(Ordering::Relaxed) {
                warn!("Device lost detected, attempting recovery...");
                self.recover_device().await?;
            }
        }
        Ok(())
    }

    /// Attempt to recover from device loss
    async fn recover_device(&self) -> Result<()> {
        info!("Attempting device recovery...");

        // Clear current device
        *self.active_device.write() = None;
        *self.active_adapter_index.write() = None;

        // Try to reinitialize with same criteria
        let criteria = self.selection_criteria.read().clone();
        self.initialize_device_with_criteria(criteria).await?;

        info!("Device recovery successful");
        Ok(())
    }

    /// Update selection criteria
    pub fn update_selection_criteria(&self, criteria: DeviceSelectionCriteria) {
        *self.selection_criteria.write() = criteria;
    }

    /// Get adapter capabilities
    pub fn get_adapter_capabilities(&self) -> Vec<GpuCapabilities> {
        self.adapters.iter().map(|(_, caps)| caps.clone()).collect()
    }

    /// Get the best available device
    pub fn get_best_device(&self) -> Option<Arc<ManagedDevice>> {
        self.get_device()
    }

    /// Get device statistics
    pub fn get_device_stats(&self) -> Option<DeviceStats> {
        self.get_device().map(|device| DeviceStats {
            device_name: device.capabilities.device_name.clone(),
            vendor: device.capabilities.vendor,
            performance_tier: device.capabilities.performance_tier,
            uptime: device.creation_time.elapsed(),
            error_count: device.health.error_count.load(Ordering::Relaxed),
            is_healthy: !device.health.is_lost.load(Ordering::Relaxed),
            memory_usage: device.health.memory_usage.read().clone(),
            temperature: device.health.temperature.read().clone(),
            power_usage: device.health.power_usage.read().clone(),
        })
    }
}

/// Device statistics for monitoring
#[derive(Debug, Clone)]
pub struct DeviceStats {
    pub device_name: String,
    pub vendor: GpuVendor,
    pub performance_tier: PerformanceTier,
    pub uptime: Duration,
    pub error_count: u64,
    pub is_healthy: bool,
    pub memory_usage: Option<f32>,
    pub temperature: Option<f32>,
    pub power_usage: Option<f32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_device_manager_creation() {
        let manager = DeviceManager::new(None, None).await;
        assert!(manager.is_ok());
    }

    #[test]
    fn test_gpu_vendor_detection() {
        assert_eq!(GpuVendor::from(0x10DE), GpuVendor::Nvidia);
        assert_eq!(GpuVendor::from(0x1002), GpuVendor::Amd);
        assert_eq!(GpuVendor::from(0x8086), GpuVendor::Intel);
    }

    #[test]
    fn test_optimization_hints() {
        let caps = GpuCapabilities {
            vendor: GpuVendor::Nvidia,
            device_name: "Test GPU".to_string(),
            device_id: 0,
            vendor_id: 0x10DE,
            performance_tier: PerformanceTier::HighEnd,
            memory_size: 8 * 1024 * 1024 * 1024,
            memory_bandwidth: None,
            compute_units: None,
            base_clock: None,
            boost_clock: None,
            supports_ray_tracing: false,
            supports_mesh_shaders: false,
            supports_variable_rate_shading: false,
            max_texture_size: 16384,
            max_texture_array_layers: 2048,
            max_bind_groups: 8,
            max_dynamic_uniform_buffers: 8,
            max_storage_buffers: 8,
            max_sampled_textures: 16,
            max_samplers: 16,
            max_storage_textures: 8,
            max_vertex_buffers: 8,
            max_vertex_attributes: 16,
            max_push_constant_size: 128,
            timestamp_period: 1.0,
            supported_features: Features::empty(),
            limits: Limits::default(),
        };

        let hints = caps.get_optimization_hints();
        assert_eq!(hints.preferred_workgroup_size, (32, 1, 1));
        assert!(hints.prefers_texture_arrays);
    }
}
