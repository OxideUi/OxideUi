use strato_widgets::{
    Widget, Column, Container, Text, Button, Flex,
    text::FontWeight,
    layout::{CrossAxisAlignment, MainAxisAlignment},
};
use strato_core::{
    types::Color,
    state::Signal,
};
use crate::theme::{AppTheme, BORDER_RADIUS_MD, SPACING_MD, SPACING_SM};

pub struct Sidebar {
    active_tab: Signal<String>,
    theme: AppTheme,
}

impl Sidebar {
    pub fn new(active_tab: Signal<String>) -> Self {
        Self {
            active_tab,
            theme: AppTheme::dark(),
        }
    }

    pub fn build(self) -> Box<dyn Widget> {
        let theme = self.theme;
        let active_tab = self.active_tab.clone();

        Box::new(Container::new()
            .background(theme.bg_secondary)
            .width(260.0)
            .padding(SPACING_MD)
            .child(
                Column::new()
                    .cross_axis_alignment(CrossAxisAlignment::Stretch)
                    .spacing(SPACING_MD)
                    .children(vec![
                        // Logo Area
                        Box::new(Container::new()
                            .padding(SPACING_SM)
                            .child(
                                Text::new("StratoUI")
                                    .font_size(24.0)
                                    .font_weight(FontWeight::Bold)
                                    .color(theme.accent)
                            )
                        ) as Box<dyn Widget>,

                        // Navigation Items
                        self.nav_item("Dashboard", "dashboard", active_tab.clone(), theme.clone()),
                        self.nav_item("Analytics", "analytics", active_tab.clone(), theme.clone()),
                        self.nav_item("Users", "users", active_tab.clone(), theme.clone()),
                        self.nav_item("Settings", "settings", active_tab.clone(), theme.clone()),
                        
                        // Spacer
                        Box::new(Flex::new(Box::new(Container::new())).flex(1.0)) as Box<dyn Widget>,
                        
                        // Bottom Area
                        Box::new(Container::new()
                            .background(theme.bg_tertiary)
                            .border_radius(BORDER_RADIUS_MD)
                            .padding(SPACING_MD)
                            .child(
                                Column::new()
                                    .spacing(SPACING_SM)
                                    .children(vec![
                                        Box::new(Text::new("Pro Plan").font_weight(FontWeight::Bold).color(theme.text_primary)) as Box<dyn Widget>,
                                        Box::new(Text::new("Expires in 12 days").font_size(12.0).color(theme.text_secondary)) as Box<dyn Widget>,
                                        Box::new(Button::new("Upgrade").primary()) as Box<dyn Widget>,
                                    ])
                            )
                        ) as Box<dyn Widget>,
                    ])
            )
        )
    }

    fn nav_item(&self, label: &str, id: &str, active_signal: Signal<String>, _theme: AppTheme) -> Box<dyn Widget> {
        // Note: Real interactivity would check active_signal value to change style
        // For now, we simulate basic structure
        let id_owned = id.to_string();
        
        Box::new(Button::new(label)
            .secondary() // Use secondary style by default
            .on_click(move || {
                active_signal.set(id_owned.clone());
            })
        )
    }
}
