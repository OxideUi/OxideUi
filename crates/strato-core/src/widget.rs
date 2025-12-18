//! Core widget traits and types for StratoUI
//!
//! This module provides the fundamental widget system that all UI components
//! are built upon. It defines the core traits and types needed for widget
//! creation, rendering, and interaction.

use crate::{
    error::StratoResult,
    event::Event,
    layout::{LayoutConstraints, Size},
    types::Rect,
};
use std::{any::Any, collections::HashMap, fmt::Debug};

/// Unique identifier for widgets
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WidgetId(pub u64);

impl WidgetId {
    /// Create a new widget ID
    pub fn new() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for WidgetId {
    fn default() -> Self {
        Self::new()
    }
}

/// Widget context containing shared state and services
#[derive(Debug)]
pub struct WidgetContext {
    /// Widget's unique identifier
    pub id: WidgetId,
    /// Current layout bounds
    pub bounds: Rect,
    /// Whether the widget is focused
    pub is_focused: bool,
    /// Whether the widget is hovered
    pub is_hovered: bool,
    /// Whether the widget is pressed
    pub is_pressed: bool,
    /// Custom properties
    pub properties: HashMap<String, Box<dyn Any>>,
}

impl WidgetContext {
    /// Create a new widget context
    pub fn new(id: WidgetId) -> Self {
        Self {
            id,
            bounds: Rect::default(),
            is_focused: false,
            is_hovered: false,
            is_pressed: false,
            properties: HashMap::new(),
        }
    }

    /// Set a custom property
    pub fn set_property<T: Any>(&mut self, key: &str, value: T) {
        self.properties.insert(key.to_string(), Box::new(value));
    }

    /// Get a custom property
    pub fn get_property<T: Any>(&self, key: &str) -> Option<&T> {
        self.properties.get(key)?.downcast_ref()
    }
}

/// Core widget trait that all widgets must implement
pub trait Widget: Debug + Send + Sync {
    /// Get the widget's unique identifier
    fn id(&self) -> WidgetId;

    /// Handle an event
    fn handle_event(&mut self, event: &Event, context: &mut WidgetContext) -> StratoResult<bool>;

    /// Update the widget's state
    fn update(&mut self, context: &mut WidgetContext) -> StratoResult<()>;

    /// Calculate the widget's layout
    fn layout(
        &mut self,
        constraints: &LayoutConstraints,
        context: &mut WidgetContext,
    ) -> StratoResult<Size>;

    /// Render the widget
    fn render(&self, context: &WidgetContext) -> StratoResult<()>;

    /// Get the widget as Any for downcasting
    fn as_any(&self) -> &dyn Any;

    /// Get the widget as mutable Any for downcasting
    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// Get child widgets
    fn children(&self) -> Vec<&dyn Widget> {
        Vec::new()
    }

    /// Get mutable child widgets
    fn children_mut(&mut self) -> Vec<&mut dyn Widget> {
        Vec::new()
    }

    /// Check if the widget can receive focus
    fn can_focus(&self) -> bool {
        false
    }

    /// Check if the widget is visible
    fn is_visible(&self) -> bool {
        true
    }

    /// Get the widget's preferred size
    fn preferred_size(&self) -> Option<Size> {
        None
    }

    /// Get the widget's minimum size
    fn min_size(&self) -> Size {
        Size::new(0.0, 0.0)
    }

    /// Get the widget's maximum size
    fn max_size(&self) -> Size {
        Size::new(f32::INFINITY, f32::INFINITY)
    }
}

/// Implement Widget for Box<T> to allow boxed widgets to be treated as widgets
/// Implement Widget for Box<T> to allow boxed widgets to be treated as widgets
impl<T: Widget + ?Sized> Widget for Box<T> {
    fn id(&self) -> WidgetId {
        (**self).id()
    }

    fn handle_event(&mut self, event: &Event, context: &mut WidgetContext) -> StratoResult<bool> {
        (**self).handle_event(event, context)
    }

    fn update(&mut self, context: &mut WidgetContext) -> StratoResult<()> {
        (**self).update(context)
    }

