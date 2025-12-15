//! Dropdown and Select widgets implementation for StratoUI

use crate::widget::{Widget, WidgetId, generate_id};
use strato_core::{
    event::{Event, EventResult, KeyCode, KeyboardEvent, MouseEvent, MouseButton},
    layout::{Size, Constraints, Layout},
    state::Signal,
    types::{Rect, Color},
    types::Transform,
    vdom::VNode,
};
use strato_renderer::{
    batch::RenderBatch,
};

/// Dropdown/Select widget for choosing from a list of options
#[derive(Debug, Clone)]
pub struct Dropdown<T: Clone + PartialEq + std::fmt::Display + std::fmt::Debug> {
    id: WidgetId,
    options: Vec<DropdownOption<T>>,
    selected_index: Signal<Option<usize>>,
    is_open: Signal<bool>,
    bounds: Signal<Rect>,
    width: f32,
    height: f32,
    max_height: f32,
    enabled: bool,
    searchable: bool,
    search_text: Signal<String>,
    placeholder: String,
    style: DropdownStyle,
}

/// Option in a dropdown
#[derive(Debug, Clone)]
pub struct DropdownOption<T: Clone + PartialEq + std::fmt::Display + std::fmt::Debug> {
    pub value: T,
    pub label: String,
    pub enabled: bool,
}

/// Styling options for dropdown
#[derive(Debug, Clone)]
pub struct DropdownStyle {
    pub background_color: [f32; 4],
    pub border_color: [f32; 4],
    pub border_width: f32,
    pub border_radius: f32,
    pub text_color: [f32; 4],
    pub placeholder_color: [f32; 4],
    pub hover_color: [f32; 4],
    pub selected_color: [f32; 4],
    pub disabled_color: [f32; 4],
    pub dropdown_background: [f32; 4],
    pub dropdown_border_color: [f32; 4],
    pub dropdown_shadow: [f32; 4],
    pub font_size: f32,
    pub padding: f32,
}

impl Default for DropdownStyle {
    fn default() -> Self {
        Self {
            background_color: [1.0, 1.0, 1.0, 1.0], // White
            border_color: [0.8, 0.8, 0.8, 1.0], // Light gray
            border_width: 1.0,
            border_radius: 4.0,
            text_color: [0.2, 0.2, 0.2, 1.0], // Dark gray
            placeholder_color: [0.6, 0.6, 0.6, 1.0], // Medium gray
            hover_color: [0.95, 0.95, 0.95, 1.0], // Light gray
            selected_color: [0.2, 0.6, 1.0, 1.0], // Blue
            disabled_color: [0.9, 0.9, 0.9, 1.0], // Light gray
            dropdown_background: [1.0, 1.0, 1.0, 1.0], // White
            dropdown_border_color: [0.7, 0.7, 0.7, 1.0], // Gray
            dropdown_shadow: [0.0, 0.0, 0.0, 0.1], // Light shadow
            font_size: 14.0,
            padding: 8.0,
        }
    }
}

impl<T: Clone + PartialEq + std::fmt::Display + std::fmt::Debug> DropdownOption<T> {
    /// Create a new dropdown option
    pub fn new(value: T, label: String) -> Self {
        Self {
            value,
            label,
            enabled: true,
        }
    }

    /// Create option with custom label
    pub fn with_label(value: T, label: String) -> Self {
        Self::new(value, label)
    }

    /// Create option using value's Display trait
    pub fn from_value(value: T) -> Self {
        let label = value.to_string();
        Self::new(value, label)
    }

    /// Set enabled state
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

impl<T: Clone + PartialEq + std::fmt::Display + std::fmt::Debug> Dropdown<T> {
    /// Create a new dropdown
    pub fn new() -> Self {
        Self {
            id: generate_id(),
            options: Vec::new(),
            selected_index: Signal::new(None),
            is_open: Signal::new(false),
            bounds: Signal::new(Rect::new(0.0, 0.0, 0.0, 0.0)),
            width: 200.0,
            height: 36.0,
            max_height: 200.0,
            enabled: true,
            searchable: false,
            search_text: Signal::new(String::new()),
            placeholder: "Select an option...".to_string(),
            style: DropdownStyle::default(),
        }
    }

    /// Add an option
    pub fn option(mut self, option: DropdownOption<T>) -> Self {
        self.options.push(option);
        self
    }

