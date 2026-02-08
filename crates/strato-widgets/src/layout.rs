//! Layout widgets for arranging child widgets

use std::any::Any;
use crate::widget::{generate_id, Widget, WidgetId};
use strato_core::taffy::{
    prelude::*,
    style::{AlignItems, Dimension, FlexDirection, JustifyContent},
};
use strato_core::{
    event::{Event, EventResult},
    layout::{
        AlignItems as CoreAlignItems, Constraints, FlexContainer, FlexDirection as CoreFlexDirection,
        FlexItem, JustifyContent as CoreJustifyContent, Layout, Size,
    },
    taffy_layout::{TaffyLayoutError, TaffyLayoutResult, TaffyWidget},
};
use strato_renderer::batch::RenderBatch;

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

impl MainAxisAlignment {
    fn to_taffy(&self) -> JustifyContent {
        match self {
            MainAxisAlignment::Start => JustifyContent::FlexStart,
            MainAxisAlignment::Center => JustifyContent::Center,
            MainAxisAlignment::End => JustifyContent::FlexEnd,
            MainAxisAlignment::SpaceBetween => JustifyContent::SpaceBetween,
            MainAxisAlignment::SpaceAround => JustifyContent::SpaceAround,
            MainAxisAlignment::SpaceEvenly => JustifyContent::SpaceEvenly,
        }
    }

    fn to_core(&self) -> CoreJustifyContent {
        match self {
            MainAxisAlignment::Start => CoreJustifyContent::FlexStart,
            MainAxisAlignment::Center => CoreJustifyContent::Center,
            MainAxisAlignment::End => CoreJustifyContent::FlexEnd,
            MainAxisAlignment::SpaceBetween => CoreJustifyContent::SpaceBetween,
            MainAxisAlignment::SpaceAround => CoreJustifyContent::SpaceAround,
            MainAxisAlignment::SpaceEvenly => CoreJustifyContent::SpaceEvenly,
        }
    }
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

impl CrossAxisAlignment {
    fn to_taffy(&self) -> AlignItems {
        match self {
            CrossAxisAlignment::Start => AlignItems::FlexStart,
            CrossAxisAlignment::Center => AlignItems::Center,
            CrossAxisAlignment::End => AlignItems::FlexEnd,
            CrossAxisAlignment::Stretch => AlignItems::Stretch,
            CrossAxisAlignment::Baseline => AlignItems::Baseline,
        }
    }

    fn to_core(&self) -> CoreAlignItems {
        match self {
            CrossAxisAlignment::Start => CoreAlignItems::FlexStart,
            CrossAxisAlignment::Center => CoreAlignItems::Center,
            CrossAxisAlignment::End => CoreAlignItems::FlexEnd,
            CrossAxisAlignment::Stretch => CoreAlignItems::Stretch,
            CrossAxisAlignment::Baseline => CoreAlignItems::Baseline,
        }
    }
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
        let engine = strato_core::layout::LayoutEngine::new();

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
            direction: CoreFlexDirection::Row,
            justify_content: self.main_axis_alignment.to_core(),
            align_items: self.cross_axis_alignment.to_core(),
            ..Default::default()
        };
        let layouts = engine.calculate_flex_layout(&container, &child_data, constraints);

