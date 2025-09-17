//! Slider and Progress widgets implementation for OxideUI

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

/// Slider widget for numeric value selection
#[derive(Clone)]
pub struct Slider {
    id: WidgetId,
    value: Signal<f32>,
    min: f32,
    max: f32,
    step: f32,
    width: f32,
    height: f32,
    enabled: bool,
    style: SliderStyle,
    dragging: Signal<bool>,
}

/// Styling options for slider
#[derive(Debug, Clone)]
pub struct SliderStyle {
    pub track_height: f32,
    pub thumb_size: f32,
    pub track_color: [f32; 4],
    pub track_fill_color: [f32; 4],
    pub thumb_color: [f32; 4],
    pub thumb_hover_color: [f32; 4],
    pub thumb_active_color: [f32; 4],
    pub disabled_color: [f32; 4],
    pub border_radius: f32,
}

impl Default for SliderStyle {
    fn default() -> Self {
        Self {
            track_height: 4.0,
            thumb_size: 20.0,
            track_color: [0.8, 0.8, 0.8, 1.0], // Light gray
            track_fill_color: [0.2, 0.6, 1.0, 1.0], // Blue
            thumb_color: [1.0, 1.0, 1.0, 1.0], // White
            thumb_hover_color: [0.95, 0.95, 0.95, 1.0], // Light gray
            thumb_active_color: [0.9, 0.9, 0.9, 1.0], // Darker gray
            disabled_color: [0.7, 0.7, 0.7, 1.0], // Gray
            border_radius: 2.0,
        }
    }
}

impl Slider {
    /// Create a new slider
    pub fn new(min: f32, max: f32) -> Self {
        Self {
            id: generate_id(),
            value: Signal::new(min),
            min,
            max,
            step: 1.0,
            width: 200.0,
            height: 40.0,
            enabled: true,
            style: SliderStyle::default(),
            dragging: Signal::new(false),
        }
    }

    /// Set the initial value
    pub fn value(mut self, value: f32) -> Self {
        let clamped = value.clamp(self.min, self.max);
        self.value.set(clamped);
        self
    }

    /// Set the step size
    pub fn step(mut self, step: f32) -> Self {
        self.step = step.max(0.01); // Minimum step
        self
    }

    /// Set the slider dimensions
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Set enabled state
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Set custom style
    pub fn style(mut self, style: SliderStyle) -> Self {
        self.style = style;
        self
    }

    /// Get the value signal
    pub fn value_signal(&self) -> &Signal<f32> {
        &self.value
    }

    /// Get current value
    pub fn get_value(&self) -> f32 {
        self.value.get()
    }

    /// Set the value
    pub fn set_value(&self, value: f32) {
        let clamped = value.clamp(self.min, self.max);
        let stepped = if self.step > 0.0 {
            (clamped / self.step).round() * self.step
        } else {
            clamped
        };
        self.value.set(stepped);
    }

    /// Calculate value from position
    fn value_from_position(&self, x: f32, track_width: f32) -> f32 {
        let ratio = (x / track_width).clamp(0.0, 1.0);
        let value = self.min + ratio * (self.max - self.min);
        
        if self.step > 0.0 {
            (value / self.step).round() * self.step
        } else {
            value
        }
    }

    /// Calculate thumb position from value
    fn thumb_position(&self, track_width: f32) -> f32 {
        let ratio = if self.max > self.min {
            (self.value.get() - self.min) / (self.max - self.min)
        } else {
            0.0
        };
        ratio * track_width
    }

    /// Handle mouse events
    fn handle_mouse_event(&self, event: &MouseEvent, layout: Layout) -> EventResult {
        if !self.enabled {
            return EventResult::Ignored;
        }

        let track_width = self.width - self.style.thumb_size;
        let track_start_x = self.style.thumb_size / 2.0;

        match event {
            MouseEvent::Press { button: MouseButton::Left, position } => {
                let local_x = position.x - layout.position.x - track_start_x;
                let new_value = self.value_from_position(local_x, track_width);
                self.set_value(new_value);
                self.dragging.set(true);
                EventResult::Handled
            }
            MouseEvent::Drag { position, .. } if self.dragging.get() => {
                let local_x = position.x - layout.position.x - track_start_x;
                let new_value = self.value_from_position(local_x, track_width);
                self.set_value(new_value);
                EventResult::Handled
            }
            MouseEvent::Release { button: MouseButton::Left, .. } => {
                self.dragging.set(false);
                EventResult::Handled
            }
            _ => EventResult::Ignored,
        }
    }
}

impl Default for Slider {
    fn default() -> Self {
        Self::new(0.0, 100.0)
    }
}

impl Widget for Slider {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn layout(&self, constraints: Constraints, _context: &WidgetContext) -> Size {
        constraints.constrain(Size::new(self.width, self.height))
    }