    /// Add multiple options
    pub fn options(mut self, options: Vec<DropdownOption<T>>) -> Self {
        self.options.extend(options);
        self
    }

    /// Add option from value
    pub fn add_value(mut self, value: T) -> Self {
        self.options.push(DropdownOption::from_value(value));
        self
    }

    /// Add option with custom label
    pub fn add_option(mut self, value: T, label: String) -> Self {
        self.options.push(DropdownOption::new(value, label));
        self
    }

    /// Set the selected value
    pub fn selected(self, value: T) -> Self {
        if let Some(index) = self.options.iter().position(|opt| opt.value == value) {
            self.selected_index.set(Some(index));
        }
        self
    }

    /// Set the selected index
    pub fn selected_index(self, index: Option<usize>) -> Self {
        if index.map_or(true, |i| i < self.options.len()) {
            self.selected_index.set(index);
        }
        self
    }

    /// Set the dropdown dimensions
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Set maximum dropdown height
    pub fn max_height(mut self, max_height: f32) -> Self {
        self.max_height = max_height;
        self
    }

    /// Set enabled state
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Enable search functionality
    pub fn searchable(mut self, searchable: bool) -> Self {
        self.searchable = searchable;
        self
    }

    /// Set placeholder text
    pub fn placeholder(mut self, placeholder: String) -> Self {
        self.placeholder = placeholder;
        self
    }

    /// Set custom style
    pub fn style(mut self, style: DropdownStyle) -> Self {
        self.style = style;
        self
    }

    /// Get the selected value
    pub fn get_selected(&self) -> Option<&T> {
        self.selected_index.get()
            .and_then(|index| self.options.get(index))
            .map(|opt| &opt.value)
    }

    /// Get the selected index
    pub fn get_selected_index(&self) -> Option<usize> {
        self.selected_index.get()
    }

    /// Get the selected index signal
    pub fn selected_index_signal(&self) -> &Signal<Option<usize>> {
        &self.selected_index
    }

    /// Check if dropdown is open
    pub fn is_open(&self) -> bool {
        self.is_open.get()
    }

    /// Open the dropdown
    pub fn open(&self) {
        if self.enabled {
            self.is_open.set(true);
        }
    }

    /// Close the dropdown
    pub fn close(&self) {
        self.is_open.set(false);
        self.search_text.set(String::new());
    }

    /// Toggle dropdown open state
    pub fn toggle(&self) {
        if self.is_open() {
            self.close();
        } else {
            self.open();
        }
    }

    /// Select an option by index
    pub fn select_index(&self, index: usize) {
        if index < self.options.len() && self.options[index].enabled {
            self.selected_index.set(Some(index));
            self.close();
        }
    }

    /// Get filtered options based on search
    fn filtered_options(&self) -> Vec<(usize, &DropdownOption<T>)> {
        let search = self.search_text.get().to_lowercase();
        
        if search.is_empty() {
            self.options.iter().enumerate().collect()
        } else {
            self.options
                .iter()
                .enumerate()
                .filter(|(_, opt)| opt.label.to_lowercase().contains(&search))
                .collect()
        }
    }

    /// Handle mouse events
    fn handle_mouse_event(&self, event: &MouseEvent, bounds: Rect) -> EventResult {
        if !self.enabled {
            return EventResult::Ignored;
        }

        if let Some(MouseButton::Left) = event.button {
            if self.is_open() {
                // Check if clicking on an option
                let dropdown_y = bounds.y + self.height;
                let option_height = self.height;
                let filtered_options = self.filtered_options();
                
                if event.position.y >= dropdown_y {
                    let option_index = ((event.position.y - dropdown_y) / option_height) as usize;
                    if let Some((original_index, _)) = filtered_options.get(option_index) {
                        self.select_index(*original_index);
                        return EventResult::Handled;
                    }
                }
                
                // Click outside dropdown - close it
                self.close();
            } else {
                // Click on dropdown button - open it
                self.toggle();
            }
            EventResult::Handled
        } else {
            EventResult::Ignored
        }
    }

