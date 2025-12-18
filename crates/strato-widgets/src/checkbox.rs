//! Checkbox widget implementation for StratoUI

use crate::control::{ControlRole, ControlState};
use crate::widget::{generate_id, Widget, WidgetContext, WidgetId, WidgetState};
use std::any::Any;
use strato_core::{
    event::{Event, EventResult, MouseButton},
    layout::{Constraints, Layout, Size},
    state::Signal,
    theme::Theme,
    types::{Color, Point, Rect, Transform},
    vdom::VNode,
};
use strato_renderer::batch::RenderBatch;

/// Checkbox widget for boolean selection
#[derive(Debug, Clone)]
pub struct Checkbox {
    id: WidgetId,
    checked: Signal<bool>,
    label: Option<String>,
    enabled: bool,
    size: f32,
    style: CheckboxStyle,
    bounds: Signal<Rect>,
    control: ControlState,
}

/// Styling options for checkbox
#[derive(Debug, Clone)]
pub struct CheckboxStyle {
    pub size: f32,
    pub border_width: f32,
    pub border_radius: f32,
    pub check_color: [f32; 4],
    pub border_color: [f32; 4],
    pub background_color: [f32; 4],
    pub hover_color: [f32; 4],
    pub disabled_color: [f32; 4],
}

impl Default for CheckboxStyle {
    fn default() -> Self {
        Self {
            size: 20.0,
            border_width: 2.0,
            border_radius: 4.0,
            check_color: [1.0, 1.0, 1.0, 1.0],      // White
            border_color: [0.5, 0.5, 0.5, 1.0],     // Gray
            background_color: [0.2, 0.6, 1.0, 1.0], // Blue
            hover_color: [0.3, 0.7, 1.0, 1.0],      // Light blue
            disabled_color: [0.7, 0.7, 0.7, 1.0],   // Light gray
        }
    }
}

fn color_from(values: [f32; 4]) -> Color {
    Color::rgba(values[0], values[1], values[2], values[3])
}

fn blend_color(a: Color, b: Color, t: f32) -> Color {
    let mix = |from: f32, to: f32| from + (to - from) * t;
    Color::rgba(mix(a.r, b.r), mix(a.g, b.g), mix(a.b, b.b), mix(a.a, b.a))
}

impl Checkbox {
    /// Create a new checkbox
    pub fn new() -> Self {
        let mut control = ControlState::new(ControlRole::Checkbox);
        control.set_toggled(false);
        Self {
            id: generate_id(),
            checked: Signal::new(false),
            label: None,
            enabled: true,
            size: 20.0,
            style: CheckboxStyle::default(),
            bounds: Signal::new(Rect::new(0.0, 0.0, 0.0, 0.0)),
            control,
        }
    }

    /// Set the checked state
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked.set(checked);
        self.control.set_toggled(checked);
        self
    }

    /// Set the label text
    pub fn label<S: Into<String>>(mut self, label: S) -> Self {
        let label = label.into();
        self.control.set_label(label.clone());
        self.label = Some(label);
        self
    }

    /// Set enabled state
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self.control.set_disabled(!enabled);
        self
    }

    /// Set the checkbox size
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self.style.size = size;
        self
    }

    /// Set custom style
    pub fn style(mut self, style: CheckboxStyle) -> Self {
        self.style = style;
        self
    }

    /// Get the checked state signal
    pub fn checked_signal(&self) -> &Signal<bool> {
        &self.checked
    }

    /// Get current checked state
    pub fn is_checked(&self) -> bool {
        self.checked.get()
    }

    /// Toggle the checkbox state
    pub fn toggle(&mut self) {
        let current = self.checked.get();
        self.checked.set(!current);
        self.control.set_toggled(!current);
    }

    /// Handle click event
    fn handle_click(&mut self) -> EventResult {
        if self.enabled {
            self.toggle();
            EventResult::Handled
        } else {
            EventResult::Ignored
        }
    }

    /// Create the checkbox visual representation
    fn create_checkbox_node(&self, theme: &Theme) -> VNode {
        let checked = self.checked.get();
        let size = self.style.size;

        let background_color = if !self.enabled {
            self.style.disabled_color
        } else if checked {
            self.style.background_color
        } else {
            [1.0, 1.0, 1.0, 1.0] // White background when unchecked
        };

        let mut checkbox = VNode::element("div")
            .attr("class", "checkbox")
            .attr("width", size.to_string())
            .attr("height", size.to_string())
            .attr(
                "background-color",
                format!(
                    "rgba({}, {}, {}, {})",
                    background_color[0],
                    background_color[1],
                    background_color[2],
                    background_color[3]
                ),
            )
            .attr(
                "border",
                format!(
                    "{}px solid rgba({}, {}, {}, {})",
                    self.style.border_width,
                    self.style.border_color[0],
                    self.style.border_color[1],
                    self.style.border_color[2],
                    self.style.border_color[3]
                ),
            )
            .attr("border-radius", format!("{}px", self.style.border_radius));

        // Add checkmark if checked
        if checked {
            let checkmark = VNode::element("div")
                .attr("class", "checkmark")
                .attr(
                    "color",
                    format!(
                        "rgba({}, {}, {}, {})",
                        self.style.check_color[0],
                        self.style.check_color[1],
                        self.style.check_color[2],
                        self.style.check_color[3]
                    ),
                )
                .children(vec![VNode::text("✓")]);

            checkbox = checkbox.children(vec![checkmark]);
        }

        checkbox
    }
}

