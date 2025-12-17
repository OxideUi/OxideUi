use std::collections::HashMap;
use strato_widgets::{
    Widget, WidgetId, Row, Column, Container, Flex,
};
use strato_core::layout::{Constraints, Layout, Size};
use strato_renderer::batch::RenderBatch;
use strato_core::state::Signal;
use crate::theme::AppTheme;
use crate::components::{sidebar::Sidebar, header::Header};
use crate::views::{
    dashboard::DashboardView,
    analytics::AnalyticsView,
    users::UsersView,
    settings::SettingsView,
};

pub struct ModernDashboardApp {
    root: Box<dyn Widget>,
}

impl ModernDashboardApp {
    pub fn new() -> Self {
        let active_tab = Signal::new("dashboard".to_string());
        let theme = AppTheme::dark();

        let sidebar = Sidebar::new(active_tab.clone()).build();
        let header = Header::new().build();

        let view_switcher = ViewSwitcher::new(active_tab);

        let root = Container::new()
            .background(theme.bg_primary)
            .child(
                Row::new()
                    .children(vec![
                        // Sidebar
                        sidebar,

                        // Main Content Area
                        Box::new(Flex::new(
                            Box::new(Column::new()
                                .children(vec![
                                    header,
                                    Box::new(Flex::new(
                                        Box::new(Container::new()
                                            .padding(crate::theme::SPACING_LG)
                                            .child(view_switcher)
                                        )
                                    ).flex(1.0)) as Box<dyn Widget>,
                                ])
                            )
                        ).flex(1.0)) as Box<dyn Widget>,
                    ])
            );

        Self {
            root: Box::new(root),
        }
    }

    pub fn build(self) -> Box<dyn Widget> {
        self.root
    }
}

// Custom Widget to handle View Switching
#[derive(Debug)]
struct ViewSwitcher {
    id: WidgetId,
    active_tab: Signal<String>,
    // We store factories or instances? Instances are easier but need to be careful about state.
    // For this example, we just store instances.
    views: HashMap<String, Box<dyn Widget>>,
    last_tab: String,
}

impl ViewSwitcher {
    fn new(active_tab: Signal<String>) -> Self {
        let mut views = HashMap::new();
        views.insert("dashboard".to_string(), DashboardView::build());
        views.insert("analytics".to_string(), AnalyticsView::build());
        views.insert("users".to_string(), UsersView::build());
        views.insert("settings".to_string(), SettingsView::build());

        Self {
            id: strato_widgets::widget::generate_id(),
            active_tab,
            views,
            last_tab: "dashboard".to_string(),
        }
    }
}

impl Widget for ViewSwitcher {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn layout(&mut self, constraints: Constraints) -> Size {
        let current_tab = self.active_tab.get();
        if let Some(view) = self.views.get_mut(&current_tab) {
            view.layout(constraints)
        } else {
            Size::zero()
        }
    }

    fn render(&self, batch: &mut RenderBatch, layout: Layout) {
        let current_tab = self.active_tab.get();
        if let Some(view) = self.views.get(&current_tab) {
            view.render(batch, layout);
        }
    }

    fn handle_event(&mut self, event: &strato_core::event::Event) -> strato_core::event::EventResult {
        let current_tab = self.active_tab.get();
        if let Some(view) = self.views.get_mut(&current_tab) {
            view.handle_event(event)
        } else {
            strato_core::event::EventResult::Ignored
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn clone_widget(&self) -> Box<dyn Widget> {
        let mut new_views = HashMap::new();
        for (k, v) in &self.views {
            new_views.insert(k.clone(), v.clone_widget());
        }

        Box::new(ViewSwitcher {
            id: strato_widgets::widget::generate_id(),
            active_tab: self.active_tab.clone(),
            views: new_views,
            last_tab: self.last_tab.clone(),
        })
    }

    fn children(&self) -> Vec<&(dyn Widget + '_)> {
        let current_tab = self.active_tab.get();
        if let Some(view) = self.views.get(&current_tab) {
            vec![view.as_ref()]
        } else {
            vec![]
        }
    }

    fn children_mut(&mut self) -> Vec<&mut (dyn Widget + '_)> {
        let current_tab = self.active_tab.get();
        if let Some(view) = self.views.get_mut(&current_tab) {
            vec![view.as_mut()]
        } else {
            vec![]
        }
    }
}
