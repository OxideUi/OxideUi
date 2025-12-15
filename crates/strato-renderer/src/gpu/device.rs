//! Device management for wgpu
//!
//! BLOCCO 1: Device Setup
//! Handles wgpu instance, adapter, device, and queue initialization

use wgpu::{
    Adapter, AdapterInfo, Backends, Device, DeviceDescriptor, Features, Instance,
    InstanceDescriptor, Limits, PowerPreference, Queue, RequestAdapterOptions,
};

/// Manages wgpu device and queue
pub struct DeviceManager {
    instance: Instance,
    adapter: Adapter,
    device: Device,
    queue: Queue,
}

impl DeviceManager {
    /// Create a new device manager
    ///
    /// # Arguments
    /// * `backend` - Backend to use (Backends::all() for auto-selection)
    ///
    /// # Errors
    /// Returns error if no suitable adapter found or device creation fails
    pub async fn new(backend: Backends) -> anyhow::Result<Self> {
        // Create wgpu instance
        let instance = Instance::new(InstanceDescriptor {
            backends: backend,
            ..Default::default()
        });

        // Request adapter
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| anyhow::anyhow!("Failed to find suitable GPU adapter"))?;

        // Log adapter info for debugging
        let adapter_info = adapter.get_info();
        println!("=== GPU ADAPTER INFO ===");
        println!("Name: {}", adapter_info.name);
        println!("Vendor: {:?}", adapter_info.vendor);
        println!("Device: {:?}", adapter_info.device);
        println!("Backend: {:?}", adapter_info.backend);
        println!("========================");

        // Request device and queue
        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: Some("StratoUI Device"),
                    required_features: Features::empty(),
                    required_limits: Limits::default(),
                },
                None,
            )
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create device: {}", e))?;

        Ok(Self {
            instance,
            adapter,
            device,
            queue,
        })
    }

    /// Get reference to the device
    pub fn device(&self) -> &Device {
        &self.device
    }

    /// Get reference to the queue
    pub fn queue(&self) -> &Queue {
        &self.queue
    }

    /// Get reference to the adapter
    pub fn adapter(&self) -> &Adapter {
        &self.adapter
    }

    /// Get reference to the instance
    pub fn instance(&self) -> &Instance {
        &self.instance
    }

    /// Get adapter information
    pub fn adapter_info(&self) -> AdapterInfo {
        self.adapter.get_info()
    }

    /// Get device limits
    pub fn limits(&self) -> Limits {
        self.device.limits()
    }

    /// Get device features
    pub fn features(&self) -> Features {
        self.device.features()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_device_creation() {
        let dm = DeviceManager::new(Backends::all())
            .await
            .expect("Failed to create device manager");

        // Verify device created successfully
        assert!(dm.device().limits().max_texture_dimension_2d > 0);
    }

    #[tokio::test]
    async fn test_device_limits_reasonable() {
        let dm = DeviceManager::new(Backends::all())
            .await
            .expect("Failed to create device manager");

        let limits = dm.limits();

        // Verify reasonable limits for 2D rendering
        assert!(limits.max_texture_dimension_2d >= 2048);
        assert!(limits.max_buffer_size > 0);
        assert!(limits.max_bind_groups > 0);
    }

    #[tokio::test]
    async fn test_adapter_info() {
        let dm = DeviceManager::new(Backends::all())
            .await
            .expect("Failed to create device manager");

        let info = dm.adapter_info();

        // Verify adapter info is populated
        assert!(!info.name.is_empty());
        println!("Adapter: {}", info.name);
        println!("Backend: {:?}", info.backend);
    }

    #[tokio::test]
    async fn test_queue_available() {
        let dm = DeviceManager::new(Backends::all())
            .await
            .expect("Failed to create device manager");

        // Verify queue is accessible
        let _queue = dm.queue();
    }
}
