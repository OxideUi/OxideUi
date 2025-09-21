//! Granular initialization system for OxideUI
//!
//! This module provides fine-grained control over the initialization process,
//! allowing developers to customize font loading, logging, and other aspects
//! of the framework initialization.

use std::sync::{Arc, RwLock, OnceLock};
use oxide_core::{Result, OxideError};
use oxide_renderer::text::TextRenderer;

/// Global text renderer instance to avoid multiple cosmic_text initializations
static GLOBAL_TEXT_RENDERER: OnceLock<Arc<RwLock<TextRenderer>>> = OnceLock::new();

/// Configuration for OxideUI initialization
#[derive(Debug, Clone)]
pub struct InitConfig {
    /// Enable detailed logging during initialization
    pub enable_logging: bool,
    /// Skip problematic system fonts (like mstmc.ttf)
    pub skip_problematic_fonts: bool,
    /// Maximum number of font faces to load (None = unlimited)
    pub max_font_faces: Option<usize>,
    /// Custom font directories to search
    pub custom_font_dirs: Vec<String>,
    /// Preferred fonts to load first
    pub preferred_fonts: Vec<String>,
}

impl Default for InitConfig {
    fn default() -> Self {
        Self {
            enable_logging: false,
            skip_problematic_fonts: true,
            max_font_faces: Some(50),
            custom_font_dirs: Vec::new(),
            preferred_fonts: vec![
                "Segoe UI".to_string(),
                "Arial".to_string(),
                "Helvetica".to_string(),
            ],
        }
    }
}

/// Builder for step-by-step initialization
pub struct InitBuilder {
    config: InitConfig,
    core_initialized: bool,
    widgets_initialized: bool,
    platform_initialized: bool,
}

impl InitBuilder {
    /// Create a new initialization builder
    pub fn new() -> Self {
        Self {
            config: InitConfig::default(),
            core_initialized: false,
            widgets_initialized: false,
            platform_initialized: false,
        }
    }

    /// Set custom configuration
    pub fn with_config(mut self, config: InitConfig) -> Self {
        self.config = config;
        self
    }

    /// Initialize only the core module
    pub fn init_core(&mut self) -> Result<&mut Self> {
        if self.core_initialized {
            return Ok(self);
        }

        // Initialize core first (which includes logging)
        oxide_core::init()?;
        self.core_initialized = true;

        if self.config.enable_logging {
            oxide_core::oxide_trace!(oxide_core::logging::LogCategory::Core, "Core initialized");
        }

        Ok(self)
    }

    /// Initialize only the widgets module
    pub fn init_widgets(&mut self) -> Result<&mut Self> {
        if !self.core_initialized {
            self.init_core()?;
        }

        if self.widgets_initialized {
            return Ok(self);
        }

        if self.config.enable_logging {
            oxide_core::oxide_trace!(oxide_core::logging::LogCategory::Core, "Initializing widgets...");
        }

        oxide_widgets::init()?;
        self.widgets_initialized = true;

        if self.config.enable_logging {
            oxide_core::oxide_trace!(oxide_core::logging::LogCategory::Core, "Widgets initialized");
        }

        Ok(self)
    }

    /// Initialize only the platform module with optimized font loading
    pub fn init_platform(&mut self) -> Result<&mut Self> {
        if !self.core_initialized {
            self.init_core()?;
        }

        if self.platform_initialized {
            return Ok(self);
        }

        if self.config.enable_logging {
            oxide_core::oxide_trace!(oxide_core::logging::LogCategory::Platform, "Initializing platform with font optimizations...");
        }

        // Initialize platform with our custom configuration
        crate::init().map_err(|e| oxide_core::OxideError::platform(format!("Platform init failed: {}", e)))?;

        // Initialize the global text renderer with optimizations
        self.init_optimized_text_renderer()?;

        self.platform_initialized = true;

        if self.config.enable_logging {
            oxide_core::oxide_trace!(oxide_core::logging::LogCategory::Platform, "Platform initialized");
        }

        Ok(self)
    }

    /// Initialize all modules at once
    pub fn init_all(&mut self) -> Result<()> {
        self.init_core()?
            .init_widgets()?
            .init_platform()?;
        Ok(())
    }

    /// Initialize the optimized text renderer
    fn init_optimized_text_renderer(&self) -> Result<()> {
        if GLOBAL_TEXT_RENDERER.get().is_some() {
            return Ok(()); // Already initialized
        }

        // Create optimized text renderer
        oxide_core::oxide_trace!(oxide_core::logging::LogCategory::Platform, "Creating optimized TextRenderer");
        let text_renderer = TextRenderer::new();

        GLOBAL_TEXT_RENDERER.set(Arc::new(RwLock::new(text_renderer)))
            .map_err(|_| OxideError::platform("Failed to set global text renderer".to_string()))?;

        Ok(())
    }
}

impl Default for InitBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Get the global text renderer instance
pub fn get_text_renderer() -> Option<Arc<RwLock<TextRenderer>>> {
    GLOBAL_TEXT_RENDERER.get().cloned()
}



/// Convenience function for full initialization with default config
pub fn init_all() -> Result<()> {
    InitBuilder::new().init_all()
}

/// Convenience function for initialization with custom config
pub fn init_with_config(config: InitConfig) -> Result<()> {
    InitBuilder::new().with_config(config).init_all()
}

/// Check if the framework is fully initialized
pub fn is_initialized() -> bool {
    GLOBAL_TEXT_RENDERER.get().is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_config_default() {
        let config = InitConfig::default();
        assert!(!config.enable_logging);
        assert!(config.skip_problematic_fonts);
        assert_eq!(config.max_font_faces, Some(50));
    }

    #[test]
    fn test_init_builder() {
        let mut builder = InitBuilder::new();
        assert!(!builder.core_initialized);
        assert!(!builder.widgets_initialized);
        assert!(!builder.platform_initialized);
    }
}