//! Flexbox-based layout engine for OxideUI
//! 
//! This module provides a comprehensive flexbox layout system that supports
//! all major flexbox properties including direction, wrap, alignment, and gaps.

use glam::Vec2;
use std::fmt::Debug;

/// Layout constraints for widgets
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Constraints {
    pub min_width: f32,
    pub max_width: f32,
    pub min_height: f32,
    pub max_height: f32,
}

impl Constraints {
    /// Create constraints with no limits
    pub fn none() -> Self {
        Self {
            min_width: 0.0,
            max_width: f32::INFINITY,
            min_height: 0.0,
            max_height: f32::INFINITY,
        }
    }

    /// Create tight constraints (fixed size)
    pub fn tight(width: f32, height: f32) -> Self {
        Self {
            min_width: width,
            max_width: width,
            min_height: height,
            max_height: height,
        }
    }

    /// Create loose constraints (maximum size)
    pub fn loose(width: f32, height: f32) -> Self {
        Self {
            min_width: 0.0,
            max_width: width,
            min_height: 0.0,
            max_height: height,
        }
    }

    /// Constrain a size to these constraints
    pub fn constrain(&self, size: Size) -> Size {
        Size {
            width: size.width.clamp(self.min_width, self.max_width),
            height: size.height.clamp(self.min_height, self.max_height),
        }
    }

    /// Check if a size satisfies these constraints
    pub fn is_satisfied_by(&self, size: Size) -> bool {
        size.width >= self.min_width && size.width <= self.max_width &&
        size.height >= self.min_height && size.height <= self.max_height
    }
}

/// Size representation
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl Size {
    /// Create a new size
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    /// Create a zero size
    pub fn zero() -> Self {
        Self::new(0.0, 0.0)
    }

    /// Convert to Vec2
    pub fn to_vec2(self) -> Vec2 {
        Vec2::new(self.width, self.height)
    }
}

impl From<Vec2> for Size {
    fn from(vec: Vec2) -> Self {
        Self::new(vec.x, vec.y)
    }
}

/// Flex direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlexDirection {
    Row,
    RowReverse,
    Column,
    ColumnReverse,
}

impl FlexDirection {
    /// Check if this is a row direction
    pub fn is_row(&self) -> bool {
        matches!(self, FlexDirection::Row | FlexDirection::RowReverse)
    }

    /// Check if this is a column direction
    pub fn is_column(&self) -> bool {
        !self.is_row()
    }

    /// Check if this is reversed
    pub fn is_reverse(&self) -> bool {
        matches!(self, FlexDirection::RowReverse | FlexDirection::ColumnReverse)
    }
}

/// Flex wrap behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlexWrap {
    NoWrap,
    Wrap,
    WrapReverse,
}

/// Main axis alignment (along flex direction)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JustifyContent {
    FlexStart,
    FlexEnd,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

/// Cross axis alignment (perpendicular to flex direction)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignItems {
    FlexStart,
    FlexEnd,
    Center,
    Stretch,
    Baseline,
}

/// Alignment for wrapped lines
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignContent {
    FlexStart,
    FlexEnd,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
    Stretch,
}

/// Individual item alignment override
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignSelf {
    Auto,
    FlexStart,
    FlexEnd,
    Center,
    Stretch,
    Baseline,
}

/// Flex properties for a widget
#[derive(Debug, Clone, Copy)]
pub struct FlexItem {
    pub flex_grow: f32,
    pub flex_shrink: f32,
    pub flex_basis: f32,
    pub align_self: AlignSelf,
    pub margin: EdgeInsets,
}

impl Default for FlexItem {
    fn default() -> Self {
        Self {
            flex_grow: 0.0,
            flex_shrink: 1.0,
            flex_basis: 0.0,
            align_self: AlignSelf::Auto,
            margin: EdgeInsets::default(),
        }
    }
}

impl FlexItem {
    /// Create a flex item with grow factor
    pub fn grow(flex_grow: f32) -> Self {
        Self {
            flex_grow,
            ..Default::default()
        }
    }

    /// Create a flex item with shrink factor
    pub fn shrink(flex_shrink: f32) -> Self {
        Self {
            flex_shrink,
            ..Default::default()
        }
    }

    /// Create a flex item with basis
    pub fn basis(flex_basis: f32) -> Self {
        Self {
            flex_basis,
            ..Default::default()
        }
    }
}

