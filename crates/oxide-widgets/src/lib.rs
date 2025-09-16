//! Widget library for OxideUI framework
//!
//! Provides a collection of reusable UI components with a declarative API.

pub mod widget;
pub mod button;
pub mod text;
pub mod container;
pub mod input;
pub mod layout;
pub mod theme;
pub mod builder;
pub mod prelude;

pub use widget::{Widget, WidgetId, WidgetState, WidgetContext};
pub use button::{Button, ButtonStyle};
pub use text::Text;
pub use container::{Container, ContainerStyle};
pub use input::{TextInput, TextInputStyle};
pub use layout::{Row, Column, Stack, Flex};
pub use theme::{Theme, ThemeProvider};
pub use builder::WidgetBuilder;

/// Initialize the widgets module
pub fn init() -> oxide_core::Result<()> {
    tracing::info!("OxideUI Widgets initialized");
    Ok(())
}

/// Create a simple app example
pub fn example_app() -> impl Widget {
    Container::new()
        .padding(20.0)
        .child(
            Column::new()
                .spacing(10.0)
                .children(vec![
                    Box::new(Text::new("Welcome to OxideUI")),
                    Box::new(Button::new("Click Me")),
                    Box::new(TextInput::new().placeholder("Enter text...")),
                ])
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_widget_creation() {
        let button = Button::new("Test");
        assert_eq!(button.text(), "Test");
    }
}