    fn layout(
        &mut self,
        constraints: &LayoutConstraints,
        context: &mut WidgetContext,
    ) -> StratoResult<Size> {
        (**self).layout(constraints, context)
    }

    fn render(&self, context: &WidgetContext) -> StratoResult<()> {
        (**self).render(context)
    }

    fn as_any(&self) -> &dyn Any {
        (**self).as_any()
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        (**self).as_any_mut()
    }

    fn children(&self) -> Vec<&dyn Widget> {
        (**self).children()
    }

    fn children_mut(&mut self) -> Vec<&mut dyn Widget> {
        (**self).children_mut()
    }

    fn can_focus(&self) -> bool {
        (**self).can_focus()
    }

    fn is_visible(&self) -> bool {
        (**self).is_visible()
    }

    fn preferred_size(&self) -> Option<Size> {
        (**self).preferred_size()
    }

    fn min_size(&self) -> Size {
        (**self).min_size()
    }

    fn max_size(&self) -> Size {
        (**self).max_size()
    }
}

/// Widget builder trait for creating widgets with a fluent API
pub trait WidgetBuilder<T: Widget> {
    /// Build the widget
    fn build(self) -> T;
}

/// Widget tree for managing widget hierarchy
#[derive(Debug)]
pub struct WidgetTree {
    root: Option<Box<dyn Widget>>,
    widgets: HashMap<WidgetId, Box<dyn Widget>>,
}

impl WidgetTree {
    /// Create a new widget tree
    pub fn new() -> Self {
        Self {
            root: None,
            widgets: HashMap::new(),
        }
    }

    /// Set the root widget
    pub fn set_root(&mut self, widget: Box<dyn Widget>) {
        let id = widget.id();
        self.widgets.insert(id, widget);
        self.root = self.widgets.remove(&id);
    }

    /// Get the root widget
    pub fn root(&self) -> Option<&dyn Widget> {
        self.root.as_ref().map(|w| w.as_ref())
    }

    /// Get a widget by ID
    pub fn get_widget(&self, id: WidgetId) -> Option<&dyn Widget> {
        self.widgets.get(&id).map(|w| w.as_ref())
    }

    /// Get a mutable widget by ID
    pub fn get_widget_mut(&mut self, id: WidgetId) -> Option<&mut Box<dyn Widget>> {
        self.widgets.get_mut(&id)
    }

    /// Add a widget to the tree
    pub fn add_widget(&mut self, widget: Box<dyn Widget>) {
        let id = widget.id();
        self.widgets.insert(id, widget);
    }

    /// Remove a widget from the tree
    pub fn remove_widget(&mut self, id: WidgetId) -> Option<Box<dyn Widget>> {
        self.widgets.remove(&id)
    }

    /// Get all widget IDs
    pub fn widget_ids(&self) -> Vec<WidgetId> {
        self.widgets.keys().copied().collect()
    }

    /// Clear all widgets
    pub fn clear(&mut self) {
        self.widgets.clear();
        self.root = None;
    }
}

impl Default for WidgetTree {
    fn default() -> Self {
        Self::new()
    }
}

/// Widget manager for handling widget lifecycle
#[derive(Debug)]
pub struct WidgetManager {
    tree: WidgetTree,
    contexts: HashMap<WidgetId, WidgetContext>,
    focused_widget: Option<WidgetId>,
}

impl WidgetManager {
    /// Create a new widget manager
    pub fn new() -> Self {
        Self {
            tree: WidgetTree::new(),
            contexts: HashMap::new(),
            focused_widget: None,
        }
    }

    /// Set the root widget
    pub fn set_root(&mut self, widget: Box<dyn Widget>) {
        let id = widget.id();
        let context = WidgetContext::new(id);
        self.contexts.insert(id, context);
        self.tree.set_root(widget);
    }

    /// Handle an event
    pub fn handle_event(&mut self, event: &Event) -> StratoResult<bool> {
        if let Some(root) = self.tree.root() {
            let id = root.id();
            if let (Some(widget), Some(context)) =
                (self.tree.get_widget_mut(id), self.contexts.get_mut(&id))
            {
                return widget.handle_event(event, context);
            }
        }
        Ok(false)
    }

