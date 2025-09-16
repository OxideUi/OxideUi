//! Text widget implementation

use crate::widget::{Widget, WidgetId, generate_id};
use oxide_core::{
    layout::{Constraints, Layout, Size},
    types::Color,
};
use oxide_renderer::batch::RenderBatch;
use std::any::Any;

#[derive(Debug, Clone)]
pub struct Text {
    content: String,
    style: oxide_renderer::TextStyle,
    max_width: Option<f32>,
}

impl Text {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            style: oxide_renderer::TextStyle::default(),
            max_width: None,
        }
    }

    pub fn size(mut self, size: f32) -> Self {
        self.style.size = size;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.style.color = color;
        self
    }

    pub fn with_max_width(mut self, max_width: f32) -> Self {
        self.max_width = Some(max_width);
        self
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn set_content(&mut self, content: impl Into<String>) {
        self.content = content.into();
    }

    pub fn style(&self) -> &oxide_renderer::TextStyle {
        &self.style
    }

    pub fn set_style(&mut self, style: oxide_renderer::TextStyle) {
        self.style = style;
    }
}

impl Widget for Text {
    fn id(&self) -> WidgetId {
        generate_id() // Temporary implementation
    }

    fn layout(&mut self, constraints: Constraints) -> Size {
        // For now, return a fixed size based on text length
        // TODO: Implement proper text measurement
        let char_width = 8.0;
        let line_height = self.style.size * 1.2;
        let width = self.content.len() as f32 * char_width;
        
        Size::new(
            width.min(constraints.max_width),
            line_height.min(constraints.max_height)
        )
    }

    fn render(&self, batch: &mut RenderBatch, layout: Layout) {
        // For now, use the batch's draw_text method
        batch.draw_text(
            &self.content,
            (layout.position.x, layout.position.y),
            self.style.color,
            self.style.size,
        );
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_widget(&self) -> Box<dyn Widget> {
        Box::new(self.clone())
    }
}
