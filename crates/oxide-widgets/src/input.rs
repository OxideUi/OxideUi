//! Text input widget implementation
//! 
//! Provides text input components with various input types, validation, and formatting options.

use oxide_core::{
    layout::{Rect, Size},
    state::{Signal},
    theme::{Theme, ColorPalette},
    types::{Color, Point},
    events::{Event, KeyEvent, MouseEvent},
};
use oxide_renderer::{
    vertex::{Vertex, VertexBuilder},
    batch::RenderBatch,
};
use std::sync::Arc;

/// Input type enumeration
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputType {
    Text,
    Password,
    Email,
    Number,
    Tel,
    Url,
    Search,
    Multiline,
}

/// Input validation state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ValidationState {
    Valid,
    Invalid,
    Warning,
    Pending,
}

/// Input state enumeration
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputState {
    Normal,
    Focused,
    Hovered,
    Disabled,
    ReadOnly,
    Error,
}

/// Text input style configuration
#[derive(Debug, Clone)]
pub struct InputStyle {
    pub background_color: Color,
    pub border_color: Color,
    pub text_color: Color,
    pub placeholder_color: Color,
    pub selection_color: Color,
    pub cursor_color: Color,
    pub border_width: f32,
    pub border_radius: f32,
    pub padding: (f32, f32, f32, f32), // top, right, bottom, left
    pub font_size: f32,
    pub font_family: String,
    pub line_height: f32,
}

impl Default for InputStyle {
    fn default() -> Self {
        Self {
            background_color: Color::rgba(1.0, 1.0, 1.0, 1.0),
            border_color: Color::rgba(0.8, 0.8, 0.8, 1.0),
            text_color: Color::rgba(0.0, 0.0, 0.0, 1.0),
            placeholder_color: Color::rgba(0.6, 0.6, 0.6, 1.0),
            selection_color: Color::rgba(0.0, 0.4, 0.8, 0.3),
            cursor_color: Color::rgba(0.0, 0.0, 0.0, 1.0),
            border_width: 1.0,
            border_radius: 4.0,
            padding: (8.0, 12.0, 8.0, 12.0),
            font_size: 14.0,
            font_family: "system-ui".to_string(),
            line_height: 1.4,
        }
    }
}

impl InputStyle {
    /// Create a style for the given state
    pub fn for_state(&self, state: InputState) -> Self {
        let mut style = self.clone();
        
        match state {
            InputState::Normal => {},
            InputState::Focused => {
                style.border_color = Color::rgba(0.0, 0.4, 0.8, 1.0);
            }
            InputState::Hovered => {
                style.border_color = Color::rgba(0.6, 0.6, 0.6, 1.0);
            }
            InputState::Disabled => {
                style.background_color = Color::rgba(0.95, 0.95, 0.95, 1.0);
                style.text_color = Color::rgba(0.6, 0.6, 0.6, 1.0);
                style.border_color = Color::rgba(0.9, 0.9, 0.9, 1.0);
            }
            InputState::ReadOnly => {
                style.background_color = Color::rgba(0.98, 0.98, 0.98, 1.0);
                style.border_color = Color::rgba(0.9, 0.9, 0.9, 1.0);
            }
            InputState::Error => {
                style.border_color = Color::rgba(0.8, 0.2, 0.2, 1.0);
                style.border_width = 2.0;
            },
        }
        
        style
    }
}

/// Validation function type
pub type ValidationFn = Box<dyn Fn(&str) -> Result<(), String> + Send + Sync>;

/// Text input widget
pub struct TextInput {
    id: String,
    input_type: InputType,
    value: Signal<String>,
    placeholder: String,
    max_length: Option<usize>,
    min_length: Option<usize>,
    pattern: Option<String>,
    required: bool,
    disabled: Signal<bool>,
    readonly: Signal<bool>,
    multiline: bool,
    rows: usize,
    cols: usize,
    
    // State
    state: Signal<InputState>,
    validation_state: Signal<ValidationState>,
    validation_message: Signal<Option<String>>,
    focused: Signal<bool>,
    hovered: Signal<bool>,
    
    // Selection and cursor
    cursor_position: Signal<usize>,
    selection_start: Signal<Option<usize>>,
    selection_end: Signal<Option<usize>>,
    
