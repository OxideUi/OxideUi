//! OxideUI Widgets - A comprehensive widget library for OxideUI
//! 
//! This crate provides a collection of UI widgets built on top of the OxideUI core framework.
//! All widgets are designed to be composable, reactive, and performant.

pub mod widget;
pub mod button;
pub mod text;
pub mod container;
pub mod input;
pub mod layout;
pub mod theme;
pub mod builder;
pub mod checkbox;
pub mod slider;
pub mod dropdown;
pub mod image;

pub mod prelude;

// Re-export all widget types for easy access
pub use widget::{Widget, WidgetContext, WidgetId};
pub use button::{Button, ButtonStyle, ButtonVariant};
pub use text::{Text, TextStyle};
pub use container::{Container, ContainerStyle};
pub use input::{Input, InputStyle, InputType};
pub use layout::{Layout, LayoutDirection, LayoutWrap};
pub use theme::{Theme, ThemeColor, ThemeSize};
pub use builder::WidgetBuilder;
pub use checkbox::{Checkbox, RadioButton, CheckboxStyle};
pub use slider::{Slider, ProgressBar, SliderStyle};
pub use dropdown::{Dropdown, DropdownOption, DropdownStyle};
pub use image::{Image, ImageBuilder, ImageFit, ImageSource, ImageData, ImageFormat, ImageFilter, ImageStyle};

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
