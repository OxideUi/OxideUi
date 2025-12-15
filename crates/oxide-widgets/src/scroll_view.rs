use crate::prelude::*;
use oxide_core::event::{Event, EventResult, MouseEvent};
use oxide_core::layout::{Constraints, Layout, Size};
use oxide_renderer::batch::RenderBatch;
use oxide_core::types::{Rect, Point, Transform, Color};

use crate::widget::BaseWidget;

#[derive(Debug)]
pub struct ScrollView {
    base: BaseWidget,
    child: Box<dyn Widget>,
    offset: Point,
    content_size: Size,
    viewport_size: Size,
}

impl ScrollView {
    pub fn new(child: impl Widget + 'static) -> Self {
        Self {
            base: BaseWidget::new(),
            child: Box::new(child),
            offset: Point::new(0.0, 0.0),
            content_size: Size::zero(),
            viewport_size: Size::zero(),
        }
    }
}

impl Widget for ScrollView {
    fn id(&self) -> WidgetId {
        self.base.id()
    }

    fn layout(&mut self, constraints: Constraints) -> Size {
        // ScrollView takes all available space or respects max size
        let self_size = Size::new(
            constraints.max_width,
            constraints.max_height
        );
        self.viewport_size = self_size;
        
        // Layout child with infinite constraints
        let child_constraints = Constraints {
            min_width: 0.0,
            max_width: f32::INFINITY,
            min_height: 0.0,
            max_height: f32::INFINITY,
        };

        self.content_size = self.child.layout(child_constraints);
        
        self_size
    }

    fn render(&self, batch: &mut RenderBatch, layout: Layout) {
        // 1. Push Clip
        batch.push_clip(Rect::new(layout.position.x, layout.position.y, layout.size.width, layout.size.height));

        // 2. Render child offset
        let draw_pos = layout.position - self.offset.to_vec2();
        
        // We use the computed content size for the child layout
        let child_layout = Layout::new(draw_pos, self.content_size);
        self.child.render(batch, child_layout);

        // 3. Pop Clip
        batch.pop_clip();
        
        // 4. Draw Scrollbar
        let viewport_height = layout.size.height;
        let content_height = self.content_size.height;
        
        if content_height > viewport_height {
            let ratio = viewport_height / content_height;
            let thumb_height = (viewport_height * ratio).max(20.0);
            let track_height = viewport_height;
            let max_offset = content_height - viewport_height;
            let thumb_y = if max_offset > 0.0 {
                (self.offset.y / max_offset) * (track_height - thumb_height)
            } else {
                0.0
            };
            
            let scrollbar_width = 10.0;
            let scrollbar_x = layout.position.x + layout.size.width - scrollbar_width;
            let scrollbar_y = layout.position.y + thumb_y;
            
            // Draw thumb
            batch.add_rect(
                Rect::new(scrollbar_x, scrollbar_y, scrollbar_width, thumb_height),
                Color::rgba(0.5, 0.5, 0.5, 0.5),
                Transform::identity(),
            );
        }
    }

    fn handle_event(&mut self, event: &Event) -> EventResult {
        match event {
            Event::MouseWheel { delta, .. } => {
                let delta_x = delta.x;
                let delta_y = delta.y;
                
                let viewport_w = self.viewport_size.width;
                let viewport_h = self.viewport_size.height;
                
                let max_x = (self.content_size.width - viewport_w).max(0.0);
                let max_y = (self.content_size.height - viewport_h).max(0.0);
                
                self.offset.x = (self.offset.x - delta_x).clamp(0.0, max_x);
                self.offset.y = (self.offset.y - delta_y).clamp(0.0, max_y);
                
                EventResult::Handled
            }
            _ => self.child.handle_event(event),
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn clone_widget(&self) -> Box<dyn Widget> {
        Box::new(Self {
            base: self.base.clone(),
            child: self.child.clone_widget(),
            offset: self.offset,
            content_size: self.content_size,
            viewport_size: self.viewport_size,
        })
    }
    
    fn children(&self) -> Vec<&(dyn Widget + '_)> {
        vec![self.child.as_ref()]
    }

    fn children_mut(&mut self) -> Vec<&mut (dyn Widget + '_)> {
        vec![self.child.as_mut()]
    }
}
