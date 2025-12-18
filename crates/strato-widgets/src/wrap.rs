//! Wrap widget for flow layout
use crate::widget::{generate_id, Widget, WidgetId};
use std::any::Any;
use strato_core::{
    event::{Event, EventResult},
    layout::{
        AlignContent, AlignItems, Constraints, FlexContainer, FlexDirection, FlexItem, FlexWrap,
        Gap, JustifyContent, Layout, Size,
    },
};
use strato_renderer::batch::RenderBatch;

/// Alignment for wrap layout
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WrapAlignment {
    Start,
    Center,
    End,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

/// Cross axis alignment for items in a run
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WrapCrossAlignment {
    Start,
    Center,
    End,
}

/// A widget that displays its children in multiple horizontal or vertical runs.
#[derive(Debug)]
pub struct Wrap {
    id: WidgetId,
    children: Vec<Box<dyn Widget>>,
    direction: FlexDirection,
    alignment: WrapAlignment,
    cross_alignment: WrapCrossAlignment,
    run_alignment: WrapAlignment,
    spacing: f32,
    run_spacing: f32,
    // Layout cache computed during layout()
    cached_child_sizes: Vec<Size>,
}

impl Wrap {
    /// Create a new wrap widget
    pub fn new() -> Self {
        Self {
            id: generate_id(),
            children: Vec::new(),
            direction: FlexDirection::Row,
            alignment: WrapAlignment::Start,
            cross_alignment: WrapCrossAlignment::Start,
            run_alignment: WrapAlignment::Start,
            spacing: 0.0,
            run_spacing: 0.0,
            cached_child_sizes: Vec::new(),
        }
    }

    /// Add children widgets
    pub fn children(mut self, children: Vec<Box<dyn Widget>>) -> Self {
        self.children = children;
        self
    }

    /// Add a single child
    pub fn child(mut self, child: Box<dyn Widget>) -> Self {
        self.children.push(child);
        self
    }

    /// Set direction
    pub fn direction(mut self, direction: FlexDirection) -> Self {
        self.direction = direction;
        self
    }

    /// Set main axis alignment
    pub fn alignment(mut self, alignment: WrapAlignment) -> Self {
        self.alignment = alignment;
        self
    }

    /// Set cross axis alignment (for items in a run)
    pub fn cross_alignment(mut self, alignment: WrapCrossAlignment) -> Self {
        self.cross_alignment = alignment;
        self
    }

    /// Set run alignment (how runs are placed in the cross axis)
    pub fn run_alignment(mut self, alignment: WrapAlignment) -> Self {
        self.run_alignment = alignment;
        self
    }

    /// Set spacing between items in the main axis
    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    /// Set spacing between runs in the cross axis
    pub fn run_spacing(mut self, spacing: f32) -> Self {
        self.run_spacing = spacing;
        self
    }
}

impl Widget for Wrap {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn layout(&mut self, constraints: Constraints) -> Size {
        let engine = strato_core::layout::LayoutEngine::new();

        // Relax constraints for children measurement
        // Wrap children can be any size, they force a wrap if they exceed width
        let child_constraints = Constraints {
            min_width: 0.0,
            max_width: constraints.max_width, // Individual items shouldn't exceed container
            min_height: 0.0,
            max_height: constraints.max_height,
        };

        // Calculate child sizes
        let mut child_data = Vec::new();
        let mut sizes = Vec::with_capacity(self.children.len());
        for child in &mut self.children {
            let child_size = child.layout(child_constraints);
            sizes.push(child_size);

            // Wrap treats all items as non-flex by default in terms of growing to fill line
            // But we can support flex basis if needed. For now, we assume simple flow.
            let flex_item = FlexItem::default();
            child_data.push((flex_item, child_size));
        }
        self.cached_child_sizes = sizes;

        // Map widget props to core layout props
        let justify_content = match self.alignment {
            WrapAlignment::Start => JustifyContent::FlexStart,
            WrapAlignment::Center => JustifyContent::Center,
            WrapAlignment::End => JustifyContent::FlexEnd,
            WrapAlignment::SpaceBetween => JustifyContent::SpaceBetween,
            WrapAlignment::SpaceAround => JustifyContent::SpaceAround,
            WrapAlignment::SpaceEvenly => JustifyContent::SpaceEvenly,
        };

        let align_items = match self.cross_alignment {
            WrapCrossAlignment::Start => AlignItems::FlexStart,
            WrapCrossAlignment::Center => AlignItems::Center,
            WrapCrossAlignment::End => AlignItems::FlexEnd,
        };

        let align_content = match self.run_alignment {
            WrapAlignment::Start => AlignContent::FlexStart,
            WrapAlignment::Center => AlignContent::Center,
            WrapAlignment::End => AlignContent::FlexEnd,
            WrapAlignment::SpaceBetween => AlignContent::SpaceBetween,
            WrapAlignment::SpaceAround => AlignContent::SpaceAround,
            WrapAlignment::SpaceEvenly => AlignContent::SpaceEvenly,
        };

        let container = FlexContainer {
            direction: self.direction,
            wrap: FlexWrap::Wrap,
            justify_content,
            align_items,
            align_content,
            gap: Gap {
                row: self.run_spacing, // Gap between runs (lines)
                column: self.spacing,  // Gap between items
            },
            ..Default::default()
        };

        let layouts = engine.calculate_flex_layout(&container, &child_data, constraints);

        // Calculate total size based on returned layouts
        let mut max_x = 0.0f32;
        let mut max_y = 0.0f32;
        for l in layouts {
            max_x = max_x.max(l.position.x + l.size.width);
            max_y = max_y.max(l.position.y + l.size.height);
        }

        Size::new(max_x, max_y)
    }

    fn render(&self, batch: &mut RenderBatch, layout: Layout) {
        let engine = strato_core::layout::LayoutEngine::new();

        // Reconstruct child data from cache
        let mut child_data = Vec::new();
        for (i, _) in self.children.iter().enumerate() {
            let child_size = self
                .cached_child_sizes
                .get(i)
                .copied()
                .unwrap_or(Size::zero());
            child_data.push((FlexItem::default(), child_size));
        }

        // Map props again (should factor this out if it gets complex)
        let justify_content = match self.alignment {
            WrapAlignment::Start => JustifyContent::FlexStart,
            WrapAlignment::Center => JustifyContent::Center,
            WrapAlignment::End => JustifyContent::FlexEnd,
            WrapAlignment::SpaceBetween => JustifyContent::SpaceBetween,
            WrapAlignment::SpaceAround => JustifyContent::SpaceAround,
            WrapAlignment::SpaceEvenly => JustifyContent::SpaceEvenly,
        };

        let align_items = match self.cross_alignment {
            WrapCrossAlignment::Start => AlignItems::FlexStart,
            WrapCrossAlignment::Center => AlignItems::Center,
            WrapCrossAlignment::End => AlignItems::FlexEnd,
        };

        let align_content = match self.run_alignment {
            WrapAlignment::Start => AlignContent::FlexStart,
            WrapAlignment::Center => AlignContent::Center,
            WrapAlignment::End => AlignContent::FlexEnd,
            WrapAlignment::SpaceBetween => AlignContent::SpaceBetween,
            WrapAlignment::SpaceAround => AlignContent::SpaceAround,
            WrapAlignment::SpaceEvenly => AlignContent::SpaceEvenly,
        };

        let container = FlexContainer {
            direction: self.direction,
            wrap: FlexWrap::Wrap,
            justify_content,
            align_items,
            align_content,
            gap: Gap {
                row: self.run_spacing,
                column: self.spacing,
            },
            ..Default::default()
        };

        // Recalculate layout inside bounds
        let layouts = engine.calculate_flex_layout(
            &container,
            &child_data,
            Constraints::tight(layout.size.width, layout.size.height), // Use actual assigned size
        );

        // Render children
        for (child, child_layout) in self.children.iter().zip(layouts.iter()) {
            let absolute_layout =
                Layout::new(layout.position + child_layout.position, child_layout.size);
            child.render(batch, absolute_layout);
        }
    }

    fn handle_event(&mut self, event: &Event) -> EventResult {
        for child in &mut self.children {
            if child.handle_event(event) == EventResult::Handled {
                return EventResult::Handled;
            }
        }
        EventResult::Ignored
    }

    fn children(&self) -> Vec<&(dyn Widget + '_)> {
        self.children.iter().map(|c| c.as_ref()).collect()
    }

    fn children_mut<'a>(&'a mut self) -> Vec<&'a mut (dyn Widget + 'a)> {
        self.children
            .iter_mut()
            .map(|c| c.as_mut() as &'a mut (dyn Widget + 'a))
            .collect()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_widget(&self) -> Box<dyn Widget> {
        Box::new(Wrap {
            id: generate_id(),
            children: self.children.iter().map(|c| c.clone_widget()).collect(),
            direction: self.direction,
            alignment: self.alignment,
            cross_alignment: self.cross_alignment,
            run_alignment: self.run_alignment,
            spacing: self.spacing,
            run_spacing: self.run_spacing,
            cached_child_sizes: Vec::new(),
        })
    }
}
