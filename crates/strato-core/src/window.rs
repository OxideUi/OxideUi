//! Window management for StratoUI
//!
//! This module provides cross-platform window creation and management
//! capabilities. It handles window lifecycle, events, and properties.

use crate::{
    error::{StratoError, StratoResult},
    event::Event,
    types::{Point, Rect, Size},
};
use std::collections::HashMap;

/// Unique identifier for windows
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WindowId(pub u64);

impl WindowId {
    /// Create a new window ID
    pub fn new() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for WindowId {
    fn default() -> Self {
        Self::new()
    }
}

/// Window configuration
#[derive(Debug, Clone)]
pub struct WindowConfig {
    /// Window title
    pub title: String,
    /// Initial window size
    pub size: Size,
    /// Initial window position
    pub position: Option<Point>,
    /// Whether the window is resizable
    pub resizable: bool,
    /// Whether the window has decorations (title bar, borders)
    pub decorated: bool,
    /// Whether the window is always on top
    pub always_on_top: bool,
    /// Whether the window is maximized
    pub maximized: bool,
    /// Whether the window is minimized
    pub minimized: bool,
    /// Whether the window is visible
    pub visible: bool,
    /// Whether the window is transparent
    pub transparent: bool,
    /// Minimum window size
    pub min_size: Option<Size>,
    /// Maximum window size
    pub max_size: Option<Size>,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: "StratoUI Window".to_string(),
            size: Size::new(800.0, 600.0),
            position: None,
            resizable: true,
            decorated: true,
            always_on_top: false,
            maximized: false,
            minimized: false,
            visible: true,
            transparent: false,
            min_size: Some(Size::new(200.0, 150.0)),
            max_size: None,
        }
    }
}

/// Window state
#[derive(Debug, Clone, PartialEq)]
pub enum WindowState {
    /// Window is normal
    Normal,
    /// Window is minimized
    Minimized,
    /// Window is maximized
    Maximized,
    /// Window is fullscreen
    Fullscreen,
}

/// Window theme
#[derive(Debug, Clone, PartialEq)]
pub enum WindowTheme {
    /// Light theme
    Light,
    /// Dark theme
    Dark,
    /// System theme
    System,
}

/// Window properties that can be changed at runtime
#[derive(Debug, Clone)]
pub struct WindowProperties {
    /// Window title
    pub title: String,
    /// Window size
    pub size: Size,
    /// Window position
    pub position: Point,
    /// Window state
    pub state: WindowState,
    /// Whether the window is visible
    pub visible: bool,
    /// Whether the window is focused
    pub focused: bool,
    /// Window theme
    pub theme: WindowTheme,
    /// Custom properties
    pub custom: HashMap<String, String>,
}

impl Default for WindowProperties {
    fn default() -> Self {
        Self {
            title: "StratoUI Window".to_string(),
            size: Size::new(800.0, 600.0),
            position: Point::new(100.0, 100.0),
            state: WindowState::Normal,
            visible: true,
            focused: false,
            theme: WindowTheme::System,
            custom: HashMap::new(),
        }
    }
}

/// Window event handler trait
pub trait WindowEventHandler: Send + Sync {
    /// Handle window close request
    fn on_close_requested(&mut self, window_id: WindowId) -> bool;

    /// Handle window resize
    fn on_resize(&mut self, window_id: WindowId, size: Size);

    /// Handle window move
    fn on_move(&mut self, window_id: WindowId, position: Point);

    /// Handle window focus change
    fn on_focus_changed(&mut self, window_id: WindowId, focused: bool);

    /// Handle window state change
    fn on_state_changed(&mut self, window_id: WindowId, state: WindowState);

    /// Handle window theme change
    fn on_theme_changed(&mut self, window_id: WindowId, theme: WindowTheme);

    /// Handle generic window event
    fn on_event(&mut self, window_id: WindowId, event: &Event);
}

/// Default window event handler that does nothing
#[derive(Debug, Default)]
pub struct DefaultWindowEventHandler;

impl WindowEventHandler for DefaultWindowEventHandler {
    fn on_close_requested(&mut self, _window_id: WindowId) -> bool {
        true // Allow close by default
    }

    fn on_resize(&mut self, _window_id: WindowId, _size: Size) {}

    fn on_move(&mut self, _window_id: WindowId, _position: Point) {}

    fn on_focus_changed(&mut self, _window_id: WindowId, _focused: bool) {}

    fn on_state_changed(&mut self, _window_id: WindowId, _state: WindowState) {}

    fn on_theme_changed(&mut self, _window_id: WindowId, _theme: WindowTheme) {}

    fn on_event(&mut self, _window_id: WindowId, _event: &Event) {}
}

/// Window trait for platform-specific implementations
pub trait Window: Send + Sync {
    /// Get the window ID
    fn id(&self) -> WindowId;

    /// Get window properties
    fn properties(&self) -> &WindowProperties;

