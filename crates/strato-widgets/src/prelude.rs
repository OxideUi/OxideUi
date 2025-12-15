//! Prelude module for StratoUI widgets
//! 
//! This module re-exports the most commonly used types and traits from the widgets crate,
//! allowing users to import everything they need with a single `use strato_widgets::prelude::*;`

// Animation
pub use crate::animation::{AnimationController, Curve, Tween, Tweenable};

// Re-export core types that are commonly used with widgets
pub use strato_core::prelude::*;

// Widget trait and common types
pub use crate::widget::{Widget, WidgetId, WidgetState};

// Layout widgets
pub use crate::container::Container;
pub use crate::layout::{Column, Row, MainAxisAlignment, CrossAxisAlignment};
pub use crate::scroll_view::ScrollView;
pub use crate::wrap::{Wrap, WrapAlignment, WrapCrossAlignment};
pub use crate::grid::{Grid, GridUnit};

// Basic widgets
pub use crate::text::Text;
pub use crate::button::{Button, ButtonStyle};
pub use crate::input::TextInput;

// Theme system
pub use crate::theme::{Theme, ColorPalette, Typography};

// Common layout types from strato-core
pub use strato_core::layout::EdgeInsets;