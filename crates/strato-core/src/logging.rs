//! Logging system for StratoUI framework
//!
//! This module provides a comprehensive logging system with rate limiting,
//! contextual error information, and category-based filtering.

use crate::config::LoggingConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, OnceLock, RwLock};
use std::time::{Duration, Instant};

/// Log levels supported by the system
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

/// Log categories for organizing log messages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LogCategory {
    Core,
    Renderer,
    Vulkan,
    Text,
    UI,
    Input,
    Audio,
    Network,
    Plugin,
    Platform,
}

impl LogCategory {
    /// Convert LogCategory to string
    pub fn as_str(&self) -> &'static str {
        match self {
            LogCategory::Core => "core",
            LogCategory::Renderer => "renderer",
            LogCategory::Vulkan => "vulkan",
            LogCategory::Text => "text",
            LogCategory::UI => "ui",
            LogCategory::Input => "input",
            LogCategory::Audio => "audio",
            LogCategory::Network => "network",
            LogCategory::Plugin => "plugin",
            LogCategory::Platform => "platform",
        }
    }
}

impl std::fmt::Display for LogCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl LogLevel {
    /// Convert string to LogLevel
    pub fn from_str(s: &str) -> Option<LogLevel> {
        match s.to_lowercase().as_str() {
            "trace" => Some(LogLevel::Trace),
            "debug" => Some(LogLevel::Debug),
            "info" => Some(LogLevel::Info),
            "warn" => Some(LogLevel::Warn),
            "error" => Some(LogLevel::Error),
            _ => None,
        }
    }

    /// Convert LogLevel to string
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Trace => "trace",
            LogLevel::Debug => "debug",
            LogLevel::Info => "info",
            LogLevel::Warn => "warn",
            LogLevel::Error => "error",
        }
    }
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Rate limiting state for a specific category
#[derive(Debug)]
struct RateLimitState {
    last_reset: Instant,
    count: u32,
    max_count: u32,
    duration: Duration,
}

impl RateLimitState {
    fn new(max_count: u32, duration: Duration) -> Self {
        Self {
            last_reset: Instant::now(),
            count: 0,
            max_count,
            duration,
        }
    }

    fn should_allow(&mut self) -> bool {
        let now = Instant::now();

        // Reset counter if duration has passed
        if now.duration_since(self.last_reset) >= self.duration {
            self.last_reset = now;
            self.count = 0;
        }

        if self.count < self.max_count {
            self.count += 1;
            true
        } else {
            false
        }
    }
}

/// Logger configuration and state
#[derive(Debug)]
pub struct LoggerConfig {
    rate_limiters: Arc<RwLock<HashMap<String, RateLimitState>>>,
    config: LoggingConfig,
}