    /// Set window title
    fn set_title(&mut self, title: &str) -> StratoResult<()>;

    /// Set window size
    fn set_size(&mut self, size: Size) -> StratoResult<()>;

    /// Set window position
    fn set_position(&mut self, position: Point) -> StratoResult<()>;

    /// Set window state
    fn set_state(&mut self, state: WindowState) -> StratoResult<()>;

    /// Set window visibility
    fn set_visible(&mut self, visible: bool) -> StratoResult<()>;

    /// Focus the window
    fn focus(&mut self) -> StratoResult<()>;

    /// Close the window
    fn close(&mut self) -> StratoResult<()>;

    /// Check if the window should close
    fn should_close(&self) -> bool;

    /// Get the window's content area
    fn content_area(&self) -> Rect;

    /// Get the window's scale factor
    fn scale_factor(&self) -> f32;

    /// Request a redraw
    fn request_redraw(&mut self);

    /// Set the window theme
    fn set_theme(&mut self, theme: WindowTheme) -> StratoResult<()>;

    /// Get the current cursor position relative to the window
    fn cursor_position(&self) -> Option<Point>;

    /// Set the cursor icon
    fn set_cursor_icon(&mut self, icon: CursorIcon) -> StratoResult<()>;

    /// Set the cursor visibility
    fn set_cursor_visible(&mut self, visible: bool) -> StratoResult<()>;
}

/// Cursor icon types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorIcon {
    /// Default cursor
    Default,
    /// Pointer cursor (hand)
    Pointer,
    /// Text cursor (I-beam)
    Text,
    /// Crosshair cursor
    Crosshair,
    /// Move cursor
    Move,
    /// Resize cursor (north-south)
    ResizeNS,
    /// Resize cursor (east-west)
    ResizeEW,
    /// Resize cursor (northeast-southwest)
    ResizeNESW,
    /// Resize cursor (northwest-southeast)
    ResizeNWSE,
    /// Wait cursor
    Wait,
    /// Not allowed cursor
    NotAllowed,
    /// Help cursor
    Help,
    /// Progress cursor
    Progress,
}

/// Window builder for creating windows with a fluent API
pub struct WindowBuilder {
    config: WindowConfig,
    event_handler: Option<Box<dyn WindowEventHandler>>,
}

impl WindowBuilder {
    /// Create a new window builder
    pub fn new() -> Self {
        Self {
            config: WindowConfig::default(),
            event_handler: None,
        }
    }

    /// Set the window title
    pub fn title<S: Into<String>>(mut self, title: S) -> Self {
        self.config.title = title.into();
        self
    }

    /// Set the window size
    pub fn size(mut self, size: Size) -> Self {
        self.config.size = size;
        self
    }

    /// Set the window position
    pub fn position(mut self, position: Point) -> Self {
        self.config.position = Some(position);
        self
    }

    /// Set whether the window is resizable
    pub fn resizable(mut self, resizable: bool) -> Self {
        self.config.resizable = resizable;
        self
    }

    /// Set whether the window has decorations
    pub fn decorated(mut self, decorated: bool) -> Self {
        self.config.decorated = decorated;
        self
    }

    /// Set whether the window is always on top
    pub fn always_on_top(mut self, always_on_top: bool) -> Self {
        self.config.always_on_top = always_on_top;
        self
    }

    /// Set whether the window starts maximized
    pub fn maximized(mut self, maximized: bool) -> Self {
        self.config.maximized = maximized;
        self
    }

    /// Set whether the window starts visible
    pub fn visible(mut self, visible: bool) -> Self {
        self.config.visible = visible;
        self
    }

    /// Set whether the window is transparent
    pub fn transparent(mut self, transparent: bool) -> Self {
        self.config.transparent = transparent;
        self
    }

    /// Set the minimum window size
    pub fn min_size(mut self, min_size: Size) -> Self {
        self.config.min_size = Some(min_size);
        self
    }

    /// Set the maximum window size
    pub fn max_size(mut self, max_size: Size) -> Self {
        self.config.max_size = Some(max_size);
        self
    }

    /// Set the event handler
    pub fn event_handler<H: WindowEventHandler + 'static>(mut self, handler: H) -> Self {
        self.event_handler = Some(Box::new(handler));
        self
    }

    /// Build the window
    pub fn build(self) -> StratoResult<Box<dyn Window>> {
        // This would be implemented by the platform-specific backend
        Err(StratoError::NotImplemented {
            message: "Window creation not implemented".to_string(),
            context: None,
        })
    }
}

impl Default for WindowBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Window manager for handling multiple windows
pub struct WindowManager {
    windows: HashMap<WindowId, Box<dyn Window>>,
    event_handlers: HashMap<WindowId, Box<dyn WindowEventHandler>>,
    active_window: Option<WindowId>,
}

impl WindowManager {
    /// Create a new window manager
    pub fn new() -> Self {
        Self {
            windows: HashMap::new(),
            event_handlers: HashMap::new(),
            active_window: None,
        }
    }

