use strato_widgets::{
    Widget, WidgetId, 
    animation::{AnimationController, Curve},
};
// Clean imports:
use strato_core::layout::{Layout, Size};
use strato_renderer::batch::RenderBatch;
use strato_core::{
    types::{Color, Point, Rect},
    event::{Event, EventResult},
};
use std::any::Any;
use std::time::Duration;

#[derive(Debug)]
pub struct AnimatedChart {
    id: WidgetId,
    data: Vec<f32>,
    bar_color: Color,
    anim_controller: AnimationController,
}

impl AnimatedChart {
    pub fn new(data: Vec<f32>) -> Self {
        let mut controller = AnimationController::new(Duration::from_millis(800))
             .with_curve(Curve::EaseOut);
        controller.start();

        Self {
            id: strato_widgets::widget::generate_id(),
            data,
            bar_color: Color::rgb(0.2, 0.6, 1.0), // Blue-ish
            anim_controller: controller,
        }
    }

    pub fn color(mut self, color: Color) -> Self {
        self.bar_color = color;
        self
    }
}

impl Widget for AnimatedChart {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn layout(&mut self, constraints: strato_core::layout::Constraints) -> Size {
        // Take all available space, or default specific size if unbounded
        Size::new(
            constraints.max_width.min(800.0).max(300.0), 
            constraints.max_height.min(400.0).max(200.0)
        )
    }

    fn render(&self, batch: &mut RenderBatch, layout: Layout) {
        let progress = self.anim_controller.value();
        let bar_count = self.data.len();
        if bar_count == 0 { return; }

        let gap = 10.0;
        let total_gap = gap * (bar_count - 1) as f32;
        let bar_width = (layout.size.width - total_gap) / bar_count as f32;
        let max_val = self.data.iter().cloned().fold(0.0f32, f32::max);

        for (i, &value) in self.data.iter().enumerate() {
            let x = layout.position.x + i as f32 * (bar_width + gap);
            
            // Animate height
            let target_h = (value / max_val) * layout.size.height;
            let current_h = target_h * progress;
            
            let y = layout.position.y + layout.size.height - current_h;

            let rect = Rect::new(x, y, bar_width, current_h);
            
            // Use different opacity based on index to show it's dynamic
            let alpha = 0.5 + 0.5 * (i as f32 / bar_count as f32);
            let color = Color {
                r: self.bar_color.r,
                g: self.bar_color.g,
                b: self.bar_color.b,
                a: alpha,
            };

            batch.add_rect(rect, color, strato_core::types::Transform::identity());
        }
    }

    fn handle_event(&mut self, event: &Event) -> EventResult {
        // Simple hover effect or restart animation on click could go here
        if let Event::MouseDown(_) = event {
             self.anim_controller.reset();
             self.anim_controller.start();
             return EventResult::Handled;
        }
        EventResult::Ignored
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_widget(&self) -> Box<dyn Widget> {
        Box::new(Self {
            id: strato_widgets::widget::generate_id(),
            data: self.data.clone(),
            bar_color: self.bar_color,
            anim_controller: self.anim_controller.clone(),
        })
    }
}
