//! Text input widget implementation
//! 
//! Provides text input components with various input types, validation, and formatting options.

use oxide_core::{
    layout::{Size, Constraints, Layout},
    state::{Signal},
    theme::{Theme},
    types::{Point, Rect, Color, Transform},
    event::{Event, EventResult, KeyboardEvent, KeyCode, KeyEvent, MouseEvent},
    vdom::{VNode},
};
use oxide_renderer::{
    vertex::{Vertex, VertexBuilder},
    batch::RenderBatch,
};
use crate::widget::{Widget, WidgetId, generate_id};
use std::{sync::Arc, any::Any};

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

/// Validation state for input
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

/// Style configuration for input widgets
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
        // Use platform-specific default fonts
        #[cfg(target_os = "windows")]
        let font_family = "Segoe UI";
        
        #[cfg(target_os = "macos")]
        let font_family = "SF Pro Display";
        
        #[cfg(target_os = "linux")]
        let font_family = "Ubuntu";
        
        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        let font_family = "Arial";
        
        Self {
            background_color: Color::WHITE,
            border_color: Color::GRAY,
            text_color: Color::BLACK,
            placeholder_color: Color::LIGHT_GRAY,
            selection_color: Color::BLUE,
            cursor_color: Color::BLACK,
            border_width: 1.0,
            border_radius: 4.0,
            padding: (8.0, 12.0, 8.0, 12.0),
            font_size: 14.0,
            font_family: font_family.to_string(),
            line_height: 1.2,
        }
    }
}

impl InputStyle {
    /// Create an outlined input style
    pub fn outlined() -> Self {
        Self {
            background_color: Color::WHITE,
            border_color: Color::GRAY,
            border_width: 1.0,
            ..Default::default()
        }
    }

    /// Create a filled input style
    pub fn filled() -> Self {
        Self {
            background_color: Color::LIGHT_GRAY,
            border_color: Color::TRANSPARENT,
            border_width: 0.0,
            ..Default::default()
        }
    }

    /// Get style for a specific input state
    pub fn for_state(&self, state: InputState) -> Self {
        let mut style = self.clone();
        match state {
            InputState::Focused => {
                style.border_color = Color::BLUE;
            }
            InputState::Hovered => {
                style.border_color = Color::DARK_GRAY;
            }
            InputState::Disabled => {
                style.background_color = Color::LIGHT_GRAY;
                style.text_color = Color::GRAY;
            }
            InputState::ReadOnly => {
                style.background_color = Color::LIGHT_GRAY;
            }
            InputState::Error => {
                style.border_color = Color::RED;
            }
            _ => {}
        }
        style
    }
}

/// Validation function type
pub type ValidationFn = Box<dyn Fn(&str) -> Result<(), String> + Send + Sync>;

/// Text input widget
pub struct TextInput {
    id: WidgetId,
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
    
    // State management
    state: Signal<InputState>,
    validation_state: Signal<ValidationState>,
    validation_message: Signal<Option<String>>,
    focused: Signal<bool>,
    hovered: Signal<bool>,
    
    // Cursor and selection
    cursor_position: Signal<usize>,
    selection_start: Signal<Option<usize>>,
    selection_end: Signal<Option<usize>>,
    
    // Layout and rendering
    bounds: Signal<Rect>,
    content_bounds: Signal<Rect>,
    visible: Signal<bool>,
    
    // Styling
    style: InputStyle,
    theme: Option<Arc<Theme>>,
    
    // Validation
    validators: Vec<ValidationFn>,
    
    // Event handlers
    on_change: Option<Box<dyn Fn(&str) + Send + Sync>>,
    on_focus: Option<Box<dyn Fn() + Send + Sync>>,
    on_blur: Option<Box<dyn Fn() + Send + Sync>>,
    on_submit: Option<Box<dyn Fn(&str) + Send + Sync>>,
    
    // Internal state
    cursor_blink_timer: Signal<f32>,
    scroll_offset: Signal<f32>,
}

