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
            Curve::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    -1.0 + (4.0 - 2.0 * t) * t
                }
            }
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

/// A handle to an animation task
pub type AnimationId = u64;

/// Animation status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationStatus {
    Playing,
    Paused,
    Completed,
}

/// Advanced Timeline for managing complex animations
pub struct Timeline {
    animations: Vec<Box<dyn Animation>>,
    status: AnimationStatus,
    start_time: Option<Instant>,
    elapsed: Duration,
    speed: f32,
}

impl Timeline {
    pub fn new() -> Self {
        Self {
            animations: Vec::new(),
            status: AnimationStatus::Paused,
            start_time: None,
            elapsed: Duration::ZERO,
            speed: 1.0,
        }
    }

    pub fn add(&mut self, anim: impl Animation + 'static) {
        self.animations.push(Box::new(anim));
    }

    pub fn play(&mut self) {
        if self.status != AnimationStatus::Playing {
            self.status = AnimationStatus::Playing;
            self.start_time = Some(Instant::now());
        }
    }

    pub fn pause(&mut self) {
        if self.status == AnimationStatus::Playing {
            self.status = AnimationStatus::Paused;
            if let Some(start) = self.start_time {
                self.elapsed += start.elapsed().mul_f32(self.speed);
                self.start_time = None;
            }
        }
    }

    pub fn update(&mut self) {
        if self.status == AnimationStatus::Playing {
            if let Some(start) = self.start_time {
                let current_elapsed = self.elapsed + start.elapsed().mul_f32(self.speed);

                let mut all_finished = true;
                for anim in &mut self.animations {
                    anim.update(current_elapsed);
                    if !anim.is_finished() {
                        all_finished = false;
                    }
                }

                if all_finished {
                    self.status = AnimationStatus::Completed;
                }
            }
        }
    }

    pub fn reset(&mut self) {
        self.status = AnimationStatus::Paused;
        self.start_time = None;
        self.elapsed = Duration::ZERO;
        for anim in &mut self.animations {
            anim.reset();
        }
    }
}

/// Trait for any animation
pub trait Animation: std::fmt::Debug {
    fn update(&mut self, elapsed: Duration);
    fn is_finished(&self) -> bool;
    fn reset(&mut self);
    fn duration(&self) -> Duration;
}

// Re-implementing KeyframeAnimation properly with state for is_finished
/// Animation that interpolates a value over time using a Signal target
#[derive(Debug)]
pub struct KeyframeAnimation<T: Tweenable + 'static + Send + Sync> {
    controller: AnimationController,
    tween: Tween<T>,
    target: strato_core::state::Signal<T>,
    finished: bool,
}

impl<T: Tweenable + std::fmt::Debug + Send + Sync> KeyframeAnimation<T> {
    pub fn new(duration: Duration, tween: Tween<T>, target: strato_core::state::Signal<T>) -> Self {
        Self {
            controller: AnimationController::new(duration),
            tween,
            target,
            finished: false,
        }
    }

    pub fn with_curve(mut self, curve: Curve) -> Self {
        self.controller = self.controller.with_curve(curve);
        self
    }
}

impl<T: Tweenable + std::fmt::Debug + Send + Sync> Animation for KeyframeAnimation<T> {
    fn update(&mut self, elapsed: Duration) {
        let d_secs = self.controller.duration.as_secs_f32();
        if d_secs == 0.0 {
            self.finished = true;
            return;
        }

        let t_secs = elapsed.as_secs_f32();
        let raw_t = t_secs / d_secs;

        self.finished = raw_t >= 1.0;

        let t = raw_t.clamp(0.0, 1.0);
        let curved_t = self.controller.curve.transform(t);
        let value = self.tween.transform(curved_t);

        self.target.set(value);
    }

    fn is_finished(&self) -> bool {
        self.finished
    }

    fn reset(&mut self) {
        self.finished = false;
        // Optionally reset value to start?
        // let value = self.tween.transform(0.0);
        // self.target.set(value);
    }

    fn duration(&self) -> Duration {
        self.controller.duration
    }
}

/// Run animations in sequence
#[derive(Debug)]
pub struct Sequence {
    animations: Vec<Box<dyn Animation>>,
}

impl Sequence {
    pub fn new(animations: Vec<Box<dyn Animation>>) -> Self {
        Self { animations }
    }
}

impl Animation for Sequence {
    fn update(&mut self, elapsed: Duration) {
        let mut time_so_far = Duration::ZERO;

        for anim in &mut self.animations {
            let duration = anim.duration();
            let anim_end_time = time_so_far + duration;

            if elapsed >= anim_end_time {
                // Ensure this animation is in its final state
                anim.update(duration);
            } else if elapsed >= time_so_far {
                // Currently active
                anim.update(elapsed - time_so_far);
            } else {
                // Future
                anim.update(Duration::ZERO);
            }

            time_so_far += duration;
        }
    }

    fn is_finished(&self) -> bool {
        if let Some(last) = self.animations.last() {
            last.is_finished()
        } else {
            true
        }
    }

    fn reset(&mut self) {
        for anim in &mut self.animations {
            anim.reset();
        }
    }

    fn duration(&self) -> Duration {
        self.animations.iter().map(|a| a.duration()).sum()
    }
}

/// Run animations in parallel
#[derive(Debug)]
pub struct Parallel {
    animations: Vec<Box<dyn Animation>>,
}

impl Parallel {
    pub fn new(animations: Vec<Box<dyn Animation>>) -> Self {
        Self { animations }
    }
}

impl Animation for Parallel {
    fn update(&mut self, elapsed: Duration) {
        for anim in &mut self.animations {
            anim.update(elapsed);
        }
    }

    fn is_finished(&self) -> bool {
        self.animations.iter().all(|a| a.is_finished())
    }

    fn reset(&mut self) {
        for anim in &mut self.animations {
            anim.reset();
        }
    }

    fn duration(&self) -> Duration {
        self.animations
            .iter()
            .map(|a| a.duration())
            .max()
            .unwrap_or(Duration::ZERO)
    }
}