    /// Create a new window
    pub fn create_window(&mut self, builder: WindowBuilder) -> StratoResult<WindowId> {
        let window = builder.build()?;
        let id = window.id();
        self.windows.insert(id, window);

        if self.active_window.is_none() {
            self.active_window = Some(id);
        }

        Ok(id)
    }

    /// Get a window by ID
    pub fn get_window(&self, id: WindowId) -> Option<&dyn Window> {
        self.windows.get(&id).map(|w| w.as_ref())
    }

    /// Get a mutable window by ID
    pub fn get_window_mut(&mut self, id: WindowId) -> Option<&mut Box<dyn Window>> {
        self.windows.get_mut(&id)
    }

    /// Close a window
    pub fn close_window(&mut self, id: WindowId) -> StratoResult<()> {
        if let Some(mut window) = self.windows.remove(&id) {
            window.close()?;
            self.event_handlers.remove(&id);

            if self.active_window == Some(id) {
                self.active_window = self.windows.keys().next().copied();
            }
        }
        Ok(())
    }

    /// Get the active window ID
    pub fn active_window(&self) -> Option<WindowId> {
        self.active_window
    }

    /// Set the active window
    pub fn set_active_window(&mut self, id: WindowId) -> StratoResult<()> {
        if self.windows.contains_key(&id) {
            self.active_window = Some(id);
            if let Some(window) = self.get_window_mut(id) {
                window.focus()?;
            }
        }
        Ok(())
    }

    /// Get all window IDs
    pub fn window_ids(&self) -> Vec<WindowId> {
        self.windows.keys().copied().collect()
    }

    /// Handle an event for a specific window
    pub fn handle_event(&mut self, window_id: WindowId, event: &Event) -> StratoResult<()> {
        if let Some(handler) = self.event_handlers.get_mut(&window_id) {
            handler.on_event(window_id, event);
        }
        Ok(())
    }

    /// Update all windows
    pub fn update(&mut self) -> StratoResult<()> {
        // Process window events and updates
        for (id, window) in &mut self.windows {
            if window.should_close() {
                // Handle close request through event handler
                if let Some(handler) = self.event_handlers.get_mut(id) {
                    if handler.on_close_requested(*id) {
                        // Window should be closed
                        continue;
                    }
                }
            }
        }

        // Remove windows that should be closed
        let mut to_remove = Vec::new();
        for (id, window) in &self.windows {
            if window.should_close() {
                to_remove.push(*id);
            }
        }

        for id in to_remove {
            self.close_window(id)?;
        }

        Ok(())
    }

    /// Check if there are any open windows
    pub fn has_windows(&self) -> bool {
        !self.windows.is_empty()
    }
}

impl Default for WindowManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_id_generation() {
        let id1 = WindowId::new();
        let id2 = WindowId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_window_config_default() {
        let config = WindowConfig::default();
        assert_eq!(config.title, "StratoUI Window");
        assert_eq!(config.size, Size::new(800.0, 600.0));
        assert!(config.resizable);
        assert!(config.decorated);
        assert!(!config.always_on_top);
    }

    #[test]
    fn test_window_builder() {
        let builder = WindowBuilder::new()
            .title("Test Window")
            .size(Size::new(1024.0, 768.0))
            .resizable(false)
            .decorated(true);

        assert_eq!(builder.config.title, "Test Window");
        assert_eq!(builder.config.size, Size::new(1024.0, 768.0));
        assert!(!builder.config.resizable);
        assert!(builder.config.decorated);
    }

    #[test]
    fn test_window_properties_default() {
        let props = WindowProperties::default();
        assert_eq!(props.title, "StratoUI Window");
        assert_eq!(props.state, WindowState::Normal);
        assert!(props.visible);
        assert!(!props.focused);
        assert_eq!(props.theme, WindowTheme::System);
    }

    #[test]
    fn test_window_manager() {
        let manager = WindowManager::new();
        assert!(!manager.has_windows());
        assert_eq!(manager.window_ids().len(), 0);
        assert_eq!(manager.active_window(), None);
    }

    #[test]
    fn test_cursor_icon_variants() {
        let icons = [
            CursorIcon::Default,
            CursorIcon::Pointer,
            CursorIcon::Text,
            CursorIcon::Crosshair,
            CursorIcon::Move,
            CursorIcon::ResizeNS,
            CursorIcon::ResizeEW,
            CursorIcon::ResizeNESW,
            CursorIcon::ResizeNWSE,
            CursorIcon::Wait,
            CursorIcon::NotAllowed,
            CursorIcon::Help,
            CursorIcon::Progress,
        ];

        // Test that all variants are distinct
        for (i, icon1) in icons.iter().enumerate() {
            for (j, icon2) in icons.iter().enumerate() {
                if i != j {
                    assert_ne!(icon1, icon2);
                }
            }
        }
    }
}
