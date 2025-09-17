//! Common types used throughout OxideUI

use glam::{Mat4, Vec2, Vec3, Vec4};
// Removed unused std::fmt import

/// Unique identifier for DOM nodes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub u64);

impl NodeId {
    /// Create a new node ID
    pub fn new() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for NodeId {
    fn default() -> Self {
        Self::new()
    }
}

/// Unique identifier for DOM elements
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ElementId(pub u64);

impl ElementId {
    /// Create a new element ID
    pub fn new() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for ElementId {
    fn default() -> Self {
        Self::new()
    }
}

/// 2D size representation
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

    /// Zero size
    pub fn zero() -> Self {
        Self::new(0.0, 0.0)
    }

    /// Convert to Vec2
    pub fn to_vec2(&self) -> Vec2 {
        Vec2::new(self.width, self.height)
    }

    /// Get the area
    pub fn area(&self) -> f32 {
        self.width * self.height
    }

    /// Check if the size is empty (zero area)
    pub fn is_empty(&self) -> bool {
        self.width <= 0.0 || self.height <= 0.0
    }
}

impl From<Vec2> for Size {
    fn from(vec: Vec2) -> Self {
        Self::new(vec.x, vec.y)
    }
}

/// RGBA color representation
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    /// Create a new color from RGBA values (0.0 to 1.0)
    pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Create a new color from RGB values (0.0 to 1.0)
    pub fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    /// Create a color from hex string
    pub fn from_hex(hex: &str) -> Result<Self, String> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 && hex.len() != 8 {
            return Err(format!("Invalid hex color: {}", hex));
        }
        
        let r = u8::from_str_radix(&hex[0..2], 16).map_err(|e| e.to_string())?;
        let g = u8::from_str_radix(&hex[2..4], 16).map_err(|e| e.to_string())?;
        let b = u8::from_str_radix(&hex[4..6], 16).map_err(|e| e.to_string())?;
        let a = if hex.len() == 8 {
            u8::from_str_radix(&hex[6..8], 16).map_err(|e| e.to_string())?
        } else {
            255
        };
        
        Ok(Self {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: a as f32 / 255.0,
        })
    }

    /// Convert to hex string
    pub fn to_hex(&self) -> String {
        format!("#{:02x}{:02x}{:02x}{:02x}",
            (self.r * 255.0) as u8,
            (self.g * 255.0) as u8,
            (self.b * 255.0) as u8,
            (self.a * 255.0) as u8,
        )
    }

    /// Convert to array for GPU usage
    pub fn to_array(&self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }

    /// Common colors
    pub const WHITE: Self = Self { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
    pub const BLACK: Self = Self { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const RED: Self = Self { r: 1.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const GREEN: Self = Self { r: 0.0, g: 1.0, b: 0.0, a: 1.0 };
    pub const BLUE: Self = Self { r: 0.0, g: 0.0, b: 1.0, a: 1.0 };
    pub const TRANSPARENT: Self = Self { r: 0.0, g: 0.0, b: 0.0, a: 0.0 };
    pub const GRAY: Self = Self { r: 0.5, g: 0.5, b: 0.5, a: 1.0 };
    pub const LIGHT_GRAY: Self = Self { r: 0.8, g: 0.8, b: 0.8, a: 1.0 };
    pub const DARK_GRAY: Self = Self { r: 0.3, g: 0.3, b: 0.3, a: 1.0 };
    
    // Material Design colors
    pub const PRIMARY: Self = Self { r: 0.129, g: 0.588, b: 0.953, a: 1.0 }; // Blue 500
    pub const SECONDARY: Self = Self { r: 0.0, g: 0.737, b: 0.831, a: 1.0 }; // Cyan 500
    pub const SUCCESS: Self = Self { r: 0.298, g: 0.686, b: 0.314, a: 1.0 }; // Green 500
    pub const WARNING: Self = Self { r: 1.0, g: 0.757, b: 0.027, a: 1.0 }; // Amber 500
    pub const ERROR: Self = Self { r: 0.956, g: 0.263, b: 0.212, a: 1.0 }; // Red 500
}

impl Default for Color {
    fn default() -> Self {
        Self::BLACK
    }
}

/// 2D point
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    /// Create a new point
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Zero point
    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    /// Convert to Vec2
    pub fn to_vec2(&self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }

    /// Calculate distance to another point
    pub fn distance_to(&self, other: Point) -> f32 {
        ((other.x - self.x).powi(2) + (other.y - self.y).powi(2)).sqrt()
    }
}

impl From<Vec2> for Point {
    fn from(vec: Vec2) -> Self {
        Self { x: vec.x, y: vec.y }
    }
}

/// Rectangle
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    /// Create a new rectangle
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }

    /// Create from position and size
    pub fn from_pos_size(pos: Point, size: (f32, f32)) -> Self {
        Self {
            x: pos.x,
            y: pos.y,
            width: size.0,
            height: size.1,
        }
    }

    /// Get the center point
    pub fn center(&self) -> Point {
        Point::new(self.x + self.width / 2.0, self.y + self.height / 2.0)
    }

    /// Check if a point is inside the rectangle
    pub fn contains(&self, point: Point) -> bool {
        point.x >= self.x && point.x <= self.x + self.width &&
        point.y >= self.y && point.y <= self.y + self.height
    }

    /// Check if this rectangle intersects with another
    pub fn intersects(&self, other: &Rect) -> bool {
        self.x < other.x + other.width &&
        self.x + self.width > other.x &&
        self.y < other.y + other.height &&
        self.y + self.height > other.y
    }

    /// Get the intersection of two rectangles
    pub fn intersection(&self, other: &Rect) -> Option<Rect> {
        if !self.intersects(other) {
            return None;
        }

        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        let right = (self.x + self.width).min(other.x + other.width);
        let bottom = (self.y + self.height).min(other.y + other.height);

        Some(Rect::new(x, y, right - x, bottom - y))
    }

    /// Expand the rectangle by a margin
    pub fn expand(&self, margin: f32) -> Self {
        Self {
            x: self.x - margin,
            y: self.y - margin,
            width: self.width + margin * 2.0,
            height: self.height + margin * 2.0,
        }
    }

    /// Contract the rectangle by a margin
    pub fn contract(&self, margin: f32) -> Self {
        self.expand(-margin)
    }
}

