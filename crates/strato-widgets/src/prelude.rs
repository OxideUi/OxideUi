//! Prelude module for StratoUI widgets
//!
//! This module re-exports the most commonly used types and traits from the widgets crate,
//! allowing users to import everything they need with a single `use strato_widgets::prelude::*;`

// Animation
pub use crate::animation::{AnimationController, Curve, Tween, Tweenable};
pub use crate::control::{ControlRole, ControlSemantics, ControlState};

// Re-export core types that are commonly used with widgets
pub use strato_core::prelude::*;
pub use strato_macros::view;

// Widget trait and common types
pub use crate::widget::{Widget, WidgetId, WidgetState};

// Layout widgets
pub use crate::container::Container;
pub use crate::grid::{Grid, GridUnit};
pub use crate::layout::{Column, CrossAxisAlignment, Flex, MainAxisAlignment, Row, Stack};
pub use crate::scroll_view::ScrollView;
pub use crate::wrap::{Wrap, WrapAlignment, WrapCrossAlignment};

// Basic widgets
pub use crate::button::{Button, ButtonStyle};
pub use crate::input::TextInput;
pub use crate::text::Text;

// Theme system
pub use crate::theme::{ColorPalette, Theme, Typography};

// Common layout types from strato-core
pub use strato_core::layout::EdgeInsets;