/// Edge insets (padding/margin)
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct EdgeInsets {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl EdgeInsets {
    /// Create uniform insets
    pub fn all(value: f32) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    /// Create symmetric insets
    pub fn symmetric(horizontal: f32, vertical: f32) -> Self {
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }

    /// Get total horizontal insets
    pub fn horizontal(&self) -> f32 {
        self.left + self.right
    }

    /// Get total vertical insets
    pub fn vertical(&self) -> f32 {
        self.top + self.bottom
    }
}

/// Gap properties for flex containers
#[derive(Debug, Clone, Copy, Default)]
pub struct Gap {
    pub row: f32,
    pub column: f32,
}

impl Gap {
    /// Create uniform gap
    pub fn all(value: f32) -> Self {
        Self {
            row: value,
            column: value,
        }
    }

    /// Create gap with different row and column values
    pub fn new(row: f32, column: f32) -> Self {
        Self { row, column }
    }
}

/// Flex container properties
#[derive(Debug, Clone, Copy)]
pub struct FlexContainer {
    pub direction: FlexDirection,
    pub wrap: FlexWrap,
    pub justify_content: JustifyContent,
    pub align_items: AlignItems,
    pub align_content: AlignContent,
    pub gap: Gap,
    pub padding: EdgeInsets,
}

impl Default for FlexContainer {
    fn default() -> Self {
        Self {
            direction: FlexDirection::Row,
            wrap: FlexWrap::NoWrap,
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::Stretch,
            align_content: AlignContent::Stretch,
            gap: Gap::default(),
            padding: EdgeInsets::default(),
        }
    }
}

/// Layout result for a widget
#[derive(Debug, Clone, Copy)]
pub struct Layout {
    pub position: Vec2,
    pub size: Size,
}

impl Layout {
    /// Create a new layout
    pub fn new(position: Vec2, size: Size) -> Self {
        Self { position, size }
    }

    /// Get the bounds as (x, y, width, height)
    pub fn bounds(&self) -> (f32, f32, f32, f32) {
        (self.position.x, self.position.y, self.size.width, self.size.height)
    }

    /// Check if a point is within this layout
    pub fn contains(&self, point: Vec2) -> bool {
        point.x >= self.position.x && point.x <= self.position.x + self.size.width &&
        point.y >= self.position.y && point.y <= self.position.y + self.size.height
    }
}

/// Flex line (row of items in a flex container)
#[derive(Debug)]
struct FlexLine {
    items: Vec<usize>,
    main_size: f32,
    cross_size: f32,
}

/// Layout engine for calculating widget positions
pub struct LayoutEngine {
    cache: dashmap::DashMap<u64, Layout>,
}

impl LayoutEngine {
    /// Create a new layout engine
    pub fn new() -> Self {
        Self {
            cache: dashmap::DashMap::new(),
        }
    }

    /// Calculate flex layout for a container and its children
    pub fn calculate_flex_layout(
        &self,
        container: &FlexContainer,
        children: &[(FlexItem, Size)],
        constraints: Constraints,
    ) -> Vec<Layout> {
        if children.is_empty() {
            return Vec::new();
        }

        // Calculate available space after padding
        let content_constraints = Constraints {
            min_width: (constraints.min_width - container.padding.horizontal()).max(0.0),
            max_width: (constraints.max_width - container.padding.horizontal()).max(0.0),
            min_height: (constraints.min_height - container.padding.vertical()).max(0.0),
            max_height: (constraints.max_height - container.padding.vertical()).max(0.0),
        };

        // Determine main and cross axis dimensions
        let (main_size, cross_size) = if container.direction.is_row() {
            (content_constraints.max_width, content_constraints.max_height)
        } else {
            (content_constraints.max_height, content_constraints.max_width)
        };

        // Create flex lines
        let lines = self.create_flex_lines(container, children, main_size);
        
        // Calculate layouts for each line
        let mut layouts = Vec::with_capacity(children.len());
        let mut cross_position = container.padding.top;

        for line in &lines {
            let line_layouts = self.calculate_line_layout(
                container,
                children,
                line,
                main_size,
                cross_position,
            );
            layouts.extend(line_layouts);
            cross_position += line.cross_size + container.gap.row;
        }

        // Apply align-content for multiple lines
        if lines.len() > 1 {
            self.apply_align_content(container, &mut layouts, &lines, cross_size);
        }

        layouts
    }

    /// Create flex lines based on wrap behavior
    fn create_flex_lines(
        &self,
        container: &FlexContainer,
        children: &[(FlexItem, Size)],
        main_size: f32,
    ) -> Vec<FlexLine> {
        let mut lines = Vec::new();
        let mut current_line = FlexLine {
            items: Vec::new(),
            main_size: 0.0,
            cross_size: 0.0,
        };

        for (i, (item, size)) in children.iter().enumerate() {
            let item_main_size = if container.direction.is_row() {
                size.width + item.margin.horizontal()
            } else {
                size.height + item.margin.vertical()
            };

            let item_cross_size = if container.direction.is_row() {
                size.height + item.margin.vertical()
            } else {
                size.width + item.margin.horizontal()
            };

            // Check if we need to wrap
            let needs_wrap = container.wrap != FlexWrap::NoWrap &&
                !current_line.items.is_empty() &&
                current_line.main_size + item_main_size + container.gap.column > main_size;

            if needs_wrap {
                lines.push(current_line);
                current_line = FlexLine {
                    items: Vec::new(),
                    main_size: 0.0,
                    cross_size: 0.0,
                };
            }

            current_line.items.push(i);
            current_line.main_size += item_main_size;
            if !current_line.items.is_empty() {
                current_line.main_size += container.gap.column;
            }
            current_line.cross_size = current_line.cross_size.max(item_cross_size);
        }

        if !current_line.items.is_empty() {
            lines.push(current_line);
        }

        lines
    }

    /// Calculate layout for a single flex line
    fn calculate_line_layout(
        &self,
        container: &FlexContainer,
        children: &[(FlexItem, Size)],
        line: &FlexLine,
        main_size: f32,
        cross_position: f32,
    ) -> Vec<Layout> {
        let mut layouts = Vec::new();
        
        // Calculate flex grow/shrink
        let total_flex_grow: f32 = line.items.iter()
            .map(|&i| children[i].0.flex_grow)
            .sum();
        
        let total_flex_shrink: f32 = line.items.iter()
            .map(|&i| children[i].0.flex_shrink)
            .sum();

        // Calculate available space
        let used_space = line.main_size - container.gap.column * (line.items.len() - 1) as f32;
        let free_space = main_size - used_space;

        // Distribute free space
        let mut main_position = container.padding.left;
        
        // Apply justify-content
        match container.justify_content {
            JustifyContent::FlexEnd => main_position += free_space,
            JustifyContent::Center => main_position += free_space / 2.0,
            JustifyContent::SpaceBetween if line.items.len() > 1 => {
                // Space will be distributed between items
            }
            JustifyContent::SpaceAround => {
                let space_per_item = free_space / line.items.len() as f32;
                main_position += space_per_item / 2.0;
            }
            JustifyContent::SpaceEvenly => {
                let space_per_gap = free_space / (line.items.len() + 1) as f32;
                main_position += space_per_gap;
            }
            _ => {}
        }

        for (idx, &item_idx) in line.items.iter().enumerate() {
            let (item, size) = &children[item_idx];
            
            // Calculate item main size with flex
            let mut item_main_size = if container.direction.is_row() {
                size.width
            } else {
                size.height
            };

            if free_space > 0.0 && total_flex_grow > 0.0 {
                item_main_size += (item.flex_grow / total_flex_grow) * free_space;
            } else if free_space < 0.0 && total_flex_shrink > 0.0 {
                item_main_size += (item.flex_shrink / total_flex_shrink) * free_space;
            }

            // Calculate cross size
            let item_cross_size = if container.direction.is_row() {
                size.height
            } else {
                size.width
            };

            // Apply align-items/align-self
            let align = if item.align_self != AlignSelf::Auto {
                match item.align_self {
                    AlignSelf::FlexStart => AlignItems::FlexStart,
                    AlignSelf::FlexEnd => AlignItems::FlexEnd,
                    AlignSelf::Center => AlignItems::Center,
                    AlignSelf::Stretch => AlignItems::Stretch,
                    AlignSelf::Baseline => AlignItems::Baseline,
                    AlignSelf::Auto => container.align_items,
                }
            } else {
                container.align_items
            };

            let mut item_cross_position = cross_position;
            match align {
                AlignItems::FlexEnd => item_cross_position += line.cross_size - item_cross_size,
                AlignItems::Center => item_cross_position += (line.cross_size - item_cross_size) / 2.0,
                AlignItems::Stretch => {
                    // Stretch to fill cross axis
                }
                _ => {}
            }

            // Create layout based on direction
            let layout = if container.direction.is_row() {
                Layout::new(
                    Vec2::new(main_position + item.margin.left, item_cross_position + item.margin.top),
                    Size::new(item_main_size, item_cross_size),
                )
            } else {
                Layout::new(
                    Vec2::new(item_cross_position + item.margin.left, main_position + item.margin.top),
                    Size::new(item_cross_size, item_main_size),
                )
            };

            layouts.push(layout);

            // Update position for next item
            main_position += item_main_size + item.margin.horizontal() + container.gap.column;
            
            // Apply justify-content spacing
            match container.justify_content {
                JustifyContent::SpaceBetween if line.items.len() > 1 && idx < line.items.len() - 1 => {
                    main_position += free_space / (line.items.len() - 1) as f32;
                }
                JustifyContent::SpaceAround => {
                    let space_per_item = free_space / line.items.len() as f32;
                    main_position += space_per_item;
                }
                JustifyContent::SpaceEvenly if idx < line.items.len() - 1 => {
                    let space_per_gap = free_space / (line.items.len() + 1) as f32;
                    main_position += space_per_gap;
                }
                _ => {}
            }
        }

        layouts
    }

    /// Apply align-content for multiple lines
    fn apply_align_content(
        &self,
        container: &FlexContainer,
        layouts: &mut [Layout],
        lines: &[FlexLine],
        cross_size: f32,
    ) {
        let total_cross_size: f32 = lines.iter().map(|line| line.cross_size).sum();
        let total_gaps = container.gap.row * (lines.len() - 1) as f32;
        let free_cross_space = cross_size - total_cross_size - total_gaps;

        let mut cross_offset = 0.0;
        match container.align_content {
            AlignContent::FlexEnd => cross_offset = free_cross_space,
            AlignContent::Center => cross_offset = free_cross_space / 2.0,
            AlignContent::SpaceBetween if lines.len() > 1 => {
                // Space will be distributed between lines
            }
            AlignContent::SpaceAround => {
                cross_offset = free_cross_space / (lines.len() * 2) as f32;
            }
            AlignContent::SpaceEvenly => {
                cross_offset = free_cross_space / (lines.len() + 1) as f32;
            }
            _ => return,
        }

        // Apply offset to all layouts
        let mut item_idx = 0;
        for (line_idx, line) in lines.iter().enumerate() {
            let line_offset = match container.align_content {
                AlignContent::SpaceBetween if lines.len() > 1 => {
                    (free_cross_space / (lines.len() - 1) as f32) * line_idx as f32
                }
                AlignContent::SpaceAround => {
                    cross_offset + (free_cross_space / lines.len() as f32) * line_idx as f32
                }
                AlignContent::SpaceEvenly => {
                    cross_offset * (line_idx + 1) as f32
                }
                _ => cross_offset,
            };

            for _ in 0..line.items.len() {
                if container.direction.is_row() {
                    layouts[item_idx].position.y += line_offset;
                } else {
                    layouts[item_idx].position.x += line_offset;
                }
                item_idx += 1;
            }
        }
    }

    /// Clear the layout cache
    pub fn clear_cache(&self) {
        self.cache.clear();
    }
}

impl Default for LayoutEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constraints() {
        let constraints = Constraints::tight(100.0, 100.0);
        let size = Size::new(150.0, 50.0);
        let constrained = constraints.constrain(size);
        assert_eq!(constrained.width, 100.0);
        assert_eq!(constrained.height, 100.0);
    }

    #[test]
    fn test_flex_layout() {
        let engine = LayoutEngine::new();
        let children = vec![
            (FlexProps { flex: 1.0, ..Default::default() }, Size::new(0.0, 50.0)),
            (FlexProps { flex: 2.0, ..Default::default() }, Size::new(0.0, 50.0)),
        ];
        
        let layouts = engine.calculate_flex(
            Direction::Horizontal,
            MainAxisAlignment::Start,
            CrossAxisAlignment::Start,
            &children,
            Constraints::loose(300.0, 100.0),
            0.0,
        );
        
        assert_eq!(layouts.len(), 2);
        assert_eq!(layouts[0].size.width, 100.0);
        assert_eq!(layouts[1].size.width, 200.0);
    }
}
