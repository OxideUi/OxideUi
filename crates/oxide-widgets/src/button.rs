//! Button widget implementation
//! 
//! Provides interactive button components with various styles, states, and event handling.

use oxide_core::{
    layout::{Rect, Size},
    state::{Signal, StateManager},
    theme::{Theme, ColorPalette},
    types::{Color, Point},
};
use oxide_renderer::{
    vertex::{Vertex, VertexBuilder},
    batch::RenderBatch,
};
use std::sync::Arc;

/// Button widget state
#[derive(Debug, Clone, PartialEq)]
pub enum ButtonState {
    Normal,
    Hovered,
    Pressed,
    Disabled,
    Focused,
}

/// Button style configuration
#[derive(Debug, Clone)]
pub struct ButtonStyle {
    pub background_color: Color,
    pub hover_color: Color,
    pub pressed_color: Color,
    pub disabled_color: Color,
    pub text_color: Color,
    pub border_radius: f32,
    pub border_width: f32,
    pub border_color: Color,
    pub padding: f32,
    pub font_size: f32,
    pub min_width: f32,
    pub min_height: f32,
}

impl Default for ButtonStyle {
    fn default() -> Self {
        Self {
            background_color: Color::new(0.2, 0.4, 0.8, 1.0),
            hover_color: Color::new(0.3, 0.5, 0.9, 1.0),
            pressed_color: Color::new(0.1, 0.3, 0.7, 1.0),
            disabled_color: Color::new(0.5, 0.5, 0.5, 1.0),
            text_color: Color::new(1.0, 1.0, 1.0, 1.0),
            border_radius: 4.0,
            border_width: 0.0,
            border_color: Color::new(0.0, 0.0, 0.0, 0.0),
            padding: 12.0,
            font_size: 14.0,
            min_width: 80.0,
            min_height: 32.0,
        }
    }
}

impl ButtonStyle {
    /// Create a primary button style
    pub fn primary() -> Self {
        Self {
            background_color: Color::new(0.0, 0.4, 0.8, 1.0),
            hover_color: Color::new(0.1, 0.5, 0.9, 1.0),
            pressed_color: Color::new(0.0, 0.3, 0.7, 1.0),
            ..Default::default()
        }
    }

    /// Create a secondary button style
    pub fn secondary() -> Self {
        Self {
            background_color: Color::new(0.6, 0.6, 0.6, 1.0),
            hover_color: Color::new(0.7, 0.7, 0.7, 1.0),
            pressed_color: Color::new(0.5, 0.5, 0.5, 1.0),
            text_color: Color::new(0.0, 0.0, 0.0, 1.0),
            ..Default::default()
        }
    }

    /// Create a danger button style
    pub fn danger() -> Self {
        Self {
            background_color: Color::new(0.8, 0.2, 0.2, 1.0),
            hover_color: Color::new(0.9, 0.3, 0.3, 1.0),
            pressed_color: Color::new(0.7, 0.1, 0.1, 1.0),
            ..Default::default()
        }
    }

    /// Create an outline button style
    pub fn outline() -> Self {
        Self {
            background_color: Color::new(0.0, 0.0, 0.0, 0.0),
            hover_color: Color::new(0.0, 0.4, 0.8, 0.1),
            pressed_color: Color::new(0.0, 0.4, 0.8, 0.2),
            text_color: Color::new(0.0, 0.4, 0.8, 1.0),
            border_width: 1.0,
            border_color: Color::new(0.0, 0.4, 0.8, 1.0),
            ..Default::default()
        }
    }

    /// Create a ghost button style
    pub fn ghost() -> Self {
        Self {
            background_color: Color::new(0.0, 0.0, 0.0, 0.0),
            hover_color: Color::new(0.0, 0.0, 0.0, 0.05),
            pressed_color: Color::new(0.0, 0.0, 0.0, 0.1),
            text_color: Color::new(0.3, 0.3, 0.3, 1.0),
            border_width: 0.0,
            ..Default::default()
        }
    }
}

/// Button widget
pub struct Button {
    id: String,
    text: String,
    style: ButtonStyle,
    state: Signal<ButtonState>,
    bounds: Signal<Rect>,
    enabled: Signal<bool>,
    visible: Signal<bool>,
    on_click: Option<Box<dyn Fn() + Send + Sync>>,
    on_hover: Option<Box<dyn Fn(bool) + Send + Sync>>,
    theme: Option<Arc<Theme>>,
}