    // Layout
    bounds: Signal<Rect>,
    content_bounds: Signal<Rect>,
    visible: Signal<bool>,
    
    // Style and theme
    style: InputStyle,
    theme: Option<Arc<Theme>>,
    
    // Validation
    validators: Vec<ValidationFn>,
    
    // Events
    on_change: Option<Box<dyn Fn(&str) + Send + Sync>>,
    on_focus: Option<Box<dyn Fn() + Send + Sync>>,
    on_blur: Option<Box<dyn Fn() + Send + Sync>>,
    on_submit: Option<Box<dyn Fn(&str) + Send + Sync>>,
    
    // Internal state
    cursor_blink_timer: Signal<f32>,
    scroll_offset: Signal<f32>,
}

impl TextInput {
    /// Create a new text input
    pub fn new() -> Self {
        Self {
            id: format!("input_{}", uuid::Uuid::new_v4()),
            input_type: InputType::Text,
            value: Signal::new(String::new()),
            placeholder: String::new(),
            max_length: None,
            min_length: None,
            pattern: None,
            required: false,
            disabled: Signal::new(false),
            readonly: Signal::new(false),
            multiline: false,
            rows: 1,
            cols: 20,
            
            state: Signal::new(InputState::Normal),
            validation_state: Signal::new(ValidationState::Valid),
            validation_message: Signal::new(None),
            focused: Signal::new(false),
            hovered: Signal::new(false),
            
            cursor_position: Signal::new(0),
            selection_start: Signal::new(None),
            selection_end: Signal::new(None),
            
            bounds: Signal::new(Rect::new(0.0, 0.0, 0.0, 0.0)),
            content_bounds: Signal::new(Rect::new(0.0, 0.0, 0.0, 0.0)),
            visible: Signal::new(true),
            
            style: InputStyle::default(),
            theme: None,
            
            validators: Vec::new(),
            
            on_change: None,
            on_focus: None,
            on_blur: None,
            on_submit: None,
            
            cursor_blink_timer: Signal::new(0.0),
            scroll_offset: Signal::new(0.0),
        }
    }

    /// Set input type
    pub fn input_type(mut self, input_type: InputType) -> Self {
        self.input_type = input_type;
        if input_type == InputType::Multiline {
            self.multiline = true;
        }
        self
    }

    /// Set placeholder text
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Set initial value
    pub fn value(self, value: impl Into<String>) -> Self {
        let val = value.into();
        self.cursor_position.set(val.len());
        self.value.set(val);
        self
    }

    /// Set maximum length
    pub fn max_length(mut self, max_length: usize) -> Self {
        self.max_length = Some(max_length);
        self
    }

    /// Set minimum length
    pub fn min_length(mut self, min_length: usize) -> Self {
        self.min_length = Some(min_length);
        self
    }

    /// Set validation pattern
    pub fn pattern(mut self, pattern: impl Into<String>) -> Self {
        self.pattern = Some(pattern.into());
        self
    }

    /// Set required flag
    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    /// Set disabled state
    pub fn disabled(self, disabled: bool) -> Self {
        self.disabled.set(disabled);
        if disabled {
            self.state.set(InputState::Disabled);
        }
        self
    }

    /// Set readonly state
    pub fn readonly(self, readonly: bool) -> Self {
        self.readonly.set(readonly);
        if readonly {
            self.state.set(InputState::ReadOnly);
        }
        self
    }

    /// Set multiline mode
    pub fn multiline(mut self, multiline: bool) -> Self {
        self.multiline = multiline;
        if multiline {
            self.input_type = InputType::Multiline;
        }
        self
    }

    /// Set rows for multiline input
    pub fn rows(mut self, rows: usize) -> Self {
        self.rows = rows;
        self
    }

    /// Set columns
    pub fn cols(mut self, cols: usize) -> Self {
        self.cols = cols;
        self
    }

    /// Set style
    pub fn style(mut self, style: InputStyle) -> Self {
        self.style = style;
        self
    }

    /// Set theme
    pub fn theme(mut self, theme: Arc<Theme>) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Add validator
    pub fn validator<F>(mut self, validator: F) -> Self 
    where
        F: Fn(&str) -> Result<(), String> + Send + Sync + 'static,
    {
        self.validators.push(Box::new(validator));
        self
    }

