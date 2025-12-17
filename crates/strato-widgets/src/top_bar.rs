use strato_core::{
    types::{Color, Point},
    layout::{Constraints, Layout, Size},
    event::{Event, EventResult},
};
use crate::{
    Widget,
    container::Container,
    layout::{Row, MainAxisAlignment, CrossAxisAlignment},
    text::{Text, FontWeight},
    widget::{WidgetId, WidgetContext, generate_id},
};
use std::any::Any;

/// A standardized top bar / header widget
#[derive(Debug)]
pub struct TopBar {
    id: WidgetId,
    inner: Option<Box<dyn Widget>>,
    
    // Props
    pub title: String,
    pub leading: Option<Box<dyn Widget>>,
    pub trailing: Option<Box<dyn Widget>>,
    pub height: f32,
    pub background: Color,
}

impl TopBar {
    pub fn new(title: String) -> Self {
        Self {
            id: generate_id(),
            inner: None,
            title,
            leading: None,
            trailing: None,
            height: 48.0,
            background: Color::rgba(0.1, 0.1, 0.1, 0.95), // Default dark nice background
        }
    }

    pub fn with_background(mut self, color: Color) -> Self {
        self.background = color;
        self.inner = None; // Invalidate inner widget
        self
    }

    pub fn with_leading(mut self, widget: impl Widget + 'static) -> Self {
        self.leading = Some(Box::new(widget));
        self.inner = None;
        self
    }

    pub fn with_trailing(mut self, widget: impl Widget + 'static) -> Self {
        self.trailing = Some(Box::new(widget));
        self.inner = None; // Invalidate inner widget
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self.inner = None;
        self
    }
    
    fn ensure_inner(&mut self) {
        if self.inner.is_some() { return; }
        
        // Build the inner widget tree
        let title_widget = Text::new(&self.title)
            .size(16.0)
            .font_weight(FontWeight::SemiBold) 
            .color(Color::WHITE);

        let mut row = Row::new()
            .main_axis_alignment(MainAxisAlignment::Center)
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .spacing(16.0);

        
        if let Some(leading) = &self.leading {
            row = row.child(leading.clone_widget());
        }

        row = row.child(Box::new(title_widget) as Box<dyn Widget>);

        if let Some(trailing) = &self.trailing {
            row = row.child(trailing.clone_widget());
        }
        
        let container = Container::new()
            .width(f32::MAX) // Full width
            .height(self.height)
            .background(self.background)
            .padding(12.0)
            .child(row);

        self.inner = Some(Box::new(container));
    }
}

impl Widget for TopBar {
    fn id(&self) -> WidgetId { self.id }
    
    fn layout(&mut self, constraints: Constraints) -> Size {
        self.ensure_inner();
        self.inner.as_mut().unwrap().layout(constraints)
    }
    
    fn render(&self, batch: &mut strato_renderer::batch::RenderBatch, layout: Layout) {
        if let Some(inner) = &self.inner {
            inner.render(batch, layout);
        }
    }
    
    fn handle_event(&mut self, event: &Event) -> EventResult {
        self.ensure_inner();
        self.inner.as_mut().unwrap().handle_event(event)
    }
    
    fn update(&mut self, ctx: &WidgetContext) {
        if let Some(inner) = &mut self.inner {
            inner.update(ctx);
        }
    }
    
    fn children(&self) -> Vec<&(dyn Widget + '_)> {
        if let Some(inner) = &self.inner {
            vec![inner.as_ref()]
        } else {
            vec![]
        }
    }
    
    fn children_mut(&mut self) -> Vec<&mut (dyn Widget + '_)> {
        if let Some(inner) = &mut self.inner {
            vec![inner.as_mut()]
        } else {
            vec![]
        }
    }
    
    fn hit_test(&self, point: Point, layout: Layout) -> bool {
        if let Some(inner) = &self.inner {
            inner.hit_test(point, layout)
        } else {
            false
        }
    }
    
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    
    fn clone_widget(&self) -> Box<dyn Widget> {
        Box::new(Self {
            id: generate_id(), // New ID
            inner: None, // Reset inner to force rebuild/fresh state
            title: self.title.clone(),
            leading: self.leading.as_ref().map(|w| w.clone_widget()),
            trailing: self.trailing.as_ref().map(|w| w.clone_widget()),
            height: self.height,
            background: self.background,
        })
    }
}
