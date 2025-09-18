//! Integration module for coordinating all advanced wgpu systems
//!
//! This module provides a unified API that coordinates all the advanced systems:
//! - Device management with automatic fallback
//! - Resource management with intelligent pooling
//! - Memory management with multi-tier allocation
//! - Shader management with hot-reload
//! - Pipeline management with render graph optimization
//! - Buffer management with lock-free operations
//! - Performance profiling and monitoring
//!
//! The integration layer ensures all systems work together seamlessly and provides
//! a clean, easy-to-use API for the rest of the framework.

use std::sync::Arc;
use anyhow::{Result, Context};
use tracing::{info, warn, error, debug, instrument};
use wgpu::*;
use slotmap::DefaultKey;

use crate::{
    device::{ManagedDevice, DeviceManager, AdapterInfo},
    resources::{ResourceManager, ResourceHandle, ResourceType},
    memory::{MemoryManager, MemoryPool, AllocationStrategy},
    shader::{ShaderManager, ShaderSource, CompiledShader},
    buffer::{BufferManager, DynamicBuffer, BufferPool},
    profiler::{Profiler, PerformanceReport, FrameStats},
    pipeline::{PipelineManager, RenderGraph, RenderNode},
};

/// Configuration for the integrated renderer system
#[derive(Debug, Clone)]
pub struct RendererConfig {
    /// Enable performance profiling
    pub enable_profiling: bool,
    /// Enable detailed profiling (higher overhead)
    pub detailed_profiling: bool,
    /// Enable automatic performance analysis
    pub auto_analysis: bool,
    /// Memory allocation strategy
    pub memory_strategy: AllocationStrategy,
    /// Maximum memory pool size in bytes
    pub max_memory_pool_size: u64,
    /// Enable shader hot-reload in debug builds
    pub enable_shader_hot_reload: bool,
    /// Preferred GPU adapter type
    pub preferred_adapter: Option<PowerPreference>,
    /// Enable validation layers
    pub enable_validation: bool,
    /// Maximum number of frames in flight
    pub max_frames_in_flight: u32,
}

impl Default for RendererConfig {
    fn default() -> Self {
        Self {
            enable_profiling: cfg!(debug_assertions),
            detailed_profiling: false,
            auto_analysis: true,
            memory_strategy: AllocationStrategy::Balanced,
            max_memory_pool_size: 512 * 1024 * 1024, // 512MB
            enable_shader_hot_reload: cfg!(debug_assertions),
            preferred_adapter: Some(PowerPreference::HighPerformance),
            enable_validation: cfg!(debug_assertions),
            max_frames_in_flight: 2,
        }
    }
}

/// Integrated renderer system that coordinates all subsystems
pub struct IntegratedRenderer {
    // Core systems
    device_manager: Arc<DeviceManager>,
    device: Arc<ManagedDevice>,
    
    // Management systems
    resource_manager: Arc<ResourceManager>,
    memory_manager: Arc<parking_lot::Mutex<MemoryManager>>,
    shader_manager: Arc<ShaderManager>,
    buffer_manager: Arc<BufferManager>,
    pipeline_manager: Arc<PipelineManager>,
    
    // Monitoring
    profiler: Option<Arc<Profiler>>,
    
    // Configuration
    config: RendererConfig,
    
    // State
    initialized: bool,
    frame_count: u64,
}

/// Render context for a single frame
pub struct RenderContext {
    pub device: Arc<ManagedDevice>,
    pub encoder: CommandEncoder,
    pub profiler: Option<Arc<Profiler>>,
    pub frame_id: u64,
    
    // Timing queries
    gpu_timer_id: Option<u32>,
}

/// Render statistics for monitoring
#[derive(Debug, Clone)]
pub struct RenderStats {
    pub frame_count: u64,
    pub average_frame_time: f64,
    pub memory_usage: u64,
    pub active_resources: u32,
    pub shader_reloads: u32,
    pub pipeline_switches: u32,
}

