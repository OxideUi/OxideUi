use glam::Vec2;
use std::collections::BTreeMap;
use strato_core::layout::{Constraints, Layout};
use strato_renderer::{batch::DrawCommand, batch::RenderBatch};
use strato_widgets::button::ButtonState;
use strato_widgets::{Button, Slider, Theme, Widget};

fn summarize_batch(batch: &RenderBatch) -> Vec<String> {
    batch
        .commands
        .iter()
        .map(|cmd| match cmd {
            DrawCommand::Rect { rect, color, .. } => format!(
                "rect {:.1},{:.1} {:.1}x{:.1} rgba({:.2},{:.2},{:.2},{:.2})",
                rect.x, rect.y, rect.width, rect.height, color.r, color.g, color.b, color.a
            ),
            DrawCommand::Text {
                text,
                position,
                color,
                font_size,
                ..
            } => format!(
                "text '{}' @({:.1},{:.1}) size {:.1} rgba({:.2},{:.2},{:.2},{:.2})",
                text, position.0, position.1, font_size, color.r, color.g, color.b, color.a
            ),
            DrawCommand::Circle {
                center,
                radius,
                color,
                ..
            } => format!(
                "circle ({:.1},{:.1}) r{:.1} rgba({:.2},{:.2},{:.2},{:.2})",
                center.0, center.1, radius, color.r, color.g, color.b, color.a
            ),
            other => format!("{:?}", other),
        })
        .collect()
}

fn render_button_with_state(state: ButtonState) -> Vec<String> {
    let theme = Theme::default();
    let mut button = Button::new(format!("{:?}", state)).size(140.0, 44.0);
    button.set_state(state);

    let mut batch = RenderBatch::new();
    let size = <Button as Widget>::layout(&mut button, Constraints::tight(140.0, 44.0));
    let layout = Layout::new(Vec2::ZERO, size);
    let ctx = strato_widgets::widget::WidgetContext {
        theme: &theme,
        state,
        is_focused: matches!(state, ButtonState::Focused),
        is_hovered: matches!(state, ButtonState::Hovered),
        delta_time: 1.0,
    };
    button.update(&ctx);
    button.render(&mut batch, layout);

    summarize_batch(&batch)
}

fn render_slider_variant(disabled: bool) -> Vec<String> {
    let mut slider = Slider::new(0.0, 100.0).size(200.0, 32.0).enabled(!disabled);
    slider.set_value(50.0);

    let mut batch = RenderBatch::new();
    let size = <Slider as Widget>::layout(&mut slider, Constraints::tight(220.0, 48.0));
    let layout = Layout::new(Vec2::ZERO, size);
    let theme = Theme::default();
    let ctx = strato_widgets::widget::WidgetContext {
        theme: &theme,
        state: if disabled {
            ButtonState::Disabled
        } else {
            ButtonState::Normal
        },
        is_focused: false,
        is_hovered: false,
        delta_time: 1.0,
    };
    slider.update(&ctx);
    slider.render(&mut batch, layout);

    summarize_batch(&batch)
}

fn format_snapshot(map: &BTreeMap<String, Vec<String>>) -> String {
    let mut lines = Vec::new();
    for (key, entries) in map {
        lines.push(format!("{}:", key));
        for entry in entries {
            lines.push(format!("  - {}", entry));
        }
    }
    lines.join("\n")
}

const EXPECTED_BUTTON_SNAPSHOT: &str = include_str!("snapshots/button_state_commands.snap");
const EXPECTED_SLIDER_SNAPSHOT: &str = include_str!("snapshots/slider_commands.snap");

#[test]
fn button_state_snapshots() {
    let mut snapshots = BTreeMap::new();
    for state in [
        ButtonState::Normal,
        ButtonState::Hovered,
        ButtonState::Pressed,
        ButtonState::Focused,
        ButtonState::Disabled,
    ] {
        snapshots.insert(format!("{:?}", state), render_button_with_state(state));
    }
    let rendered = format_snapshot(&snapshots);
    assert_eq!(rendered.trim(), EXPECTED_BUTTON_SNAPSHOT.trim());
}

#[test]
fn slider_snapshots() {
    let mut snapshots = BTreeMap::new();
    snapshots.insert("enabled".to_string(), render_slider_variant(false));
    snapshots.insert("disabled".to_string(), render_slider_variant(true));
    let rendered = format_snapshot(&snapshots);
    assert_eq!(rendered.trim(), EXPECTED_SLIDER_SNAPSHOT.trim());
}
