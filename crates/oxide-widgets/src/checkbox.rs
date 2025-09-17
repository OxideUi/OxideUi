//! Checkbox widget implementation for OxideUI

use crate::widget::{Widget, WidgetId, WidgetState, WidgetContext, generate_id};
use crate::theme::Theme;
use oxide_core::{
    event::{Event, MouseEvent, MouseButton, EventResult},
    layout::{Size, Constraints, Layout},
    state::Signal,
    vdom::{VNode, VNodeBuilder},
};
use glam::Vec2;
use std::sync::Arc;

/// Checkbox widget for boolean selection
#[derive(Clone)]
pub struct Checkbox {
    id: WidgetId,
    checked: Signal<bool>,
    label: Option<String>,
    enabled: bool,
    size: f32,
    style: CheckboxStyle,
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
            check_color: [1.0, 1.0, 1.0, 1.0], // White
            border_color: [0.5, 0.5, 0.5, 1.0], // Gray
            background_color: [0.2, 0.6, 1.0, 1.0], // Blue
            hover_color: [0.3, 0.7, 1.0, 1.0], // Light blue
            disabled_color: [0.7, 0.7, 0.7, 1.0], // Light gray
        }
    }
}

impl Checkbox {
    /// Create a new checkbox
    pub fn new() -> Self {
        Self {
            id: generate_id(),
            checked: Signal::new(false),
            label: None,
            enabled: true,
            size: 20.0,
            style: CheckboxStyle::default(),
        }
    }

    /// Set the checked state
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked.set(checked);
        self
    }

    /// Set the label text
    pub fn label<S: Into<String>>(mut self, label: S) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Set enabled state
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
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
    pub fn toggle(&self) {
        let current = self.checked.get();
        self.checked.set(!current);
    }

    /// Handle click event
    fn handle_click(&self) -> EventResult {
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
            .attr("background-color", format!("rgba({}, {}, {}, {})", 
                background_color[0], background_color[1], 
                background_color[2], background_color[3]))
            .attr("border", format!("{}px solid rgba({}, {}, {}, {})", 
                self.style.border_width,
                self.style.border_color[0], self.style.border_color[1],
                self.style.border_color[2], self.style.border_color[3]))
            .attr("border-radius", format!("{}px", self.style.border_radius));

        // Add checkmark if checked
        if checked {
            let checkmark = VNode::element("div")
                .attr("class", "checkmark")
                .attr("color", format!("rgba({}, {}, {}, {})",
                    self.style.check_color[0], self.style.check_color[1],
                    self.style.check_color[2], self.style.check_color[3]))
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

    fn layout(&self, constraints: Constraints, _context: &WidgetContext) -> Size {
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

    fn render(&self, layout: Layout, context: &WidgetContext) -> VNode {
        let theme = context.theme;
        let checkbox_node = self.create_checkbox_node(theme);
        
        if let Some(ref label_text) = self.label {
            // Create container with checkbox and label
            let label_node = VNode::element("span")
                .attr("class", "checkbox-label")
                .attr("margin-left", "8px")
                .children(vec![VNode::text(label_text.clone())]);
            
            VNode::element("div")
                .attr("class", "checkbox-container")
                .attr("display", "flex")
                .attr("align-items", "center")
                .attr("cursor", if self.enabled { "pointer" } else { "not-allowed" })
                .children(vec![checkbox_node, label_node])
        } else {
            checkbox_node
                .attr("cursor", if self.enabled { "pointer" } else { "not-allowed" })
        }
    }

    fn handle_event(&mut self, event: &Event, _layout: Layout, _context: &WidgetContext) -> EventResult {
        match event {
            Event::MouseDown(mouse_event) => {
                if let Some(MouseButton::Left) = mouse_event.button {
                    self.handle_click()
                } else {
                    EventResult::Ignored
                }
            }
            _ => EventResult::Ignored,
        }
    }

    fn state(&self) -> WidgetState {
        if !self.enabled {
            WidgetState::Disabled
        } else {
            WidgetState::Normal
        }
    }
}

/// Radio button widget for single selection from a group
#[derive(Clone)]
pub struct RadioButton {
    id: WidgetId,
    selected: Signal<bool>,
    group: String,
    value: String,
    label: Option<String>,
    enabled: bool,
    style: RadioStyle,
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
            dot_color: [1.0, 1.0, 1.0, 1.0], // White
            border_color: [0.5, 0.5, 0.5, 1.0], // Gray
            background_color: [0.2, 0.6, 1.0, 1.0], // Blue
            hover_color: [0.3, 0.7, 1.0, 1.0], // Light blue
            disabled_color: [0.7, 0.7, 0.7, 1.0], // Light gray
        }
    }
}

