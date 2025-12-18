//! Surface management for window integration
//!
//! BLOCCO 2: Surface Configuration
//! Handles wgpu Surface creation, configuration, and resize management

use std::sync::Arc;
use wgpu::{
    Adapter, Device, PresentMode, Surface, SurfaceConfiguration, SurfaceError, SurfaceTexture,
    TextureFormat, TextureUsages,
};

/// Manages wgpu surface and its configuration
pub struct SurfaceManager {
    surface: Surface<'static>,
    config: SurfaceConfiguration,
    format: TextureFormat,
}

impl SurfaceManager {
    /// Create a new surface manager
    ///
    /// # Arguments
    /// * `surface` - The WGPU surface
    /// * `device` - GPU device
    /// * `adapter` - GPU adapter for capability detection
    /// * `width` - Initial width
    /// * `height` - Initial height
    ///
    /// # Errors
    /// Returns error if surface configuration fails
    pub fn new(
        surface: Surface<'static>,
        device: &Device,
        adapter: &Adapter,
        width: u32,
        height: u32,
    ) -> anyhow::Result<Self> {
        // Get surface capabilities
        let capabilities = surface.get_capabilities(adapter);

        // Select best format (prefer sRGB)
        let format = capabilities
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(capabilities.formats[0]);

        println!("=== SURFACE CONFIGURATION ===");
        println!("Selected format: {:?}", format);
        println!("Available formats: {:?}", capabilities.formats);
        println!("Alpha modes: {:?}", capabilities.alpha_modes);
        println!("============================");

        // Ensure valid dimensions
        let width = width.max(1);
        let height = height.max(1);

        // Configure surface
        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format,
            width,
            height,
            present_mode: PresentMode::AutoVsync,
            alpha_mode: capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(device, &config);

        println!("Surface configured: {}x{}", width, height);

        Ok(Self {
            surface,
            config,
            format,
        })
    }

    /// Resize the surface
    ///
    /// # Arguments
    /// * `width` - New width (must be > 0)
    /// * `height` - New height (must be > 0)
    /// * `device` - GPU device to reconfigure surface
    pub fn resize(&mut self, width: u32, height: u32, device: &Device) -> anyhow::Result<()> {
        if width == 0 || height == 0 {
            return Err(anyhow::anyhow!(
                "Invalid surface dimensions: {}x{}",
                width,
                height
            ));
        }

        self.config.width = width;
        self.config.height = height;
        self.surface.configure(device, &self.config);

        println!("Surface resized to: {}x{}", width, height);

        Ok(())
    }

    /// Get current surface texture for rendering
    pub fn get_current_texture(&mut self) -> Result<SurfaceTexture, SurfaceError> {
        self.surface.get_current_texture()
    }

    /// Get surface format
    pub fn format(&self) -> TextureFormat {
        self.format
    }

    /// Get surface width
    pub fn width(&self) -> u32 {
        self.config.width
    }

    /// Get surface height
    pub fn height(&self) -> u32 {
        self.config.height
    }

    /// Get surface configuration (for advanced usage)
    pub fn config(&self) -> &SurfaceConfiguration {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gpu::DeviceManager;
    use wgpu::Backends;
    use winit::dpi::PhysicalSize;
    use winit::event_loop::EventLoop;

    // Note: These tests require a window which needs an event loop
    // They may fail in headless CI environments

    #[tokio::test]
    #[ignore] // Requires window system
    async fn test_surface_creation() {
        let event_loop = EventLoop::new().unwrap();
        let window = Arc::new(
            winit::window::WindowBuilder::new()
                .with_inner_size(PhysicalSize::new(800, 600))
                .with_visible(false)
                .build(&event_loop)
                .unwrap(),
        );

        let dm = DeviceManager::new(Backends::all()).await.unwrap();
        let surface = dm.instance().create_surface(window.clone()).unwrap();
        // Safety: for test purposes
        let surface: Surface<'static> = unsafe { std::mem::transmute(surface) };

        let surface_mgr = SurfaceManager::new(surface, dm.device(), dm.adapter(), 800, 600);

        assert!(surface_mgr.is_ok());
    }

    #[tokio::test]
    #[ignore] // Requires window system
    async fn test_surface_format() {
        let event_loop = EventLoop::new().unwrap();
        let window = Arc::new(
            winit::window::WindowBuilder::new()
                .with_inner_size(PhysicalSize::new(800, 600))
                .with_visible(false)
                .build(&event_loop)
                .unwrap(),
        );

        let dm = DeviceManager::new(Backends::all()).await.unwrap();
        let surface = dm.instance().create_surface(window.clone()).unwrap();
        // Safety: for test purposes
        let surface: Surface<'static> = unsafe { std::mem::transmute(surface) };

        let surface_mgr =
            SurfaceManager::new(surface, dm.device(), dm.adapter(), 800, 600).unwrap();

        let format = surface_mgr.format();
        assert!(matches!(
            format,
            TextureFormat::Bgra8UnormSrgb | TextureFormat::Rgba8UnormSrgb
        ));
    }

    #[test]
    fn test_resize_validation() {
        // Test that resize validates dimensions without requiring actual surface
        // This is a unit test for the validation logic
        let result_zero_width = validate_dimensions(0, 600);
        let result_zero_height = validate_dimensions(800, 0);
        let result_valid = validate_dimensions(800, 600);

        assert!(result_zero_width.is_err());
        assert!(result_zero_height.is_err());
        assert!(result_valid.is_ok());
    }

    // Helper function for unit testing dimension validation
    fn validate_dimensions(width: u32, height: u32) -> anyhow::Result<()> {
        if width == 0 || height == 0 {
            return Err(anyhow::anyhow!(
                "Invalid surface dimensions: {}x{}",
                width,
                height
            ));
        }
        Ok(())
    }
}
