//! StratoUI Widgets - A comprehensive widget library for StratoUI
//! 
//! This crate provides a collection of UI widgets built on top of the StratoUI core framework.
//! All widgets are designed to be composable, reactive, and performant.

pub mod widget;
pub mod button;
pub mod text;
pub mod container;
pub mod input;
pub mod layout;
pub mod theme;
pub mod wrap;
pub mod grid;
pub mod animation;
pub mod builder;
pub mod checkbox;
pub mod slider;
pub mod dropdown;
pub mod image;
pub mod scroll_view;

pub mod prelude;
use crate::prelude::*;

// Re-export all widget types for easy access
pub use widget::{Widget, WidgetContext, WidgetId};
pub use button::{Button, ButtonStyle};
pub use text::{Text, TextStyle};
pub use container::{Container, ContainerStyle};
pub use input::{TextInput, InputStyle, InputType};
pub use layout::{Row, Column, Stack, Flex};
pub use theme::{Theme};
pub use builder::WidgetBuilder;
pub use checkbox::{Checkbox, RadioButton, CheckboxStyle};
pub use slider::{Slider, ProgressBar, SliderStyle};
pub use dropdown::{Dropdown, DropdownOption, DropdownStyle};
pub use image::{Image, ImageBuilder, ImageFit, ImageSource, ImageData, ImageFormat, ImageFilter, ImageStyle};
pub use scroll_view::ScrollView;
pub use grid::{Grid, GridUnit};


/// Initialize the widgets module
pub fn init() -> strato_core::Result<()> {
    tracing::info!("StratoUI Widgets initialized");
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
                    Box::new(Text::new("Welcome to StratoUI")),
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