/// 2D transformation matrix
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    matrix: Mat4,
}

impl Transform {
    /// Create an identity transform
    pub fn identity() -> Self {
        Self {
            matrix: Mat4::IDENTITY,
        }
    }

    /// Create a translation transform
    pub fn translate(x: f32, y: f32) -> Self {
        Self {
            matrix: Mat4::from_translation(Vec3::new(x, y, 0.0)),
        }
    }

    /// Create a rotation transform (in radians)
    pub fn rotate(angle: f32) -> Self {
        Self {
            matrix: Mat4::from_rotation_z(angle),
        }
    }

    /// Create a scale transform
    pub fn scale(x: f32, y: f32) -> Self {
        Self {
            matrix: Mat4::from_scale(Vec3::new(x, y, 1.0)),
        }
    }

    /// Combine with another transform
    pub fn combine(&self, other: &Transform) -> Self {
        Self {
            matrix: self.matrix * other.matrix,
        }
    }

    /// Apply transform to a point
    pub fn transform_point(&self, point: Point) -> Point {
        let transformed = self.matrix * Vec4::new(point.x, point.y, 0.0, 1.0);
        Point::new(transformed.x, transformed.y)
    }

    /// Get the matrix
    pub fn matrix(&self) -> &Mat4 {
        &self.matrix
    }

    /// Get as array for GPU usage
    pub fn to_array(&self) -> [[f32; 4]; 4] {
        self.matrix.to_cols_array_2d()
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::identity()
    }
}

/// Border radius for rounded rectangles
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct BorderRadius {
    pub top_left: f32,
    pub top_right: f32,
    pub bottom_right: f32,
    pub bottom_left: f32,
}

impl BorderRadius {
    /// Create uniform border radius
    pub fn all(radius: f32) -> Self {
        Self {
            top_left: radius,
            top_right: radius,
            bottom_right: radius,
            bottom_left: radius,
        }
    }

    /// Create border radius with individual values
    pub fn new(top_left: f32, top_right: f32, bottom_right: f32, bottom_left: f32) -> Self {
        Self {
            top_left,
            top_right,
            bottom_right,
            bottom_left,
        }
    }
}

/// Gradient stop
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GradientStop {
    pub color: Color,
    pub position: f32,
}

/// Linear gradient
#[derive(Debug, Clone, PartialEq)]
pub struct LinearGradient {
    pub start: Point,
    pub end: Point,
    pub stops: Vec<GradientStop>,
}

impl LinearGradient {
    /// Create a new linear gradient
    pub fn new(start: Point, end: Point, stops: Vec<GradientStop>) -> Self {
        Self { start, end, stops }
    }

    /// Create a vertical gradient
    pub fn vertical(stops: Vec<GradientStop>) -> Self {
        Self {
            start: Point::new(0.0, 0.0),
            end: Point::new(0.0, 1.0),
            stops,
        }
    }

    /// Create a horizontal gradient
    pub fn horizontal(stops: Vec<GradientStop>) -> Self {
        Self {
            start: Point::new(0.0, 0.0),
            end: Point::new(1.0, 0.0),
            stops,
        }
    }
}

/// Shadow effect
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Shadow {
    pub color: Color,
    pub offset: Point,
    pub blur_radius: f32,
    pub spread_radius: f32,
}

impl Shadow {
    /// Create a new shadow
    pub fn new(color: Color, offset: Point, blur_radius: f32, spread_radius: f32) -> Self {
        Self {
            color,
            offset,
            blur_radius,
            spread_radius,
        }
    }

    /// Create a drop shadow
    pub fn drop(blur_radius: f32) -> Self {
        Self {
            color: Color::rgba(0.0, 0.0, 0.0, 0.3),
            offset: Point::new(0.0, 2.0),
            blur_radius,
            spread_radius: 0.0,
        }
    }
}

impl Default for Shadow {
    fn default() -> Self {
        Self::drop(4.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_from_hex() {
        let color = Color::from_hex("#FF0000").unwrap();
        assert_eq!(color.r, 1.0);
        assert_eq!(color.g, 0.0);
        assert_eq!(color.b, 0.0);
        assert_eq!(color.a, 1.0);
    }

    #[test]
    fn test_rect_contains() {
        let rect = Rect::new(10.0, 10.0, 20.0, 20.0);
        assert!(rect.contains(Point::new(15.0, 15.0)));
        assert!(!rect.contains(Point::new(5.0, 5.0)));
    }

    #[test]
    fn test_rect_intersection() {
        let rect1 = Rect::new(0.0, 0.0, 10.0, 10.0);
        let rect2 = Rect::new(5.0, 5.0, 10.0, 10.0);
        let intersection = rect1.intersection(&rect2).unwrap();
        assert_eq!(intersection, Rect::new(5.0, 5.0, 5.0, 5.0));
    }

    #[test]
    fn test_transform() {
        let transform = Transform::translate(10.0, 20.0);
        let point = Point::new(5.0, 5.0);
        let transformed = transform.transform_point(point);
        assert_eq!(transformed.x, 15.0);
        assert_eq!(transformed.y, 25.0);
    }
}