    fn render(&self, layout: Layout, context: &WidgetContext) -> VNode {
        let track_width = self.width - self.style.thumb_size;
        let thumb_pos = self.thumb_position(track_width);
        let fill_width = thumb_pos;
        
        let track_y = (self.height - self.style.track_height) / 2.0;
        let thumb_y = (self.height - self.style.thumb_size) / 2.0;

        // Track background
        let track_bg = VNode::element("div")
            .attr("class", "slider-track-bg")
            .attr("position", "absolute")
            .attr("left", format!("{}px", self.style.thumb_size / 2.0))
            .attr("top", format!("{}px", track_y))
            .attr("width", format!("{}px", track_width))
            .attr("height", format!("{}px", self.style.track_height))
            .attr("background-color", format!("rgba({}, {}, {}, {})",
                self.style.track_color[0], self.style.track_color[1],
                self.style.track_color[2], self.style.track_color[3]))
            .attr("border-radius", format!("{}px", self.style.border_radius));

        // Track fill
        let track_fill = VNode::element("div")
            .attr("class", "slider-track-fill")
            .attr("position", "absolute")
            .attr("left", format!("{}px", self.style.thumb_size / 2.0))
            .attr("top", format!("{}px", track_y))
            .attr("width", format!("{}px", fill_width))
            .attr("height", format!("{}px", self.style.track_height))
            .attr("background-color", format!("rgba({}, {}, {}, {})",
                self.style.track_fill_color[0], self.style.track_fill_color[1],
                self.style.track_fill_color[2], self.style.track_fill_color[3]))
            .attr("border-radius", format!("{}px", self.style.border_radius));

        // Thumb
        let thumb_color = if !self.enabled {
            self.style.disabled_color
        } else if self.dragging.get() {
            self.style.thumb_active_color
        } else {
            self.style.thumb_color
        };

        let thumb = VNode::element("div")
            .attr("class", "slider-thumb")
            .attr("position", "absolute")
            .attr("left", format!("{}px", thumb_pos))
            .attr("top", format!("{}px", thumb_y))
            .attr("width", format!("{}px", self.style.thumb_size))
            .attr("height", format!("{}px", self.style.thumb_size))
            .attr("background-color", format!("rgba({}, {}, {}, {})",
                thumb_color[0], thumb_color[1], thumb_color[2], thumb_color[3]))
            .attr("border-radius", "50%")
            .attr("border", "2px solid rgba(0, 0, 0, 0.1)")
            .attr("cursor", if self.enabled { "pointer" } else { "not-allowed" })
            .attr("box-shadow", "0 2px 4px rgba(0, 0, 0, 0.2)");

        VNode::element("div")
            .attr("class", "slider")
            .attr("position", "relative")
            .attr("width", format!("{}px", self.width))
            .attr("height", format!("{}px", self.height))
            .children(vec![track_bg, track_fill, thumb])
    }

    fn handle_event(&mut self, event: &Event, layout: Layout, _context: &WidgetContext) -> EventResult {
        match event {
            Event::MouseDown(mouse_event) | Event::MouseUp(mouse_event) | Event::MouseMove(mouse_event) => {
                self.handle_mouse_event(mouse_event, layout)
            },
            _ => EventResult::Ignored,
        }
    }

    fn state(&self) -> WidgetState {
        if !self.enabled {
            WidgetState::Disabled
        } else if self.dragging.get() {
            WidgetState::Pressed
        } else {
            WidgetState::Normal
        }
    }
}

/// Progress bar widget for showing completion status
#[derive(Debug, Clone)]
pub struct ProgressBar {
    id: WidgetId,
    value: Signal<f32>,
    max: f32,
    width: f32,
    height: f32,
    indeterminate: bool,
    style: ProgressStyle,
}

/// Styling options for progress bar
#[derive(Debug, Clone)]
pub struct ProgressStyle {
    pub background_color: [f32; 4],
    pub fill_color: [f32; 4],
    pub border_radius: f32,
    pub border_width: f32,
    pub border_color: [f32; 4],
}

impl Default for ProgressStyle {
    fn default() -> Self {
        Self {
            background_color: [0.9, 0.9, 0.9, 1.0], // Light gray
            fill_color: [0.2, 0.6, 1.0, 1.0], // Blue
            border_radius: 4.0,
            border_width: 1.0,
            border_color: [0.8, 0.8, 0.8, 1.0], // Gray
        }
    }
}

impl ProgressBar {
    /// Create a new progress bar
    pub fn new(max: f32) -> Self {
        Self {
            id: generate_id(),
            value: Signal::new(0.0),
            max,
            width: 200.0,
            height: 20.0,
            indeterminate: false,
            style: ProgressStyle::default(),
        }
    }

