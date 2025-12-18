//! Reactive programming primitives for StratoUI

use parking_lot::RwLock;
use smallvec::SmallVec;
use std::sync::Arc;
// Removed unused std::fmt::Debug import
use std::marker::PhantomData;

/// Effect function that runs when dependencies change
pub type EffectFn = Box<dyn Fn() + Send + Sync>;

/// Dependency tracking for reactive values
pub trait Reactive: Send + Sync {
    /// Track this reactive value as a dependency
    fn track(&self);

    /// Trigger updates for all dependents
    fn trigger(&self);
}

/// Computed value that derives from other reactive values
pub struct Computed<T: Clone + Send + Sync + 'static> {
    value: Arc<RwLock<Option<T>>>,
    compute_fn: Arc<dyn Fn() -> T + Send + Sync>,
    dependencies: Arc<RwLock<SmallVec<[Box<dyn Reactive>; 4]>>>,
}

impl<T: Clone + Send + Sync + 'static> Computed<T> {
    /// Create a new computed value
    pub fn new<F>(compute_fn: F) -> Self
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        Self {
            value: Arc::new(RwLock::new(None)),
            compute_fn: Arc::new(compute_fn),
            dependencies: Arc::new(RwLock::new(SmallVec::new())),
        }
    }

    /// Get the computed value, recomputing if necessary
    pub fn get(&self) -> T {
        let mut value = self.value.write();
        if value.is_none() {
            *value = Some((self.compute_fn)());
        }
        value.as_ref().unwrap().clone()
    }

    /// Invalidate the cached value
    pub fn invalidate(&self) {
        *self.value.write() = None;
    }

    /// Add a dependency
    pub fn add_dependency(&self, dep: Box<dyn Reactive>) {
        self.dependencies.write().push(dep);
    }
}

impl<T: Clone + Send + Sync + 'static> Reactive for Computed<T> {
    fn track(&self) {
        // Track this computed as a dependency
    }

    fn trigger(&self) {
        self.invalidate();
    }
}

/// Effect that runs when dependencies change
pub struct Effect {
    effect_fn: Arc<EffectFn>,
    dependencies: Arc<RwLock<SmallVec<[Box<dyn Reactive>; 4]>>>,
    active: Arc<RwLock<bool>>,
}

impl Effect {
    /// Create a new effect
    pub fn new<F>(effect_fn: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        let effect = Self {
            effect_fn: Arc::new(Box::new(effect_fn)),
            dependencies: Arc::new(RwLock::new(SmallVec::new())),
            active: Arc::new(RwLock::new(true)),
        };

        // Run the effect immediately
        effect.run();

        effect
    }

    /// Run the effect
    pub fn run(&self) {
        if *self.active.read() {
            (self.effect_fn)();
        }
    }

    /// Stop the effect
    pub fn stop(&self) {
        *self.active.write() = false;
    }

    /// Resume the effect
    pub fn resume(&self) {
        *self.active.write() = true;
        self.run();
    }

    /// Add a dependency
    pub fn add_dependency(&self, dep: Box<dyn Reactive>) {
        self.dependencies.write().push(dep);
    }
}

/// Memo for caching expensive computations
pub struct Memo<T: Clone + PartialEq + Send + Sync + 'static> {
    value: Arc<RwLock<Option<T>>>,
    compute_fn: Arc<dyn Fn() -> T + Send + Sync>,
    _phantom: PhantomData<T>,
}

impl<T: Clone + PartialEq + Send + Sync + 'static> Memo<T> {
    /// Create a new memo
    pub fn new<F>(compute_fn: F) -> Self
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        Self {
            value: Arc::new(RwLock::new(None)),
            compute_fn: Arc::new(compute_fn),
            _phantom: PhantomData,
        }
    }

    /// Get the memoized value
    pub fn get(&self) -> T {
        let mut value = self.value.write();
        if value.is_none() {
            *value = Some((self.compute_fn)());
        }
        value.as_ref().unwrap().clone()
    }

    /// Clear the memoized value
    pub fn clear(&self) {
        *self.value.write() = None;
    }
}

/// Watch for changes in a reactive value
pub struct Watch<T: Clone + Send + Sync + 'static> {
    value: Arc<RwLock<T>>,
    callbacks: Arc<RwLock<SmallVec<[Box<dyn Fn(&T) + Send + Sync>; 2]>>>,
}

impl<T: Clone + Send + Sync + 'static> Watch<T> {
    /// Create a new watch
    pub fn new(initial: T) -> Self {
        Self {
            value: Arc::new(RwLock::new(initial)),
            callbacks: Arc::new(RwLock::new(SmallVec::new())),
        }
    }

    /// Get the current value
    pub fn get(&self) -> T {
        self.value.read().clone()
    }

    /// Set a new value and trigger callbacks
    pub fn set(&self, value: T) {
        *self.value.write() = value.clone();
        let callbacks = self.callbacks.read();
        for callback in callbacks.iter() {
            callback(&value);
        }
    }

    /// Watch for changes
    pub fn on_change<F>(&self, callback: F)
    where
        F: Fn(&T) + Send + Sync + 'static,
    {
        self.callbacks.write().push(Box::new(callback));
    }
}

/// Batch multiple updates together
pub struct Batch {
    updates: Arc<RwLock<Vec<Box<dyn FnOnce() + Send>>>>,
}

impl Batch {
    /// Create a new batch
    pub fn new() -> Self {
        Self {
            updates: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Add an update to the batch
    pub fn add<F>(&self, update: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.updates.write().push(Box::new(update));
    }

    /// Execute all batched updates
    pub fn flush(&self) {
        let updates = std::mem::take(&mut *self.updates.write());
        for update in updates {
            update();
        }
    }
}

impl Default for Batch {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_computed() {
        let counter = Arc::new(RwLock::new(0));
        let counter_clone = counter.clone();

        let computed = Computed::new(move || *counter_clone.read() * 2);

        assert_eq!(computed.get(), 0);

        *counter.write() = 5;
        computed.invalidate();
        assert_eq!(computed.get(), 10);
    }

    #[test]
    fn test_watch() {
        use std::sync::atomic::{AtomicI32, Ordering};

        let watch = Watch::new(0);
        let received = Arc::new(AtomicI32::new(0));
        let received_clone = received.clone();

        watch.on_change(move |value| {
            received_clone.store(*value, Ordering::SeqCst);
        });

        watch.set(42);
        assert_eq!(received.load(Ordering::SeqCst), 42);
    }

    #[test]
    fn test_memo() {
        let call_count = Arc::new(RwLock::new(0));
        let call_count_clone = call_count.clone();

        let memo = Memo::new(move || {
            *call_count_clone.write() += 1;
            42
        });

        assert_eq!(memo.get(), 42);
        assert_eq!(memo.get(), 42); // Should not recompute
        assert_eq!(*call_count.read(), 1);

        memo.clear();
        assert_eq!(memo.get(), 42); // Recomputes after clear
        assert_eq!(*call_count.read(), 2);
    }
}
