//! Control gallery demonstrating integrated interaction states and accessibility semantics.

use strato_sdk::prelude::*;
use strato_sdk::strato_widgets::{Checkbox, Slider};
use strato_sdk::InitBuilder;

fn main() {
    InitBuilder::new()
        .init_all()
        .expect("Failed to initialize StratoSDK");

    ApplicationBuilder::new()
        .title("Control gallery")
        .window(WindowBuilder::new().with_size(640.0, 420.0))
        .run(build_ui());
}

fn stateful_buttons() -> impl Widget {
    let mut focus_preview = Button::new("Focused").style(ButtonStyle::outline());
    focus_preview.set_state(WidgetState::Focused);

    let mut pressed_preview = Button::new("Pressed").style(ButtonStyle::ghost());
    pressed_preview.set_state(WidgetState::Pressed);

    Column::new().spacing(12.0).children(vec![
        Box::new(
            Button::new("Primary")
                .primary()
                .accessibility_hint("Activates the primary action"),
        ),
        Box::new(
            Button::new("Disabled")
                .enabled(false)
                .accessibility_hint("Disabled to show the dimmed state"),
        ),
        Box::new(focus_preview),
        Box::new(pressed_preview),
    ])
}

fn toggles() -> impl Widget {
    let checked = Checkbox::new().label("Notifications").checked(true);
    let disabled = Checkbox::new().label("Location access").enabled(false);

    Column::new()
        .spacing(8.0)
        .children(vec![Box::new(checked), Box::new(disabled)])
}

fn sliders() -> impl Widget {
    let mut disabled_slider = Slider::new(0.0, 100.0).enabled(false);
    disabled_slider.set_value(45.0);

    Column::new().spacing(10.0).children(vec![
        Box::new(Slider::new(0.0, 100.0).size(260.0, 32.0)),
        Box::new(disabled_slider),
    ])
}

fn build_ui() -> impl Widget {
    Container::new()
        .padding(24.0)
        .child(Row::new().spacing(32.0).children(vec![
            Box::new(stateful_buttons()),
            Box::new(toggles()),
            Box::new(sliders()),
        ]))
}
