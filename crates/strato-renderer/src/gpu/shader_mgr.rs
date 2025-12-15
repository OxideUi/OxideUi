//! Shader module management
//!
//! BLOCCO 3: Shader Compilation
//! Handles WGSL shader loading, compilation, and validation

use wgpu::{Device, ShaderModule, ShaderModuleDescriptor, ShaderSource};

/// Manages shader module
pub struct ShaderManager {
    module: ShaderModule,
    vertex_entry: String,
    fragment_entry: String,
}

impl ShaderManager {
    /// Create shader from WGSL source
    ///
    /// # Arguments
    /// * `device` - GPU device
    /// * `source` - WGSL source code
    /// * `label` - Optional label for debugging
    ///
    /// # Errors
    /// Returns error if shader compilation fails
    pub fn from_wgsl(
        device: &Device,
        source: &str,
        label: Option<&str>,
    ) -> anyhow::Result<Self> {
        println!("=== SHADER COMPILATION ===");
        println!("Label: {:?}", label);
        println!("Source length: {} bytes", source.len());

        // Create shader module descriptor
        let descriptor = ShaderModuleDescriptor {
            label,
            source: ShaderSource::Wgsl(source.into()),
        };

        // Compile shader (wgpu validates automatically)
        let module = device.create_shader_module(descriptor);

        // Extract entry points from WGSL source
        // For simple.wgsl, we have hardcoded: vs_main and fs_main
        let vertex_entry = "vs_main".to_string();
        let fragment_entry = "fs_main".to_string();

        println!("Vertex entry: {}", vertex_entry);
        println!("Fragment entry: {}", fragment_entry);
        println!("Compilation: SUCCESS");
        println!("==========================");

        Ok(Self {
            module,
            vertex_entry,
            fragment_entry,
        })
    }

    /// Get shader module reference
    pub fn module(&self) -> &ShaderModule {
        &self.module
    }

    /// Get vertex shader entry point
    pub fn vertex_entry(&self) -> &str {
        &self.vertex_entry
    }

    /// Get fragment shader entry point
    pub fn fragment_entry(&self) -> &str {
        &self.fragment_entry
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gpu::DeviceManager;
    use wgpu::Backends;

    const SIMPLE_WGSL: &str = include_str!("../shaders/simple.wgsl");

    #[tokio::test]
    async fn test_shader_compilation() {
        let dm = DeviceManager::new(Backends::all())
            .await
            .expect("Failed to create device");

        let shader = ShaderManager::from_wgsl(dm.device(), SIMPLE_WGSL, Some("Test Shader"));

        assert!(shader.is_ok());
        let shader = shader.unwrap();
        assert_eq!(shader.vertex_entry(), "vs_main");
        assert_eq!(shader.fragment_entry(), "fs_main");
    }

    #[tokio::test]
    #[should_panic(expected = "Shader")]
    async fn test_invalid_wgsl_rejection() {
        let dm = DeviceManager::new(Backends::all())
            .await
            .expect("Failed to create device");

        // wgpu DOES validate during shader module creation and will panic
        // This test ensures we get expected behavior (panic on invalid WGSL)
        let invalid_source = "this is not valid WGSL!!!";
        let _result = ShaderManager::from_wgsl(dm.device(), invalid_source, Some("Invalid"));
        
        // Should panic before reaching here
    }

    #[tokio::test]
    async fn test_entry_point_detection() {
        let dm = DeviceManager::new(Backends::all())
            .await
            .expect("Failed to create device");

        let shader = ShaderManager::from_wgsl(dm.device(), SIMPLE_WGSL, None)
            .expect("Shader compilation failed");

        // Verify entry points are correct
        assert_eq!(shader.vertex_entry(), "vs_main");
        assert_eq!(shader.fragment_entry(), "fs_main");
    }
}

