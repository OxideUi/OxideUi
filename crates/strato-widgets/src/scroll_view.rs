use crate::prelude::*;
use strato_core::event::{Event, EventResult, MouseEvent};
use strato_core::layout::{Constraints, Layout, Size};
use strato_core::types::{Color, Point, Rect, Transform};
use strato_renderer::batch::RenderBatch;

use crate::widget::BaseWidget;

#[derive(Debug)]
pub struct ScrollView {
    base: BaseWidget,
    child: Box<dyn Widget>,
    offset: Point,
    content_size: Size,
    viewport_size: Size,

    // Interaction state
    bounds: strato_core::state::Signal<Rect>,
    scrollbar_rect: strato_core::state::Signal<Rect>,
    is_dragging: bool,
    drag_start_y: f32,
    offset_start_y: f32,
}

impl ScrollView {
    pub fn new(child: impl Widget + 'static) -> Self {
        Self {
            base: BaseWidget::new(),
            child: Box::new(child),
            offset: Point::new(0.0, 0.0),
            content_size: Size::zero(),
            viewport_size: Size::zero(),
            bounds: strato_core::state::Signal::new(Rect::new(0.0, 0.0, 0.0, 0.0)),
            scrollbar_rect: strato_core::state::Signal::new(Rect::new(0.0, 0.0, 0.0, 0.0)),
            is_dragging: false,
            drag_start_y: 0.0,
            offset_start_y: 0.0,
        }
    }

    fn update_scrollbar_rect(
        &self,
        content_height: f32,
        viewport_height: f32,
        offset_y: f32,
        bounds: Rect,
    ) {
        if content_height > viewport_height {
            let ratio = viewport_height / content_height;
            let thumb_height = (viewport_height * ratio).max(20.0);
            let track_height = viewport_height;
            let max_offset = content_height - viewport_height;
            let thumb_y = if max_offset > 0.0 {
                (offset_y / max_offset) * (track_height - thumb_height)
            } else {
                0.0
            };

            let scrollbar_width = 10.0;
            let scrollbar_x = bounds.x + bounds.width - scrollbar_width;
            let scrollbar_y = bounds.y + thumb_y;

            self.scrollbar_rect.set(Rect::new(
                scrollbar_x,
                scrollbar_y,
                scrollbar_width,
                thumb_height,
            ));
        } else {
            self.scrollbar_rect.set(Rect::new(0.0, 0.0, 0.0, 0.0));
        }
    }
}

impl Widget for ScrollView {
    fn id(&self) -> WidgetId {
        self.base.id()
    }

    fn layout(&mut self, constraints: Constraints) -> Size {
        // ScrollView takes all available space or respects max size
        let self_size = Size::new(constraints.max_width, constraints.max_height);
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
        let bounds = Rect::new(
            layout.position.x,
            layout.position.y,
            layout.size.width,
            layout.size.height,
        );
        self.bounds.set(bounds);

        // Update scrollbar rect
        self.update_scrollbar_rect(
            self.content_size.height,
            layout.size.height,
            self.offset.y,
            bounds,
        );

        // 1. Push Clip
        batch.push_clip(bounds);

        // 2. Render child offset
        let draw_pos = layout.position - self.offset.to_vec2();

        // We use the computed content size for the child layout
        let child_layout = Layout::new(draw_pos, self.content_size);
        self.child.render(batch, child_layout);

        // 3. Pop Clip
        batch.pop_clip();

        // 4. Draw Scrollbar
        let scrollbar = self.scrollbar_rect.get();
        if scrollbar.width > 0.0 {
            // Draw thumb
            batch.add_rect(
                scrollbar,
                if self.is_dragging {
                    Color::rgba(0.4, 0.4, 0.4, 0.8)
                } else {
                    Color::rgba(0.5, 0.5, 0.5, 0.5)
                },
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

                // Update scrollbar rect immediately for responsiveness if we were running a single loop
                // but render will handle it.

                EventResult::Handled
            }
            Event::MouseDown(mouse) => {
                let point = Point::new(mouse.position.x, mouse.position.y);
                let scrollbar = self.scrollbar_rect.get();

                if scrollbar.contains(point) {
                    self.is_dragging = true;
                    self.drag_start_y = point.y;
                    self.offset_start_y = self.offset.y;
                    return EventResult::Handled;
                }

                self.child.handle_event(event)
            }
            Event::MouseMove(mouse) => {
                if self.is_dragging {
                    let point = Point::new(mouse.position.x, mouse.position.y);
                    let delta_y = point.y - self.drag_start_y;

                    let viewport_h = self.viewport_size.height;
                    let content_h = self.content_size.height;

                    if content_h > viewport_h {
                        // Calculate how much offset changes per pixel of scrollbar movement
                        let track_height = viewport_h;
                        // We need the thumb height to know track range
                        let ratio = viewport_h / content_h;
                        let thumb_height = (viewport_h * ratio).max(20.0);
                        let track_range = track_height - thumb_height;

                        if track_range > 0.0 {
                            let max_offset = content_h - viewport_h;
                            let offset_delta = (delta_y / track_range) * max_offset;

                            self.offset.y =
                                (self.offset_start_y + offset_delta).clamp(0.0, max_offset);
                        }
                    }

                    return EventResult::Handled;
                }
                self.child.handle_event(event)
            }
            Event::MouseUp(_) => {
                if self.is_dragging {
                    self.is_dragging = false;
                    return EventResult::Handled;
                }
                self.child.handle_event(event)
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
            bounds: strato_core::state::Signal::new(self.bounds.get()),
            scrollbar_rect: strato_core::state::Signal::new(self.scrollbar_rect.get()),
            is_dragging: false,
            drag_start_y: 0.0,
            offset_start_y: 0.0,
        })
    }

    fn children(&self) -> Vec<&(dyn Widget + '_)> {
        vec![self.child.as_ref()]
    }

    fn children_mut(&mut self) -> Vec<&mut (dyn Widget + '_)> {
        vec![self.child.as_mut()]
    }
}