    /// Set change callback
    pub fn on_change<F>(mut self, callback: F) -> Self
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.on_change = Some(Box::new(callback));
        self
    }

    /// Set focus callback
    pub fn on_focus<F>(mut self, callback: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_focus = Some(Box::new(callback));
        self
    }

    /// Set blur callback
    pub fn on_blur<F>(mut self, callback: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_blur = Some(Box::new(callback));
        self
    }

    /// Set submit callback
    pub fn on_submit<F>(mut self, callback: F) -> Self
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.on_submit = Some(Box::new(callback));
        self
    }

    /// Get input ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get current value
    pub fn get_value(&self) -> String {
        self.value.get()
    }

    /// Set value programmatically
    pub fn set_value(&self, value: impl Into<String>) {
        let val = value.into();
        
        // Validate length constraints
        if let Some(max_len) = self.max_length {
            if val.len() > max_len {
                return;
            }
        }
        
        self.value.set(val.clone());
        self.cursor_position.set(val.len());
        self.validate();
        
        if let Some(callback) = &self.on_change {
            callback(&val);
        }
    }

    /// Check if input is focused
    pub fn is_focused(&self) -> bool {
        self.focused.get()
    }

    /// Check if input is disabled
    pub fn is_disabled(&self) -> bool {
        self.disabled.get()
    }

    /// Check if input is readonly
    pub fn is_readonly(&self) -> bool {
        self.readonly.get()
    }

    /// Get current selection
    pub fn get_selection(&self) -> Option<(usize, usize)> {
        match (self.selection_start.get(), self.selection_end.get()) {
            (Some(start), Some(end)) => Some((start.min(end), start.max(end))),
            _ => None,
        }
    }

    /// Set selection
    pub fn set_selection(&self, start: Option<usize>, end: Option<usize>) {
        self.selection_start.set(start);
        self.selection_end.set(end);
    }

    /// Clear selection
    pub fn clear_selection(&self) {
        self.selection_start.set(None);
        self.selection_end.set(None);
    }

    /// Focus the input
    pub fn focus(&self) {
        if self.is_disabled() || self.is_readonly() {
            return;
        }
        
        self.focused.set(true);
        self.state.set(InputState::Focused);
        
        if let Some(callback) = &self.on_focus {
            callback();
        }
    }

    /// Blur the input
    pub fn blur(&self) {
        self.focused.set(false);
        self.clear_selection();
        
        let new_state = if self.is_disabled() {
            InputState::Disabled
        } else if self.is_readonly() {
            InputState::ReadOnly
        } else if self.validation_state.get() == ValidationState::Invalid {
            InputState::Error
        } else {
            InputState::Normal
        };
        
        self.state.set(new_state);
        
        if let Some(callback) = &self.on_blur {
            callback();
        }
    }

    /// Validate input value
    pub fn validate(&self) -> bool {
        let value = self.value.get();
        
        // Check required
        if self.required && value.is_empty() {
            self.validation_state.set(ValidationState::Invalid);
            self.validation_message.set(Some("This field is required".to_string()));
            return false;
        }
        
        // Check length constraints
        if let Some(min_len) = self.min_length {
            if value.len() < min_len {
                self.validation_state.set(ValidationState::Invalid);
                self.validation_message.set(Some(format!("Minimum length is {}", min_len)));
                return false;
            }
        }
        
        if let Some(max_len) = self.max_length {
            if value.len() > max_len {
                self.validation_state.set(ValidationState::Invalid);
                self.validation_message.set(Some(format!("Maximum length is {}", max_len)));
                return false;
            }
        }
        
        // Check pattern
        if let Some(pattern) = &self.pattern {
            // Simple pattern matching (in a real implementation, use regex)
            if !value.contains(pattern) {
                self.validation_state.set(ValidationState::Invalid);
                self.validation_message.set(Some("Invalid format".to_string()));
                return false;
            }
        }
        
        // Check input type specific validation
        match self.input_type {
            InputType::Email => {
                if !value.is_empty() && !value.contains('@') {
                    self.validation_state.set(ValidationState::Invalid);
                    self.validation_message.set(Some("Invalid email format".to_string()));
                    return false;
                }
            },
            InputType::Number => {
                if !value.is_empty() && value.parse::<f64>().is_err() {
                    self.validation_state.set(ValidationState::Invalid);
                    self.validation_message.set(Some("Invalid number format".to_string()));
                    return false;
                }
            },
            InputType::Url => {
                if !value.is_empty() && !value.starts_with("http") {
                    self.validation_state.set(ValidationState::Invalid);
                    self.validation_message.set(Some("Invalid URL format".to_string()));
                    return false;
                }
            },
            _ => {},
        }
        
        // Run custom validators
        for validator in &self.validators {
            if let Err(message) = validator(&value) {
                self.validation_state.set(ValidationState::Invalid);
                self.validation_message.set(Some(message));
                return false;
            }
        }
        
        self.validation_state.set(ValidationState::Valid);
        self.validation_message.set(None);
        true
    }

    /// Calculate input size
    pub fn calculate_size(&self, available_size: Size) -> Size {
        let char_width = self.style.font_size * 0.6;
        let line_height = self.style.font_size * self.style.line_height;
        
        let padding_h = self.style.padding.1 + self.style.padding.3;
        let padding_v = self.style.padding.0 + self.style.padding.2;
        
        let width = if self.multiline {
            (self.cols as f32 * char_width + padding_h).min(available_size.width)
        } else {
            available_size.width.min(300.0) // Default single-line width
        };
        
        let height = if self.multiline {
            self.rows as f32 * line_height + padding_v
        } else {
            line_height + padding_v
        };
        
        Size::new(width, height.min(available_size.height))
    }

    /// Layout the input
    pub fn layout(&self, bounds: Rect) {
        self.bounds.set(bounds);
        
        let padding = &self.style.padding;
        let content_bounds = Rect::new(
            bounds.x + padding.3,
            bounds.y + padding.0,
            bounds.width - padding.1 - padding.3,
            bounds.height - padding.0 - padding.2,
        );
        self.content_bounds.set(content_bounds);
    }

    /// Handle mouse events
    pub fn handle_mouse_event(&self, event: &MouseEvent) -> bool {
        let bounds = self.bounds.get();
        
        match event {
            MouseEvent::Press { point, .. } => {
                if bounds.contains(*point) {
                    self.focus();
                    
                    // Calculate cursor position
                    let content_bounds = self.content_bounds.get();
                    let relative_x = point.x - content_bounds.x;
                    let char_width = self.style.font_size * 0.6;
                    let position = (relative_x / char_width) as usize;
                    
                    let value = self.value.get();
                    self.cursor_position.set(position.min(value.len()));
                    self.clear_selection();
                    
                    return true;
                } else {
                    self.blur();
                }
            },
            MouseEvent::Move { point } => {
                let was_hovered = self.hovered.get();
                let is_hovered = bounds.contains(*point);
                
                if is_hovered != was_hovered {
                    self.hovered.set(is_hovered);
                    
                    if !self.is_focused() {
                        let new_state = if is_hovered {
                            InputState::Hovered
                        } else if self.is_disabled() {
                            InputState::Disabled
                        } else if self.is_readonly() {
                            InputState::ReadOnly
                        } else {
                            InputState::Normal
                        };
                        self.state.set(new_state);
                    }
                }
            },
            _ => {},
        }
        
        false
    }

    /// Handle key events
    pub fn handle_key_event(&self, event: &KeyEvent) -> bool {
        if !self.is_focused() || self.is_disabled() || self.is_readonly() {
            return false;
        }

        match event {
            KeyEvent::Char(ch) => {
                self.insert_char(*ch);
                true
            },
            KeyEvent::Backspace => {
                self.delete_backward();
                true
            },
            KeyEvent::Delete => {
                self.delete_forward();
                true
            },
            KeyEvent::Enter => {
                if self.multiline {
                    self.insert_char('\n');
                } else if let Some(callback) = &self.on_submit {
                    callback(&self.value.get());
                }
                true
            },
            KeyEvent::Left => {
                self.move_cursor_left();
                true
            },
            KeyEvent::Right => {
                self.move_cursor_right();
                true
            },
            KeyEvent::Home => {
                self.cursor_position.set(0);
                self.clear_selection();
                true
            },
            KeyEvent::End => {
                self.cursor_position.set(self.value.get().len());
                self.clear_selection();
                true
            },
            _ => false,
        }
    }

    /// Insert character at cursor position
    fn insert_char(&self, ch: char) {
        let mut value = self.value.get();
        let cursor_pos = self.cursor_position.get();
        
        // Check max length
        if let Some(max_len) = self.max_length {
            if value.len() >= max_len {
                return;
            }
        }
        
        // Handle selection replacement
        if let Some((start, end)) = self.get_selection() {
            value.replace_range(start..end, &ch.to_string());
            self.cursor_position.set(start + 1);
            self.clear_selection();
        } else {
            value.insert(cursor_pos, ch);
            self.cursor_position.set(cursor_pos + 1);
        }
        
        self.value.set(value.clone());
        self.validate();
        
        if let Some(callback) = &self.on_change {
            callback(&value);
        }
    }

    /// Delete character backward
    fn delete_backward(&self) {
        let mut value = self.value.get();
        let cursor_pos = self.cursor_position.get();
        
        if let Some((start, end)) = self.get_selection() {
            value.replace_range(start..end, "");
            self.cursor_position.set(start);
            self.clear_selection();
        } else if cursor_pos > 0 {
            value.remove(cursor_pos - 1);
            self.cursor_position.set(cursor_pos - 1);
        }
        
        self.value.set(value.clone());
        self.validate();
        
        if let Some(callback) = &self.on_change {
            callback(&value);
        }
    }

    /// Delete character forward
    fn delete_forward(&self) {
        let mut value = self.value.get();
        let cursor_pos = self.cursor_position.get();
        
        if let Some((start, end)) = self.get_selection() {
            value.replace_range(start..end, "");
            self.cursor_position.set(start);
            self.clear_selection();
        } else if cursor_pos < value.len() {
            value.remove(cursor_pos);
        }
        
        self.value.set(value.clone());
        self.validate();
        
        if let Some(callback) = &self.on_change {
            callback(&value);
        }
    }

    /// Move cursor left
    fn move_cursor_left(&self) {
        let cursor_pos = self.cursor_position.get();
        if cursor_pos > 0 {
            self.cursor_position.set(cursor_pos - 1);
        }
        self.clear_selection();
    }

    /// Move cursor right
    fn move_cursor_right(&self) {
        let cursor_pos = self.cursor_position.get();
        let value_len = self.value.get().len();
        if cursor_pos < value_len {
            self.cursor_position.set(cursor_pos + 1);
        }
        self.clear_selection();
    }

    /// Update cursor blink timer
    pub fn update(&self, delta_time: f32) {
        if self.is_focused() {
            let mut timer = self.cursor_blink_timer.get();
            timer += delta_time;
            if timer > 1.0 {
                timer = 0.0;
            }
            self.cursor_blink_timer.set(timer);
        }
    }

    /// Render the input
    pub fn render(&self, batch: &mut RenderBatch) {
        if !self.visible.get() {
            return;
        }

        let bounds = self.bounds.get();
        let content_bounds = self.content_bounds.get();
        let current_style = self.style.for_state(self.state.get());
        
        // Render background
        let (bg_vertices, bg_indices) = VertexBuilder::rounded_rectangle(
            bounds.x,
            bounds.y,
            bounds.width,
            bounds.height,
            current_style.border_radius,
            current_style.background_color.to_array(),
            8, // corner segments
        );
        batch.add_vertices(&bg_vertices, &bg_indices);
        
        // Render border
        if current_style.border_width > 0.0 {
            let (border_vertices, border_indices) = VertexBuilder::rounded_rectangle_outline(
                bounds.x,
                bounds.y,
                bounds.width,
                bounds.height,
                current_style.border_radius,
                current_style.border_color.to_array(),
                current_style.border_width,
                8, // corner segments
            );
            batch.add_vertices(&border_vertices, &border_indices);
        }
        
        let value = self.value.get();
        let display_text = if self.input_type == InputType::Password {
            "*".repeat(value.len())
        } else {
            value.clone()
        };
        
        // Render selection background
        if let Some((start, end)) = self.get_selection() {
            if start != end {
                let char_width = current_style.font_size * 0.6;
                let selection_x = content_bounds.x + start as f32 * char_width;
                let selection_width = (end - start) as f32 * char_width;
                
                let (sel_vertices, sel_indices) = VertexBuilder::rectangle(
                    selection_x,
                    content_bounds.y,
                    selection_width,
                    content_bounds.height,
                    current_style.selection_color.to_array(),
                );
                batch.add_vertices(&sel_vertices, &sel_indices);
            }
        }
        
        // Render text or placeholder
        if !display_text.is_empty() {
            batch.add_text(
                display_text.clone(),
                (content_bounds.x, content_bounds.y + current_style.font_size * 0.8),
                current_style.text_color,
                current_style.font_size,
            );
        } else if !self.placeholder.is_empty() {
            batch.add_text(
                self.placeholder.clone(),
                (content_bounds.x, content_bounds.y + current_style.font_size * 0.8),
                current_style.placeholder_color,
                current_style.font_size,
            );
        }
        
        // Render cursor
        if self.is_focused() && self.cursor_blink_timer.get() < 0.5 {
            let cursor_pos = self.cursor_position.get();
            let char_width = current_style.font_size * 0.6;
            let cursor_x = content_bounds.x + cursor_pos as f32 * char_width;
            
            let (cursor_vertices, cursor_indices) = VertexBuilder::line(
                cursor_x,
                content_bounds.y,
                cursor_x,
                content_bounds.y + content_bounds.height,
                1.0,
                current_style.cursor_color.to_array(),
            );
            batch.add_vertices(&cursor_vertices, &cursor_indices);
        }
    }

    /// Apply theme to input
    pub fn apply_theme(&mut self, theme: &Theme) {
        self.style.background_color = theme.colors.surface.to_types_color();
        self.style.border_color = theme.colors.outline.to_types_color();
        self.style.text_color = theme.colors.on_surface.to_types_color();
        self.style.selection_color = Color::rgba(
            theme.colors.primary.r, 
            theme.colors.primary.g, 
            theme.colors.primary.b, 
            0.3
        );
    }
}

