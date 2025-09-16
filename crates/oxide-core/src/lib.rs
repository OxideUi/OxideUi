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

pub use event::{Event, EventHandler, EventResult};
pub use layout::{Constraints, Layout, LayoutEngine, Size};
pub use state::{Signal, State, StateManager};
pub use reactive::{Computed, Effect, Reactive};
pub use types::{Color, Point, Rect, Transform};
pub use error::{OxideError, Result};

/// Re-export commonly used types
pub mod prelude {
    pub use crate::{
        event::{Event, EventHandler, EventResult},
        layout::{Constraints, Layout, Size},
        state::{Signal, State},
        reactive::{Computed, Effect},
        types::{Color, Point, Rect},
        error::{OxideError, Result},
    };
}

/// Framework version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize the core framework
pub fn init() -> Result<()> {
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
