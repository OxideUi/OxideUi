//! Event handling system for StratoUI

use std::any::Any;
use std::sync::Arc;
use std::fmt::Debug;
use glam::Vec2;

/// Result of event handling
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventResult {
    /// Event was handled, stop propagation
    Handled,
    /// Event was not handled, continue propagation
    Ignored,
}

/// Mouse button types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Other(u16),
}

/// Keyboard key codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyCode {
    // Letters
    A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    // Numbers
    Num0, Num1, Num2, Num3, Num4, Num5, Num6, Num7, Num8, Num9,
    // Function keys
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    // Control keys
    Enter, Escape, Backspace, Tab, Space,
    Left, Right, Up, Down,
    Shift, Control, Alt, Super,
    // Special
    Delete, Insert, Home, End, PageUp, PageDown,
}

/// High-level key event for text input and navigation
#[derive(Debug, Clone, PartialEq)]
pub enum KeyEvent {
    /// Character input
    Char(char),
    /// Backspace key
    Backspace,
    /// Delete key
    Delete,
    /// Enter key
    Enter,
    /// Left arrow
    Left,
    /// Right arrow
    Right,
    /// Up arrow
    Up,
    /// Down arrow
    Down,
    /// Home key
    Home,
    /// End key
    End,
    /// Tab key
    Tab,
    /// Escape key
    Escape,
}

/// Keyboard modifiers
#[derive(Debug, Clone, Copy, Default)]
pub struct Modifiers {
    pub shift: bool,
    pub control: bool,
    pub alt: bool,
    pub super_key: bool,
}

/// Mouse event data
#[derive(Debug, Clone)]
pub struct MouseEvent {
    pub position: Vec2,
    pub button: Option<MouseButton>,
    pub modifiers: Modifiers,
    pub delta: Vec2,
}

/// Keyboard event data
#[derive(Debug, Clone)]
pub struct KeyboardEvent {
    pub key_code: KeyCode,
    pub modifiers: Modifiers,
    pub is_repeat: bool,
    pub text: Option<String>,
}

/// Window event data
#[derive(Debug, Clone)]
pub enum WindowEvent {
    Resize { width: u32, height: u32 },
    Move { x: i32, y: i32 },
    Focus(bool),
    Close,
    Minimize,
    Maximize,
}

/// Touch event data
#[derive(Debug, Clone)]
pub struct TouchEvent {
    pub id: u64,
    pub position: Vec2,
    pub force: Option<f32>,
}

/// Main event type
#[derive(Debug, Clone)]
pub enum Event {
    /// Mouse button pressed
    MouseDown(MouseEvent),
    /// Mouse button released
    MouseUp(MouseEvent),
    /// Mouse moved
    MouseMove(MouseEvent),
    /// Mouse wheel scrolled
    MouseWheel { delta: Vec2, modifiers: Modifiers },
    /// Mouse entered widget
    MouseEnter,
    /// Mouse left widget
    MouseExit,
    
    /// Key pressed
    KeyDown(KeyboardEvent),
    /// Key released
    KeyUp(KeyboardEvent),
    /// Text input
    TextInput(String),
    
    /// Window event
    Window(WindowEvent),
    
    /// Touch started
    TouchStart(TouchEvent),
    /// Touch moved
    TouchMove(TouchEvent),
    /// Touch ended
    TouchEnd(TouchEvent),
    /// Touch cancelled
    TouchCancel(TouchEvent),
    
    /// Custom user event
    Custom(Arc<dyn Any + Send + Sync>),
}

/// Event handler trait
pub trait EventHandler: Send + Sync {
    /// Handle an event
    fn handle(&mut self, event: &Event) -> EventResult;
    
    /// Check if handler can handle this event type
    fn can_handle(&self, _event: &Event) -> bool {
        true
    }
}

/// Event dispatcher with priority and filtering
pub struct EventDispatcher {
    handlers: Vec<(Box<dyn EventHandler>, i32)>, // (handler, priority)
    event_filters: Vec<Box<dyn Fn(&Event) -> bool + Send + Sync>>,
}

impl EventDispatcher {
    /// Create a new event dispatcher
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
            event_filters: Vec::new(),
        }
    }

    /// Add a handler with priority (higher priority = processed first)
    pub fn add_handler_with_priority(&mut self, handler: Box<dyn EventHandler>, priority: i32) {
        self.handlers.push((handler, priority));
        // Sort by priority (descending)
        self.handlers.sort_by(|a, b| b.1.cmp(&a.1));
    }

    /// Add a handler with default priority (0)
    pub fn add_handler(&mut self, handler: Box<dyn EventHandler>) {
        self.add_handler_with_priority(handler, 0);
    }

    /// Add an event filter
    pub fn add_filter<F>(&mut self, filter: F)
    where
        F: Fn(&Event) -> bool + Send + Sync + 'static,
    {
        self.event_filters.push(Box::new(filter));
    }

    /// Dispatch an event to all handlers
    pub fn dispatch(&mut self, event: &Event) -> EventResult {
        // Apply filters first
        for filter in &self.event_filters {
            if !filter(event) {
                return EventResult::Ignored;
            }
        }

        // Dispatch to handlers in priority order
        for (handler, _) in &mut self.handlers {
            if handler.can_handle(event) {
                if handler.handle(event) == EventResult::Handled {
                    return EventResult::Handled;
                }
            }
        }
        EventResult::Ignored
    }

    /// Remove all handlers
    pub fn clear_handlers(&mut self) {
        self.handlers.clear();
    }

    /// Remove all filters
    pub fn clear_filters(&mut self) {
        self.event_filters.clear();
    }
}

impl Default for EventDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestHandler {
        handled_count: usize,
    }

    impl EventHandler for TestHandler {
        fn handle(&mut self, _event: &Event) -> EventResult {
            self.handled_count += 1;
            EventResult::Handled
        }
    }

    #[test]
    fn test_event_dispatcher() {
        let mut dispatcher = EventDispatcher::new();
        dispatcher.add_handler(Box::new(TestHandler { handled_count: 0 }));
        
        let event = Event::MouseEnter;
        let result = dispatcher.dispatch(&event);
        assert_eq!(result, EventResult::Handled);
    }
}
