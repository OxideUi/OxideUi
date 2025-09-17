//! Container widget for layout and styling

use crate::widget::{Widget, WidgetId, generate_id};
use oxide_core::{
    event::{Event, EventResult},
    layout::{Constraints, EdgeInsets, Layout, Size},
    types::{Color, Rect, BorderRadius, Shadow},
    Transform,
};
use oxide_renderer::batch::RenderBatch;
use std::any::Any;

/// Container widget for grouping and styling child widgets
#[derive(Debug)]
pub struct Container {
    id: WidgetId,
    child: Option<Box<dyn Widget>>,
    style: ContainerStyle,
    constraints: Option<Constraints>,
}

impl Container {
    /// Create a new container
    pub fn new() -> Self {
        Self {
            id: generate_id(),
            child: None,
            style: ContainerStyle::default(),
            constraints: None,
        }
    }

    /// Set the child widget
    pub fn child(mut self, child: impl Widget + 'static) -> Self {
        self.child = Some(Box::new(child));
        self
    }

    /// Set padding
    pub fn padding(mut self, padding: f32) -> Self {
        self.style.padding = EdgeInsets::all(padding);
        self
    }

    /// Set padding with individual values
    pub fn padding_values(mut self, top: f32, right: f32, bottom: f32, left: f32) -> Self {
        self.style.padding = EdgeInsets { top, right, bottom, left };
        self
    }

    /// Set margin
    pub fn margin(mut self, margin: f32) -> Self {
        self.style.margin = EdgeInsets::all(margin);
        self
    }

    /// Set background color
    pub fn background(mut self, color: Color) -> Self {
        self.style.background_color = color;
        self
    }

    /// Set border
    pub fn border(mut self, width: f32, color: Color) -> Self {
        self.style.border_width = width;
        self.style.border_color = color;
        self
    }

    /// Set border radius
    pub fn border_radius(mut self, radius: f32) -> Self {
        self.style.border_radius = BorderRadius::all(radius);
        self
    }

    /// Set shadow
    pub fn shadow(mut self, shadow: Shadow) -> Self {
        self.style.shadow = Some(shadow);
        self
    }

    /// Set width
    pub fn width(mut self, width: f32) -> Self {
        self.style.width = Some(width);
        self
    }

    /// Set height
    pub fn height(mut self, height: f32) -> Self {
        self.style.height = Some(height);
        self
    }

    /// Set both width and height
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.style.width = Some(width);
        self.style.height = Some(height);
        self
    }

    /// Set style
    pub fn style(mut self, style: ContainerStyle) -> Self {
        self.style = style;
        self
    }

    /// Set constraints
    pub fn constraints(mut self, constraints: Constraints) -> Self {
        self.constraints = Some(constraints);
        self
    }
}

impl Widget for Container {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn layout(&mut self, constraints: Constraints) -> Size {
        let constraints = self.constraints.unwrap_or(constraints);
        
        // Apply margin to constraints
        let margin = self.style.margin;
        let inner_constraints = Constraints {
            min_width: (constraints.min_width - margin.horizontal()).max(0.0),
            max_width: (constraints.max_width - margin.horizontal()).max(0.0),
            min_height: (constraints.min_height - margin.vertical()).max(0.0),
            max_height: (constraints.max_height - margin.vertical()).max(0.0),
        };
        
        // Apply padding to child constraints
        let padding = self.style.padding;
        let child_constraints = Constraints {
            min_width: (inner_constraints.min_width - padding.horizontal()).max(0.0),
            max_width: (inner_constraints.max_width - padding.horizontal()).max(0.0),
            min_height: (inner_constraints.min_height - padding.vertical()).max(0.0),
            max_height: (inner_constraints.max_height - padding.vertical()).max(0.0),
        };
        
        // Calculate child size
        let child_size = if let Some(child) = &mut self.child {
            child.layout(child_constraints)
        } else {
            Size::zero()
        };
        
        // Calculate container size
        let mut width = child_size.width + padding.horizontal();
        let mut height = child_size.height + padding.vertical();
        
        // Apply fixed dimensions if specified
        if let Some(fixed_width) = self.style.width {
            width = fixed_width;
        }
        if let Some(fixed_height) = self.style.height {
            height = fixed_height;
        }
        