impl RadioButton {
    /// Create a new radio button
    pub fn new<S: Into<String>>(group: S, value: S) -> Self {
        Self {
            id: generate_id(),
            selected: Signal::new(false),
            group: group.into(),
            value: value.into(),
            label: None,
            enabled: true,
            style: RadioStyle::default(),
        }
    }

    /// Set the selected state
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected.set(selected);
        self
    }

    /// Set the label text
    pub fn label<S: Into<String>>(mut self, label: S) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Set enabled state
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
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
    pub fn select(&self) {
        self.selected.set(true);
    }

    /// Deselect this radio button
    pub fn deselect(&self) {
        self.selected.set(false);
    }
}

impl Widget for RadioButton {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn layout(&self, constraints: Constraints, _context: &WidgetContext) -> Size {
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

    fn render(&self, layout: Layout, context: &WidgetContext) -> VNode {
        let selected = self.selected.get();
        let size = self.style.size;
        
        let background_color = if !self.enabled {
            self.style.disabled_color
        } else if selected {
            self.style.background_color
        } else {
            [1.0, 1.0, 1.0, 1.0]
        };

        let mut radio = VNode::element("div")
            .attr("class", "radio")
            .attr("width", size.to_string())
            .attr("height", size.to_string())
            .attr("border-radius", "50%") // Circular
            .attr("background-color", format!("rgba({}, {}, {}, {})", 
                background_color[0], background_color[1], 
                background_color[2], background_color[3]))
            .attr("border", format!("{}px solid rgba({}, {}, {}, {})", 
                self.style.border_width,
                self.style.border_color[0], self.style.border_color[1],
                self.style.border_color[2], self.style.border_color[3]));

        // Add dot if selected
        if selected {
            let dot = VNode::element("div")
                .attr("class", "radio-dot")
                .attr("width", format!("{}px", size * 0.5))
                .attr("height", format!("{}px", size * 0.5))
                .attr("border-radius", "50%")
                .attr("background-color", format!("rgba({}, {}, {}, {})",
                    self.style.dot_color[0], self.style.dot_color[1],
                    self.style.dot_color[2], self.style.dot_color[3]))
                .attr("margin", "auto");
            
            radio = radio.children(vec![dot]);
        }

        if let Some(ref label_text) = self.label {
            let label_node = VNode::element("span")
                .attr("class", "radio-label")
                .attr("margin-left", "8px")
                .children(vec![VNode::text(label_text.clone())]);
            
            VNode::element("div")
                .attr("class", "radio-container")
                .attr("display", "flex")
                .attr("align-items", "center")
                .attr("cursor", if self.enabled { "pointer" } else { "not-allowed" })
                .children(vec![radio, label_node])
        } else {
            radio.attr("cursor", if self.enabled { "pointer" } else { "not-allowed" })
        }
    }

    fn handle_event(&mut self, event: &Event, _layout: Layout, _context: &WidgetContext) -> EventResult {
        match event {
            Event::MouseDown(mouse_event) => {
                if let Some(MouseButton::Left) = mouse_event.button {
                    if self.enabled {
                        self.select();
                        EventResult::Handled
                    } else {
                        EventResult::Ignored
                    }
                } else {
                    EventResult::Ignored
                }
            }
            _ => EventResult::Ignored,
        }
    }

    fn state(&self) -> WidgetState {
        if !self.enabled {
            WidgetState::Disabled
        } else {
            WidgetState::Normal
        }
    }
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
        let checkbox = Checkbox::new();
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
        let radio = RadioButton::new("group1", "value1");
        assert!(!radio.is_selected());
        
        radio.select();
        assert!(radio.is_selected());
        
        radio.deselect();
        assert!(!radio.is_selected());
    }
}