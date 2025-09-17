//! Dropdown and Select widgets implementation for OxideUI

use crate::widget::{Widget, WidgetId, generate_id};
use oxide_core::{
    event::{Event, EventResult, KeyCode, KeyboardEvent, MouseEvent, MouseButton},
    layout::{Size, Constraints, Layout},
    state::Signal,
    types::Rect,
    vdom::VNode,
};
use oxide_renderer::batch::RenderBatch;

/// Dropdown/Select widget for choosing from a list of options
#[derive(Debug, Clone)]
pub struct Dropdown<T: Clone + PartialEq + std::fmt::Display + std::fmt::Debug> {
    id: WidgetId,
    options: Vec<DropdownOption<T>>,
    selected_index: Signal<Option<usize>>,
    is_open: Signal<bool>,
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
    pub fn selected_index(mut self, index: Option<usize>) -> Self {
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

    /// Render the dropdown button
    fn render_button(&self, _layout: Layout) -> VNode {
        let selected_text = if let Some(index) = self.selected_index.get() {
            self.options.get(index)
                .map(|opt| opt.label.clone())
                .unwrap_or_else(|| self.placeholder.clone())
        } else {
            self.placeholder.clone()
        };

        let is_placeholder = self.selected_index.get().is_none();
        let text_color = if is_placeholder {
            self.style.placeholder_color
        } else {
            self.style.text_color
        };

        let background_color = if !self.enabled {
            self.style.disabled_color
        } else {
            self.style.background_color
        };

        VNode::element("div")
            .attr("class", "dropdown-button")
            .attr("width", format!("{}px", self.width))
            .attr("height", format!("{}px", self.height))
            .attr("background-color", format!("rgba({}, {}, {}, {})",
                background_color[0], background_color[1], background_color[2], background_color[3]))
            .attr("border", format!("{}px solid rgba({}, {}, {}, {})",
                self.style.border_width,
                self.style.border_color[0], self.style.border_color[1],
                self.style.border_color[2], self.style.border_color[3]))
            .attr("border-radius", format!("{}px", self.style.border_radius))
            .attr("padding", format!("{}px", self.style.padding))
            .attr("cursor", if self.enabled { "pointer" } else { "not-allowed" })
            .attr("display", "flex")
            .attr("align-items", "center")
            .attr("justify-content", "space-between")
            .children(vec![
                VNode::text(&selected_text)
                    .attr("color", format!("rgba({}, {}, {}, {})",
                        text_color[0], text_color[1], text_color[2], text_color[3]))
                    .attr("font-size", format!("{}px", self.style.font_size)),
                VNode::element("div")
                    .attr("class", "dropdown-arrow")
                    .attr("width", "0")
                    .attr("height", "0")
                    .attr("border-left", "4px solid transparent")
                    .attr("border-right", "4px solid transparent")
                    .attr("border-top", format!("4px solid rgba({}, {}, {}, {})",
                        text_color[0], text_color[1], text_color[2], text_color[3]))
                    .attr("transform", if self.is_open() { "rotate(180deg)" } else { "rotate(0deg)" })
            ])
    }

    /// Render the dropdown list
    fn render_dropdown(&self, _layout: Layout) -> Option<VNode> {
        if !self.is_open() {
            return None;
        }

        let filtered_options = self.filtered_options();
        let option_height: f32 = self.height;
        let _dropdown_height = (filtered_options.len() as f32 * option_height).min(self.max_height);

        let mut children = Vec::new();

        // Search input if searchable
        if self.searchable {
            let search_input = VNode::element("input")
                .attr("class", "dropdown-search")
                .attr("type", "text")
                .attr("placeholder", "Search...")
                .attr("value", &self.search_text.get())
                .attr("width", "100%")
                .attr("height", format!("{}px", option_height))
                .attr("padding", format!("{}px", self.style.padding))
                .attr("border", "none")
                .attr("border-bottom", format!("1px solid rgba({}, {}, {}, {})",
                    self.style.border_color[0], self.style.border_color[1],
                    self.style.border_color[2], self.style.border_color[3]))
                .attr("font-size", format!("{}px", self.style.font_size))
                .attr("outline", "none");
            children.push(search_input);
        }

        // Options
        for (original_index, option) in filtered_options {
            let is_selected = self.selected_index.get() == Some(original_index);
            let background_color = if is_selected {
                self.style.selected_color
            } else {
                self.style.dropdown_background
            };

            let text_color = if option.enabled {
                if is_selected {
                    [1.0, 1.0, 1.0, 1.0] // White for selected
                } else {
                    self.style.text_color
                }
            } else {
                self.style.disabled_color
            };

            let option_node = VNode::element("div")
                .attr("class", "dropdown-option")
                .attr("width", "100%")
                .attr("height", format!("{}px", option_height))
                .attr("padding", format!("{}px", self.style.padding))
                .attr("background-color", format!("rgba({}, {}, {}, {})",
                    background_color[0], background_color[1], background_color[2], background_color[3]))
                .attr("cursor", if option.enabled { "pointer" } else { "not-allowed" })
                .attr("display", "flex")
                .attr("align-items", "center")
                .children(vec![
                    VNode::text(&option.label)
                        .attr("color", format!("rgba({}, {}, {}, {})",
                            text_color[0], text_color[1], text_color[2], text_color[3]))
                        .attr("font-size", format!("{}px", self.style.font_size))
                ]);

            children.push(option_node);
        }

        Some(VNode::element("div")
            .attr("class", "dropdown-list")
            .attr("position", "absolute")
            .attr("top", format!("{}px", self.height))
            .attr("left", "0")
            .attr("width", format!("{}px", self.width))
            .attr("max-height", format!("{}px", self.max_height))
            .attr("background-color", format!("rgba({}, {}, {}, {})",
                self.style.dropdown_background[0], self.style.dropdown_background[1],
                self.style.dropdown_background[2], self.style.dropdown_background[3]))
            .attr("border", format!("{}px solid rgba({}, {}, {}, {})",
                self.style.border_width,
                self.style.dropdown_border_color[0], self.style.dropdown_border_color[1],
                self.style.dropdown_border_color[2], self.style.dropdown_border_color[3]))
            .attr("border-radius", format!("{}px", self.style.border_radius))
            .attr("box-shadow", format!("0 4px 8px rgba({}, {}, {}, {})",
                self.style.dropdown_shadow[0], self.style.dropdown_shadow[1],
                self.style.dropdown_shadow[2], self.style.dropdown_shadow[3]))
            .attr("overflow-y", "auto")
            .attr("z-index", "1000")
            .children(children))
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

    fn render(&self, _batch: &mut RenderBatch, _layout: Layout) {
        // TODO: Implement proper rendering with RenderBatch
    }

    fn handle_event(&mut self, event: &Event) -> EventResult {
        match event {
            Event::MouseDown(_mouse_event) | Event::MouseUp(_mouse_event) | Event::MouseMove(_mouse_event) => {
                // TODO: Fix bounds access - need to get bounds from layout
                // self.handle_mouse_event(mouse_event, bounds)
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