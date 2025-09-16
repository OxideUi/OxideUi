//! Button widget implementation

use crate::widget::{Widget, WidgetId, generate_id, BaseWidget};
use oxide_core::{
    event::{Event, EventResult, MouseEvent},
    layout::{Constraints, Layout, Size},
    types::{Color},
};
use oxide_renderer::batch::RenderBatch;
use std::any::Any;
use std::fmt;
use std::sync::Arc;

/// Button widget
#[derive(Clone)]
pub struct Button {
    id: WidgetId,
    base: BaseWidget,
    text: String,
    style: ButtonStyle,
    on_click: Option<Arc<dyn Fn() + Send + Sync>>,
    is_pressed: bool,
    is_hovered: bool,
    is_disabled: bool,
}

impl fmt::Debug for Button {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Button")
            .field("id", &self.id)
            .field("base", &self.base)
            .field("text", &self.text)
            .field("style", &self.style)
            .field("has_on_click", &self.on_click.is_some())
            .field("is_pressed", &self.is_pressed)
            .field("is_hovered", &self.is_hovered)
            .field("is_disabled", &self.is_disabled)
            .finish()
    }
}

impl Button {
    /// Create a new button with text
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            id: generate_id(),
            base: BaseWidget::new(),
            text: text.into(),
            style: ButtonStyle::default(),
            on_click: None,
            is_pressed: false,
            is_hovered: false,
            is_disabled: false,
        }
    }

    /// Set button style
    pub fn style(mut self, style: ButtonStyle) -> Self {
        self.style = style;
        self
    }

    /// Set click handler
    pub fn on_click<F>(mut self, handler: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_click = Some(Arc::new(handler));
        self
    }

    /// Enable or disable the button
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.is_disabled = !enabled;
        self
    }

    /// Set button size
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.base = self.base.with_min_size(width, height);
        self
    }

    /// Get button text
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Get current button color based on state
    fn get_color(&self) -> Color {
        if self.is_disabled {
            self.style.disabled_color
        } else if self.is_pressed {
            self.style.pressed_color
        } else if self.is_hovered {
            self.style.hover_color
        } else {
            self.style.normal_color
        }
    }
}

impl Widget for Button {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn layout(&mut self, constraints: Constraints) -> Size {
        // Calculate button size based on text
        let padding = self.style.padding;
        let text_width = self.text.len() as f32 * 8.0; // Approximate
        let text_height = 16.0; // Default font size
        
        let width = (text_width + padding * 2.0)
            .max(self.style.min_width)
            .min(constraints.max_width);
        let height = (text_height + padding * 2.0)
            .max(self.style.min_height)
            .min(constraints.max_height);
        
        Size::new(width, height)
    }

    fn render(&self, batch: &mut RenderBatch, layout: Layout) {
        let rect = oxide_core::types::Rect::new(
            layout.position.x,
            layout.position.y,
            layout.size.width,
            layout.size.height,
        );
        
        // Draw button background
        let bg_color = self.get_color();
        
        batch.draw_rounded_rect(rect, bg_color, self.style.border_radius);
        
        // Draw border if present
        if self.style.border_width > 0.0 {
            let border_rect = oxide_core::types::Rect::new(
                layout.position.x + self.style.border_width / 2.0,
                layout.position.y + self.style.border_width / 2.0,
                layout.size.width - self.style.border_width,
                layout.size.height - self.style.border_width,
            );
            // Draw border as a slightly smaller rect on top
            batch.draw_rounded_rect(border_rect, self.style.border_color, self.style.border_radius);
        }
        
        // Draw button text centered
        let text_width = self.text.len() as f32 * 8.0; // Approximate text width
        let text_height = 16.0;
        let text_x = layout.position.x + (layout.size.width - text_width) / 2.0;
        let text_y = layout.position.y + (layout.size.height - text_height) / 2.0;
        
        batch.draw_text(
            &self.text,
            (text_x, text_y),
            self.style.text_color,
            16.0,
        );
    }

    fn handle_event(&mut self, event: &Event) -> EventResult {
        if self.is_disabled {
            return EventResult::Ignored;
        }

        match event {
            Event::MouseDown(MouseEvent { button: Some(_), .. }) => {
                self.is_pressed = true;
                EventResult::Handled
            }
            Event::MouseUp(MouseEvent { .. }) => {
                if self.is_pressed {
                    self.is_pressed = false;
                    if let Some(handler) = &self.on_click {
                        (handler)();
                    }
                    EventResult::Handled
                } else {
                    EventResult::Ignored
                }
            }
            Event::MouseEnter => {
                self.is_hovered = true;
                EventResult::Handled
            }
            Event::MouseExit => {
                self.is_hovered = false;
                self.is_pressed = false;
                EventResult::Handled
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

/// Button style configuration
#[derive(Debug, Clone)]
pub struct ButtonStyle {
    pub normal_color: Color,
    pub hover_color: Color,
    pub pressed_color: Color,
    pub disabled_color: Color,
    pub text_color: Color,
    pub border_color: Color,
    pub border_width: f32,
    pub border_radius: f32,
    pub padding: f32,
    pub min_width: f32,
    pub min_height: f32,
}

impl ButtonStyle {
    /// Primary button style
    pub fn primary() -> Self {
        Self {
            normal_color: Color::PRIMARY,
            hover_color: Color::rgb(0.1, 0.5, 0.9),
            pressed_color: Color::rgb(0.08, 0.4, 0.8),
            disabled_color: Color::rgb(0.5, 0.5, 0.5),
            text_color: Color::WHITE,
            border_color: Color::rgba(0.0, 0.0, 0.0, 0.2),
            border_width: 1.0,
            border_radius: 4.0,
            padding: 12.0,
            min_width: 80.0,
            min_height: 36.0,
        }
    }

    /// Secondary button style
    pub fn secondary() -> Self {
        Self {
            normal_color: Color::rgb(0.9, 0.9, 0.9),
            hover_color: Color::rgb(0.85, 0.85, 0.85),
            pressed_color: Color::rgb(0.8, 0.8, 0.8),
            disabled_color: Color::rgb(0.5, 0.5, 0.5),
            text_color: Color::BLACK,
            border_color: Color::rgba(0.0, 0.0, 0.0, 0.15),
            border_width: 1.0,
            border_radius: 4.0,
            padding: 12.0,
            min_width: 80.0,
            min_height: 36.0,
        }
    }

    /// Text button style (no background)
    pub fn text() -> Self {
        Self {
            normal_color: Color::TRANSPARENT,
            hover_color: Color::rgba(0.0, 0.0, 0.0, 0.05),
            pressed_color: Color::rgba(0.0, 0.0, 0.0, 0.1),
            disabled_color: Color::TRANSPARENT,
            text_color: Color::PRIMARY,
            border_color: Color::TRANSPARENT,
            border_width: 0.0,
            border_radius: 4.0,
            padding: 8.0,
            min_width: 0.0,
            min_height: 32.0,
        }
    }
}

impl Default for ButtonStyle {
    fn default() -> Self {
        Self::primary()
    }
}