impl Default for Checkbox {
    fn default() -> Self {
        Self::new()
    }
}

impl Widget for Checkbox {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn layout(&mut self, constraints: Constraints) -> Size {
        let checkbox_size = self.style.size;
        let label_width = if let Some(ref label) = self.label {
            // Estimate label width (this would be more accurate with actual text measurement)
            label.len() as f32 * 8.0 + 8.0 // 8px per char + 8px spacing
        } else {
            0.0
        };

        let total_width = checkbox_size + label_width;
        let height = checkbox_size.max(20.0); // Minimum height for text

        constraints.constrain(Size::new(total_width, height))
    }

    fn render(&self, batch: &mut RenderBatch, layout: Layout) {
        let bounds = Rect::new(
            layout.position.x,
            layout.position.y,
            layout.size.width,
            layout.size.height,
        );
        self.bounds.set(bounds);

        // Draw checkbox background
        // Center vertically
        let box_y = bounds.y + (bounds.height - self.style.size) / 2.0;
        let box_rect = Rect::new(bounds.x, box_y, self.style.size, self.style.size);
        let state = self.control.state();
        let base_color = if !self.enabled {
            color_from(self.style.disabled_color)
        } else if self.is_checked() {
            color_from(self.style.background_color)
        } else {
            Color::WHITE
        };

        let hover_color = if self.enabled {
            color_from(self.style.hover_color)
        } else {
            base_color
        };

        let target_color = match state {
            WidgetState::Hovered | WidgetState::Pressed => hover_color,
            WidgetState::Disabled => color_from(self.style.disabled_color),
            _ => base_color,
        };
        let bg_color = blend_color(base_color, target_color, self.control.interaction_factor());

        batch.add_rect(box_rect, bg_color, Transform::identity());

        // Draw label
        if let Some(label) = &self.label {
            let text_x = bounds.x + self.style.size + 8.0;
            let text_y = bounds.y + bounds.height / 2.0 - 7.0; // approx center
            let mut label_color = Color::BLACK;
            if !self.enabled {
                label_color.a = 0.6;
            }
            batch.add_text(label.clone(), (text_x, text_y), label_color, 14.0, 0.0);
        }
    }

    fn update(&mut self, ctx: &WidgetContext) {
        self.control.update(ctx.delta_time);
    }

    fn handle_event(&mut self, event: &Event) -> EventResult {
        if let EventResult::Handled = self.control.handle_pointer_event(event, self.bounds.get()) {
            if matches!(event, Event::MouseUp(_)) {
                if self.enabled {
                    self.handle_click();
                }
            }
            return EventResult::Handled;
        }

        if let EventResult::Handled = self.control.handle_keyboard_activation(event) {
            if matches!(event, Event::KeyUp(_)) && self.enabled {
                self.handle_click();
            }
            return EventResult::Handled;
        }

        EventResult::Ignored
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

    // Removed state method as it's not part of Widget trait
}

/// Radio button widget for single selection from a group
#[derive(Debug, Clone)]
pub struct RadioButton {
    id: WidgetId,
    selected: Signal<bool>,
    group: String,
    value: String,
    label: Option<String>,
    enabled: bool,
    style: RadioStyle,
    bounds: Signal<Rect>,
    control: ControlState,
}

/// Styling options for radio button
#[derive(Debug, Clone)]
pub struct RadioStyle {
    pub size: f32,
    pub border_width: f32,
    pub dot_color: [f32; 4],
    pub border_color: [f32; 4],
    pub background_color: [f32; 4],
    pub hover_color: [f32; 4],
    pub disabled_color: [f32; 4],
}

impl Default for RadioStyle {
    fn default() -> Self {
        Self {
            size: 20.0,
            border_width: 2.0,
            dot_color: [1.0, 1.0, 1.0, 1.0],        // White
            border_color: [0.5, 0.5, 0.5, 1.0],     // Gray
            background_color: [0.2, 0.6, 1.0, 1.0], // Blue
            hover_color: [0.3, 0.7, 1.0, 1.0],      // Light blue
            disabled_color: [0.7, 0.7, 0.7, 1.0],   // Light gray
        }
    }
}

impl RadioButton {
    /// Create a new radio button
    pub fn new<S: Into<String>>(group: S, value: S) -> Self {
        let mut control = ControlState::new(ControlRole::Radio);
        control.set_toggled(false);
        Self {
            id: generate_id(),
            selected: Signal::new(false),
            group: group.into(),
            value: value.into(),
            label: None,
            enabled: true,
            style: RadioStyle::default(),
            bounds: Signal::new(Rect::new(0.0, 0.0, 0.0, 0.0)),
            control,
        }
    }