impl Button {
    /// Create a new button with text
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            id: format!("button_{}", uuid::Uuid::new_v4()),
            text: text.into(),
            style: ButtonStyle::default(),
            state: Signal::new(ButtonState::Normal),
            bounds: Signal::new(Rect::new(0.0, 0.0, 0.0, 0.0)),
            enabled: Signal::new(true),
            visible: Signal::new(true),
            on_click: None,
            on_hover: None,
            theme: None,
        }
    }

    /// Set button style
    pub fn style(mut self, style: ButtonStyle) -> Self {
        self.style = style;
        self
    }

    /// Set primary style
    pub fn primary(mut self) -> Self {
        self.style = ButtonStyle::primary();
        self
    }

    /// Set secondary style
    pub fn secondary(mut self) -> Self {
        self.style = ButtonStyle::secondary();
        self
    }

    /// Set danger style
    pub fn danger(mut self) -> Self {
        self.style = ButtonStyle::danger();
        self
    }

    /// Set outline style
    pub fn outline(mut self) -> Self {
        self.style = ButtonStyle::outline();
        self
    }

    /// Set ghost style
    pub fn ghost(mut self) -> Self {
        self.style = ButtonStyle::ghost();
        self
    }

    /// Set click handler
    pub fn on_click<F>(mut self, handler: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_click = Some(Box::new(handler));
        self
    }

    /// Set hover handler
    pub fn on_hover<F>(mut self, handler: F) -> Self
    where
        F: Fn(bool) + Send + Sync + 'static,
    {
        self.on_hover = Some(Box::new(handler));
        self
    }

    /// Set enabled state
    pub fn enabled(self, enabled: bool) -> Self {
        self.enabled.set(enabled);
        self
    }

    /// Set visible state
    pub fn visible(self, visible: bool) -> Self {
        self.visible.set(visible);
        self
    }

    /// Set theme
    pub fn theme(mut self, theme: Arc<Theme>) -> Self {
        self.theme = Some(theme);
        self
    }

    /// Get button ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get button text
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Set button text
    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
    }

    /// Get current state
    pub fn get_state(&self) -> ButtonState {
        self.state.get()
    }

    /// Set button state
    pub fn set_state(&self, state: ButtonState) {
        self.state.set(state);
    }

    /// Check if button is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled.get()
    }

    /// Check if button is visible
    pub fn is_visible(&self) -> bool {
        self.visible.get()
    }

    /// Handle mouse enter event
    pub fn on_mouse_enter(&self) {
        if self.is_enabled() && self.get_state() != ButtonState::Pressed {
            self.set_state(ButtonState::Hovered);
            if let Some(ref handler) = self.on_hover {
                handler(true);
            }
        }
    }

    /// Handle mouse leave event
    pub fn on_mouse_leave(&self) {
        if self.is_enabled() {
            self.set_state(ButtonState::Normal);
            if let Some(ref handler) = self.on_hover {
                handler(false);
            }
        }
    }

    /// Handle mouse press event
    pub fn on_mouse_press(&self, point: Point) -> bool {
        if !self.is_enabled() || !self.is_visible() {
            return false;
        }

        let bounds = self.bounds.get();
        if bounds.contains(point) {
            self.set_state(ButtonState::Pressed);
            return true;
        }
        false
    }

    /// Handle mouse release event
    pub fn on_mouse_release(&self, point: Point) -> bool {
        if !self.is_enabled() || !self.is_visible() {
            return false;
        }

        let bounds = self.bounds.get();
        if bounds.contains(point) && self.get_state() == ButtonState::Pressed {
            self.set_state(ButtonState::Hovered);
            if let Some(ref handler) = self.on_click {
                handler();
            }
            return true;
        }
        false
    }

    /// Calculate button size
    pub fn calculate_size(&self, available_size: Size) -> Size {
        // Simple text measurement (in a real implementation, this would use font metrics)
        let text_width = self.text.len() as f32 * self.style.font_size * 0.6;
        let text_height = self.style.font_size;

        let width = (text_width + self.style.padding * 2.0).max(self.style.min_width);
        let height = (text_height + self.style.padding * 2.0).max(self.style.min_height);

        Size::new(
            width.min(available_size.width),
            height.min(available_size.height),
        )
    }

    /// Layout the button
    pub fn layout(&self, bounds: Rect) {
        self.bounds.set(bounds);
    }

    /// Render the button
    pub fn render(&self, batch: &mut RenderBatch) {
        if !self.is_visible() {
            return;
        }

        let bounds = self.bounds.get();
        let state = self.get_state();
        
        // Determine colors based on state
        let background_color = match state {
            ButtonState::Normal => self.style.background_color,
            ButtonState::Hovered => self.style.hover_color,
            ButtonState::Pressed => self.style.pressed_color,
            ButtonState::Disabled => self.style.disabled_color,
            ButtonState::Focused => self.style.hover_color,
        };

        // Render background
        if self.style.border_radius > 0.0 {
            let (vertices, indices) = VertexBuilder::rounded_rectangle(
                bounds.x,
                bounds.y,
                bounds.width,
                bounds.height,
                self.style.border_radius,
                background_color.to_array(),
                8, // corner segments
            );
            batch.add_vertices(&vertices, &indices);
        } else {
            let (vertices, indices) = VertexBuilder::rectangle(
                bounds.x,
                bounds.y,
                bounds.width,
                bounds.height,
                background_color.to_array(),
            );
            batch.add_vertices(&vertices, &indices);
        }

        // Render border if needed
        if self.style.border_width > 0.0 {
            let border_bounds = Rect::new(
                bounds.x - self.style.border_width / 2.0,
                bounds.y - self.style.border_width / 2.0,
                bounds.width + self.style.border_width,
                bounds.height + self.style.border_width,
            );

            if self.style.border_radius > 0.0 {
                // Render rounded border (simplified - would need proper border rendering)
                let (vertices, indices) = VertexBuilder::rounded_rectangle(
                    border_bounds.x,
                    border_bounds.y,
                    border_bounds.width,
                    border_bounds.height,
                    self.style.border_radius + self.style.border_width / 2.0,
                    self.style.border_color.to_array(),
                    8,
                );
                batch.add_vertices(&vertices, &indices);
            }
        }

        // Render text (simplified - would need proper text rendering)
        let text_x = bounds.x + bounds.width / 2.0 - (self.text.len() as f32 * self.style.font_size * 0.3);
        let text_y = bounds.y + bounds.height / 2.0 - self.style.font_size / 2.0;
        
        // For now, just add a placeholder for text rendering
        // In a real implementation, this would use a text renderer
        batch.add_text(
            &self.text,
            text_x,
            text_y,
            self.style.font_size,
            self.style.text_color.to_array(),
        );
    }

    /// Apply theme to button
    pub fn apply_theme(&mut self, theme: &Theme) {
        // Update style based on theme
        if let Some(primary_color) = theme.color_palette().primary() {
            self.style.background_color = *primary_color;
        }
        
        if let Some(text_color) = theme.color_palette().on_primary() {
            self.style.text_color = *text_color;
        }

        self.style.border_radius = theme.spacing().border_radius().medium;
        self.style.font_size = theme.typography().body().size;
    }
}