impl LoggerConfig {
    /// Create a new logger configuration
    pub fn new(config: LoggingConfig) -> Self {
        Self {
            rate_limiters: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Check if a log message should be allowed based on rate limiting
    pub fn should_allow_log(&self, category: &str) -> bool {
        let mut limiters = self.rate_limiters.write().unwrap();

        let limiter = limiters.entry(category.to_string()).or_insert_with(|| {
            RateLimitState::new(
                self.config.max_rate_limit_count,
                Duration::from_secs(self.config.rate_limit_seconds),
            )
        });

        limiter.should_allow()
    }

    /// Check if a log level is enabled for a category
    pub fn is_level_enabled(&self, category: &str, level: LogLevel) -> bool {
        if let Some(category_level_str) = self.config.category_levels.get(category) {
            if let Some(category_level) = LogLevel::from_str(category_level_str) {
                return level >= category_level;
            }
        }

        // Default to Info level if category not found
        level >= LogLevel::Info
    }

    /// Update the configuration
    pub fn update_config(&mut self, config: LoggingConfig) {
        self.config = config;
        // Clear rate limiters to apply new settings
        self.rate_limiters.write().unwrap().clear();
    }
}

/// Global logger instance
static LOGGER: OnceLock<Arc<RwLock<LoggerConfig>>> = OnceLock::new();

/// Initialize the logging system
pub fn init(config: &LoggingConfig) -> Result<(), Box<dyn std::error::Error>> {
    let logger_config = LoggerConfig::new(config.clone());
    LOGGER.set(Arc::new(RwLock::new(logger_config))).ok();
    Ok(())
}

/// Get the global logger instance
fn get_logger() -> Option<Arc<RwLock<LoggerConfig>>> {
    LOGGER.get().cloned()
}

/// Internal logging function
pub fn log_internal(level: LogLevel, category: &str, message: &str, rate_limited: bool) {
    if let Some(logger) = get_logger() {
        let logger_guard = logger.read().unwrap();

        // Check if level is enabled for this category
        if !logger_guard.is_level_enabled(category, level) {
            return;
        }

        // Check rate limiting if requested
        if rate_limited && !logger_guard.should_allow_log(category) {
            return;
        }

        drop(logger_guard); // Release the lock before printing

        // Format and print the log message
        let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f");
        println!(
            "[{}] [{}] [{}] {}",
            timestamp,
            level.as_str().to_uppercase(),
            category,
            message
        );
    } else {
        // Fallback if logger not initialized
        println!(
            "[UNINITIALIZED] [{}] [{}] {}",
            level.as_str().to_uppercase(),
            category,
            message
        );
    }
}

/// Update logger configuration
pub fn update_config(config: LoggingConfig) {
    if let Some(logger) = get_logger() {
        let mut logger_guard = logger.write().unwrap();
        logger_guard.update_config(config);
    }
}

// Core logging macros
#[macro_export]
macro_rules! strato_trace {
    ($category:expr, $($arg:tt)*) => {
        $crate::logging::log_internal($crate::logging::LogLevel::Trace, &$category.to_string(), &format!($($arg)*), false);
    };
}

#[macro_export]
macro_rules! strato_debug {
    ($category:expr, $($arg:tt)*) => {
        $crate::logging::log_internal($crate::logging::LogLevel::Debug, &$category.to_string(), &format!($($arg)*), false);
    };
}

#[macro_export]
macro_rules! strato_info {
    ($category:expr, $($arg:tt)*) => {
        $crate::logging::log_internal($crate::logging::LogLevel::Info, &$category.to_string(), &format!($($arg)*), false);
    };
}

#[macro_export]
macro_rules! strato_warn {
    ($category:expr, $($arg:tt)*) => {
        $crate::logging::log_internal($crate::logging::LogLevel::Warn, &$category.to_string(), &format!($($arg)*), false);
    };
}

#[macro_export]
macro_rules! strato_error {
    ($category:expr, $($arg:tt)*) => {
        $crate::logging::log_internal($crate::logging::LogLevel::Error, &$category.to_string(), &format!($($arg)*), false);
    };
}

// Rate-limited logging macros
#[macro_export]
macro_rules! strato_trace_rate_limited {
    ($category:expr, $($arg:tt)*) => {
        $crate::logging::log_internal($crate::logging::LogLevel::Trace, &$category.to_string(), &format!($($arg)*), true);
    };
}

#[macro_export]
macro_rules! strato_debug_rate_limited {
    ($category:expr, $($arg:tt)*) => {
        $crate::logging::log_internal($crate::logging::LogLevel::Debug, &$category.to_string(), &format!($($arg)*), true);
    };
}

#[macro_export]
macro_rules! strato_info_rate_limited {
    ($category:expr, $($arg:tt)*) => {
        $crate::logging::log_internal($crate::logging::LogLevel::Info, &$category.to_string(), &format!($($arg)*), true);
    };
}

#[macro_export]
macro_rules! strato_warn_rate_limited {
    ($category:expr, $($arg:tt)*) => {
        $crate::logging::log_internal($crate::logging::LogLevel::Warn, &$category.to_string(), &format!($($arg)*), true);
    };
}

#[macro_export]
macro_rules! strato_error_rate_limited {
    ($category:expr, $($arg:tt)*) => {
        $crate::logging::log_internal($crate::logging::LogLevel::Error, &$category.to_string(), &format!($($arg)*), true);
    };
}

// Category-specific macros
#[macro_export]
macro_rules! strato_text_debug {
    ($($arg:tt)*) => {
        $crate::logging::log_internal($crate::logging::LogLevel::Debug, "text", &format!($($arg)*), false);
    };
}

// Re-export macros for easier use
pub use strato_debug;
pub use strato_debug_rate_limited;
pub use strato_error;
pub use strato_error_rate_limited;
pub use strato_info;
pub use strato_info_rate_limited;
pub use strato_text_debug;
pub use strato_trace;
pub use strato_trace_rate_limited;
pub use strato_warn;
pub use strato_warn_rate_limited;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_log_level_conversion() {
        assert_eq!(LogLevel::from_str("info"), Some(LogLevel::Info));
        assert_eq!(LogLevel::from_str("INFO"), Some(LogLevel::Info));
        assert_eq!(LogLevel::from_str("invalid"), None);

        assert_eq!(LogLevel::Info.as_str(), "info");
        assert_eq!(LogLevel::Error.as_str(), "error");
    }

    #[test]
    fn test_rate_limiting() {
        let mut state = RateLimitState::new(2, Duration::from_millis(100));

        // First two should be allowed
        assert!(state.should_allow());
        assert!(state.should_allow());

        // Third should be blocked
        assert!(!state.should_allow());

        // After waiting, should be allowed again
        std::thread::sleep(Duration::from_millis(150));
        assert!(state.should_allow());
    }

    #[test]
    fn test_logger_config() {
        let mut category_levels = HashMap::new();
        category_levels.insert("test".to_string(), "debug".to_string());

        let config = LoggingConfig {
            category_levels,
            enable_text_debug: true,
            enable_layout_debug: false,
            rate_limit_seconds: 1,
            max_rate_limit_count: 5,
        };

        let logger_config = LoggerConfig::new(config);

        // Test level checking
        assert!(logger_config.is_level_enabled("test", LogLevel::Debug));
        assert!(logger_config.is_level_enabled("test", LogLevel::Error));
        assert!(!logger_config.is_level_enabled("test", LogLevel::Trace));

        // Test rate limiting
        assert!(logger_config.should_allow_log("test"));
    }
}
