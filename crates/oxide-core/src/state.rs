//! State management system for OxideUI
//!
//! Provides reactive state primitives similar to signals/observers pattern

use std::sync::Arc;
use std::any::Any;
// Removed unused std::fmt::Debug import
use parking_lot::RwLock;
use dashmap::DashMap;
use smallvec::SmallVec;

/// Unique identifier for state values
pub type StateId = slotmap::DefaultKey;

/// Callback function triggered on state changes
pub type StateCallback = Box<dyn Fn(&dyn Any) + Send + Sync>;

/// Enhanced signal with computed values and effects
pub struct Signal<T: Clone + Send + Sync + 'static> {
    value: Arc<RwLock<T>>,
    subscribers: Arc<RwLock<SmallVec<[StateCallback; 4]>>>,
    computed_signals: Arc<RwLock<Vec<Box<dyn Fn(&T) -> Box<dyn Any + Send + Sync> + Send + Sync>>>>,
}

impl<T: Clone + Send + Sync + 'static> Signal<T> {
    /// Create a new signal with initial value
    pub fn new(initial: T) -> Self {
        Self {
            value: Arc::new(RwLock::new(initial)),
            subscribers: Arc::new(RwLock::new(SmallVec::new())),
            computed_signals: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Get current value
    pub fn get(&self) -> T {
        self.value.read().clone()
    }

    /// Set new value and notify subscribers
    pub fn set(&self, value: T) {
        *self.value.write() = value.clone();
        self.notify(&value);
    }

    /// Update value with a function
    pub fn update(&self, f: impl FnOnce(&mut T)) {
        let mut guard = self.value.write();
        f(&mut *guard);
        let value = guard.clone();
        drop(guard);
        self.notify(&value);
    }

    /// Subscribe to value changes
    pub fn subscribe(&self, callback: StateCallback) {
        self.subscribers.write().push(callback);
    }

    /// Create a computed signal that derives from this signal
    pub fn computed<U, F>(&self, f: F) -> Signal<U>
    where
        U: Clone + Send + Sync + 'static,
        F: Fn(&T) -> U + Send + Sync + 'static,
    {
        let computed = Signal::new(f(&self.get()));
        let computed_clone = computed.clone();
        
        self.subscribe(Box::new(move |value: &dyn Any| {
            if let Some(typed_value) = value.downcast_ref::<T>() {
                computed_clone.set(f(typed_value));
            }
        }));
        
        computed
    }

    /// Create an effect that runs when the signal changes
    pub fn effect<F>(&self, f: F)
    where
        F: Fn(&T) + Send + Sync + 'static,
    {
        self.subscribe(Box::new(move |value: &dyn Any| {
            if let Some(typed_value) = value.downcast_ref::<T>() {
                f(typed_value);
            }
        }));
    }

    /// Notify all subscribers
    fn notify(&self, value: &T) {
        let subscribers = self.subscribers.read();
        for callback in subscribers.iter() {
            callback(value as &dyn Any);
        }
    }
}

impl<T: Clone + Send + Sync + 'static> Clone for Signal<T> {
    fn clone(&self) -> Self {
        Self {
            value: Arc::clone(&self.value),
            subscribers: Arc::clone(&self.subscribers),
            computed_signals: Arc::clone(&self.computed_signals),
        }
    }
}

/// Component state trait
pub trait State: Any + Send + Sync {
    /// Get the state as Any for downcasting
    fn as_any(&self) -> &dyn Any;
    
    /// Clone the state
    fn clone_state(&self) -> Box<dyn State>;
}

/// Global state manager
pub struct StateManager {
    states: DashMap<String, Box<dyn State>>,
}

impl StateManager {
    /// Create a new state manager
    pub fn new() -> Self {
        Self {
            states: DashMap::new(),
        }
    }

    /// Register a state with a key
    pub fn register<T: State + Clone + 'static>(&self, key: String, state: T) {
        self.states.insert(key, Box::new(state));
    }

    /// Get a state by key
    pub fn get<T: State + Clone + 'static>(&self, key: &str) -> Option<Box<T>> {
        self.states.get(key).and_then(|state| {
            state.as_any().downcast_ref::<T>().map(|s| {
                Box::new(s.clone())
            })
        })
    }

    /// Update a state by key
    pub fn update<T: State + Clone + 'static>(&self, key: &str, f: impl FnOnce(&mut T)) {
        if let Some(mut entry) = self.states.get_mut(key) {
            if let Some(state) = entry.as_any().downcast_ref::<T>() {
                let mut cloned = state.clone();
                f(&mut cloned);
                *entry = Box::new(cloned);
            }
        }
    }
}

impl Default for StateManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal() {
        let signal = Signal::new(42);
        assert_eq!(signal.get(), 42);
        
        signal.set(100);
        assert_eq!(signal.get(), 100);
    }

    #[test]
    fn test_signal_subscribe() {
        use std::sync::atomic::{AtomicI32, Ordering};
        
        let signal = Signal::new(0);
        let received = Arc::new(AtomicI32::new(0));
        let received_clone = received.clone();
        
        signal.subscribe(Box::new(move |value| {
            if let Some(v) = value.downcast_ref::<i32>() {
                received_clone.store(*v, Ordering::SeqCst);
            }
        }));
        
        signal.set(42);
        assert_eq!(received.load(Ordering::SeqCst), 42);
    }
}