/// Text input builder for fluent API
pub struct TextInputBuilder {
    input: TextInput,
}

impl TextInputBuilder {
    /// Create a new text input builder
    pub fn new() -> Self {
        Self {
            input: TextInput::new(),
        }
    }

    /// Set input type
    pub fn input_type(mut self, input_type: InputType) -> Self {
        self.input = self.input.input_type(input_type);
        self
    }

    /// Set placeholder
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.input = self.input.placeholder(placeholder);
        self
    }

    /// Set initial value
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.input = self.input.value(value);
        self
    }

    /// Set required
    pub fn required(mut self, required: bool) -> Self {
        self.input = self.input.required(required);
        self
    }

    /// Set disabled
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.input = self.input.disabled(disabled);
        self
    }

    /// Add validator
    pub fn validator<F>(mut self, validator: F) -> Self 
    where
        F: Fn(&str) -> Result<(), String> + Send + Sync + 'static,
    {
        self.input = self.input.validator(validator);
        self
    }

    /// Set change callback
    pub fn on_change<F>(mut self, callback: F) -> Self
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.input = self.input.on_change(callback);
        self
    }

    /// Build the input
    pub fn build(self) -> TextInput {
        self.input
    }
}

impl Default for TextInputBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_creation() {
        let input = TextInput::new();
        assert_eq!(input.get_value(), "");
        assert!(!input.is_focused());
        assert!(!input.is_disabled());
    }

    #[test]
    fn test_input_value() {
        let input = TextInput::new().value("test");
        assert_eq!(input.get_value(), "test");
    }

    #[test]
    fn test_input_validation() {
        let input = TextInput::new()
            .required(true)
            .min_length(3);
        
        // Empty value should be invalid (required)
        assert!(!input.validate());
        
        // Short value should be invalid
        input.set_value("ab");
        assert!(!input.validate());
        
        // Valid value
        input.set_value("abc");
        assert!(input.validate());
    }

    #[test]
    fn test_input_builder() {
        let input = TextInputBuilder::new()
            .placeholder("Enter text")
            .required(true)
            .build();
            
        assert_eq!(input.placeholder, "Enter text");
        assert!(input.required);
    }
}