    /// Handle keyboard events
    fn handle_keyboard_event(&self, event: &KeyboardEvent) -> EventResult {
        if !self.enabled {
            return EventResult::Ignored;
        }

        match event.key_code {
            KeyCode::Escape => {
                if self.is_open() {
                    self.close();
                    EventResult::Handled
                } else {
                    EventResult::Ignored
                }
            }
            KeyCode::Enter => {
                if !self.is_open() {
                    self.open();
                    EventResult::Handled
                } else {
                    EventResult::Ignored
                }
            }
            KeyCode::Down => {
                if self.is_open() {
                    let filtered = self.filtered_options();
                    let current = self.selected_index.get();
                    
                    let next_index = if let Some(current_idx) = current {
                        filtered.iter()
                            .position(|(idx, _)| *idx == current_idx)
                            .map(|pos| (pos + 1).min(filtered.len() - 1))
                            .unwrap_or(0)
                    } else {
                        0
                    };
                    
                    if let Some((original_idx, _)) = filtered.get(next_index) {
                        self.selected_index.set(Some(*original_idx));
                    }
                } else {
                    self.open();
                }
                EventResult::Handled
            }
            KeyCode::Up => {
                if self.is_open() {
                    let filtered = self.filtered_options();
                    let current = self.selected_index.get();
                    
                    let prev_index = if let Some(current_idx) = current {
                        filtered.iter()
                            .position(|(idx, _)| *idx == current_idx)
                            .map(|pos| pos.saturating_sub(1))
                            .unwrap_or(0)
                    } else {
                        filtered.len().saturating_sub(1)
                    };
                    
                    if let Some((original_idx, _)) = filtered.get(prev_index) {
                        self.selected_index.set(Some(*original_idx));
                    }
                }
                EventResult::Handled
            }
            KeyCode::Backspace if self.searchable && self.is_open() => {
                let mut search = self.search_text.get();
                search.pop();
                self.search_text.set(search);
                EventResult::Handled
            }
            _ => {
                // Handle text input from KeyboardEvent
                if let Some(ref text) = event.text {
                    if self.searchable && self.is_open() {
                        for ch in text.chars() {
                            if !ch.is_control() {
                                let mut search = self.search_text.get();
                                search.push(ch);
                                self.search_text.set(search);
                            }
                        }
                        EventResult::Handled
                    } else {
                        EventResult::Ignored
                    }
                } else {
                    EventResult::Ignored
                }
            }
        }
    }


}

impl<T: Clone + PartialEq + std::fmt::Display + std::fmt::Debug> Default for Dropdown<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone + PartialEq + std::fmt::Display + std::fmt::Debug + Send + Sync + 'static> Widget for Dropdown<T> {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn layout(&mut self, constraints: Constraints) -> Size {
        let size = Size::new(self.width, self.height);
        constraints.constrain(size)
    }

    fn render(&self, batch: &mut RenderBatch, layout: Layout) {
        let bounds = Rect::new(layout.position.x, layout.position.y, layout.size.width, layout.size.height);
        self.bounds.set(bounds);

        // Background
        let bg_color = if !self.enabled {
            self.style.disabled_color
        } else {
            self.style.background_color
        };
        
        batch.add_rounded_rect(
            bounds,
            Color::rgba(bg_color[0], bg_color[1], bg_color[2], bg_color[3]),
            self.style.border_radius,
            Transform::identity(),
        );

        // Border (simple)
        if self.style.border_width > 0.0 {
            // TODO: Proper border rendering
        }

        // Text
        let selected_text = if let Some(index) = self.selected_index.get() {
            self.options.get(index)
                .map(|opt| opt.label.clone())
                .unwrap_or_else(|| self.placeholder.clone())
        } else {
            self.placeholder.clone()
        };
        
        let text_color = if self.selected_index.get().is_none() {
            self.style.placeholder_color
        } else {
            self.style.text_color
        };

        batch.add_text_aligned(
            selected_text,
            (bounds.x + self.style.padding, bounds.y + bounds.height / 2.0 - self.style.font_size / 2.0),
            Color::rgba(text_color[0], text_color[1], text_color[2], text_color[3]),
            self.style.font_size,
            0.0,
            strato_core::text::TextAlign::Left,
        );

        // Arrow (Simple triangle)
        let arrow_color = self.style.text_color;
        let arrow_x = bounds.x + bounds.width - self.style.padding - 10.0;
        let arrow_y = bounds.y + bounds.height / 2.0;
        let _arrow_size = 5.0;
        
        // Vertices for arrow
        // This requires manual vertex adding or a shape primitive
        // For now, let's skip drawing arrow or use a small rect
        batch.add_rect(
            Rect::new(arrow_x, arrow_y - 2.0, 10.0, 4.0),
            Color::rgba(arrow_color[0], arrow_color[1], arrow_color[2], arrow_color[3]),
            Transform::identity()
        );

        // Dropdown List
        if self.is_open.get() {
            let filtered_options = self.filtered_options();
            let option_height = self.height;
            let list_height = (filtered_options.len() as f32 * option_height).min(self.max_height);
            
            let list_bounds = Rect::new(
                bounds.x,
                bounds.y + bounds.height,
                bounds.width,
                list_height
            );

            // List Background
            let list_bg = self.style.dropdown_background;
            batch.add_overlay_rect(
                list_bounds,
                Color::rgba(list_bg[0], list_bg[1], list_bg[2], list_bg[3]),
                Transform::identity(),
            );

            // Options
            let mut y = list_bounds.y;
            for (original_index, option) in filtered_options {
                if y + option_height > list_bounds.y + list_bounds.height {
                    break; // Clip
                }

                let is_selected = self.selected_index.get() == Some(original_index);
                let opt_bg = if is_selected {
                    self.style.selected_color
                } else {
                    self.style.dropdown_background
                };

                let opt_rect = Rect::new(list_bounds.x, y, list_bounds.width, option_height);
                batch.add_overlay_rect(
                    opt_rect,
                    Color::rgba(opt_bg[0], opt_bg[1], opt_bg[2], opt_bg[3]),
                    Transform::identity(),
                );

                let opt_text_color = if is_selected {
                    [1.0, 1.0, 1.0, 1.0]
                } else {
                    self.style.text_color
                };

                batch.add_overlay_text_aligned(
                    option.label.clone(),
                    (opt_rect.x + self.style.padding, opt_rect.y + opt_rect.height / 2.0 - self.style.font_size / 2.0),
                    Color::rgba(opt_text_color[0], opt_text_color[1], opt_text_color[2], opt_text_color[3]),
                    self.style.font_size,
                    0.0,
                    strato_core::text::TextAlign::Left,
                );

                y += option_height;
            }
        }
    }

    fn handle_event(&mut self, event: &Event) -> EventResult {
        let bounds = self.bounds.get();
        match event {
            Event::MouseDown(mouse_event) => {
                // Check if click is outside
                let point = strato_core::types::Point::new(mouse_event.position.x, mouse_event.position.y);
                
                // If open, check if we clicked inside the list
                if self.is_open.get() {
                    let list_height = (self.filtered_options().len() as f32 * self.height).min(self.max_height);
                    let list_bounds = Rect::new(
                        bounds.x,
                        bounds.y + bounds.height,
                        bounds.width,
                        list_height
                    );
                    
                    if list_bounds.contains(point) {
                        return self.handle_mouse_event(mouse_event, bounds);
                    }
                }

                if bounds.contains(point) {
                    return self.handle_mouse_event(mouse_event, bounds);
                } else if self.is_open.get() {
                    // Click outside closes
                    self.close();
                    return EventResult::Handled;
                }
                
                EventResult::Ignored
            },
            Event::KeyDown(keyboard_event) | Event::KeyUp(keyboard_event) => {
                self.handle_keyboard_event(keyboard_event)
            },
            _ => EventResult::Ignored,
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn clone_widget(&self) -> Box<dyn Widget> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dropdown_creation() {
        let dropdown: Dropdown<String> = Dropdown::new();
        assert_eq!(dropdown.get_selected(), None);
        assert!(!dropdown.is_open());
    }

    #[test]
    fn test_dropdown_options() {
        let dropdown = Dropdown::new()
            .add_value("Option 1".to_string())
            .add_value("Option 2".to_string())
            .add_option("Option 3".to_string(), "Custom Label".to_string());
        
        assert_eq!(dropdown.options.len(), 3);
        assert_eq!(dropdown.options[2].label, "Custom Label");
    }

    #[test]
    fn test_dropdown_selection() {
        let dropdown = Dropdown::new()
            .add_value("Option 1".to_string())
            .add_value("Option 2".to_string())
            .selected("Option 2".to_string());
        
        assert_eq!(dropdown.get_selected_index(), Some(1));
        assert_eq!(dropdown.get_selected(), Some(&"Option 2".to_string()));
    }

    #[test]
    fn test_dropdown_toggle() {
        let dropdown: Dropdown<String> = Dropdown::new();
        
        assert!(!dropdown.is_open());
        dropdown.open();
        assert!(dropdown.is_open());
        dropdown.close();
        assert!(!dropdown.is_open());
    }
}