use strato_widgets::{
    Widget, Column, Row, Text, Flex, Grid, GridUnit,
    layout::{CrossAxisAlignment, MainAxisAlignment},
};
use crate::components::{
    stats_card::StatsCard,
    activity_list::ActivityList,
    animated_chart::AnimatedChart,
};
use crate::theme::{AppTheme, SPACING_LG, SPACING_MD};
use strato_core::types::Color;

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

                // Stats Grid (New Layout System)
                Box::new(Grid::new()
                    .rows(vec![GridUnit::Auto])
                    .columns(vec![
                        GridUnit::Fraction(1.0),
                        GridUnit::Fraction(1.0),
                        GridUnit::Fraction(1.0),
                        GridUnit::Fraction(1.0),
                    ])
                    .row_gap(SPACING_MD)
                    .col_gap(SPACING_MD)
                    .children(vec![
                        Box::new(StatsCard::new("Total Revenue", "$45,231.89", "+20.1%", true).build()),
                        Box::new(StatsCard::new("Active Users", "2,350", "+15.2%", true).build()),
                        Box::new(StatsCard::new("Bounce Rate", "42.3%", "-5.4%", true).build()),
                        Box::new(StatsCard::new("Server Uptime", "99.9%", "+0.1%", true).build()),
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
                                Box::new(ActivityList::new().build()) // Box required for Flex::new
                            ).flex(2.0)) as Box<dyn Widget>,

                            // Right Column (Animated Chart)
                            Box::new(Flex::new(
                                Box::new(strato_widgets::Container::new()
                                    .background(theme.bg_secondary)
                                    .padding(SPACING_MD)
                                    .border_radius(crate::theme::BORDER_RADIUS_MD)
                                    .height(400.0) // Fixed height for chart area
                                    .child(
                                        Column::new()
                                            .spacing(SPACING_MD)
                                            .children(vec![
                                                Box::new(Text::new("Growth Analytics")
                                                    .font_size(20.0)
                                                    .color(theme.text_primary)) as Box<dyn Widget>,

                                                Box::new(AnimatedChart::new(vec![
                                                    65.0, 40.0, 100.0, 85.0, 45.0, 92.0,
                                                    55.0, 70.0, 30.0, 60.0, 95.0, 80.0
                                                ]).color(Color::rgb(0.4, 0.7, 1.0))) as Box<dyn Widget>
                                            ])
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