impl IntegratedRenderer {
    /// Create a new integrated renderer with default configuration
    pub async fn new() -> Result<Self> {
        Self::with_config(RendererConfig::default()).await
    }
    
    /// Create a new integrated renderer with custom configuration
    pub async fn with_config(config: RendererConfig) -> Result<Self> {
        info!("Initializing integrated renderer system");
        
        // Initialize device manager
        let device_manager = Arc::new(DeviceManager::new().await?);
        
        // Get the best available device
        let device = device_manager
            .get_best_device()
            .context("Failed to get GPU device")?;
        
        info!("Selected GPU device: {}", device.capabilities.device_name);
        
        // Initialize management systems  
        let resource_manager = Arc::new(ResourceManager::new(device.clone())?);
        let memory_manager = Arc::new(parking_lot::Mutex::new(MemoryManager::new(device.clone())));
        
        let shader_manager = Arc::new(ShaderManager::new(device.clone())?);
        
        let memory_manager_shared = memory_manager.clone();
        let buffer_manager = Arc::new(BufferManager::new(
            device.clone(),
            memory_manager_shared.clone(),
        ));
        
        let pipeline_manager = Arc::new(PipelineManager::new(
            &device.device,
            wgpu::TextureFormat::Bgra8UnormSrgb, // Default surface format
        ));
        
        // Initialize profiler if enabled
        let profiler = if config.enable_profiling {
            let profiler = Arc::new(Profiler::new(device.clone())?);
            profiler.set_detailed_profiling(config.detailed_profiling);
            Some(profiler)
        } else {
            None
        };
        
        Ok(Self {
            device_manager,
            device,
            resource_manager,
            memory_manager: memory_manager_shared,
            shader_manager,
            buffer_manager,
            pipeline_manager,
            profiler,
            config,
            initialized: false,
            frame_count: 0,
        })
    }
    
    /// Initialize the renderer (call after window creation)
    #[instrument(skip(self))]
    pub async fn initialize(&mut self) -> Result<()> {
        if self.initialized {
            warn!("Renderer already initialized");
            return Ok(());
        }
        
        info!("Initializing renderer subsystems");
        
        // Initialize shader manager (load default shaders)
        self.shader_manager.initialize()?;
        
        // Initialize pipeline manager (create default pipelines)
        self.pipeline_manager.initialize()?;
        
        // Initialize buffer manager (create default pools)
        self.buffer_manager.initialize()?;
        
        self.initialized = true;
        info!("Renderer initialization complete");
        
        Ok(())
    }
    
