//! StratoUI Widgets - A comprehensive widget library for StratoUI
//!
//! This crate provides a collection of UI widgets built on top of the StratoUI core framework.
//! All widgets are designed to be composable, reactive, and performant.

pub mod animation;
pub mod builder;
pub mod button;
pub mod checkbox;
pub mod container;
pub mod dropdown;
pub mod grid;
pub mod image;
pub mod input;
pub mod inspector;
pub mod layout;
pub mod registry;
pub mod scroll_view;
pub mod slider;
pub mod text;
pub mod theme;
pub mod widget;
pub mod wrap;
pub mod top_bar;

pub mod prelude;
use crate::prelude::*;

// Re-export all widget types for easy access
pub use builder::WidgetBuilder;
pub use button::{Button, ButtonStyle};
pub use checkbox::{Checkbox, CheckboxStyle, RadioButton};
pub use container::{Container, ContainerStyle};
pub use dropdown::{Dropdown, DropdownOption, DropdownStyle};
pub use grid::{Grid, GridUnit};
pub use image::{
    Image, ImageBuilder, ImageData, ImageFilter, ImageFit, ImageFormat, ImageSource, ImageStyle,
};
pub use input::{InputStyle, InputType, TextInput};
pub use inspector::InspectorOverlay;
pub use layout::{Column, Flex, Row, Stack};
pub use scroll_view::ScrollView;
pub use slider::{ProgressBar, Slider, SliderStyle};
pub use strato_macros::view;
pub use text::{Text, TextStyle};
pub use theme::Theme;
pub use widget::{Widget, WidgetContext, WidgetId};
pub use top_bar::TopBar;

/// Initialize the widgets module
pub fn init() -> strato_core::Result<()> {
    tracing::info!("StratoUI Widgets initialized");
    Ok(())
}

/// Create a simple app example
pub fn example_app() -> impl Widget {
    Container::new()
        .padding(20.0)
        .child(Column::new().spacing(10.0).children(vec![
            Box::new(Text::new("Welcome to StratoUI")),
            Box::new(Button::new("Click Me")),
            Box::new(TextInput::new().placeholder("Enter text...")),
        ]))
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