        // Calculate total size
        let width = layouts
            .iter()
            .map(|l| l.position.x + l.size.width)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);
        let height = layouts
            .iter()
            .map(|l| l.size.height)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);

        Size::new(width, height)
    }

    fn render(&self, batch: &mut RenderBatch, layout: Layout) {
        let engine = strato_core::layout::LayoutEngine::new();

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
            direction: CoreFlexDirection::Row,
            justify_content: self.main_axis_alignment.to_core(),
            align_items: self.cross_axis_alignment.to_core(),
            ..Default::default()
        };
        let layouts = engine.calculate_flex_layout(
            &container,
            &child_data,
            Constraints::loose(layout.size.width, layout.size.height),
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
        Box::new(Row {
            id: generate_id(),
            children: self.children.iter().map(|c| c.clone_widget()).collect(),
            main_axis_alignment: self.main_axis_alignment,
            cross_axis_alignment: self.cross_axis_alignment,
            spacing: self.spacing,
            cached_child_sizes: Vec::new(),
        })
    }

    fn as_taffy(&self) -> Option<&dyn TaffyWidget> {
        Some(self)
    }

    fn render_taffy(
        &self,
        batch: &mut RenderBatch,
        tree: &TaffyTree<()>,
        node: NodeId,
        parent_offset: strato_core::types::Point,
    ) {
        if let Ok(layout) = tree.layout(node) {
            let my_position = parent_offset + strato_core::types::Point::new(layout.location.x, layout.location.y);
            
            // Render children
            if let Ok(children_nodes) = tree.children(node) {
                let mut child_node_idx = 0;
                for child in &self.children {
                    if child.as_taffy().is_some() {
                        if child_node_idx < children_nodes.len() {
                            let child_node = children_nodes[child_node_idx];
                            child.render_taffy(batch, tree, child_node, my_position);
                            child_node_idx += 1;
                        }
                    }
                }
            }
        }
    }
}

impl TaffyWidget for Row {
    fn build_layout(&self, tree: &mut TaffyTree<()>) -> TaffyLayoutResult<NodeId> {
        let mut children_nodes = Vec::with_capacity(self.children.len());
        for child in &self.children {
            if let Some(taffy_child) = child.as_taffy() {
                let node = taffy_child.build_layout(tree)?;
                children_nodes.push(node);
            }
        }

        let style = Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            justify_content: Some(self.main_axis_alignment.to_taffy()),
            align_items: Some(self.cross_axis_alignment.to_taffy()),
            gap: strato_core::taffy::prelude::Size {
                width: length(self.spacing),
                height: length(0.0),
            },
            ..Default::default()
        };

        tree.new_with_children(style, &children_nodes)
            .map_err(|e| TaffyLayoutError::from(e))
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
        let engine = strato_core::layout::LayoutEngine::new();

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
            direction: CoreFlexDirection::Column,
            justify_content: self.main_axis_alignment.to_core(),
            align_items: self.cross_axis_alignment.to_core(),
            ..Default::default()
        };
        let layouts = engine.calculate_flex_layout(&container, &child_data, constraints);

        // Calculate total size
        let width = layouts
            .iter()
            .map(|l| l.size.width)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);
        let height = layouts
            .iter()
            .map(|l| l.position.y + l.size.height)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);

        Size::new(width, height)
    }

    fn render(&self, batch: &mut RenderBatch, layout: Layout) {
        let engine = strato_core::layout::LayoutEngine::new();

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
            direction: CoreFlexDirection::Column,
            justify_content: self.main_axis_alignment.to_core(),
            align_items: self.cross_axis_alignment.to_core(),
            ..Default::default()
        };
        let layouts = engine.calculate_flex_layout(
            &container,
            &child_data,
            Constraints::loose(layout.size.width, layout.size.height),
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
        Box::new(Column {
            id: generate_id(),
            children: self.children.iter().map(|c| c.clone_widget()).collect(),
            main_axis_alignment: self.main_axis_alignment,
            cross_axis_alignment: self.cross_axis_alignment,
            spacing: self.spacing,
            cached_child_sizes: Vec::new(),
        })
    }

    fn as_taffy(&self) -> Option<&dyn TaffyWidget> {
        Some(self)
    }

    fn render_taffy(
        &self,
        batch: &mut RenderBatch,
        tree: &TaffyTree<()>,
        node: NodeId,
        parent_offset: strato_core::types::Point,
    ) {
        if let Ok(layout) = tree.layout(node) {
            let my_position = parent_offset + strato_core::types::Point::new(layout.location.x, layout.location.y);
            
            // Render children
            if let Ok(children_nodes) = tree.children(node) {
                let mut child_node_idx = 0;
                for child in &self.children {
                    if child.as_taffy().is_some() {
                        if child_node_idx < children_nodes.len() {
                            let child_node = children_nodes[child_node_idx];
                            child.render_taffy(batch, tree, child_node, my_position);
                            child_node_idx += 1;
                        }
                    }
                }
            }
        }
    }
}