    /// Set the selected state
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected.set(selected);
        self.control.set_toggled(selected);
        self
    }

    /// Set the label text
    pub fn label<S: Into<String>>(mut self, label: S) -> Self {
        let label = label.into();
        self.control.set_label(label.clone());
        self.label = Some(label);
        self
    }

    /// Set enabled state
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self.control.set_disabled(!enabled);
        self
    }

    /// Set custom style
    pub fn style(mut self, style: RadioStyle) -> Self {
        self.style = style;
        self
    }

    /// Get the selected state signal
    pub fn selected_signal(&self) -> &Signal<bool> {
        &self.selected
    }

    /// Get current selected state
    pub fn is_selected(&self) -> bool {
        self.selected.get()
    }

    /// Get the group name
    pub fn group(&self) -> &str {
        &self.group
    }

    /// Get the value
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Select this radio button
    pub fn select(&mut self) {
        self.selected.set(true);
        self.control.set_toggled(true);
    }

    /// Deselect this radio button
    pub fn deselect(&mut self) {
        self.selected.set(false);
        self.control.set_toggled(false);
    }
}

impl Widget for RadioButton {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn layout(&mut self, constraints: Constraints) -> Size {
        let radio_size = self.style.size;
        let label_width = if let Some(ref label) = self.label {
            label.len() as f32 * 8.0 + 8.0
        } else {
            0.0
        };

        let total_width = radio_size + label_width;
        let height = radio_size.max(20.0);

        constraints.constrain(Size::new(total_width, height))
    }

    fn render(&self, batch: &mut RenderBatch, layout: Layout) {
        let bounds = Rect::new(
            layout.position.x,
            layout.position.y,
            layout.size.width,
            layout.size.height,
        );
        self.bounds.set(bounds);

        // Draw radio background (circle)
        let radio_y = bounds.y + (bounds.height - self.style.size) / 2.0;
        let center = (
            bounds.x + self.style.size / 2.0,
            radio_y + self.style.size / 2.0,
        );
        let radius = self.style.size / 2.0;

        let state = self.control.state();
        let base_color = if !self.enabled {
            color_from(self.style.disabled_color)
        } else if self.is_selected() {
            color_from(self.style.background_color)
        } else {
            Color::WHITE
        };

        let hover_color = if self.enabled {
            color_from(self.style.hover_color)
        } else {
            base_color
        };

        let target_color = match state {
            WidgetState::Hovered | WidgetState::Pressed => hover_color,
            WidgetState::Disabled => color_from(self.style.disabled_color),
            _ => base_color,
        };
        let bg_color = blend_color(base_color, target_color, self.control.interaction_factor());

        batch.add_circle(
            center,
            radius,
            bg_color,
            16,
            strato_core::types::Transform::default(),
        );

        // Draw label
        if let Some(label) = &self.label {
            let text_x = bounds.x + self.style.size + 8.0;
            let text_y = bounds.y + bounds.height / 2.0 - 7.0; // approx center
            let mut label_color = Color::BLACK;
            if !self.enabled {
                label_color.a = 0.6;
            }
            batch.add_text(label.clone(), (text_x, text_y), label_color, 14.0, 0.0);
        }
    }

    fn update(&mut self, ctx: &WidgetContext) {
        self.control.update(ctx.delta_time);
    }

    fn handle_event(&mut self, event: &Event) -> EventResult {
        if let EventResult::Handled = self.control.handle_pointer_event(event, self.bounds.get()) {
            if matches!(event, Event::MouseUp(_)) {
                if self.enabled {
                    self.select();
                }
            }
            return EventResult::Handled;
        }

        if let EventResult::Handled = self.control.handle_keyboard_activation(event) {
            if matches!(event, Event::KeyUp(_)) && self.enabled {
                self.select();
            }
            return EventResult::Handled;
        }

        EventResult::Ignored
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

    // Removed state method as it's not part of Widget trait
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checkbox_creation() {
        let checkbox = Checkbox::new();
        assert!(!checkbox.is_checked());
        assert!(checkbox.enabled);
    }

    #[test]
    fn test_checkbox_toggle() {
        let mut checkbox = Checkbox::new();
        assert!(!checkbox.is_checked());

        checkbox.toggle();
        assert!(checkbox.is_checked());

        checkbox.toggle();
        assert!(!checkbox.is_checked());
    }

    #[test]
    fn test_radio_button_creation() {
        let radio = RadioButton::new("group1", "value1");
        assert!(!radio.is_selected());
        assert_eq!(radio.group(), "group1");
        assert_eq!(radio.value(), "value1");
    }

    #[test]
    fn test_radio_button_selection() {
        let mut radio = RadioButton::new("group1", "value1");
        assert!(!radio.is_selected());

        radio.select();
        assert!(radio.is_selected());

        radio.deselect();
        assert!(!radio.is_selected());
    }
}
