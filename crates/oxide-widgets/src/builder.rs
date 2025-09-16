//! Widget builder utilities

use crate::widget::Widget;
// Removed unused oxide_core::layout::Constraints import

/// Widget builder for fluent API
pub struct WidgetBuilder<W: Widget> {
    widget: W,
}

impl<W: Widget> WidgetBuilder<W> {
    /// Create a new widget builder
    pub fn new(widget: W) -> Self {
        Self { widget }
    }

    /// Build the widget
    pub fn build(self) -> W {
        self.widget
    }
}

/// Extension trait for widget building
pub trait BuilderExt: Sized {
    /// Wrap in a builder
    fn builder(self) -> WidgetBuilder<Self>
    where
        Self: Widget,
    {
        WidgetBuilder::new(self)
    }
}

impl<T> BuilderExt for T where T: Widget {}
