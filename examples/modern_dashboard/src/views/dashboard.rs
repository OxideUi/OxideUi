use strato_widgets::{
    Widget, Column, Row, Text, Flex,
    layout::{CrossAxisAlignment, MainAxisAlignment},
};
use crate::components::{
    stats_card::StatsCard,
    activity_list::ActivityList,
};
use crate::theme::{AppTheme, SPACING_LG, SPACING_MD};

pub struct DashboardView;

impl DashboardView {
    pub fn build() -> Box<dyn Widget> {
        let theme = AppTheme::dark();

        Box::new(Column::new()
            .spacing(SPACING_LG)
            .cross_axis_alignment(CrossAxisAlignment::Stretch)
            .children(vec![
                // Page Title
                Box::new(Text::new("Dashboard Overview")
                    .font_size(32.0)
                    .color(theme.text_primary)) as Box<dyn Widget>,

                // Stats Grid
                Box::new(Row::new()
                    .spacing(SPACING_MD)
                    .children(vec![
                        StatsCard::new("Total Revenue", "$45,231.89", "+20.1%", true).build(),
                        StatsCard::new("Active Users", "2,350", "+15.2%", true).build(),
                        StatsCard::new("Bounce Rate", "42.3%", "-5.4%", true).build(),
                        StatsCard::new("Server Uptime", "99.9%", "+0.1%", true).build(),
                    ])
                ) as Box<dyn Widget>,

                // Main Content Area (Charts & Lists)
                Box::new(Flex::new(
                    Box::new(Row::new()
                        .spacing(SPACING_MD)
                        .cross_axis_alignment(CrossAxisAlignment::Start) // Top align
                        .children(vec![
                            // Left Column (Activity List)
                            Box::new(Flex::new(
                                ActivityList::new().build()
                            ).flex(2.0)) as Box<dyn Widget>,

                            // Right Column (Placeholder for Chart)
                            Box::new(Flex::new(
                                Box::new(strato_widgets::Container::new()
                                    .background(theme.bg_secondary)
                                    .padding(SPACING_MD)
                                    .border_radius(crate::theme::BORDER_RADIUS_MD)
                                    .height(400.0) // Fixed height for chart area
                                    .child(
                                        Text::new("Chart Area (Coming Soon)")
                                            .color(theme.text_secondary)
                                    )
                                )
                            ).flex(3.0)) as Box<dyn Widget>,
                        ])
                    )
                ).flex(1.0)) as Box<dyn Widget>,
            ])
        )
    }
}