impl std::fmt::Debug for TextInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TextInput")
            .field("id", &self.id)
            .field("input_type", &self.input_type)
            .field("value", &self.value)
            .field("placeholder", &self.placeholder)
            .field("max_length", &self.max_length)
            .field("min_length", &self.min_length)
            .field("pattern", &self.pattern)
            .field("required", &self.required)
            .field("disabled", &self.disabled)
            .field("readonly", &self.readonly)
            .field("multiline", &self.multiline)
            .field("rows", &self.rows)
            .field("cols", &self.cols)
            .field("state", &self.state)
            .field("validation_state", &self.validation_state)
            .field("validation_message", &self.validation_message)
            .field("focused", &self.focused)
            .field("hovered", &self.hovered)
            .field("cursor_position", &self.cursor_position)
            .field("selection_start", &self.selection_start)
            .field("selection_end", &self.selection_end)
            .field("bounds", &self.bounds)
            .field("content_bounds", &self.content_bounds)
            .field("visible", &self.visible)
            .field("style", &self.style)
            .field("theme", &self.theme)
            .field("validators", &format!("{} validators", self.validators.len()))
            .field("on_change", &self.on_change.as_ref().map(|_| "Some(callback)"))
            .field("on_focus", &self.on_focus.as_ref().map(|_| "Some(callback)"))
            .field("on_blur", &self.on_blur.as_ref().map(|_| "Some(callback)"))
            .field("on_submit", &self.on_submit.as_ref().map(|_| "Some(callback)"))
            .field("cursor_blink_timer", &self.cursor_blink_timer)
            .field("scroll_offset", &self.scroll_offset)
            .finish()
    }
}

