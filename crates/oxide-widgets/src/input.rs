//! Text input widget implementation

use crate::widget::{Widget, WidgetId, generate_id};
use oxide_core::{
    event::{Event, EventResult, KeyboardEvent, KeyCode},
    layout::{Constraints, Layout, Size},
    types::{Color, Rect},
};
use oxide_renderer::batch::RenderBatch;
use std::any::Any;
use std::fmt;
use std::sync::Arc;

/// Text input widget
#[derive(Clone)]
pub struct TextInput {
    id: WidgetId,
    value: String,
    placeholder: String,
    style: TextInputStyle,
    cursor_position: usize,
    selection_start: Option<usize>,
    is_focused: bool,
    is_readonly: bool,
    max_length: Option<usize>,
    on_change: Option<Arc<dyn Fn(&str) + Send + Sync>>,
    on_submit: Option<Arc<dyn Fn(&str) + Send + Sync>>,
}

impl fmt::Debug for TextInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TextInput")
            .field("id", &self.id)
            .field("value", &self.value)
            .field("placeholder", &self.placeholder)
            .field("style", &self.style)
            .field("cursor_position", &self.cursor_position)
            .field("selection_start", &self.selection_start)
            .field("is_focused", &self.is_focused)
            .field("is_readonly", &self.is_readonly)
            .field("max_length", &self.max_length)
            .field("has_on_change", &self.on_change.is_some())
            .field("has_on_submit", &self.on_submit.is_some())
            .finish()
    }
}

impl TextInput {
    /// Create a new text input
    pub fn new() -> Self {
        Self {
            id: generate_id(),
            value: String::new(),
            placeholder: String::new(),
            style: TextInputStyle::default(),
            cursor_position: 0,
            selection_start: None,
            is_focused: false,
            is_readonly: false,
            max_length: None,
            on_change: None,
            on_submit: None,
        }
    }

    /// Set the input value
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self.cursor_position = self.value.len();
        self
    }

    /// Set placeholder text
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Set style
    pub fn style(mut self, style: TextInputStyle) -> Self {
        self.style = style;
        self
    }

    /// Set readonly state
    pub fn readonly(mut self, readonly: bool) -> Self {
        self.is_readonly = readonly;
        self
    }

    /// Set maximum length
    pub fn max_length(mut self, max_length: usize) -> Self {
        self.max_length = Some(max_length);
        self
    }

    /// Set change handler
    pub fn on_change<F>(mut self, handler: F) -> Self
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.on_change = Some(Arc::new(handler));
        self
    }

    /// Set submit handler
    pub fn on_submit<F>(mut self, handler: F) -> Self
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.on_submit = Some(Arc::new(handler));
        self
    }

    /// Get the current value
    pub fn get_value(&self) -> &str {
        &self.value
    }

    /// Set the current value
    pub fn set_value(&mut self, value: impl Into<String>) {
        self.value = value.into();
        self.cursor_position = self.cursor_position.min(self.value.len());
        
        if let Some(handler) = &self.on_change {
            (handler)(&self.value);
        }
    }

    /// Insert text at cursor position
    fn insert_text(&mut self, text: &str) {
        if self.is_readonly {
            return;
        }

        if let Some(max_len) = self.max_length {
            if self.value.len() + text.len() > max_len {
                return;
            }
        }

        // Handle selection replacement
        if let Some(selection_start) = self.selection_start {
            let start = selection_start.min(self.cursor_position);
            let end = selection_start.max(self.cursor_position);
            self.value.replace_range(start..end, text);
            self.cursor_position = start + text.len();
            self.selection_start = None;
        } else {
            self.value.insert_str(self.cursor_position, text);
            self.cursor_position += text.len();
        }

        if let Some(handler) = &self.on_change {
            handler(&self.value);
        }
    }

    /// Delete character at cursor position
    fn delete_char(&mut self, forward: bool) {
        if self.is_readonly || self.value.is_empty() {
            return;
        }

        if let Some(selection_start) = self.selection_start {
            let start = selection_start.min(self.cursor_position);
            let end = selection_start.max(self.cursor_position);
            self.value.drain(start..end);
            self.cursor_position = start;
            self.selection_start = None;
        } else if forward && self.cursor_position < self.value.len() {
            self.value.remove(self.cursor_position);
        } else if !forward && self.cursor_position > 0 {
            self.cursor_position -= 1;
            self.value.remove(self.cursor_position);
        }

        if let Some(handler) = &self.on_change {
            handler(&self.value);
        }
    }

    /// Move cursor
    fn move_cursor(&mut self, forward: bool, word_mode: bool) {
        if forward {
            if word_mode {
                // Move to next word boundary
                while self.cursor_position < self.value.len() &&
                      !self.value.chars().nth(self.cursor_position).unwrap().is_whitespace() {
                    self.cursor_position += 1;
                }
                while self.cursor_position < self.value.len() &&
                      self.value.chars().nth(self.cursor_position).unwrap().is_whitespace() {
                    self.cursor_position += 1;
                }
            } else {
                self.cursor_position = (self.cursor_position + 1).min(self.value.len());
            }
        } else {
            if word_mode {
                // Move to previous word boundary
                while self.cursor_position > 0 &&
                      self.value.chars().nth(self.cursor_position - 1).unwrap().is_whitespace() {
                    self.cursor_position -= 1;
                }
                while self.cursor_position > 0 &&
                      !self.value.chars().nth(self.cursor_position - 1).unwrap().is_whitespace() {
                    self.cursor_position -= 1;
                }
            } else {
                self.cursor_position = self.cursor_position.saturating_sub(1);
            }
        }
    }
}

