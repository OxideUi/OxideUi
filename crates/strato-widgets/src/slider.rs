//! Slider and Progress widgets implementation for StratoUI

use std::any::Any;
use crate::widget::{Widget, WidgetId, generate_id};
use strato_core::{
    event::{Event, EventResult, MouseButton},
    layout::{Size, Constraints, Layout},
    state::Signal,
    types::{Point, Rect, Color, Transform},
};
use strato_renderer::batch::RenderBatch;

/// Slider widget for numeric value selection
#[derive(Debug, Clone)]
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
    bounds: Signal<Rect>,
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
            bounds: Signal::new(Rect::new(0.0, 0.0, 0.0, 0.0)),
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

    /// Handle mouse events using stored bounds
    fn handle_mouse_event(&self, event: &Event) -> EventResult {
        if !self.enabled {
            return EventResult::Ignored;
        }

        let bounds = self.bounds.get();
        let track_width = (bounds.width - self.style.thumb_size).max(0.0);
        let track_start_x = bounds.x + self.style.thumb_size * 0.5;

        match event {
            Event::MouseDown(mouse_event) => {
                let point = Point::new(mouse_event.position.x, mouse_event.position.y);
                if !bounds.contains(point) {
                    return EventResult::Ignored;
                }
                
                if let Some(MouseButton::Left) = mouse_event.button {
                    let local_x = mouse_event.position.x - track_start_x;
                    let new_value = self.value_from_position(local_x, track_width);
                    self.set_value(new_value);
                    self.dragging.set(true);
                    EventResult::Handled
                } else {
                    EventResult::Ignored
                }
            }
            Event::MouseMove(mouse_event) if self.dragging.get() => {
                let local_x = mouse_event.position.x - track_start_x;
                let new_value = self.value_from_position(local_x, track_width);
                self.set_value(new_value);
                EventResult::Handled
            }
            Event::MouseUp(mouse_event) => {
                if let Some(MouseButton::Left) = mouse_event.button {
                    self.dragging.set(false);
                    EventResult::Handled
                } else {
                    EventResult::Ignored
                }
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

    fn layout(&mut self, constraints: Constraints) -> Size {
        constraints.constrain(Size::new(self.width, self.height))
    }

    fn render(&self, batch: &mut RenderBatch, layout: Layout) {
        let bounds = Rect::new(
            layout.position.x,
            layout.position.y,
            layout.size.width,
            layout.size.height,
        );
        self.bounds.set(bounds);

        if !self.enabled {
            let disabled_color = Color::rgba(
                self.style.disabled_color[0],
                self.style.disabled_color[1],
                self.style.disabled_color[2],
                self.style.disabled_color[3],
            );
            batch.add_rect(bounds, disabled_color, Transform::identity());
            return;
        }

        let track_width = (bounds.width - self.style.thumb_size).max(0.0);
        let track_x = bounds.x + self.style.thumb_size * 0.5;
        let track_y = bounds.y + (bounds.height - self.style.track_height) * 0.5;

        let track_rect = Rect::new(
            track_x,
            track_y,
            track_width,
            self.style.track_height,
        );

        let track_color = Color::rgba(
            self.style.track_color[0],
            self.style.track_color[1],
            self.style.track_color[2],
            self.style.track_color[3],
        );

        batch.add_rect(track_rect, track_color, Transform::identity());

        let thumb_offset = self.thumb_position(track_width);
        let fill_rect = Rect::new(
            track_x,
            track_y,
            thumb_offset.clamp(0.0, track_width),
            self.style.track_height,
        );

        let fill_color = Color::rgba(
            self.style.track_fill_color[0],
            self.style.track_fill_color[1],
            self.style.track_fill_color[2],
            self.style.track_fill_color[3],
        );

        batch.add_rect(fill_rect, fill_color, Transform::identity());

        let thumb_center_x = track_x + thumb_offset;
        let thumb_center_y = bounds.y + bounds.height * 0.5;
        let thumb_radius = self.style.thumb_size * 0.5;

        let thumb_color = Color::rgba(
            self.style.thumb_color[0],
            self.style.thumb_color[1],
            self.style.thumb_color[2],
            self.style.thumb_color[3],
        );

        batch.add_circle(
            (thumb_center_x, thumb_center_y),
            thumb_radius,
            thumb_color, // Use state-aware color
            16,
            strato_core::types::Transform::default(),
        );
    }

    fn handle_event(&mut self, event: &Event) -> EventResult {
        match event {
            Event::MouseDown(_) | Event::MouseUp(_) | Event::MouseMove(_) => {
                self.handle_mouse_event(event)
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

    // Removed state method as it's not part of Widget trait
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

    fn layout(&mut self, constraints: Constraints) -> Size {
        constraints.constrain(Size::new(self.width, self.height))
    }

    fn render(&self, batch: &mut RenderBatch, layout: Layout) {
        let bounds = Rect::new(
            layout.position.x,
            layout.position.y,
            layout.size.width,
            layout.size.height,
        );

        let bg_color = Color::rgba(
            self.style.background_color[0],
            self.style.background_color[1],
            self.style.background_color[2],
            self.style.background_color[3],
        );

        batch.add_rect(bounds, bg_color, Transform::identity());

        let progress = self.progress();
        if progress > 0.0 {
            let fill_width = bounds.width * progress;
            let fill_rect = Rect::new(bounds.x, bounds.y, fill_width, bounds.height);

            let fill_color = Color::rgba(
                self.style.fill_color[0],
                self.style.fill_color[1],
                self.style.fill_color[2],
                self.style.fill_color[3],
            );

            batch.add_rect(fill_rect, fill_color, Transform::identity());
        }
    }

    fn handle_event(&mut self, _event: &Event) -> EventResult {
        EventResult::Ignored // Progress bars don't handle events
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
