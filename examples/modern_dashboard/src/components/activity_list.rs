use strato_widgets::{
    Widget, Column, Container, Text, Row, Flex,
    text::FontWeight,
    layout::{CrossAxisAlignment, MainAxisAlignment},
};
use crate::theme::{AppTheme, BORDER_RADIUS_MD, SPACING_MD, SPACING_SM};

pub struct ActivityList {
    theme: AppTheme,
}

impl ActivityList {
    pub fn new() -> Self {
        Self {
            theme: AppTheme::dark(),
        }
    }

    pub fn build(self) -> impl Widget {
        let theme = self.theme;

        Container::new()
            .background(theme.bg_secondary)
            .border_radius(BORDER_RADIUS_MD)
            .padding(SPACING_MD)
            .child(
                Column::new()
                    .spacing(SPACING_MD)
                    .cross_axis_alignment(CrossAxisAlignment::Stretch)
                    .children(vec![
                        Box::new(Text::new("Recent Activity")
                            .font_size(18.0)
                            .font_weight(FontWeight::Bold)
                            .color(theme.text_primary)) as Box<dyn Widget>,

                        self.activity_item("New user registered", "2 min ago", theme.success),
                        self.activity_item("Server rebooted", "15 min ago", theme.warning),
                        self.activity_item("Database backup completed", "1 hour ago", theme.accent),
                        self.activity_item("Payment failed", "2 hours ago", theme.error),
                    ])
            )
    }

    fn activity_item(&self, title: &str, time: &str, dot_color: strato_core::types::Color) -> Box<dyn Widget> {
        let theme = &self.theme;

        Box::new(Container::new()
            .padding(SPACING_SM)
            .child(
                Row::new()
                    .spacing(SPACING_MD)
                    .cross_axis_alignment(CrossAxisAlignment::Center)
                    .children(vec![
                        // Status Dot
                        Box::new(Container::new()
                            .width(10.0)
                            .height(10.0)
                            .border_radius(5.0)
                            .background(dot_color)) as Box<dyn Widget>,

                        // Content
                        Box::new(Flex::new(
                            Box::new(Column::new()
                                .spacing(4.0)
                                .children(vec![
                                    Box::new(Text::new(title)
                                        .color(theme.text_primary)
                                        .font_weight(FontWeight::SemiBold)) as Box<dyn Widget>,
                                    Box::new(Text::new(time)
                                        .color(theme.text_secondary)
                                        .font_size(12.0)) as Box<dyn Widget>,
                                ])
                            )
                        ).flex(1.0)) as Box<dyn Widget>,
                    ])
            )
        )
    }
}
