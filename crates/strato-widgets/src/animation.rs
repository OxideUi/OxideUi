//! Animation system for widgets
use std::time::{Duration, Instant};
use strato_core::types::Color;

/// Animation curve
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Curve {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
}

impl Curve {
    /// Calculate progress value (0.0 to 1.0) based on curve
    pub fn transform(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Curve::Linear => t,
            Curve::EaseIn => t * t,
            Curve::EaseOut => t * (2.0 - t),
            Curve::EaseInOut => if t < 0.5 { 2.0 * t * t } else { -1.0 + (4.0 - 2.0 * t) * t },
        }
    }
}

/// Controls an animation's state and progress
#[derive(Debug, Clone)]
pub struct AnimationController {
    duration: Duration,
    start_time: Option<Instant>,
    curve: Curve,
    is_repeating: bool,
    is_reversed: bool,
}

impl AnimationController {
    /// Create a new controller
    pub fn new(duration: Duration) -> Self {
        Self {
            duration,
            start_time: None,
            curve: Curve::Linear,
            is_repeating: false,
            is_reversed: false,
        }
    }

    /// Set animation curve
    pub fn with_curve(mut self, curve: Curve) -> Self {
        self.curve = curve;
        self
    }

    /// Set repeating
    pub fn loop_forever(mut self) -> Self {
        self.is_repeating = true;
        self
    }

    /// Start the animation
    pub fn start(&mut self) {
        self.start_time = Some(Instant::now());
    }

    /// Reset the animation
    pub fn reset(&mut self) {
        self.start_time = None;
    }

    /// Get current progress value (0.0 to 1.0)
    pub fn value(&self) -> f32 {
        let Some(start) = self.start_time else {
            return 0.0;
        };

        let elapsed = start.elapsed().as_secs_f32();
        let duration = self.duration.as_secs_f32();
        
        if duration == 0.0 {
            return 1.0;
        }

        let raw_t = elapsed / duration;
        
        let t = if self.is_repeating {
            let cycle = raw_t % 2.0;
            if cycle > 1.0 {
                2.0 - cycle
            } else {
                cycle
            }
        } else {
            raw_t.clamp(0.0, 1.0)
        };

        self.curve.transform(t)
    }

    /// Check if animation is finished
    pub fn is_completed(&self) -> bool {
        if self.is_repeating {
            return false;
        }
        if let Some(start) = self.start_time {
            start.elapsed() >= self.duration
        } else {
            false
        }
    }
}

/// Tween interface for interpolating values
pub trait Tweenable: Copy {
    fn lerp(start: Self, end: Self, t: f32) -> Self;
}

impl Tweenable for f32 {
    fn lerp(start: Self, end: Self, t: f32) -> Self {
        start + (end - start) * t
    }
}

impl Tweenable for Color {
    fn lerp(start: Self, end: Self, t: f32) -> Self {
        Color::rgba(
            f32::lerp(start.r, end.r, t),
            f32::lerp(start.g, end.g, t),
            f32::lerp(start.b, end.b, t),
            f32::lerp(start.a, end.a, t),
        )
    }
}

/// Simple tween object
#[derive(Debug, Clone, Copy)]
pub struct Tween<T: Tweenable> {
    pub begin: T,
    pub end: T,
}

impl<T: Tweenable> Tween<T> {
    pub fn new(begin: T, end: T) -> Self {
        Self { begin, end }
    }

    pub fn transform(&self, t: f32) -> T {
        T::lerp(self.begin, self.end, t)
    }
}
