use oxide_widgets::{
    Widget, Column, Container, Text, Flex,
    text::FontWeight,
};
use oxide_core::types::Color;
use crate::theme::{AppTheme, BORDER_RADIUS_MD, SPACING_MD, SPACING_SM};

pub struct StatsCard {
    title: String,
    value: String,
    trend: String,
    is_positive: bool,
    theme: AppTheme,
}

impl StatsCard {
    pub fn new(title: &str, value: &str, trend: &str, is_positive: bool) -> Self {
        Self {
            title: title.to_string(),
            value: value.to_string(),
            trend: trend.to_string(),
            is_positive,
            theme: AppTheme::dark(),
        }
    }

    pub fn build(self) -> Box<dyn Widget> {
        let theme = self.theme;
        let trend_color = if self.is_positive { theme.success } else { theme.error };

        // Wrap in Flex(1.0) so it expands in a Row
        Box::new(Flex::new(
            Box::new(Container::new()
                .background(theme.bg_secondary)
                .border_radius(BORDER_RADIUS_MD)
                .padding(SPACING_MD)
                .child(
                    Column::new()
                        .spacing(SPACING_SM)
                        .children(vec![
                            Box::new(Text::new(&self.title)
                                .color(theme.text_secondary)
                                .font_size(14.0)) as Box<dyn Widget>,
                            
                            Box::new(Text::new(&self.value)
                                .color(theme.text_primary)
                                .font_size(28.0)
                                .font_weight(FontWeight::Bold)) as Box<dyn Widget>,
                            
                            Box::new(Text::new(&self.trend)
                                .color(trend_color)
                                .font_size(12.0)) as Box<dyn Widget>,
                        ])
                )
            )
        ).flex(1.0))
    }
}