impl TaffyWidget for Column {
    fn build_layout(&self, tree: &mut TaffyTree<()>) -> TaffyLayoutResult<NodeId> {
        let mut children_nodes = Vec::with_capacity(self.children.len());
        for child in &self.children {
            if let Some(taffy_child) = child.as_taffy() {
                let node = taffy_child.build_layout(tree)?;
                children_nodes.push(node);
            }
        }

        let style = Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            justify_content: Some(self.main_axis_alignment.to_taffy()),
            align_items: Some(self.cross_axis_alignment.to_taffy()),
            gap: strato_core::taffy::prelude::Size {
                width: length(0.0),
                height: length(self.spacing),
            },
            ..Default::default()
        };

        tree.new_with_children(style, &children_nodes)
            .map_err(|e| TaffyLayoutError::from(e))
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

    fn as_taffy(&self) -> Option<&dyn TaffyWidget> {
        Some(self)
    }

    fn render_taffy(
        &self,
        batch: &mut RenderBatch,
        tree: &TaffyTree<()>,
        node: NodeId,
        parent_offset: strato_core::types::Point,
    ) {
        if let Ok(layout) = tree.layout(node) {
            let my_position = parent_offset + strato_core::types::Point::new(layout.location.x, layout.location.y);
            
            // Render children
            if let Ok(children_nodes) = tree.children(node) {
                let mut child_node_idx = 0;
                for child in &self.children {
                    if child.as_taffy().is_some() {
                        if child_node_idx < children_nodes.len() {
                            let child_node = children_nodes[child_node_idx];
                            child.render_taffy(batch, tree, child_node, my_position);
                            child_node_idx += 1;
                        }
                    }
                }
            }
        }
    }
}

impl TaffyWidget for Stack {
    fn build_layout(&self, tree: &mut TaffyTree<()>) -> TaffyLayoutResult<NodeId> {
        let mut children_nodes = Vec::with_capacity(self.children.len());
        for child in &self.children {
            if let Some(taffy_child) = child.as_taffy() {
                let node = taffy_child.build_layout(tree)?;
                // Force absolute positioning for Stack children
                let mut style = tree.style(node).cloned().unwrap_or_default();
                style.position = Position::Absolute;
                tree.set_style(node, style).map_err(|e| TaffyLayoutError::from(e))?;
                children_nodes.push(node);
            }
        }

        let style = Style {
            display: Display::Flex,
            size: strato_core::taffy::prelude::Size {
                width: percent(1.0),
                height: percent(1.0),
            },
            ..Default::default()
        };

        tree.new_with_children(style, &children_nodes)
            .map_err(|e| TaffyLayoutError::from(e))
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

    fn as_taffy(&self) -> Option<&dyn TaffyWidget> {
        Some(self)
    }

    fn render_taffy(
        &self,
        batch: &mut RenderBatch,
        tree: &TaffyTree<()>,
        node: NodeId,
        parent_offset: strato_core::types::Point,
    ) {
        // Flex doesn't have its own node in Taffy (it configures the child's node).
        // So we pass the same node to the child.
        self.child.render_taffy(batch, tree, node, parent_offset);
    }
}

impl TaffyWidget for Flex {
    fn build_layout(&self, tree: &mut TaffyTree<()>) -> TaffyLayoutResult<NodeId> {
        if let Some(taffy_child) = self.child.as_taffy() {
            let node = taffy_child.build_layout(tree)?;
            let mut style = tree.style(node).map_err(|e| TaffyLayoutError::from(e))?.clone();
            style.flex_grow = self.flex;
            tree.set_style(node, style).map_err(|e| TaffyLayoutError::from(e))?;
            Ok(node)
        } else {
            Err(TaffyLayoutError::NodeBuildFailed)
        }
    }
}


