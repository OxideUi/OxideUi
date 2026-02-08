//! Validated rectangle type with GPU-safe coordinate guarantees.
//!
//! # Safety Guarantees
//!
//! `ValidatedRect` provides the following invariants:
//! - All coordinates are finite (no NaN, no Infinity)
//! - Width and height are non-negative
//! - All values are clamped to prevent GPU overflow
//!
//! # Example
//!
//! ```rust
//! use strato_core::validated_rect::ValidatedRect;
//!
//! // Valid construction
//! let rect = ValidatedRect::new(10.0, 20.0, 100.0, 50.0).unwrap();
//!
//! // Invalid construction (NaN)
//! assert!(ValidatedRect::new(f32::NAN, 0.0, 10.0, 10.0).is_err());
//! ```

use crate::error::TaffyValidationError;

/// Maximum coordinate value to prevent GPU overflow.
///
/// @note This is a conservative limit that all major GPUs can handle.
/// @rationale Prevents floating-point precision issues at extreme values.
const MAX_COORD: f32 = 1_000_000.0;

/// Rectangle with GUARANTEED validity of all coordinates.
///
/// # Invariants
///
/// All instances of `ValidatedRect` satisfy:
/// - `x`, `y` are finite and in range `[-MAX_COORD, MAX_COORD]`
/// - `width`, `height` are finite, non-negative, and `<= MAX_COORD`
///
/// These invariants are enforced at construction time and cannot be violated
/// through safe Rust code.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ValidatedRect {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

impl ValidatedRect {
    /// Create a new validated rectangle.
    ///
    /// # Arguments
    ///
    /// * `x` - X position (must be finite)
    /// * `y` - Y position (must be finite)
    /// * `width` - Width (must be finite and >= 0.0)
    /// * `height` - Height (must be finite and >= 0.0)
    ///
    /// # Returns
    ///
    /// * `Ok(ValidatedRect)` - If all values are valid
    /// * `Err(TaffyValidationError)` - If any validation fails
    ///
    /// # Errors
    ///
    /// * `TaffyValidationError::NonFiniteValue` - If any value is NaN or Infinity
    /// * `TaffyValidationError::NegativeDimension` - If width or height < 0.0
    ///
    /// # Safety
    ///
    /// Thread-safety: This function is thread-safe.
    ///
    /// # Performance
    ///
    /// WCET: O(1), ~10 floating-point comparisons
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Result<Self, TaffyValidationError> {
        // STEP 1: Check is_finite() for all 4 values
        if !x.is_finite() || !y.is_finite() || !width.is_finite() || !height.is_finite() {
            return Err(TaffyValidationError::NonFiniteValue);
        }

        // STEP 2: Check width >= 0.0 && height >= 0.0
        if width < 0.0 || height < 0.0 {
            return Err(TaffyValidationError::NegativeDimension { width, height });
        }

        // STEP 3: Clamp all values to safe range
        let x_clamped = x.clamp(-MAX_COORD, MAX_COORD);
        let y_clamped = y.clamp(-MAX_COORD, MAX_COORD);
        let width_clamped = width.min(MAX_COORD);
        let height_clamped = height.min(MAX_COORD);

        Ok(Self {
            x: x_clamped,
            y: y_clamped,
            width: width_clamped,
            height: height_clamped,
        })
    }

    /// Create a validated rectangle from Taffy layout output.
    ///
    /// # Arguments
    ///
    /// * `layout` - Taffy layout result
    ///
    /// # Returns
    ///
    /// * `Ok(ValidatedRect)` - If Taffy output is valid
    /// * `Err(TaffyValidationError)` - If Taffy produced invalid coordinates
    ///
    /// # Note
    ///
    /// Taffy COULD potentially produce invalid values in edge cases
    /// (e.g., overflow from extreme percentages). This function catches those.
    pub fn from_taffy(layout: &taffy::Layout) -> Result<Self, TaffyValidationError> {
        Self::new(
            layout.location.x,
            layout.location.y,
            layout.size.width,
            layout.size.height,
        )
    }

    /// Zero-size rectangle at origin.
    ///
    /// # Returns
    ///
    /// A valid rectangle with x=0, y=0, width=0, height=0.
    #[inline]
    pub fn zero() -> Self {
        // SAFETY: All values are 0.0, which is finite and non-negative
        Self {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        }
    }

    /// Get X position.
    #[inline]
    pub fn x(&self) -> f32 {
        self.x
    }

    /// Get Y position.
    #[inline]
    pub fn y(&self) -> f32 {
        self.y
    }

    /// Get width.
    #[inline]
    pub fn width(&self) -> f32 {
        self.width
    }

    /// Get height.
    #[inline]
    pub fn height(&self) -> f32 {
        self.height
    }