    /// Begin a new frame
    #[instrument(skip(self))]
    pub fn begin_frame(&mut self) -> Result<RenderContext> {
        if !self.initialized {
            return Err(anyhow::anyhow!("Renderer not initialized"));
        }
        
        self.frame_count += 1;
        
        // Begin profiling if enabled
        if let Some(ref profiler) = self.profiler {
            profiler.begin_frame();
        }
        
        // Create command encoder
        let encoder = self.device.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some(&format!("Frame {}", self.frame_count)),
        });
        
        // Begin GPU timing
        let gpu_timer_id = if let Some(ref profiler) = self.profiler {
            // Note: encoder is moved, so we need to handle this differently
            None // Placeholder - would need to restructure for actual GPU timing
        } else {
            None
        };
        
        Ok(RenderContext {
            device: self.device.clone(),
            encoder,
            profiler: self.profiler.clone(),
            frame_id: self.frame_count,
            gpu_timer_id,
        })
    }
    
    /// End the current frame and submit commands
    #[instrument(skip(self, context))]
    pub fn end_frame(&mut self, context: RenderContext) -> Result<()> {
        // End GPU timing if active
        if let (Some(profiler), Some(timer_id)) = (&context.profiler, context.gpu_timer_id) {
            // profiler.end_gpu_timing(&mut context.encoder, timer_id);
        }
        
        // Submit command buffer
        let command_buffer = context.encoder.finish();
        self.device.queue.submit(std::iter::once(command_buffer));
        
        // End profiling if enabled
        if let Some(ref profiler) = self.profiler {
            profiler.end_frame();
        }
        
        // Perform maintenance tasks periodically
        if self.frame_count % 60 == 0 {
            self.perform_maintenance()?;
        }
        
        Ok(())
    }
    
    /// Get render statistics
    pub fn get_stats(&self) -> RenderStats {
        let memory_usage = self.memory_manager.lock().get_total_allocated();
        let active_resources = self.resource_manager.get_active_count() as u32;
        
        let (average_frame_time, shader_reloads, pipeline_switches) = if let Some(ref profiler) = self.profiler {
            let report = profiler.get_performance_report();
            (
                report.frame_stats.average_frame_time,
                0, // Would need to track in shader manager
                0, // Would need to track in pipeline manager
            )
        } else {
            (0.0, 0, 0)
        };
        
        RenderStats {
            frame_count: self.frame_count,
            average_frame_time,
            memory_usage,
            active_resources,
            shader_reloads,
            pipeline_switches,
        }
    }
    
    /// Get performance report (if profiling is enabled)
    pub fn get_performance_report(&self) -> Option<PerformanceReport> {
        self.profiler.as_ref().map(|p| p.get_performance_report())
    }
    
    /// Create a buffer with automatic management
    pub fn create_buffer(&self, size: u64, usage: BufferUsages) -> Result<ResourceHandle> {
        let config = crate::buffer::BufferConfig {
            name: "auto_buffer".to_string(),
            size,
            usage,
            usage_pattern: crate::buffer::BufferUsagePattern::Dynamic,
            allocation_strategy: crate::buffer::AllocationStrategy::BestFit,
            alignment: 256,
            mapped_at_creation: false,
            persistent_mapping: false,
        };
        self.buffer_manager.create_buffer(&config)
    }
    
    /// Create a texture with automatic management
    pub fn create_texture(&self, descriptor: &TextureDescriptor) -> DefaultKey {
        self.resource_manager.create_texture(descriptor)
    }
    
    /// Load and compile a shader
    pub fn load_shader(&self, path: &std::path::Path, stage: crate::shader::ShaderStage, variant: crate::shader::ShaderVariant) -> Result<Arc<crate::shader::CompiledShader>> {
        self.shader_manager.load_shader(path, stage, variant)
    }
    
    /// Create a render pipeline
    pub fn create_render_pipeline(&self) -> Result<()> {
        self.pipeline_manager.create_render_pipeline()
    }
    
    /// Get device information
    pub fn get_device_info(&self) -> &str {
        &self.device.capabilities.device_name
    }
    
    /// Check if a feature is supported
    pub fn supports_feature(&self, feature: Features) -> bool {
        self.device.capabilities.supported_features.contains(feature)
    }
    
    /// Perform maintenance tasks
    #[instrument(skip(self))]
    fn perform_maintenance(&mut self) -> Result<()> {
        debug!("Performing maintenance tasks");
        
        // Clean up unused resources
        self.resource_manager.cleanup_unused();
        
        // Defragment memory pools
        let _ = self.memory_manager.lock().defragment();
        
        // Check for shader hot-reloads
        if self.config.enable_shader_hot_reload {
            if let Err(e) = self.shader_manager.check_for_reloads() {
                warn!("Shader hot-reload check failed: {}", e);
            }
        }
        
        // Collect garbage in buffer pools
        self.buffer_manager.collect_garbage();
        
        Ok(())
    }
    
    /// Resize surface (call when window is resized)
    pub fn resize(&mut self, new_size: (u32, u32)) -> Result<()> {
        info!("Resizing renderer to {}x{}", new_size.0, new_size.1);
        
        // Update any size-dependent resources
        // This would typically involve recreating swap chain, depth buffers, etc.
        
        Ok(())
    }
    
    /// Shutdown the renderer gracefully
    #[instrument(skip(self))]
    pub fn shutdown(&mut self) {
        info!("Shutting down integrated renderer");
        
        if let Some(ref profiler) = self.profiler {
            let report = profiler.get_performance_report();
            info!("Final performance report: {:#?}", report);
        }
        
        // Cleanup resources
        self.resource_manager.cleanup_all();
        let _ = self.memory_manager.lock().cleanup();
        
        self.initialized = false;
        info!("Renderer shutdown complete");
    }
}