impl Default for TextInput {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for TextInput {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn layout(&mut self, constraints: Constraints) -> Size {
        let height = self.style.height.unwrap_or(40.0);
        let width = self.style.width.unwrap_or(200.0)
            .min(constraints.max_width)
            .max(constraints.min_width);
        
        Size::new(width, height)
    }

    fn render(&self, batch: &mut RenderBatch, layout: Layout) {
        let rect = Rect::new(
            layout.position.x,
            layout.position.y,
            layout.size.width,
            layout.size.height,
        );
        
        // Draw background
        if self.style.border_radius > 0.0 {
            batch.draw_rounded_rect(rect, self.style.background_color, self.style.border_radius);
        } else {
            batch.draw_rect(rect, self.style.background_color);
        }
        
        // Draw border
        if self.is_focused {
            // TODO: Draw focus border
        }
        
        // Draw text or placeholder
        // TODO: Implement text rendering
        
        // Draw cursor if focused
        if self.is_focused {
            // TODO: Draw blinking cursor
        }
    }

    fn handle_event(&mut self, event: &Event) -> EventResult {
        match event {
            Event::KeyDown(KeyboardEvent { key_code, .. }) => {
                if !self.is_focused {
                    return EventResult::Ignored;
                }

                match key_code {
                    KeyCode::Left => {
                        self.move_cursor(false, false);
                        EventResult::Handled
                    }
                    KeyCode::Right => {
                        self.move_cursor(true, false);
                        EventResult::Handled
                    }
                    KeyCode::Home => {
                        self.cursor_position = 0;
                        EventResult::Handled
                    }
                    KeyCode::End => {
                        self.cursor_position = self.value.len();
                        EventResult::Handled
                    }
                    KeyCode::Backspace => {
                        self.delete_char(false);
                        EventResult::Handled
                    }
                    KeyCode::Delete => {
                        self.delete_char(true);
                        EventResult::Handled
                    }
                    KeyCode::Enter => {
                        if let Some(handler) = &self.on_submit {
                            (handler)(&self.value);
                        }
                        EventResult::Handled
                    }
                    _ => EventResult::Ignored,
                }
            }
            Event::TextInput(text) => {
                if self.is_focused {
                    self.insert_text(text);
                    EventResult::Handled
                } else {
                    EventResult::Ignored
                }
            }
            _ => EventResult::Ignored,
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_widget(&self) -> Box<dyn Widget> {
        Box::new(self.clone())
    }
}

/// Text input style configuration
#[derive(Debug, Clone)]
pub struct TextInputStyle {
    pub background_color: Color,
    pub text_color: Color,
    pub placeholder_color: Color,
    pub border_color: Color,
    pub focus_border_color: Color,
    pub border_width: f32,
    pub border_radius: f32,
    pub padding: f32,
    pub font_size: f32,
    pub width: Option<f32>,
    pub height: Option<f32>,
}

impl Default for TextInputStyle {
    fn default() -> Self {
        Self {
            background_color: Color::WHITE,
            text_color: Color::BLACK,
            placeholder_color: Color::rgba(0.0, 0.0, 0.0, 0.5),
            border_color: Color::rgba(0.0, 0.0, 0.0, 0.2),
            focus_border_color: Color::PRIMARY,
            border_width: 1.0,
            border_radius: 4.0,
            padding: 8.0,
            font_size: 14.0,
            width: None,
            height: Some(36.0),
        }
    }
}

impl TextInputStyle {
    /// Outlined style
    pub fn outlined() -> Self {
        Self::default()
    }

    /// Filled style
    pub fn filled() -> Self {
        Self {
            background_color: Color::rgba(0.0, 0.0, 0.0, 0.05),
            text_color: Color::BLACK,
            placeholder_color: Color::rgba(0.0, 0.0, 0.0, 0.5),
            border_color: Color::TRANSPARENT,
            focus_border_color: Color::PRIMARY,
            border_width: 0.0,
            border_radius: 4.0,
            padding: 12.0,
            font_size: 14.0,
            width: None,
            height: Some(42.0),
        }
    }
}