        // Add margin
        width += margin.horizontal();
        height += margin.vertical();
        
        // Constrain to limits
        Size::new(
            width.clamp(constraints.min_width, constraints.max_width),
            height.clamp(constraints.min_height, constraints.max_height),
        )
    }

    fn render(&self, batch: &mut RenderBatch, layout: Layout) {
        let margin = self.style.margin;
        let padding = self.style.padding;
        
        // Calculate content rect (excluding margin)
        let content_rect = Rect::new(
            layout.position.x + margin.left,
            layout.position.y + margin.top,
            layout.size.width - margin.horizontal(),
            layout.size.height - margin.vertical(),
        );
        
        // Draw shadow if present
        if let Some(shadow) = &self.style.shadow {
            let _shadow_rect = content_rect.expand(shadow.spread_radius);
            // TODO: Implement proper shadow rendering
        }
        
        // Draw background
        if self.style.background_color.a > 0.0 {
            batch.add_rect(content_rect, self.style.background_color, Transform::identity());
        }
        
        // Draw border
        if self.style.border_width > 0.0 {
            // TODO: Implement proper border rendering
        }
        
        // Render child
        if let Some(child) = &self.child {
            let child_layout = Layout::new(
                glam::Vec2::new(
                    content_rect.x + padding.left,
                    content_rect.y + padding.top,
                ),
                Size::new(
                    content_rect.width - padding.horizontal(),
                    content_rect.height - padding.vertical(),
                ),
            );
            child.render(batch, child_layout);
        }
    }

    fn handle_event(&mut self, event: &Event) -> EventResult {
        if let Some(child) = &mut self.child {
            child.handle_event(event)
        } else {
            EventResult::Ignored
        }
    }

    fn children(&self) -> Vec<&(dyn Widget + '_)> {
        if let Some(child) = &self.child {
            vec![child.as_ref()]
        } else {
            vec![]
        }
    }

    fn children_mut(&mut self) -> Vec<&mut (dyn Widget + '_)> {
        if let Some(child) = &mut self.child {
            vec![child.as_mut()]
        } else {
            vec![]
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_widget(&self) -> Box<dyn Widget> {
        Box::new(Container {
            id: generate_id(),
            child: self.child.as_ref().map(|c| c.clone_widget()),
            style: self.style.clone(),
            constraints: self.constraints,
        })
    }
}

impl Default for Container {
    fn default() -> Self {
        Self::new()
    }
}

/// Container style configuration
#[derive(Debug, Clone)]
pub struct ContainerStyle {
    pub background_color: Color,
    pub border_color: Color,
    pub border_width: f32,
    pub border_radius: BorderRadius,
    pub padding: EdgeInsets,
    pub margin: EdgeInsets,
    pub shadow: Option<Shadow>,
    pub width: Option<f32>,
    pub height: Option<f32>,
}

impl Default for ContainerStyle {
    fn default() -> Self {
        Self {
            background_color: Color::TRANSPARENT,
            border_color: Color::TRANSPARENT,
            border_width: 0.0,
            border_radius: BorderRadius::all(0.0),
            padding: EdgeInsets::all(0.0),
            margin: EdgeInsets::all(0.0),
            shadow: None,
            width: None,
            height: None,
        }
    }
}

impl ContainerStyle {
    /// Card style with shadow
    pub fn card() -> Self {
        Self {
            background_color: Color::WHITE,
            border_color: Color::rgba(0.0, 0.0, 0.0, 0.1),
            border_width: 1.0,
            border_radius: BorderRadius::all(8.0),
            padding: EdgeInsets::all(16.0),
            margin: EdgeInsets::all(8.0),
            shadow: Some(Shadow::drop(4.0)),
            width: None,
            height: None,
        }
    }

    /// Panel style
    pub fn panel() -> Self {
        Self {
            background_color: Color::rgba(0.95, 0.95, 0.95, 1.0),
            border_color: Color::rgba(0.0, 0.0, 0.0, 0.2),
            border_width: 1.0,
            border_radius: BorderRadius::all(4.0),
            padding: EdgeInsets::all(12.0),
            margin: EdgeInsets::all(0.0),
            shadow: None,
            width: None,
            height: None,
        }
    }
}
