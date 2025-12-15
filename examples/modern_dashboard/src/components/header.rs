use oxide_widgets::{
    Widget, Row, Container, Text, Button, InputType, TextInput,
    text::FontWeight,
    layout::{CrossAxisAlignment, MainAxisAlignment},
};
use crate::theme::{AppTheme, BORDER_RADIUS_MD, SPACING_MD, SPACING_SM};

pub struct Header {
    theme: AppTheme,
}

impl Header {
    pub fn new() -> Self {
        Self {
            theme: AppTheme::dark(),
        }
    }

    pub fn build(self) -> Box<dyn Widget> {
        let theme = self.theme;

        Box::new(Container::new()
            .background(theme.bg_secondary)
            .padding(SPACING_MD)
            .child(
                Row::new()
                    .main_axis_alignment(MainAxisAlignment::SpaceBetween)
                    .cross_axis_alignment(CrossAxisAlignment::Center)
                    .children(vec![
                        // Search Bar
                        Box::new(Container::new()
                            .width(300.0)
                            .child(
                                TextInput::new()
                                    .placeholder("Search anything...")
                            )
                        ) as Box<dyn Widget>,

                        // Right Actions
                        Box::new(Row::new()
                            .spacing(SPACING_SM)
                            .children(vec![
                                Box::new(Button::new("Notifications").outline()) as Box<dyn Widget>,
                                Box::new(Container::new()
                                    .width(40.0)
                                    .height(40.0)
                                    .background(theme.accent)
                                    .border_radius(20.0) // Circle
                                    .child(
                                        Text::new("JD")
                                            .color(theme.text_primary)
                                            .font_weight(FontWeight::Bold)
                                            .align(oxide_widgets::text::TextAlign::Center)
                                    )
                                ) as Box<dyn Widget>,
                            ])
                        ) as Box<dyn Widget>,
                    ])
            )
        )
    }
}
