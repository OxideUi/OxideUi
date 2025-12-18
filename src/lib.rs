//! StratoSDK - A lightweight, secure, and reactive UI framework
//!
//! StratoSDK provides a modern, declarative approach to building user interfaces
//! with a focus on performance, security, and developer experience.

pub use strato_core;
pub use strato_platform;
pub use strato_widgets;

// Re-export the new granular initialization system
pub use strato_platform::init::{
    get_text_renderer, init_all, init_with_config, is_initialized, InitBuilder, InitConfig,
};

use strato_core::Result;

/// Unified prelude module that exports all commonly used types
pub mod prelude {
    pub use strato_core::prelude::*;
    pub use strato_platform::{Application, ApplicationBuilder, Window, WindowBuilder};
    pub use strato_widgets::prelude::*;
}

/// Legacy initialization function - now uses the new granular system
///
/// This function is kept for backward compatibility but internally uses
/// the new InitBuilder system with default configuration.
///
/// For better control over initialization, use InitBuilder directly:
/// ```rust
/// use strato_sdk::{InitBuilder, InitConfig};
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let config = InitConfig {
///         skip_problematic_fonts: true,
///         max_font_faces: Some(50),
///         ..Default::default()
///     };
///
///     InitBuilder::new()
///         .with_config(config)
///         .init_all()?;
///     Ok(())
/// }
/// ```
#[deprecated(
    since = "0.2.0",
    note = "Use InitBuilder for better control over initialization"
)]
pub fn init_all_legacy() -> Result<()> {
    strato_core::init()?;
    strato_widgets::init()?;
    strato_platform::init()
        .map_err(|e| strato_core::StratoError::platform(format!("Platform init failed: {}", e)))?;
    Ok(())
}

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_init_all() {
        // This test ensures init_all doesn't panic
        let result = init_all();
        assert!(result.is_ok());
    }
}
