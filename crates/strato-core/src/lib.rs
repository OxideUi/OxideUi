//! Core functionality for StratoUI framework
//!
//! This crate provides the fundamental building blocks for the StratoUI framework,
//! including state management, event handling, and layout calculations.

pub mod event;
pub mod layout;
pub mod state;
pub mod reactive;
pub mod types;
pub mod error;
pub mod vdom;
pub mod widget;
pub mod window;
pub mod hot_reload;
pub mod theme;
pub mod plugin;
pub mod text;
pub mod logging;
pub mod config;
pub mod ui_node;

pub use event::{Event, EventHandler, EventResult};
pub use layout::{Constraints, Layout, LayoutEngine, LayoutConstraints, Size};
pub use state::{Signal, State};
pub use reactive::{Computed, Effect, Reactive};
pub use types::{Color, Point, Rect, Transform};
pub use error::{StratoError, StratoResult, Result};
pub use logging::{LogLevel, LogCategory};

/// Re-export commonly used types
pub mod prelude {
    pub use crate::{
        event::{Event, EventHandler, EventResult},
        layout::{Constraints, Layout, Size},
        state::{Signal, State},
        reactive::{Computed, Effect},
        types::{Color, Point, Rect},
        error::{StratoError, Result},
        logging::{LogLevel},
    };
}

/// Framework version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize the core framework
pub fn init() -> Result<()> {
    // Initialize logging system with default config
    let config = config::LoggingConfig::default();
    if let Err(e) = logging::init(&config) {
        return Err(StratoError::Initialization { 
            message: format!("Failed to initialize logging: {}", e),
            context: None,
        });
    }
    
    // Initialize tracing
    tracing::info!("StratoUI Core v{} initialized", VERSION);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}
