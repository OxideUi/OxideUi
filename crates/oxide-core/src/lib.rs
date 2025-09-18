//! Core functionality for OxideUI framework
//!
//! This crate provides the fundamental building blocks for the OxideUI framework,
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

pub use event::{Event, EventHandler, EventResult};
pub use layout::{Constraints, Layout, LayoutEngine, LayoutConstraints, Size};
pub use state::{Signal, State};
pub use reactive::{Computed, Effect, Reactive};
pub use types::{Color, Point, Rect, Transform};
pub use error::{OxideError, OxideResult, Result};
pub use logging::{Logger, LogLevel, LogCategory};

/// Re-export commonly used types
pub mod prelude {
    pub use crate::{
        event::{Event, EventHandler, EventResult},
        layout::{Constraints, Layout, Size},
        state::{Signal, State},
        reactive::{Computed, Effect},
        types::{Color, Point, Rect},
        error::{OxideError, Result},
        logging::{Logger, LogLevel, LogCategory},
    };
}

/// Framework version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize the core framework
pub fn init() -> Result<()> {
    // Initialize logging system
    logging::init();
    
    // Initialize tracing
    tracing::info!("OxideUI Core v{} initialized", VERSION);
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
