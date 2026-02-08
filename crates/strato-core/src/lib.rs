//! Core functionality for StratoUI framework
//!
//! This crate provides the fundamental building blocks for the StratoUI framework,
//! including state management, event handling, and layout calculations.

pub mod config;
pub mod error;
pub mod event;
pub mod hot_reload;
pub mod inspector;
pub mod layout;
pub mod logging;
pub mod plugin;
pub mod reactive;
pub mod state;
pub mod taffy_layout;
pub mod text;
pub mod theme;
pub mod types;
pub mod ui_node;
pub mod validated_rect;
pub mod vdom;
pub mod widget;
pub mod window;

pub use error::{
    Result, StratoError, StratoResult, TaffyLayoutError, TaffyLayoutResult,
    TaffyRenderError, TaffyRenderResult, TaffyValidationError, TaffyValidationResult,
};
pub use event::{Event, EventHandler, EventResult};
pub use layout::{Constraints, Layout, LayoutConstraints, LayoutEngine, Size};
pub use logging::{LogCategory, LogLevel};
pub use reactive::{Computed, Effect, Reactive};
pub use state::{Signal, State};
pub use taffy;
pub use taffy_layout::{ComputedLayout, DrawCommand, TaffyLayoutManager, TaffyWidget};
pub use types::{Color, Point, Rect, Transform};
pub use validated_rect::ValidatedRect;

/// Re-export commonly used types
pub mod prelude {
    pub use crate::{
        error::{Result, StratoError},
        event::{Event, EventHandler, EventResult},
        inspector::{inspector, InspectorConfig, InspectorSnapshot},
        layout::{Constraints, Layout, Size},
        logging::LogLevel,
        reactive::{Computed, Effect},
        state::{Signal, State},
        types::{Color, Point, Rect},
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
