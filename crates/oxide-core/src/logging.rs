//! Logging system for OxideUI framework
//!
//! Provides structured logging with configurable levels and categories
//! to improve debugging experience and reduce noise in production

use std::sync::OnceLock;
use tracing::Level;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Log levels for controlling verbosity
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl From<LogLevel> for Level {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Trace => Level::TRACE,
            LogLevel::Debug => Level::DEBUG,
            LogLevel::Info => Level::INFO,
            LogLevel::Warn => Level::WARN,
            LogLevel::Error => Level::ERROR,
        }
    }
}

/// Categories for organizing log messages
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogCategory {
    Core,
    Renderer,
    Platform,
    Text,
    Layout,
    Event,
    Animation,
    Performance,
}

impl LogCategory {
    pub const fn as_str(&self) -> &'static str {
        match self {
            LogCategory::Core => "CORE",
            LogCategory::Renderer => "RENDERER",
            LogCategory::Platform => "PLATFORM",
            LogCategory::Text => "TEXT",
            LogCategory::Layout => "LAYOUT",
            LogCategory::Event => "EVENT",
            LogCategory::Animation => "ANIMATION",
            LogCategory::Performance => "PERF",
        }
    }
}

/// Global logger instance
static LOGGER: OnceLock<Logger> = OnceLock::new();

/// Logger configuration and state
pub struct Logger {
    level: LogLevel,
    enabled_categories: Vec<LogCategory>,
}

impl Logger {
    pub fn new(level: LogLevel) -> Self {
        Self {
            level,
            enabled_categories: vec![
                LogCategory::Core,
                LogCategory::Renderer,
                LogCategory::Platform,
                LogCategory::Text,
                LogCategory::Layout,
                LogCategory::Event,
                LogCategory::Animation,
                LogCategory::Performance,
            ],
        }
    }

    pub fn with_categories(mut self, categories: Vec<LogCategory>) -> Self {
        self.enabled_categories = categories;
        self
    }

    pub fn is_enabled(&self, level: LogLevel, category: LogCategory) -> bool {
        level as u8 >= self.level as u8 && self.enabled_categories.contains(&category)
    }

    pub fn set_level(&mut self, level: LogLevel) {
        self.level = level;
    }

    pub fn enable_category(&mut self, category: LogCategory) {
        if !self.enabled_categories.contains(&category) {
            self.enabled_categories.push(category);
        }
    }

    pub fn disable_category(&mut self, category: LogCategory) {
        self.enabled_categories.retain(|&c| c != category);
    }
}

/// Convenience macros for structured logging
#[macro_export]
macro_rules! oxide_trace {
    ($category:expr, $($arg:tt)*) => {
        if $crate::logging::get_logger().is_enabled($crate::logging::LogLevel::Trace, $category) {
            tracing::trace!(target: $category.as_str(), $($arg)*);
        }
    };
}

#[macro_export]
macro_rules! oxide_debug {
    ($category:expr, $($arg:tt)*) => {
        if $crate::logging::get_logger().is_enabled($crate::logging::LogLevel::Debug, $category) {
            tracing::debug!(target: $category.as_str(), $($arg)*);
        }
    };
}

#[macro_export]
macro_rules! oxide_info {
    ($category:expr, $($arg:tt)*) => {
        if $crate::logging::get_logger().is_enabled($crate::logging::LogLevel::Info, $category) {
            tracing::info!(target: $category.as_str(), $($arg)*);
        }
    };
}

#[macro_export]
macro_rules! oxide_warn {
    ($category:expr, $($arg:tt)*) => {
        if $crate::logging::get_logger().is_enabled($crate::logging::LogLevel::Warn, $category) {
            tracing::warn!(target: $category.as_str(), $($arg)*);
        }
    };
}

#[macro_export]
macro_rules! oxide_error {
    ($category:expr, $($arg:tt)*) => {
        if $crate::logging::get_logger().is_enabled($crate::logging::LogLevel::Error, $category) {
            tracing::error!(target: $category.as_str(), $($arg)*);
        }
    };
}

/// Initialize the logging system
pub fn init() {
    // Set up tracing subscriber with custom formatting
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(filter)
        .with(
            fmt::layer()
                .with_target(true)
                .with_thread_ids(false)
                .with_thread_names(false)
                .with_file(false)
                .with_line_number(false)
                .compact()
        )
        .init();

    // Initialize global logger with INFO level by default
    let _ = LOGGER.set(Logger::new(LogLevel::Info));
}

/// Get the global logger instance
pub fn get_logger() -> &'static Logger {
    LOGGER.get().unwrap_or_else(|| {
        panic!("Logger not initialized. Call oxide_core::logging::init() first.");
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_levels() {
        Logger::set_level(LogLevel::Warn);
        assert!(!Logger::should_log(LogLevel::Debug));
        assert!(Logger::should_log(LogLevel::Error));
        assert!(Logger::should_log(LogLevel::Warn));
    }

    #[test]
    fn test_log_level_ordering() {
        assert!(LogLevel::Error < LogLevel::Warn);
        assert!(LogLevel::Warn < LogLevel::Info);
        assert!(LogLevel::Info < LogLevel::Debug);
        assert!(LogLevel::Debug < LogLevel::Trace);
    }
}