impl Drop for IntegratedRenderer {
    fn drop(&mut self) {
        if self.initialized {
            self.shutdown();
        }
    }
}

impl RenderContext {
    /// Begin a render pass with profiling
    pub fn begin_render_pass<'a>(&'a mut self, descriptor: &RenderPassDescriptor<'a, '_>) -> RenderPass<'a> {
        // Begin CPU timing if profiler is available
        if let Some(ref profiler) = self.profiler {
            profiler.cpu_profiler.begin_section("render_pass");
        }
        
        self.encoder.begin_render_pass(descriptor)
    }
    
    /// End render pass timing
    pub fn end_render_pass(&self) {
        if let Some(ref profiler) = self.profiler {
            profiler.cpu_profiler.end_section("render_pass");
        }
    }
    
    /// Begin compute pass with profiling
    pub fn begin_compute_pass<'a>(&'a mut self, descriptor: &ComputePassDescriptor<'a>) -> ComputePass<'a> {
        if let Some(ref profiler) = self.profiler {
            profiler.cpu_profiler.begin_section("compute_pass");
        }
        
        self.encoder.begin_compute_pass(descriptor)
    }
    
    /// End compute pass timing
    pub fn end_compute_pass(&self) {
        if let Some(ref profiler) = self.profiler {
            profiler.cpu_profiler.end_section("compute_pass");
        }
    }
}

/// Builder for creating an integrated renderer with custom configuration
pub struct RendererBuilder {
    config: RendererConfig,
}

impl RendererBuilder {
    /// Create a new renderer builder
    pub fn new() -> Self {
        Self {
            config: RendererConfig::default(),
        }
    }
    
    /// Enable or disable profiling
    pub fn with_profiling(mut self, enabled: bool) -> Self {
        self.config.enable_profiling = enabled;
        self
    }
    
    /// Enable or disable detailed profiling
    pub fn with_detailed_profiling(mut self, enabled: bool) -> Self {
        self.config.detailed_profiling = enabled;
        self
    }
    
    /// Set memory allocation strategy
    pub fn with_memory_strategy(mut self, strategy: AllocationStrategy) -> Self {
        self.config.memory_strategy = strategy;
        self
    }
    
    /// Set maximum memory pool size
    pub fn with_max_memory_pool_size(mut self, size: u64) -> Self {
        self.config.max_memory_pool_size = size;
        self
    }
    
    /// Set preferred GPU adapter
    pub fn with_preferred_adapter(mut self, preference: PowerPreference) -> Self {
        self.config.preferred_adapter = Some(preference);
        self
    }
    
    /// Enable or disable validation layers
    pub fn with_validation(mut self, enabled: bool) -> Self {
        self.config.enable_validation = enabled;
        self
    }
    
    /// Build the integrated renderer
    pub async fn build(self) -> Result<IntegratedRenderer> {
        IntegratedRenderer::with_config(self.config).await
    }
}

impl Default for RendererBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience macro for creating a renderer with common configurations
#[macro_export]
macro_rules! create_renderer {
    (debug) => {
        RendererBuilder::new()
            .with_profiling(true)
            .with_detailed_profiling(true)
            .with_validation(true)
            .build()
            .await
    };
    
    (release) => {
        RendererBuilder::new()
            .with_profiling(false)
            .with_validation(false)
            .build()
            .await
    };
    
    (performance) => {
        RendererBuilder::new()
            .with_profiling(true)
            .with_detailed_profiling(false)
            .with_preferred_adapter(PowerPreference::HighPerformance)
            .with_memory_strategy(AllocationStrategy::Performance)
            .build()
            .await
    };
}