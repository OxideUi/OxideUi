//! OxideUI - A lightweight, secure, and reactive UI framework
//!
//! OxideUI provides a modern, declarative approach to building user interfaces
//! with a focus on performance, security, and developer experience.

pub use oxide_core as core;
pub use oxide_widgets as widgets;
pub use oxide_platform as platform;

// Re-export commonly used types
pub use oxide_core::{OxideError, Result};
pub use oxide_widgets::prelude::*;
pub use oxide_platform::{Application, ApplicationBuilder, Window, WindowBuilder};

/// Unified prelude module that exports all commonly used types
pub mod prelude {
    pub use oxide_core::prelude::*;
    pub use oxide_widgets::prelude::*;
    pub use oxide_platform::{Application, ApplicationBuilder, Window, WindowBuilder};
}

/// Initialize all OxideUI modules at once
/// 
/// This is a convenience function that initializes all core modules
/// in the correct order. Use this for simple applications that don't
/// need fine-grained control over initialization.
pub fn init_all() -> Result<()> {
    oxide_core::init()?;
    oxide_widgets::init()?;
    oxide_platform::init().map_err(|e| OxideError::platform(e.to_string()))?;
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