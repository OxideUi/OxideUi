//! Flexbox-based layout engine for OxideUI

use glam::Vec2;
use std::fmt::Debug;

/// Size constraints for layout calculation
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Constraints {
    pub min_width: f32,
    pub max_width: f32,
    pub min_height: f32,
    pub max_height: f32,
}

impl Constraints {
    /// Create unconstrained constraints
    pub fn none() -> Self {
        Self {
            min_width: 0.0,
            max_width: f32::INFINITY,
            min_height: 0.0,
            max_height: f32::INFINITY,
        }
    }

    /// Create tight constraints (exact size)
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

/// Size of a layout element
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
        Self { width: 0.0, height: 0.0 }
    }

    /// Convert to Vec2
    pub fn to_vec2(&self) -> Vec2 {
        Vec2::new(self.width, self.height)
    }
}

impl From<Vec2> for Size {
    fn from(vec: Vec2) -> Self {
        Self { width: vec.x, height: vec.y }
    }
}

/// Layout direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Horizontal,
    Vertical,
}

/// Main axis alignment (along the flex direction)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MainAxisAlignment {
    Start,
    Center,
    End,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

/// Cross axis alignment (perpendicular to flex direction)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrossAxisAlignment {
    Start,
    Center,
    End,
    Stretch,
}

/// Flex properties for a widget
#[derive(Debug, Clone, Copy)]
pub struct FlexProps {
    pub flex: f32,
    pub grow: f32,
    pub shrink: f32,
    pub basis: f32,
}

impl Default for FlexProps {
    fn default() -> Self {
        Self {
            flex: 0.0,
            grow: 0.0,
            shrink: 1.0,
            basis: 0.0,
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

    /// Calculate flex layout
    pub fn calculate_flex(
        &self,
        direction: Direction,
        main_axis_alignment: MainAxisAlignment,
        _cross_axis_alignment: CrossAxisAlignment,
        children: &[(FlexProps, Size)],
        constraints: Constraints,
        spacing: f32,
    ) -> Vec<Layout> {
        let mut layouts = Vec::with_capacity(children.len());
        
        if children.is_empty() {
            return layouts;
        }

        // Calculate total flex and fixed space
        let mut total_flex = 0.0;
        let mut fixed_space = 0.0;
        
        for (props, size) in children {
            if props.flex > 0.0 {
                total_flex += props.flex;
            } else {
                fixed_space += match direction {
                    Direction::Horizontal => size.width,
                    Direction::Vertical => size.height,
                };
            }
        }
        
        // Add spacing
        fixed_space += spacing * (children.len() - 1) as f32;
        
        // Calculate available space for flex items
        let available_space = match direction {
            Direction::Horizontal => constraints.max_width - fixed_space,
            Direction::Vertical => constraints.max_height - fixed_space,
        }.max(0.0);
        
        // Calculate positions
        let mut position = 0.0;
        
        for (props, size) in children {
            let item_size = if props.flex > 0.0 && total_flex > 0.0 {
                (props.flex / total_flex) * available_space
            } else {
                match direction {
                    Direction::Horizontal => size.width,
                    Direction::Vertical => size.height,
                }
            };
            
            let layout = match direction {
                Direction::Horizontal => Layout::new(
                    Vec2::new(position, 0.0),
                    Size::new(item_size, size.height),
                ),
                Direction::Vertical => Layout::new(
                    Vec2::new(0.0, position),
                    Size::new(size.width, item_size),
                ),
            };
            
            layouts.push(layout);
            position += item_size + spacing;
        }
        
        // Apply main axis alignment
        let total_size = position - spacing;
        let extra_space = match direction {
            Direction::Horizontal => constraints.max_width - total_size,
            Direction::Vertical => constraints.max_height - total_size,
        }.max(0.0);
        
        match main_axis_alignment {
            MainAxisAlignment::Center => {
                let offset = extra_space / 2.0;
                for layout in &mut layouts {
                    match direction {
                        Direction::Horizontal => layout.position.x += offset,
                        Direction::Vertical => layout.position.y += offset,
                    }
                }
            }
            MainAxisAlignment::End => {
                for layout in &mut layouts {
                    match direction {
                        Direction::Horizontal => layout.position.x += extra_space,
                        Direction::Vertical => layout.position.y += extra_space,
                    }
                }
            }
            MainAxisAlignment::SpaceBetween if children.len() > 1 => {
                let spacing = extra_space / (children.len() - 1) as f32;
                for (i, layout) in layouts.iter_mut().enumerate() {
                    let offset = spacing * i as f32;
                    match direction {
                        Direction::Horizontal => layout.position.x += offset,
                        Direction::Vertical => layout.position.y += offset,
                    }
                }
            }
            MainAxisAlignment::SpaceAround => {
                let spacing = extra_space / children.len() as f32;
                for (i, layout) in layouts.iter_mut().enumerate() {
                    let offset = spacing * (i as f32 + 0.5);
                    match direction {
                        Direction::Horizontal => layout.position.x += offset,
                        Direction::Vertical => layout.position.y += offset,
                    }
                }
            }
            MainAxisAlignment::SpaceEvenly => {
                let spacing = extra_space / (children.len() + 1) as f32;
                for (i, layout) in layouts.iter_mut().enumerate() {
                    let offset = spacing * (i + 1) as f32;
                    match direction {
                        Direction::Horizontal => layout.position.x += offset,
                        Direction::Vertical => layout.position.y += offset,
                    }
                }
            }
            _ => {}
        }
        
        layouts
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
