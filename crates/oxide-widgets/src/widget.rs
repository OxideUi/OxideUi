//! Base widget trait and common functionality

use oxide_core::{
    event::{Event, EventResult},
    layout::{Constraints, Layout, Size},
    types::{Point}, // Removed unused Color and Rect imports
};
use oxide_renderer::batch::RenderBatch;
use std::any::Any;
use std::fmt::Debug;

/// Unique widget identifier
pub type WidgetId = u64;

/// Widget state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WidgetState {
    Normal,
    Hovered,
    Pressed,
    Focused,
    Disabled,
}

/// Widget context passed during updates
pub struct WidgetContext<'a> {
    pub theme: &'a crate::theme::Theme,
    pub state: WidgetState,
    pub is_focused: bool,
    pub is_hovered: bool,
    pub delta_time: f32,
}

/// Base trait for all widgets
pub trait Widget: Debug + Send + Sync {
    /// Get the widget's unique ID
    fn id(&self) -> WidgetId;

    /// Calculate the widget's size given constraints
    fn layout(&mut self, constraints: Constraints) -> Size;

    /// Render the widget
    fn render(&self, batch: &mut RenderBatch, layout: Layout);

    /// Handle an event
    fn handle_event(&mut self, _event: &Event) -> EventResult {
        EventResult::Ignored
    }

    /// Update the widget state
    fn update(&mut self, _ctx: &WidgetContext) {}

    /// Get children widgets
    fn children(&self) -> Vec<&(dyn Widget + '_)> {
        vec![]
    }

    /// Get mutable children widgets
    fn children_mut(&mut self) -> Vec<&mut (dyn Widget + '_)> {
        vec![]
    }

    /// Check if point is inside widget
    fn hit_test(&self, point: Point, layout: Layout) -> bool {
        layout.contains(point.to_vec2())
    }

    /// Get widget as Any for downcasting
    fn as_any(&self) -> &dyn Any;

    /// Get mutable widget as Any for downcasting
    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// Clone the widget
    fn clone_widget(&self) -> Box<dyn Widget>;
}

/// Generate a unique widget ID
pub fn generate_id() -> WidgetId {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    COUNTER.fetch_add(1, Ordering::SeqCst)
}

/// Base widget implementation helper
#[derive(Debug, Clone)]
pub struct BaseWidget {
    id: WidgetId,
    size: Size,
    min_size: Size,
    max_size: Size,
    flex_grow: f32,
    flex_shrink: f32,
}

impl BaseWidget {
    /// Create a new base widget
    pub fn new() -> Self {
        Self {
            id: generate_id(),
            size: Size::zero(),
            min_size: Size::zero(),
            max_size: Size::new(f32::INFINITY, f32::INFINITY),
            flex_grow: 0.0,
            flex_shrink: 1.0,
        }
    }

    /// Set minimum size
    pub fn with_min_size(mut self, width: f32, height: f32) -> Self {
        self.min_size = Size::new(width, height);
        self
    }

    /// Set maximum size
    pub fn with_max_size(mut self, width: f32, height: f32) -> Self {
        self.max_size = Size::new(width, height);
        self
    }

    /// Set flex grow factor
    pub fn with_flex_grow(mut self, factor: f32) -> Self {
        self.flex_grow = factor;
        self
    }

    /// Set flex shrink factor
    pub fn with_flex_shrink(mut self, factor: f32) -> Self {
        self.flex_shrink = factor;
        self
    }

    /// Calculate size within constraints
    pub fn calculate_size(&self, constraints: Constraints) -> Size {
        Size::new(
            self.size.width.clamp(
                self.min_size.width.max(constraints.min_width),
                self.max_size.width.min(constraints.max_width),
            ),
            self.size.height.clamp(
                self.min_size.height.max(constraints.min_height),
                self.max_size.height.min(constraints.max_height),
            ),
        )
    }
}

impl Default for BaseWidget {
    fn default() -> Self {
        Self::new()
    }
}

/// Widget event handler
pub trait EventHandler: Send + Sync {
    /// Handle widget event
    fn handle(&mut self, event: &Event, widget_id: WidgetId) -> EventResult;
}

/// Widget lifecycle
pub trait Lifecycle {
    /// Called when widget is mounted
    fn on_mount(&mut self) {}

    /// Called when widget is unmounted
    fn on_unmount(&mut self) {}

    /// Called when widget needs to update
    fn on_update(&mut self, _delta_time: f32) {}
}

/// Focusable widget trait
pub trait Focusable {
    /// Check if widget can receive focus
    fn can_focus(&self) -> bool {
        true
    }

    /// Called when widget gains focus
    fn on_focus(&mut self) {}

    /// Called when widget loses focus
    fn on_blur(&mut self) {}
}

/// Hoverable widget trait
pub trait Hoverable {
    /// Called when mouse enters widget
    fn on_mouse_enter(&mut self) {}

    /// Called when mouse leaves widget
    fn on_mouse_exit(&mut self) {}

    /// Called when mouse moves over widget
    fn on_mouse_move(&mut self, _position: Point) {}
}