    /// Get right edge (x + width).
    #[inline]
    pub fn right(&self) -> f32 {
        self.x + self.width
    }

    /// Get bottom edge (y + height).
    #[inline]
    pub fn bottom(&self) -> f32 {
        self.y + self.height
    }

    /// Check if a point is inside this rectangle.
    ///
    /// # Arguments
    ///
    /// * `px` - Point X coordinate
    /// * `py` - Point Y coordinate
    ///
    /// # Returns
    ///
    /// `true` if point is inside or on the boundary.
    #[inline]
    pub fn contains(&self, px: f32, py: f32) -> bool {
        px >= self.x && px <= self.right() && py >= self.y && py <= self.bottom()
    }

    /// Convert to tuple (x, y, width, height).
    #[inline]
    pub fn to_tuple(&self) -> (f32, f32, f32, f32) {
        (self.x, self.y, self.width, self.height)
    }

    /// Convert to array [x, y, width, height].
    #[inline]
    pub fn to_array(&self) -> [f32; 4] {
        [self.x, self.y, self.width, self.height]
    }
}

impl Default for ValidatedRect {
    fn default() -> Self {
        Self::zero()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validated_rect_valid_creation() {
        let rect = ValidatedRect::new(10.0, 20.0, 100.0, 50.0);
        assert!(rect.is_ok());
        let rect = rect.unwrap();
        assert_eq!(rect.x(), 10.0);
        assert_eq!(rect.y(), 20.0);
        assert_eq!(rect.width(), 100.0);
        assert_eq!(rect.height(), 50.0);
    }

    #[test]
    fn test_validated_rect_rejects_nan() {
        assert!(ValidatedRect::new(f32::NAN, 0.0, 10.0, 10.0).is_err());
        assert!(ValidatedRect::new(0.0, f32::NAN, 10.0, 10.0).is_err());
        assert!(ValidatedRect::new(0.0, 0.0, f32::NAN, 10.0).is_err());
        assert!(ValidatedRect::new(0.0, 0.0, 10.0, f32::NAN).is_err());
    }

    #[test]
    fn test_validated_rect_rejects_infinity() {
        assert!(ValidatedRect::new(f32::INFINITY, 0.0, 10.0, 10.0).is_err());
        assert!(ValidatedRect::new(f32::NEG_INFINITY, 0.0, 10.0, 10.0).is_err());
        assert!(ValidatedRect::new(0.0, 0.0, f32::INFINITY, 10.0).is_err());
    }

    #[test]
    fn test_validated_rect_rejects_negative_dimensions() {
        let result = ValidatedRect::new(0.0, 0.0, -10.0, 10.0);
        assert!(matches!(result, Err(TaffyValidationError::NegativeDimension { .. })));

        let result = ValidatedRect::new(0.0, 0.0, 10.0, -5.0);
        assert!(matches!(result, Err(TaffyValidationError::NegativeDimension { .. })));
    }

    #[test]
    fn test_validated_rect_allows_negative_position() {
        // Negative x/y is valid (off-screen rendering)
        let rect = ValidatedRect::new(-50.0, -30.0, 100.0, 50.0);
        assert!(rect.is_ok());
        let rect = rect.unwrap();
        assert_eq!(rect.x(), -50.0);
        assert_eq!(rect.y(), -30.0);
    }

    #[test]
    fn test_validated_rect_clamps_extreme_values() {
        let rect = ValidatedRect::new(2_000_000.0, -2_000_000.0, 10.0, 10.0).unwrap();
        assert!(rect.x() <= MAX_COORD);
        assert!(rect.y() >= -MAX_COORD);
    }

    #[test]
    fn test_validated_rect_zero() {
        let rect = ValidatedRect::zero();
        assert_eq!(rect.x(), 0.0);
        assert_eq!(rect.y(), 0.0);
        assert_eq!(rect.width(), 0.0);
        assert_eq!(rect.height(), 0.0);
    }

    #[test]
    fn test_validated_rect_contains() {
        let rect = ValidatedRect::new(10.0, 10.0, 20.0, 20.0).unwrap();

        // Inside
        assert!(rect.contains(15.0, 15.0));
        assert!(rect.contains(20.0, 20.0));

        // On boundary
        assert!(rect.contains(10.0, 10.0));
        assert!(rect.contains(30.0, 30.0));

        // Outside
        assert!(!rect.contains(5.0, 15.0));
        assert!(!rect.contains(15.0, 5.0));
        assert!(!rect.contains(35.0, 15.0));
    }

    #[test]
    fn test_validated_rect_edges() {
        let rect = ValidatedRect::new(10.0, 20.0, 100.0, 50.0).unwrap();
        assert_eq!(rect.right(), 110.0);
        assert_eq!(rect.bottom(), 70.0);
    }
}