impl TextInput {
    /// Creates a new text input widget
    pub fn new() -> Self {
        Self {
            id: generate_id(),
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
            
            // State management
            state: Signal::new(InputState::Normal),
            validation_state: Signal::new(ValidationState::Valid),
            validation_message: Signal::new(None),
            focused: Signal::new(false),
            hovered: Signal::new(false),
            
            // Cursor and selection
            cursor_position: Signal::new(0),
            selection_start: Signal::new(None),
            selection_end: Signal::new(None),
            
            // Layout and rendering
            bounds: Signal::new(Rect::new(0.0, 0.0, 0.0, 0.0)),
            content_bounds: Signal::new(Rect::new(0.0, 0.0, 0.0, 0.0)),
            visible: Signal::new(true),
            
            // Styling
            style: InputStyle::default(),
            theme: None,
            
            // Validation
            validators: Vec::new(),
            
            // Event handlers
            on_change: None,
            on_focus: None,
            on_blur: None,
            on_submit: None,
            
            // Internal state
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

    /// Set number of rows (for multiline)
    pub fn rows(mut self, rows: usize) -> Self {
        self.rows = rows;
        self
    }

    /// Set number of columns
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

    /// Gets the widget ID
    pub fn id(&self) -> WidgetId {
        self.id
    }

    /// Get current value
    pub fn get_value(&self) -> String {
        self.value.get()
    }

    /// Set value programmatically
    pub fn set_value(&self, value: impl Into<String>) {
        let new_value = value.into();
        
        // Validate length constraints
        if let Some(max_len) = self.max_length {
            if new_value.len() > max_len {
                return;
            }
        }
        
        self.value.set(new_value.clone());
        
        // Trigger validation
        self.validate();
        
        // Trigger change callback
        if let Some(ref callback) = self.on_change {
            callback(&new_value);
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
        if let (Some(start), Some(end)) = (self.selection_start.get(), self.selection_end.get()) {
            Some((start, end))
        } else {
            None
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
        if !self.is_disabled() && !self.is_readonly() {
            self.focused.set(true);
            self.state.set(InputState::Focused);
            
            // Trigger focus callback
            if let Some(ref callback) = self.on_focus {
                callback();
            }
        }
    }

    /// Blur the input
    pub fn blur(&self) {
        self.focused.set(false);
        self.clear_selection();
        
        // Update state
        if self.is_disabled() {
            self.state.set(InputState::Disabled);
        } else if self.is_readonly() {
            self.state.set(InputState::ReadOnly);
        } else if self.validation_state.get() == ValidationState::Invalid {
            self.state.set(InputState::Error);
        } else {
            self.state.set(InputState::Normal);
        }
        
        // Trigger blur callback
        if let Some(ref callback) = self.on_blur {
            callback();
        }
    }

    /// Validate input
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
        
        // Run custom validators
        for validator in &self.validators {
            if let Err(error) = validator(&value) {
                self.validation_state.set(ValidationState::Invalid);
                self.validation_message.set(Some(error));
                return false;
            }
        }
        
        self.validation_state.set(ValidationState::Valid);
        self.validation_message.set(None);
        true
    }

    /// Calculate preferred size
    pub fn calculate_size(&self, available_size: Size) -> Size {
        let style = self.style.for_state(self.state.get());
        let padding = style.padding;
        
        let text_width = if self.multiline {
            available_size.width - padding.1 - padding.3
        } else {
            (self.cols as f32) * (style.font_size * 0.6) // Approximate character width
        };
        
        let text_height = if self.multiline {
            (self.rows as f32) * (style.font_size * style.line_height)
        } else {
            style.font_size * style.line_height
        };
        
        Size::new(
            text_width + padding.1 + padding.3,
            text_height + padding.0 + padding.2,
        )
    }

    /// Layout the input
    pub fn layout(&self, bounds: Rect) {
        self.bounds.set(bounds);
        
        let style = self.style.for_state(self.state.get());
        let padding = style.padding;
        
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
        let point = Point::new(event.position.x, event.position.y);
        
        if !bounds.contains(point) {
            return false;
        }
        
        match event.button {
            Some(oxide_core::event::MouseButton::Left) => {
                // For mouse down events, we need to check if this is a press event
                // Since MouseEvent doesn't have a pressed field, we'll assume this is called for press events
                self.focus();
                
                // Calculate cursor position from click
                let content_bounds = self.content_bounds.get();
                let relative_x = point.x - content_bounds.x;
                
                // Simple cursor positioning (would need proper text measurement)
                let char_width = self.style.font_size * 0.6;
                let cursor_pos = ((relative_x / char_width) as usize).min(self.value.get().len());
                self.cursor_position.set(cursor_pos);
                
                return true;
            }
            _ => {}
        }
        
        false
    }

    /// Handle keyboard events
    pub fn handle_key_event(&self, event: &KeyboardEvent) -> bool {
        if !self.is_focused() || self.is_disabled() || self.is_readonly() {
            return false;
        }
        
        // Handle text input from KeyboardEvent
        if let Some(ref text) = event.text {
            for ch in text.chars() {
                if ch.is_control() {
                    continue;
                }
                self.insert_char(ch);
            }
            return true;
        }
        
        // Handle special keys
        match event.key_code {
            KeyCode::Backspace => {
                self.delete_backward();
                true
            }
            KeyCode::Delete => {
                self.delete_forward();
                true
            }
            KeyCode::Left => {
                self.move_cursor_left();
                true
            }
            KeyCode::Right => {
                self.move_cursor_right();
                true
            }
            KeyCode::Enter => {
                if self.multiline {
                    self.insert_char('\n');
                } else if let Some(ref callback) = self.on_submit {
                    callback(&self.value.get());
                }
                true
            }
            KeyCode::Home => {
                self.cursor_position.set(0);
                true
            }
            KeyCode::End => {
                self.cursor_position.set(self.value.get().len());
                true
            }
            _ => false,
        }
    }

    /// Insert character at cursor
    fn insert_char(&self, ch: char) {
        let mut value = self.value.get();
        let cursor_pos = self.cursor_position.get();
        
        // Check max length
        if let Some(max_len) = self.max_length {
            if value.len() >= max_len {
                return;
            }
        }
        
        // Insert character
        if cursor_pos <= value.len() {
            value.insert(cursor_pos, ch);
            self.value.set(value.clone());
            self.cursor_position.set(cursor_pos + 1);
            
            // Trigger change callback
            if let Some(ref callback) = self.on_change {
                callback(&value);
            }
            
            // Validate
            self.validate();
        }
    }

    /// Delete character before cursor
    fn delete_backward(&self) {
        let mut value = self.value.get();
        let cursor_pos = self.cursor_position.get();
        
        if cursor_pos > 0 && cursor_pos <= value.len() {
            value.remove(cursor_pos - 1);
            self.value.set(value.clone());
            self.cursor_position.set(cursor_pos - 1);
            
            // Trigger change callback
            if let Some(ref callback) = self.on_change {
                callback(&value);
            }
            
            // Validate
            self.validate();
        }
    }

    /// Delete character after cursor
    fn delete_forward(&self) {
        let mut value = self.value.get();
        let cursor_pos = self.cursor_position.get();
        
        if cursor_pos < value.len() {
            value.remove(cursor_pos);
            self.value.set(value.clone());
            
            // Trigger change callback
            if let Some(ref callback) = self.on_change {
                callback(&value);
            }
            
            // Validate
            self.validate();
        }
    }

    /// Move cursor left
    fn move_cursor_left(&self) {
        let cursor_pos = self.cursor_position.get();
        if cursor_pos > 0 {
            self.cursor_position.set(cursor_pos - 1);
        }
    }

    /// Move cursor right
    fn move_cursor_right(&self) {
        let cursor_pos = self.cursor_position.get();
        let value_len = self.value.get().len();
        if cursor_pos < value_len {
            self.cursor_position.set(cursor_pos + 1);
        }
    }

    /// Update input (called each frame)
    pub fn update(&self, delta_time: f32) {
        // Update cursor blink timer
        let mut timer = self.cursor_blink_timer.get();
        timer += delta_time;
        if timer >= 1.0 {
            timer = 0.0;
        }
        self.cursor_blink_timer.set(timer);
    }

    /// Render the input
    pub fn render(&self, batch: &mut RenderBatch) {
        let bounds = self.bounds.get();
        let content_bounds = self.content_bounds.get();
        let style = self.style.for_state(self.state.get());
        
        // Render background
        batch.add_rect(
            bounds,
            style.background_color,
            Transform::identity(),
        );
        
        // Render text or placeholder
        let value = self.value.get();
        let text_to_render = if value.is_empty() && !self.placeholder.is_empty() {
            &self.placeholder
        } else {
            &value
        };
        
        let text_color = if value.is_empty() && !self.placeholder.is_empty() {
            style.placeholder_color
        } else {
            style.text_color
        };
        
        if !text_to_render.is_empty() {
            let text_x = content_bounds.x;
            let text_y = content_bounds.y;
            batch.add_text(
                text_to_render.to_string(),
                (text_x, text_y),
                text_color,
                14.0,
                0.0, // Default letter spacing
            );
        }
        
        // Render cursor if focused
        if self.is_focused() && self.cursor_blink_timer.get() < 0.5 {
            let cursor_pos = self.cursor_position.get();
            let char_width = style.font_size * 0.6;
            let cursor_x = content_bounds.x + (cursor_pos as f32) * char_width;
            
            batch.add_line(
                (cursor_x, content_bounds.y),
                (cursor_x, content_bounds.y + content_bounds.height),
                style.cursor_color,
                1.0,
            );
        }
        
        // Render selection if any
        if let Some((start, end)) = self.get_selection() {
            let char_width = style.font_size * 0.6;
            let selection_start_x = content_bounds.x + (start as f32) * char_width;
            let selection_end_x = content_bounds.x + (end as f32) * char_width;
            
            batch.add_rect(
                Rect::new(
                    selection_start_x,
                    content_bounds.y,
                    selection_end_x - selection_start_x,
                    content_bounds.height,
                ),
                style.selection_color,
                Transform::identity(),
            );
        }
    }

    /// Apply theme to input
    pub fn apply_theme(&mut self, theme: &Theme) {
        // Apply theme colors to style
        // This would depend on the Theme structure
        self.theme = Some(Arc::new(theme.clone()));
    }
}

impl Widget for TextInput {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn layout(&mut self, constraints: Constraints) -> Size {
        let available_size = Size::new(constraints.max_width, constraints.max_height);
        let size = self.calculate_size(available_size);
        let bounds = Rect::new(0.0, 0.0, size.width, size.height);
        TextInput::layout(self, bounds);
        size
    }

    fn render(&self, batch: &mut RenderBatch, _layout: Layout) {
        self.render(batch);
    }

    fn handle_event(&mut self, event: &Event) -> EventResult {
        match event {
            Event::MouseDown(mouse_event) => {
                if self.handle_mouse_event(mouse_event) {
                    EventResult::Handled
                } else {
                    EventResult::Ignored
                }
            }
            Event::KeyDown(key_event) => {
                if self.handle_key_event(key_event) {
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

impl Clone for TextInput {
    fn clone(&self) -> Self {
        Self {
            id: generate_id(), // Generate new ID for clone
            input_type: self.input_type,
            value: Signal::new(self.value.get()),
            placeholder: self.placeholder.clone(),
            max_length: self.max_length,
            min_length: self.min_length,
            pattern: self.pattern.clone(),
            required: self.required,
            disabled: Signal::new(self.disabled.get()),
            readonly: Signal::new(self.readonly.get()),
            multiline: self.multiline,
            rows: self.rows,
            cols: self.cols,
            state: Signal::new(self.state.get()),
            validation_state: Signal::new(self.validation_state.get()),
            validation_message: Signal::new(self.validation_message.get()),
            focused: Signal::new(self.focused.get()),
            hovered: Signal::new(self.hovered.get()),
            cursor_position: Signal::new(self.cursor_position.get()),
            selection_start: Signal::new(self.selection_start.get()),
            selection_end: Signal::new(self.selection_end.get()),
            bounds: Signal::new(self.bounds.get()),
            content_bounds: Signal::new(self.content_bounds.get()),
            visible: Signal::new(self.visible.get()),
            style: self.style.clone(),
            theme: self.theme.clone(),
            validators: Vec::new(), // Don't clone validators as they contain closures
            on_change: None, // Don't clone event handlers
            on_focus: None,
            on_blur: None,
            on_submit: None,
            cursor_blink_timer: Signal::new(self.cursor_blink_timer.get()),
            scroll_offset: Signal::new(self.scroll_offset.get()),
        }
    }
}

/// Builder for TextInput
pub struct TextInputBuilder {
    input: TextInput,
}

impl TextInputBuilder {
    /// Create new builder
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

    /// Set value
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
            .validator(|value| {
                if value.len() < 3 {
                    Err("Too short".to_string())
                } else {
                    Ok(())
                }
            });
        
        // Empty value should fail validation
        assert!(!input.validate());
        
        // Set valid value
        input.set_value("test");
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
