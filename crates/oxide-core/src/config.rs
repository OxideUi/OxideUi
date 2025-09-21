//! Configuration system for OxideUI framework

use std::collections::HashMap;
use std::sync::{Arc, RwLock, OnceLock};
use serde::{Serialize, Deserialize};

/// Global configuration for OxideUI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OxideConfig {
    /// Logging configuration
    pub logging: LoggingConfig,
    /// Performance and optimization settings
    pub performance: PerformanceConfig,
    /// Debug and development settings
    pub debug: DebugConfig,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Category-specific log levels (category name -> level string)
    pub category_levels: HashMap<String, String>,
    /// Rate limiting duration in seconds
    pub rate_limit_seconds: u64,
    /// Maximum number of messages before rate limiting kicks in
    pub max_rate_limit_count: u32,
    /// Enable text rendering debug logs
    pub enable_text_debug: bool,
    /// Enable layout debug logs
    pub enable_layout_debug: bool,
}

/// Performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Enable GPU profiling
    pub enable_gpu_profiling: bool,
    /// Enable CPU profiling
    pub enable_cpu_profiling: bool,
    /// Maximum frame rate (0 = unlimited)
    pub max_fps: u32,
    /// Enable VSync
    pub enable_vsync: bool,
}

/// Debug configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugConfig {
    /// Show debug overlay
    pub show_debug_overlay: bool,
    /// Show frame timing information
    pub show_frame_timing: bool,
    /// Show memory usage
    pub show_memory_usage: bool,
    /// Enable wireframe rendering
    pub enable_wireframe: bool,
}

impl Default for OxideConfig {
    fn default() -> Self {
        let mut category_levels = HashMap::new();
        
        // Set default levels for each category
        category_levels.insert("core".to_string(), "info".to_string());
        category_levels.insert("renderer".to_string(), "info".to_string());
        category_levels.insert("ui".to_string(), "info".to_string());
        category_levels.insert("input".to_string(), "info".to_string());
        category_levels.insert("audio".to_string(), "info".to_string());
        category_levels.insert("network".to_string(), "info".to_string());
        category_levels.insert("vulkan".to_string(), "warn".to_string()); // Reduce Vulkan noise
        category_levels.insert("text".to_string(), "error".to_string()); // Disable text debug by default
        category_levels.insert("layout".to_string(), "error".to_string()); // Disable layout debug by default
        
        Self {
            logging: LoggingConfig {
                category_levels,
                enable_text_debug: false,
                enable_layout_debug: false,
                rate_limit_seconds: 5,
                max_rate_limit_count: 10,
            },
            performance: PerformanceConfig {
                enable_gpu_profiling: false,
                enable_cpu_profiling: false,
                max_fps: 60,
                enable_vsync: true,
            },
            debug: DebugConfig {
                show_debug_overlay: false,
                show_frame_timing: false,
                show_memory_usage: false,
                enable_wireframe: false,
            },
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        OxideConfig::default().logging
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        OxideConfig::default().performance
    }
}

impl Default for DebugConfig {
    fn default() -> Self {
        OxideConfig::default().debug
    }
}

/// Global configuration manager
pub struct ConfigManager {
    config: Arc<RwLock<OxideConfig>>,
}

impl ConfigManager {
    /// Create a new configuration manager with default settings
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(OxideConfig::default())),
        }
    }
    
    /// Create a configuration manager with custom config
    pub fn with_config(config: OxideConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
        }
    }
    
    /// Get the global configuration manager instance
    pub fn instance() -> Option<&'static ConfigManager> {
        CONFIG_MANAGER.get()
    }
    
    /// Get a copy of the current configuration
    pub fn get_config(&self) -> OxideConfig {
        self.config.read().unwrap().clone()
    }
    
    /// Update the configuration
    pub fn update_config<F>(&self, updater: F) 
    where 
        F: FnOnce(&mut OxideConfig),
    {
        let mut config = self.config.write().unwrap();
        updater(&mut *config);
    }
    
    /// Get the current logging configuration
    pub fn get_logging_config(&self) -> LoggingConfig {
        self.config.read().unwrap().logging.clone()
    }
    
    /// Update logging configuration
    pub fn update_logging_config<F>(&self, updater: F)
    where
        F: FnOnce(&mut LoggingConfig),
    {
        let mut config = self.config.write().unwrap();
        updater(&mut config.logging);
    }
    
    /// Enable or disable text debug logging
    pub fn set_text_debug(&self, enabled: bool) {
        self.update_logging_config(|logging| {
            logging.enable_text_debug = enabled;
            if enabled {
                logging.category_levels.insert("text".to_string(), "debug".to_string());
            } else {
                logging.category_levels.insert("text".to_string(), "error".to_string());
            }
        });
    }
    
    /// Enable or disable layout debug logging
    pub fn set_layout_debug(&self, enabled: bool) {
        self.update_logging_config(|logging| {
            logging.enable_layout_debug = enabled;
            if enabled {
                logging.category_levels.insert("layout".to_string(), "debug".to_string());
            } else {
                logging.category_levels.insert("layout".to_string(), "error".to_string());
            }
        });
    }
    
    /// Set log level for a specific category
    pub fn set_category_level(&self, category: &str, level: &str) {
        self.update_logging_config(|logging| {
            logging.category_levels.insert(category.to_string(), level.to_string());
        });
    }
    
    /// Get log level for a specific category
    pub fn get_category_level(&self, category: &str) -> Option<String> {
        let config = self.config.read().unwrap();
        config.logging.category_levels.get(category).cloned()
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Global configuration instance
static CONFIG_MANAGER: OnceLock<ConfigManager> = OnceLock::new();

/// Initialize the global configuration manager
pub fn init_config() -> &'static ConfigManager {
    CONFIG_MANAGER.get_or_init(ConfigManager::new)
}

/// Initialize the global configuration manager with custom config
pub fn init_config_with(config: OxideConfig) -> &'static ConfigManager {
    CONFIG_MANAGER.get_or_init(|| ConfigManager::with_config(config))
}

/// Get the global configuration manager
pub fn get_config_manager() -> Option<&'static ConfigManager> {
    CONFIG_MANAGER.get()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = OxideConfig::default();
        
        assert!(!config.logging.enable_text_debug);
        assert!(!config.logging.enable_layout_debug);
        
        // Text and Layout should be disabled by default
        assert_eq!(config.logging.category_levels.get("text"), Some(&"error".to_string()));
        assert_eq!(config.logging.category_levels.get("layout"), Some(&"error".to_string()));
        
        // Vulkan should be at Warn level to reduce noise
        assert_eq!(config.logging.category_levels.get("vulkan"), Some(&"warn".to_string()));
    }
    
    #[test]
    fn test_config_manager() {
        let manager = ConfigManager::new();
        
        // Test text debug toggle
        manager.set_text_debug(true);
        assert_eq!(manager.get_category_level("text"), Some("debug".to_string()));
        
        manager.set_text_debug(false);
        assert_eq!(manager.get_category_level("text"), Some("error".to_string()));
        
        // Test category level setting
        manager.set_category_level("renderer", "trace");
        assert_eq!(manager.get_category_level("renderer"), Some("trace".to_string()));
    }
}