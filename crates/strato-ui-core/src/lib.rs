//! Strato UI core primitives.
//!
//! This crate is the clean Strato-owned seed that replaces the quarantined
//! import surface. It intentionally contains no dependencies on non-MIT Warp
//! workspace crates.

use std::fmt;

/// Stable identifier for an application instance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppContext {
    app_name: String,
}

impl AppContext {
    /// Create a new application context.
    pub fn new(app_name: impl Into<String>) -> Self {
        Self {
            app_name: app_name.into(),
        }
    }

    /// Return the human-readable application name.
    pub fn app_name(&self) -> &str {
        &self.app_name
    }
}

/// Two-dimensional size in logical pixels.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl Size {
    pub const ZERO: Self = Self {
        width: 0.0,
        height: 0.0,
    };

    pub const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

/// Layout rectangle in logical pixels.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub const fn size(&self) -> Size {
        Size::new(self.width, self.height)
    }
}

/// A renderable Strato UI element.
pub trait Element: fmt::Debug {
    fn layout(&self, available: Size, app: &AppContext) -> Rect;
    fn describe(&self) -> String;
}

/// A basic text element used by smoke tests and early integration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TextElement {
    text: String,
}

impl TextElement {
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    pub fn text(&self) -> &str {
        &self.text
    }
}

impl Element for TextElement {
    fn layout(&self, available: Size, _app: &AppContext) -> Rect {
        let width = available
            .width
            .min((self.text.chars().count() as f32 * 8.0).max(1.0));
        let height = available.height.min(20.0);
        Rect::new(0.0, 0.0, width, height)
    }

    fn describe(&self) -> String {
        format!("text:{}", self.text)
    }
}

/// Convenience constructor for text elements.
pub fn text(value: impl Into<String>) -> TextElement {
    TextElement::new(value)
}
