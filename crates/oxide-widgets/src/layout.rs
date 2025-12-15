//! Layout widgets for arranging child widgets

use crate::widget::{Widget, WidgetId, generate_id};
use oxide_core::{
    event::{Event, EventResult},
    layout::{Constraints, Layout, Size, FlexItem, FlexContainer, FlexDirection, JustifyContent, AlignItems},
};
use oxide_renderer::batch::RenderBatch;
use std::any::Any;

/// Main axis alignment for flex layouts
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MainAxisAlignment {
    Start,
    Center,
    End,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

/// Cross axis alignment for flex layouts
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CrossAxisAlignment {
    Start,
    Center,
    End,
    Stretch,
    Baseline,
}

/// Row widget for horizontal layout
#[derive(Debug)]
pub struct Row {
    id: WidgetId,
    children: Vec<Box<dyn Widget>>,
    main_axis_alignment: MainAxisAlignment,
    cross_axis_alignment: CrossAxisAlignment,
    spacing: f32,
    // Layout cache computed during layout()
    cached_child_sizes: Vec<Size>,
}

impl Row {
    /// Create a new row
    pub fn new() -> Self {
        Self {
            id: generate_id(),
            children: Vec::new(),
            main_axis_alignment: MainAxisAlignment::Start,
            cross_axis_alignment: CrossAxisAlignment::Center,
            spacing: 0.0,
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

    /// Set main axis alignment
    pub fn main_axis_alignment(mut self, alignment: MainAxisAlignment) -> Self {
        self.main_axis_alignment = alignment;
        self
    }

    /// Set cross axis alignment
    pub fn cross_axis_alignment(mut self, alignment: CrossAxisAlignment) -> Self {
        self.cross_axis_alignment = alignment;
        self
    }

    /// Set spacing between children
    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }
}

impl Widget for Row {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn layout(&mut self, constraints: Constraints) -> Size {
        let engine = oxide_core::layout::LayoutEngine::new();
        
        // Relax constraints for children measurement
        let child_constraints = Constraints {
            min_width: 0.0,
            max_width: constraints.max_width,
            min_height: 0.0,
            max_height: constraints.max_height,
        };
        
        // Calculate child sizes
        let mut child_data = Vec::new();
        let mut sizes = Vec::with_capacity(self.children.len());
        for child in &mut self.children {
            let child_size = child.layout(child_constraints);
            sizes.push(child_size);
            
            let mut flex_item = FlexItem::default();
            if let Some(flex) = child.as_any().downcast_ref::<Flex>() {
                flex_item = FlexItem::grow(flex.flex);
            }
            child_data.push((flex_item, child_size));
        }
        // Cache sizes for use during render()
        self.cached_child_sizes = sizes;
        
        // Calculate layout
        let container = FlexContainer {
            direction: FlexDirection::Row,
            justify_content: match self.main_axis_alignment {
                MainAxisAlignment::Start => JustifyContent::FlexStart,
                MainAxisAlignment::Center => JustifyContent::Center,
                MainAxisAlignment::End => JustifyContent::FlexEnd,
                MainAxisAlignment::SpaceBetween => JustifyContent::SpaceBetween,
                MainAxisAlignment::SpaceAround => JustifyContent::SpaceAround,
                MainAxisAlignment::SpaceEvenly => JustifyContent::SpaceEvenly,
            },
            align_items: match self.cross_axis_alignment {
                CrossAxisAlignment::Start => AlignItems::FlexStart,
                CrossAxisAlignment::Center => AlignItems::Center,
                CrossAxisAlignment::End => AlignItems::FlexEnd,
                CrossAxisAlignment::Stretch => AlignItems::Stretch,
                CrossAxisAlignment::Baseline => AlignItems::Baseline,
            },
            ..Default::default()
        };
        let layouts = engine.calculate_flex_layout(&container, &child_data, constraints);
        
        // Calculate total size
        let width = layouts.iter()
            .map(|l| l.position.x + l.size.width)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);
        let height = layouts.iter()
            .map(|l| l.size.height)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);
        
        Size::new(width, height)
    }

    fn render(&self, batch: &mut RenderBatch, layout: Layout) {
        let engine = oxide_core::layout::LayoutEngine::new();
        
        // Calculate child layouts using cached sizes measured in layout()
        let mut child_data = Vec::new();
        for (i, child) in self.children.iter().enumerate() {
            let child_size = self
                .cached_child_sizes
                .get(i)
                .copied()
                .unwrap_or_else(|| Size::new(100.0, 50.0));
                
            let mut flex_item = FlexItem::default();
            if let Some(flex) = child.as_any().downcast_ref::<Flex>() {
                flex_item = FlexItem::grow(flex.flex);
            }
            child_data.push((flex_item, child_size));
        }
        
        let container = FlexContainer {
            direction: FlexDirection::Row,
            justify_content: match self.main_axis_alignment {
                MainAxisAlignment::Start => JustifyContent::FlexStart,
                MainAxisAlignment::Center => JustifyContent::Center,
                MainAxisAlignment::End => JustifyContent::FlexEnd,
                MainAxisAlignment::SpaceBetween => JustifyContent::SpaceBetween,
                MainAxisAlignment::SpaceAround => JustifyContent::SpaceAround,
                MainAxisAlignment::SpaceEvenly => JustifyContent::SpaceEvenly,
            },
            align_items: match self.cross_axis_alignment {
                CrossAxisAlignment::Start => AlignItems::FlexStart,
                CrossAxisAlignment::Center => AlignItems::Center,
                CrossAxisAlignment::End => AlignItems::FlexEnd,
                CrossAxisAlignment::Stretch => AlignItems::Stretch,
                CrossAxisAlignment::Baseline => AlignItems::Baseline,
            },
            ..Default::default()
        };
        let layouts = engine.calculate_flex_layout(&container, &child_data, Constraints::loose(layout.size.width, layout.size.height));
        
        // Render children
        for (child, child_layout) in self.children.iter().zip(layouts.iter()) {
            let absolute_layout = Layout::new(
                layout.position + child_layout.position,
                child_layout.size,
            );
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
        Box::new(Row {
            id: generate_id(),
            children: self.children.iter().map(|c| c.clone_widget()).collect(),
            main_axis_alignment: self.main_axis_alignment,
            cross_axis_alignment: self.cross_axis_alignment,
            spacing: self.spacing,
            cached_child_sizes: Vec::new(),
        })
    }
}

/// Column widget for vertical layout
#[derive(Debug)]
pub struct Column {
    id: WidgetId,
    children: Vec<Box<dyn Widget>>,
    main_axis_alignment: MainAxisAlignment,
    cross_axis_alignment: CrossAxisAlignment,
    spacing: f32,
    // Layout cache computed during layout()
    cached_child_sizes: Vec<Size>,
}

impl Column {
    /// Create a new column
    pub fn new() -> Self {
        Self {
            id: generate_id(),
            children: Vec::new(),
            main_axis_alignment: MainAxisAlignment::Start,
            cross_axis_alignment: CrossAxisAlignment::Center,
            spacing: 0.0,
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

    /// Set main axis alignment
    pub fn main_axis_alignment(mut self, alignment: MainAxisAlignment) -> Self {
        self.main_axis_alignment = alignment;
        self
    }

    /// Set cross axis alignment
    pub fn cross_axis_alignment(mut self, alignment: CrossAxisAlignment) -> Self {
        self.cross_axis_alignment = alignment;
        self
    }

    /// Set spacing between children
    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }
}

impl Widget for Column {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn layout(&mut self, constraints: Constraints) -> Size {
        let engine = oxide_core::layout::LayoutEngine::new();
        
        // Relax constraints for children measurement
        let child_constraints = Constraints {
            min_width: 0.0,
            max_width: constraints.max_width,
            min_height: 0.0,
            max_height: constraints.max_height,
        };
        
        // Calculate child sizes
        let mut child_data = Vec::new();
        let mut sizes = Vec::with_capacity(self.children.len());
        for child in &mut self.children {
            let child_size = child.layout(child_constraints);
            sizes.push(child_size);
            
            let mut flex_item = FlexItem::default();
            if let Some(flex) = child.as_any().downcast_ref::<Flex>() {
                flex_item = FlexItem::grow(flex.flex);
            }
            child_data.push((flex_item, child_size));
        }
        // Cache sizes for render()
        self.cached_child_sizes = sizes;
        
        // Calculate layout
        let container = FlexContainer {
            direction: FlexDirection::Column,
            justify_content: match self.main_axis_alignment {
                MainAxisAlignment::Start => JustifyContent::FlexStart,
                MainAxisAlignment::Center => JustifyContent::Center,
                MainAxisAlignment::End => JustifyContent::FlexEnd,
                MainAxisAlignment::SpaceBetween => JustifyContent::SpaceBetween,
                MainAxisAlignment::SpaceAround => JustifyContent::SpaceAround,
                MainAxisAlignment::SpaceEvenly => JustifyContent::SpaceEvenly,
            },
            align_items: match self.cross_axis_alignment {
                CrossAxisAlignment::Start => AlignItems::FlexStart,
                CrossAxisAlignment::Center => AlignItems::Center,
                CrossAxisAlignment::End => AlignItems::FlexEnd,
                CrossAxisAlignment::Stretch => AlignItems::Stretch,
                CrossAxisAlignment::Baseline => AlignItems::Baseline,
            },
            ..Default::default()
        };
        let layouts = engine.calculate_flex_layout(&container, &child_data, constraints);
        
        // Calculate total size
        let width = layouts.iter()
            .map(|l| l.size.width)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);
        let height = layouts.iter()
            .map(|l| l.position.y + l.size.height)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);
        
        Size::new(width, height)
    }

    fn render(&self, batch: &mut RenderBatch, layout: Layout) {
        let engine = oxide_core::layout::LayoutEngine::new();
        
        // Calculate child layouts using cached sizes computed during layout()
        let mut child_data = Vec::new();
        for (i, child) in self.children.iter().enumerate() {
            let child_size = self
                .cached_child_sizes
                .get(i)
                .copied()
                .unwrap_or_else(|| Size::new(100.0, 50.0));
                
            let mut flex_item = FlexItem::default();
            if let Some(flex) = child.as_any().downcast_ref::<Flex>() {
                flex_item = FlexItem::grow(flex.flex);
            }
            child_data.push((flex_item, child_size));
        }
        
        let container = FlexContainer {
            direction: FlexDirection::Column,
            justify_content: match self.main_axis_alignment {
                MainAxisAlignment::Start => JustifyContent::FlexStart,
                MainAxisAlignment::Center => JustifyContent::Center,
                MainAxisAlignment::End => JustifyContent::FlexEnd,
                MainAxisAlignment::SpaceBetween => JustifyContent::SpaceBetween,
                MainAxisAlignment::SpaceAround => JustifyContent::SpaceAround,
                MainAxisAlignment::SpaceEvenly => JustifyContent::SpaceEvenly,
            },
            align_items: match self.cross_axis_alignment {
                CrossAxisAlignment::Start => AlignItems::FlexStart,
                CrossAxisAlignment::Center => AlignItems::Center,
                CrossAxisAlignment::End => AlignItems::FlexEnd,
                CrossAxisAlignment::Stretch => AlignItems::Stretch,
                CrossAxisAlignment::Baseline => AlignItems::Baseline,
            },
            ..Default::default()
        };
        let layouts = engine.calculate_flex_layout(&container, &child_data, Constraints::loose(layout.size.width, layout.size.height));
        
        // Render children
        for (child, child_layout) in self.children.iter().zip(layouts.iter()) {
            let absolute_layout = Layout::new(
                layout.position + child_layout.position,
                child_layout.size,
            );
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
        Box::new(Column {
            id: generate_id(),
            children: self.children.iter().map(|c| c.clone_widget()).collect(),
            main_axis_alignment: self.main_axis_alignment,
            cross_axis_alignment: self.cross_axis_alignment,
            spacing: self.spacing,
            cached_child_sizes: Vec::new(),
        })
    }
}

/// Stack widget for layered layout
#[derive(Debug)]
pub struct Stack {
    id: WidgetId,
    children: Vec<Box<dyn Widget>>,
}

impl Stack {
    /// Create a new stack
    pub fn new() -> Self {
        Self {
            id: generate_id(),
            children: Vec::new(),
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
}

impl Widget for Stack {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn layout(&mut self, constraints: Constraints) -> Size {
        let mut max_width: f32 = 0.0;
        let mut max_height: f32 = 0.0;
        
        for child in &mut self.children {
            let size = child.layout(constraints);
            max_width = max_width.max(size.width);
            max_height = max_height.max(size.height);
        }
        
        Size::new(max_width, max_height)
    }

    fn render(&self, batch: &mut RenderBatch, layout: Layout) {
        // Render all children at the same position
        for child in &self.children {
            child.render(batch, layout);
        }
    }

    fn handle_event(&mut self, event: &Event) -> EventResult {
        // Events are handled from top to bottom (reverse order)
        for child in self.children.iter_mut().rev() {
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
        Box::new(Stack {
            id: generate_id(),
            children: self.children.iter().map(|c| c.clone_widget()).collect(),
        })
    }
}

/// Flexible widget for flex layout
#[derive(Debug)]
pub struct Flex {
    id: WidgetId,
    child: Box<dyn Widget>,
    flex: f32,
}

impl Flex {
    /// Create a new flex widget
    pub fn new(child: Box<dyn Widget>) -> Self {
        Self {
            id: generate_id(),
            child,
            flex: 1.0,
        }
    }

    /// Set flex factor
    pub fn flex(mut self, flex: f32) -> Self {
        self.flex = flex;
        self
    }
}

impl Widget for Flex {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn layout(&mut self, constraints: Constraints) -> Size {
        self.child.layout(constraints)
    }

    fn render(&self, batch: &mut RenderBatch, layout: Layout) {
        self.child.render(batch, layout);
    }

    fn handle_event(&mut self, event: &Event) -> EventResult {
        self.child.handle_event(event)
    }

    fn children(&self) -> Vec<&(dyn Widget + '_)> {
        vec![self.child.as_ref()]
    }

    fn children_mut<'a>(&'a mut self) -> Vec<&'a mut (dyn Widget + 'a)> {
        vec![self.child.as_mut() as &'a mut (dyn Widget + 'a)]
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_widget(&self) -> Box<dyn Widget> {
        Box::new(Flex {
            id: generate_id(),
            child: self.child.clone_widget(),
            flex: self.flex,
        })
    }
}
