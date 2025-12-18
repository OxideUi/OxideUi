//! Common control interaction and accessibility primitives.
//!
//! This module provides a lightweight state machine that widgets can embed to
//! unify pressed/hover/disabled handling and animate visual responses. It also
//! carries accessibility semantics (role, label, hint) so controls can expose
//! intent to higher level tooling.

use crate::animation::Tween;
use crate::widget::WidgetState;
use strato_core::event::{Event, EventResult, KeyCode, MouseButton};
use strato_core::state::Signal;
use strato_core::types::{Point, Rect};

/// The ARIA-like role associated with a control.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlRole {
    Button,
    Checkbox,
    Radio,
    Slider,
    Input,
    Toggle,
}

impl Default for ControlRole {
    fn default() -> Self {
        ControlRole::Button
    }
}

/// Accessibility semantics for a control.
#[derive(Debug, Clone, Default)]
pub struct ControlSemantics {
    pub role: ControlRole,
    pub label: Option<String>,
    pub hint: Option<String>,
    pub value: Option<String>,
    pub toggled: Option<bool>,
}

impl ControlSemantics {
    pub fn new(role: ControlRole) -> Self {
        Self {
            role,
            ..Default::default()
        }
    }
}

/// Shared interaction state for focusable/pressable controls.
#[derive(Debug, Clone)]
pub struct ControlState {
    state: Signal<WidgetState>,
    interaction_progress: Signal<f32>,
    focus_progress: Signal<f32>,
    semantics: ControlSemantics,
}

impl ControlState {
    /// Create a new control state for the given role.
    pub fn new(role: ControlRole) -> Self {
        Self {
            state: Signal::new(WidgetState::Normal),
            interaction_progress: Signal::new(0.0),
            focus_progress: Signal::new(0.0),
            semantics: ControlSemantics::new(role),
        }
    }

    /// Current widget state.
    pub fn state(&self) -> WidgetState {
        self.state.get()
    }

    /// Set the widget state directly (used by tests/demo code).
    pub fn set_state(&self, state: WidgetState) {
        self.state.set(state);
    }

    /// Mark the control as disabled/enabled.
    pub fn set_disabled(&self, disabled: bool) {
        if disabled {
            self.state.set(WidgetState::Disabled);
        } else if self.state.get() == WidgetState::Disabled {
            self.state.set(WidgetState::Normal);
        }
    }

    /// Update the semantics label.
    pub fn set_label(&mut self, label: impl Into<String>) {
        self.semantics.label = Some(label.into());
    }

    /// Update the semantics hint/description.
    pub fn set_hint(&mut self, hint: impl Into<String>) {
        self.semantics.hint = Some(hint.into());
    }

    /// Update the semantics value (e.g., slider percentage).
    pub fn set_value(&mut self, value: impl Into<String>) {
        self.semantics.value = Some(value.into());
    }

    /// Mark whether the control is toggled/checked.
    pub fn set_toggled(&mut self, toggled: bool) {
        self.semantics.toggled = Some(toggled);
    }

    /// Access semantics for inspectors or higher-level layers.
    pub fn semantics(&self) -> &ControlSemantics {
        &self.semantics
    }

    /// Smooth interaction animation state toward the target.
    pub fn update(&self, delta_time: f32) {
        let target_interaction = match self.state.get() {
            WidgetState::Pressed => 1.0,
            WidgetState::Hovered => 0.65,
            WidgetState::Focused => 0.4,
            WidgetState::Disabled => 0.0,
            WidgetState::Normal => 0.0,
        };
        let target_focus = if matches!(self.state.get(), WidgetState::Focused) {
            1.0
        } else {
            0.0
        };

        let smooth = |current: f32, target: f32| {
            let step = (target - current) * (delta_time * 8.0).clamp(0.0, 1.0);
            (current + step).clamp(0.0, 1.0)
        };

        self.interaction_progress
            .set(smooth(self.interaction_progress.get(), target_interaction));
        self.focus_progress
            .set(smooth(self.focus_progress.get(), target_focus));
    }

    /// Overall interaction factor used for color/opacity blending.
    pub fn interaction_factor(&self) -> f32 {
        Tween::new(0.0, 1.0).transform(
            self.interaction_progress
                .get()
                .max(self.focus_progress.get()),
        )
    }

    /// Update hover state when the cursor enters/leaves the control bounds.
    pub fn hover(&self, within: bool) {
        if self.state.get() == WidgetState::Disabled {
            return;
        }
        match (within, self.state.get()) {
            (true, WidgetState::Normal) => self.state.set(WidgetState::Hovered),
            (false, WidgetState::Hovered) => self.state.set(WidgetState::Normal),
            (false, WidgetState::Pressed) => self.state.set(WidgetState::Normal),
            _ => {}
        }
    }

    /// Start a press interaction if the point is inside.
    pub fn press(&self, point: Point, bounds: Rect) -> bool {
        if self.state.get() == WidgetState::Disabled {
            return false;
        }
        if bounds.contains(point) {
            self.state.set(WidgetState::Pressed);
            return true;
        }
        false
    }

    /// Finish a press interaction and report whether activation should occur.
    pub fn release(&self, point: Point, bounds: Rect) -> bool {
        if self.state.get() == WidgetState::Disabled {
            return false;
        }
        if self.state.get() == WidgetState::Pressed {
            let is_hovered = bounds.contains(point);
            self.state.set(if is_hovered {
                WidgetState::Hovered
            } else {
                WidgetState::Normal
            });
            return is_hovered;
        }
        false
    }

    /// Update focus state based on keyboard navigation.
    pub fn focus(&self) {
        if self.state.get() != WidgetState::Disabled {
            self.state.set(WidgetState::Focused);
        }
    }

    /// Handle blur.
    pub fn blur(&self) {
        if self.state.get() == WidgetState::Focused {
            self.state.set(WidgetState::Normal);
        }
    }

    /// Keyboard activations for accessible triggers.
    pub fn handle_keyboard_activation(&self, event: &Event) -> EventResult {
        match event {
            Event::KeyDown(key) if matches!(key.key_code, KeyCode::Enter | KeyCode::Space) => {
                if self.state.get() != WidgetState::Disabled {
                    self.state.set(WidgetState::Pressed);
                    return EventResult::Handled;
                }
            }
            Event::KeyUp(key) if matches!(key.key_code, KeyCode::Enter | KeyCode::Space) => {
                if self.state.get() == WidgetState::Pressed {
                    self.state.set(WidgetState::Focused);
                    return EventResult::Handled;
                }
            }
            _ => {}
        }
        EventResult::Ignored
    }

    /// Pointer-based interaction dispatcher.
    pub fn handle_pointer_event(&self, event: &Event, bounds: Rect) -> EventResult {
        match event {
            Event::MouseMove(mouse) => {
                let point = Point::new(mouse.position.x, mouse.position.y);
                self.hover(bounds.contains(point));
                EventResult::Ignored
            }
            Event::MouseDown(mouse) => {
                if let Some(MouseButton::Left) = mouse.button {
                    let point = Point::new(mouse.position.x, mouse.position.y);
                    if self.press(point, bounds) {
                        return EventResult::Handled;
                    }
                }
                EventResult::Ignored
            }
            Event::MouseUp(mouse) => {
                if let Some(MouseButton::Left) = mouse.button {
                    let point = Point::new(mouse.position.x, mouse.position.y);
                    if self.release(point, bounds) {
                        return EventResult::Handled;
                    }
                }
                EventResult::Ignored
            }
            _ => EventResult::Ignored,
        }
    }
}
