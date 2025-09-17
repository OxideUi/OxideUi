//! Prelude module for OxideUI widgets
//! 
//! This module re-exports the most commonly used types and traits from the widgets crate,
//! allowing users to import everything they need with a single `use oxide_widgets::prelude::*;`

// Re-export core types that are commonly used with widgets
pub use oxide_core::prelude::*;

// Widget trait and common types
pub use crate::widget::{Widget, WidgetId, WidgetState};

// Layout widgets
pub use crate::container::Container;
pub use crate::layout::{Column, Row, MainAxisAlignment, CrossAxisAlignment};

// Basic widgets
pub use crate::text::Text;
pub use crate::button::{Button, ButtonStyle};
pub use crate::input::TextInput;

// Theme system
pub use crate::theme::{Theme, ColorPalette, Typography};

// Common layout types from oxide-core
pub use oxide_core::layout::EdgeInsets;