    /// Set the current value
    pub fn value(mut self, value: f32) -> Self {
        let clamped = value.clamp(0.0, self.max);
        self.value.set(clamped);
        self
    }

    /// Set the progress bar dimensions
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Set indeterminate mode (animated)
    pub fn indeterminate(mut self, indeterminate: bool) -> Self {
        self.indeterminate = indeterminate;
        self
    }

    /// Set custom style
    pub fn style(mut self, style: ProgressStyle) -> Self {
        self.style = style;
        self
    }

    /// Get the value signal
    pub fn value_signal(&self) -> &Signal<f32> {
        &self.value
    }

    /// Get current value
    pub fn get_value(&self) -> f32 {
        self.value.get()
    }

    /// Set the value
    pub fn set_value(&self, value: f32) {
        let clamped = value.clamp(0.0, self.max);
        self.value.set(clamped);
    }

    /// Get progress percentage (0.0 to 1.0)
    pub fn progress(&self) -> f32 {
        if self.max > 0.0 {
            (self.value.get() / self.max).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }
}

impl Default for ProgressBar {
    fn default() -> Self {
        Self::new(100.0)
    }
}

impl Widget for ProgressBar {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn layout(&self, constraints: Constraints, _context: &WidgetContext) -> Size {
        constraints.constrain(Size::new(self.width, self.height))
    }

    fn render(&self, layout: Layout, context: &WidgetContext) -> VNode {
        let progress = self.progress();
        let fill_width = if self.indeterminate {
            self.width * 0.3 // Fixed width for indeterminate animation
        } else {
            self.width * progress
        };

        // Background
        let background = VNode::element("div")
            .attr("class", "progress-background")
            .attr("width", format!("{}px", self.width))
            .attr("height", format!("{}px", self.height))
            .attr("background-color", format!("rgba({}, {}, {}, {})",
                self.style.background_color[0], self.style.background_color[1],
                self.style.background_color[2], self.style.background_color[3]))
            .attr("border-radius", format!("{}px", self.style.border_radius))
            .attr("border", format!("{}px solid rgba({}, {}, {}, {})",
                self.style.border_width,
                self.style.border_color[0], self.style.border_color[1],
                self.style.border_color[2], self.style.border_color[3]))
            .attr("overflow", "hidden");

        // Fill
        let mut fill = VNode::element("div")
            .attr("class", "progress-fill")
            .attr("width", format!("{}px", fill_width))
            .attr("height", "100%")
            .attr("background-color", format!("rgba({}, {}, {}, {})",
                self.style.fill_color[0], self.style.fill_color[1],
                self.style.fill_color[2], self.style.fill_color[3]))
            .attr("border-radius", format!("{}px", self.style.border_radius));

        if self.indeterminate {
            fill = fill.attr("animation", "progress-slide 2s infinite linear");
        }

        VNode::element("div")
            .attr("class", "progress-bar")
            .attr("position", "relative")
            .children(vec![
                background.children(vec![fill])
            ])
    }

    fn handle_event(&mut self, _event: &Event, _layout: Layout, _context: &WidgetContext) -> EventResult {
        EventResult::Ignored // Progress bars don't handle events
    }

    fn state(&self) -> WidgetState {
        WidgetState::Normal
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slider_creation() {
        let slider = Slider::new(0.0, 100.0);
        assert_eq!(slider.get_value(), 0.0);
        assert_eq!(slider.min, 0.0);
        assert_eq!(slider.max, 100.0);
    }

    #[test]
    fn test_slider_value_clamping() {
        let slider = Slider::new(0.0, 100.0);
        
        slider.set_value(150.0);
        assert_eq!(slider.get_value(), 100.0);
        
        slider.set_value(-50.0);
        assert_eq!(slider.get_value(), 0.0);
    }

    #[test]
    fn test_slider_step() {
        let slider = Slider::new(0.0, 100.0).step(10.0);
        
        slider.set_value(23.0);
        assert_eq!(slider.get_value(), 20.0); // Rounded to nearest step
        
        slider.set_value(27.0);
        assert_eq!(slider.get_value(), 30.0);
    }

    #[test]
    fn test_progress_bar_creation() {
        let progress = ProgressBar::new(100.0);
        assert_eq!(progress.get_value(), 0.0);
        assert_eq!(progress.max, 100.0);
        assert_eq!(progress.progress(), 0.0);
    }

    #[test]
    fn test_progress_bar_progress() {
        let progress = ProgressBar::new(100.0);
        
        progress.set_value(50.0);
        assert_eq!(progress.progress(), 0.5);
        
        progress.set_value(100.0);
        assert_eq!(progress.progress(), 1.0);
        
        progress.set_value(150.0); // Should clamp
        assert_eq!(progress.progress(), 1.0);
    }
}