/// Button builder for fluent API
pub struct ButtonBuilder {
    button: Button,
}

impl ButtonBuilder {
    /// Create a new button builder
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            button: Button::new(text),
        }
    }

    /// Set style
    pub fn style(mut self, style: ButtonStyle) -> Self {
        self.button = self.button.style(style);
        self
    }

    /// Set as primary button
    pub fn primary(mut self) -> Self {
        self.button = self.button.primary();
        self
    }

    /// Set as secondary button
    pub fn secondary(mut self) -> Self {
        self.button = self.button.secondary();
        self
    }

    /// Set as danger button
    pub fn danger(mut self) -> Self {
        self.button = self.button.danger();
        self
    }

    /// Set as outline button
    pub fn outline(mut self) -> Self {
        self.button = self.button.outline();
        self
    }

    /// Set as ghost button
    pub fn ghost(mut self) -> Self {
        self.button = self.button.ghost();
        self
    }

    /// Set click handler
    pub fn on_click<F>(mut self, handler: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.button = self.button.on_click(handler);
        self
    }

    /// Set hover handler
    pub fn on_hover<F>(mut self, handler: F) -> Self
    where
        F: Fn(bool) + Send + Sync + 'static,
    {
        self.button = self.button.on_hover(handler);
        self
    }

    /// Set enabled state
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.button = self.button.enabled(enabled);
        self
    }

    /// Set visible state
    pub fn visible(mut self, visible: bool) -> Self {
        self.button = self.button.visible(visible);
        self
    }

    /// Build the button
    pub fn build(self) -> Button {
        self.button
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_button_creation() {
        let button = Button::new("Test Button");
        assert_eq!(button.text(), "Test Button");
        assert_eq!(button.get_state(), ButtonState::Normal);
        assert!(button.is_enabled());
        assert!(button.is_visible());
    }

    #[test]
    fn test_button_styles() {
        let primary = Button::new("Primary").primary();
        let secondary = Button::new("Secondary").secondary();
        let danger = Button::new("Danger").danger();
        
        // Styles should be different
        assert_ne!(primary.style.background_color, secondary.style.background_color);
        assert_ne!(secondary.style.background_color, danger.style.background_color);
    }

    #[test]
    fn test_button_state_changes() {
        let button = Button::new("Test");
        
        assert_eq!(button.get_state(), ButtonState::Normal);
        
        button.on_mouse_enter();
        assert_eq!(button.get_state(), ButtonState::Hovered);
        
        button.on_mouse_leave();
        assert_eq!(button.get_state(), ButtonState::Normal);
    }

    #[test]
    fn test_button_builder() {
        let button = ButtonBuilder::new("Builder Test")
            .primary()
            .enabled(true)
            .build();
            
        assert_eq!(button.text(), "Builder Test");
        assert!(button.is_enabled());
    }

    #[test]
    fn test_button_size_calculation() {
        let button = Button::new("Test");
        let available = Size::new(200.0, 100.0);
        let size = button.calculate_size(available);
        
        assert!(size.width >= button.style.min_width);
        assert!(size.height >= button.style.min_height);
        assert!(size.width <= available.width);
        assert!(size.height <= available.height);
    }
}