    /// Update all widgets
    pub fn update(&mut self) -> StratoResult<()> {
        for id in self.tree.widget_ids() {
            if let (Some(widget), Some(context)) =
                (self.tree.get_widget_mut(id), self.contexts.get_mut(&id))
            {
                widget.update(context)?;
            }
        }
        Ok(())
    }

    /// Layout all widgets
    pub fn layout(&mut self, constraints: &LayoutConstraints) -> StratoResult<()> {
        if let Some(root) = self.tree.root() {
            let id = root.id();
            if let (Some(widget), Some(context)) =
                (self.tree.get_widget_mut(id), self.contexts.get_mut(&id))
            {
                widget.layout(constraints, context)?;
            }
        }
        Ok(())
    }

    /// Render all widgets
    pub fn render(&self) -> StratoResult<()> {
        for id in self.tree.widget_ids() {
            if let (Some(widget), Some(context)) =
                (self.tree.get_widget(id), self.contexts.get(&id))
            {
                widget.render(context)?;
            }
        }
        Ok(())
    }

    /// Set focus to a widget
    pub fn set_focus(&mut self, id: Option<WidgetId>) -> StratoResult<()> {
        // Clear previous focus
        if let Some(old_id) = self.focused_widget {
            if let Some(context) = self.contexts.get_mut(&old_id) {
                context.is_focused = false;
            }
        }

        // Set new focus
        if let Some(new_id) = id {
            if let Some(context) = self.contexts.get_mut(&new_id) {
                context.is_focused = true;
            }
        }

        self.focused_widget = id;
        Ok(())
    }

    /// Get the currently focused widget
    pub fn focused_widget(&self) -> Option<WidgetId> {
        self.focused_widget
    }

    /// Get widget context
    pub fn get_context(&self, id: WidgetId) -> Option<&WidgetContext> {
        self.contexts.get(&id)
    }

    /// Get mutable widget context
    pub fn get_context_mut(&mut self, id: WidgetId) -> Option<&mut WidgetContext> {
        self.contexts.get_mut(&id)
    }
}

impl Default for WidgetManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct TestWidget {
        id: WidgetId,
    }

    impl TestWidget {
        fn new() -> Self {
            Self {
                id: WidgetId::new(),
            }
        }
    }

    impl Widget for TestWidget {
        fn id(&self) -> WidgetId {
            self.id
        }

        fn handle_event(
            &mut self,
            _event: &Event,
            _context: &mut WidgetContext,
        ) -> StratoResult<bool> {
            Ok(false)
        }

        fn update(&mut self, _context: &mut WidgetContext) -> StratoResult<()> {
            Ok(())
        }

        fn layout(
            &mut self,
            _constraints: &LayoutConstraints,
            _context: &mut WidgetContext,
        ) -> StratoResult<Size> {
            Ok(Size::new(100.0, 50.0))
        }

        fn render(&self, _context: &WidgetContext) -> StratoResult<()> {
            Ok(())
        }

        fn as_any(&self) -> &dyn Any {
            self
        }

        fn as_any_mut(&mut self) -> &mut dyn Any {
            self
        }
    }

    #[test]
    fn test_widget_id_generation() {
        let id1 = WidgetId::new();
        let id2 = WidgetId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_widget_context() {
        let id = WidgetId::new();
        let mut context = WidgetContext::new(id);

        context.set_property("test", 42i32);
        assert_eq!(context.get_property::<i32>("test"), Some(&42));
    }

    #[test]
    fn test_widget_tree() {
        let mut tree = WidgetTree::new();
        let widget = Box::new(TestWidget::new());
        let id = widget.id();

        tree.add_widget(widget);
        assert!(tree.get_widget(id).is_some());

        tree.remove_widget(id);
        assert!(tree.get_widget(id).is_none());
    }

    #[test]
    fn test_widget_manager() {
        let mut manager = WidgetManager::new();
        let widget = Box::new(TestWidget::new());
        let id = widget.id();

        manager.set_root(widget);
        assert!(manager.get_context(id).is_some());

        manager.set_focus(Some(id)).unwrap();
        assert_eq!(manager.focused_widget(), Some(id));
    }
}
