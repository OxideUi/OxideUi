//! Advanced reactive state management system for OxideUI
//!
//! Provides reactive state primitives with signals, stores, computed values,
//! effects, and automatic dependency tracking similar to modern reactive frameworks

use std::sync::Arc;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use parking_lot::{RwLock, Mutex};
use dashmap::DashMap;
use smallvec::SmallVec;
use std::sync::atomic::{AtomicU64, Ordering};

/// Unique identifier for state values
pub type StateId = slotmap::DefaultKey;

/// Unique identifier for reactive computations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ComputationId(u64);

impl ComputationId {
    fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

/// Callback function triggered on state changes
pub type StateCallback = Box<dyn Fn(&dyn Any) + Send + Sync>;

/// Effect function that can be disposed
pub type EffectFn = Box<dyn Fn() + Send + Sync>;

/// Disposable handle for effects and subscriptions
pub struct Disposable {
    dispose_fn: Box<dyn FnOnce() + Send>,
}

impl Disposable {
    pub fn new(dispose_fn: impl FnOnce() + Send + 'static) -> Self {
        Self {
            dispose_fn: Box::new(dispose_fn),
        }
    }

    pub fn dispose(self) {
        (self.dispose_fn)();
    }
}

/// Reactive context for tracking dependencies
#[derive(Default)]
pub struct ReactiveContext {
    current_computation: Arc<Mutex<Option<ComputationId>>>,
    dependencies: Arc<RwLock<HashMap<ComputationId, Vec<StateId>>>>,
    dependents: Arc<RwLock<HashMap<StateId, Vec<ComputationId>>>>,
}

impl ReactiveContext {
    pub fn new() -> Self {
        Self::default()
    }

    /// Track a dependency for the current computation
    pub fn track_dependency(&self, state_id: StateId) {
        if let Some(computation_id) = *self.current_computation.lock() {
            self.dependencies
                .write()
                .entry(computation_id)
                .or_default()
                .push(state_id);
            
            self.dependents
                .write()
                .entry(state_id)
                .or_default()
                .push(computation_id);
        }
    }

    /// Run a computation with dependency tracking
    pub fn run_with_tracking<T>(&self, computation_id: ComputationId, f: impl FnOnce() -> T) -> T {
        let _guard = ComputationGuard::new(self, computation_id);
        f()
    }

    /// Invalidate all computations that depend on a state
    pub fn invalidate_dependents(&self, state_id: StateId) {
        if let Some(dependents) = self.dependents.read().get(&state_id) {
            for &computation_id in dependents {
                // Trigger recomputation
                self.recompute(computation_id);
            }
        }
    }

    fn recompute(&self, _computation_id: ComputationId) {
        // Implementation for recomputing dependent values
        // This would trigger the recomputation of computed signals and effects
    }
}

/// RAII guard for computation tracking
struct ComputationGuard<'a> {
    context: &'a ReactiveContext,
    previous: Option<ComputationId>,
}

impl<'a> ComputationGuard<'a> {
    fn new(context: &'a ReactiveContext, computation_id: ComputationId) -> Self {
        let previous = context.current_computation.lock().replace(computation_id);
        Self { context, previous }
    }
}

impl<'a> Drop for ComputationGuard<'a> {
    fn drop(&mut self) {
        *self.context.current_computation.lock() = self.previous;
    }
}

/// Enhanced signal with automatic dependency tracking
pub struct Signal<T: Clone + Send + Sync + 'static> {
    id: StateId,
    value: Arc<RwLock<T>>,
    subscribers: Arc<RwLock<SmallVec<[StateCallback; 4]>>>,
    context: Arc<ReactiveContext>,
}

impl<T: Clone + Send + Sync + 'static> Signal<T> {
    /// Create a new signal with initial value
    pub fn new(initial: T) -> Self {
        Self::with_context(initial, Arc::new(ReactiveContext::new()))
    }

    /// Create a new signal with a specific reactive context
    pub fn with_context(initial: T, context: Arc<ReactiveContext>) -> Self {
        use slotmap::SlotMap;
        static mut SLOT_MAP: Option<SlotMap<StateId, ()>> = None;
        static INIT: std::sync::Once = std::sync::Once::new();
        
        INIT.call_once(|| unsafe {
            SLOT_MAP = Some(SlotMap::new());
        });
        
        let id = unsafe {
            SLOT_MAP.as_mut().unwrap().insert(())
        };

        Self {
            id,
            value: Arc::new(RwLock::new(initial)),
            subscribers: Arc::new(RwLock::new(SmallVec::new())),
            context,
        }
    }

    /// Get current value and track dependency
    pub fn get(&self) -> T {
        self.context.track_dependency(self.id);
        self.value.read().clone()
    }

    /// Get current value without tracking dependency
    pub fn peek(&self) -> T {
        self.value.read().clone()
    }

    /// Set new value and notify subscribers
    pub fn set(&self, value: T) {
        {
            let mut guard = self.value.write();
            *guard = value.clone();
        }
        self.notify(&value);
        self.context.invalidate_dependents(self.id);
    }

    /// Update value with a function
    pub fn update(&self, f: impl FnOnce(&mut T)) {
        let value = {
            let mut guard = self.value.write();
            f(&mut *guard);
            guard.clone()
        };
        self.notify(&value);
        self.context.invalidate_dependents(self.id);
    }

    /// Subscribe to value changes
    pub fn subscribe(&self, callback: StateCallback) -> Disposable {
        let subscribers = Arc::clone(&self.subscribers);
        let callback_id = {
            let mut subs = subscribers.write();
            let id = subs.len();
            subs.push(callback);
            id
        };

        Disposable::new(move || {
            // Remove callback by replacing with no-op
            if let Some(callback) = subscribers.write().get_mut(callback_id) {
                *callback = Box::new(|_| {});
            }
        })
    }

    /// Create a computed signal that derives from this signal
    pub fn computed<U, F>(&self, f: F) -> Signal<U>
    where
        U: Clone + Send + Sync + 'static,
        F: Fn(&T) -> U + Send + Sync + 'static,
    {
        let computation_id = ComputationId::new();
        let computed = Signal::with_context(
            self.context.run_with_tracking(computation_id, || f(&self.get())),
            Arc::clone(&self.context),
        );
        
        let computed_clone = computed.clone();
        let f = Arc::new(f);
        
        self.subscribe(Box::new(move |value: &dyn Any| {
            if let Some(typed_value) = value.downcast_ref::<T>() {
                let new_value = f(typed_value);
                computed_clone.set(new_value);
            }
        }));
        
        computed
    }

    /// Create an effect that runs when the signal changes
    pub fn effect<F>(&self, f: F) -> Disposable
    where
        F: Fn(&T) + Send + Sync + 'static,
    {
        // Run effect immediately
        f(&self.get());
        
        // Subscribe to future changes
        self.subscribe(Box::new(move |value: &dyn Any| {
            if let Some(typed_value) = value.downcast_ref::<T>() {
                f(typed_value);
            }
        }))
    }

    /// Create a derived signal that transforms this signal's value
    pub fn map<U, F>(&self, f: F) -> Signal<U>
    where
        U: Clone + Send + Sync + 'static,
        F: Fn(&T) -> U + Send + Sync + 'static,
    {
        self.computed(f)
    }

    /// Filter signal updates based on a predicate
    pub fn filter<F>(&self, predicate: F) -> Signal<Option<T>>
    where
        F: Fn(&T) -> bool + Send + Sync + 'static,
    {
        self.computed(move |value| {
            if predicate(value) {
                Some(value.clone())
            } else {
                None
            }
        })
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
            id: self.id,
            value: Arc::clone(&self.value),
            subscribers: Arc::clone(&self.subscribers),
            context: Arc::clone(&self.context),
        }
    }
}

/// Store for managing multiple related state values
pub struct Store {
    states: DashMap<String, Box<dyn Any + Send + Sync>>,
    context: Arc<ReactiveContext>,
}

impl Store {
    /// Create a new store
    pub fn new() -> Self {
        Self {
            states: DashMap::new(),
            context: Arc::new(ReactiveContext::new()),
        }
    }

    /// Add a signal to the store
    pub fn add_signal<T: Clone + Send + Sync + 'static>(&self, key: &str, initial: T) -> Signal<T> {
        let signal = Signal::with_context(initial, Arc::clone(&self.context));
        self.states.insert(key.to_string(), Box::new(signal.clone()));
        signal
    }

    /// Get a signal from the store
    pub fn get_signal<T: Clone + Send + Sync + 'static>(&self, key: &str) -> Option<Signal<T>> {
        self.states.get(key).and_then(|entry| {
            entry.value().downcast_ref::<Signal<T>>().cloned()
        })
    }

    /// Create a computed value that depends on multiple signals in the store
    pub fn computed<T, F>(&self, f: F) -> Signal<T>
    where
        T: Clone + Send + Sync + 'static,
        F: Fn(&Store) -> T + Send + Sync + 'static,
    {
        let computation_id = ComputationId::new();
        let initial_value = self.context.run_with_tracking(computation_id, || f(self));
        
        Signal::with_context(initial_value, Arc::clone(&self.context))
    }

    /// Remove a signal from the store
    pub fn remove(&self, key: &str) -> bool {
        self.states.remove(key).is_some()
    }

    /// Clear all signals from the store
    pub fn clear(&self) {
        self.states.clear();
    }

    /// Get the number of signals in the store
    pub fn len(&self) -> usize {
        self.states.len()
    }

    /// Check if the store is empty
    pub fn is_empty(&self) -> bool {
        self.states.is_empty()
    }
}

impl Default for Store {
    fn default() -> Self {
        Self::new()
    }
}

/// Batch multiple state updates to minimize notifications
pub struct Batch {
    updates: Vec<Box<dyn FnOnce() + Send>>,
}

impl Batch {
    /// Create a new batch
    pub fn new() -> Self {
        Self {
            updates: Vec::new(),
        }
    }

    /// Add an update to the batch
    pub fn add<F>(&mut self, update: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.updates.push(Box::new(update));
    }

    /// Execute all updates in the batch
    pub fn execute(self) {
        for update in self.updates {
            update();
        }
    }
}

impl Default for Batch {
    fn default() -> Self {
        Self::new()
    }
}

/// Global reactive context for the application
static GLOBAL_CONTEXT: std::sync::OnceLock<Arc<ReactiveContext>> = std::sync::OnceLock::new();

/// Get the global reactive context
pub fn global_context() -> Arc<ReactiveContext> {
    GLOBAL_CONTEXT
        .get_or_init(|| Arc::new(ReactiveContext::new()))
        .clone()
}

/// Create a signal with the global context
pub fn signal<T: Clone + Send + Sync + 'static>(initial: T) -> Signal<T> {
    Signal::with_context(initial, global_context())
}

/// Create a computed signal with the global context
pub fn computed<T, F>(f: F) -> Signal<T>
where
    T: Clone + Send + Sync + 'static,
    F: Fn() -> T + Send + Sync + 'static,
{
    let computation_id = ComputationId::new();
    let context = global_context();
    let initial_value = context.run_with_tracking(computation_id, f);
    
    Signal::with_context(initial_value, context)
}

/// Create an effect with the global context
pub fn effect<F>(f: F) -> Disposable
where
    F: Fn() + Send + Sync + 'static,
{
    // Run effect immediately
    f();
    
    // Return a disposable that does nothing for now
    // In a real implementation, this would track the effect for disposal
    Disposable::new(|| {})
}

/// State trait for type-erased state management
pub trait State: Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn clone_state(&self) -> Box<dyn State>;
}

impl<T: Clone + Send + Sync + 'static> State for T {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_state(&self) -> Box<dyn State> {
        Box::new(self.clone())
    }
}

/// State manager for managing multiple state values
pub struct StateManager {
    states: slotmap::SlotMap<StateId, Box<dyn State>>,
}

impl StateManager {
    /// Create a new state manager
    pub fn new() -> Self {
        Self {
            states: slotmap::SlotMap::new(),
        }
    }

    /// Register a new state value
    pub fn register<T: State + 'static>(&mut self, state: T) -> StateId {
        self.states.insert(Box::new(state))
    }

    /// Get a state value by ID
    pub fn get<T: 'static>(&self, id: StateId) -> Option<&T> {
        self.states.get(id)?.as_any().downcast_ref()
    }

    /// Update a state value
    pub fn update<T: State + 'static>(&mut self, id: StateId, new_state: T) -> bool {
        if let Some(state) = self.states.get_mut(id) {
            *state = Box::new(new_state);
            true
        } else {
            false
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
    use std::sync::atomic::{AtomicI32, Ordering};

    #[test]
    fn test_signal_basic() {
        let signal = Signal::new(42);
        assert_eq!(signal.get(), 42);
        
        signal.set(100);
        assert_eq!(signal.get(), 100);
    }

    #[test]
    fn test_signal_subscribe() {
        let signal = Signal::new(0);
        let counter = Arc::new(AtomicI32::new(0));
        let counter_clone = Arc::clone(&counter);
        
        let _disposable = signal.subscribe(Box::new(move |value: &dyn Any| {
            if let Some(&val) = value.downcast_ref::<i32>() {
                counter_clone.store(val, Ordering::Relaxed);
            }
        }));
        
        signal.set(42);
        assert_eq!(counter.load(Ordering::Relaxed), 42);
    }

    #[test]
    fn test_computed_signal() {
        let base = Signal::new(10);
        let doubled = base.computed(|&x| x * 2);
        
        assert_eq!(doubled.get(), 20);
        
        base.set(15);
        assert_eq!(doubled.get(), 30);
    }

    #[test]
    fn test_store() {
        let store = Store::new();
        let counter = store.add_signal("counter", 0);
        let name = store.add_signal("name", "test".to_string());
        
        assert_eq!(counter.get(), 0);
        assert_eq!(name.get(), "test");
        
        counter.set(42);
        assert_eq!(store.get_signal::<i32>("counter").unwrap().get(), 42);
    }

    #[test]
    fn test_batch_updates() {
        let signal1 = Signal::new(0);
        let signal2 = Signal::new(0);
        
        let mut batch = Batch::new();
        batch.add(move || signal1.set(10));
        batch.add(move || signal2.set(20));
        
        batch.execute();
        
        assert_eq!(signal1.get(), 10);
        assert_eq!(signal2.get(), 20);
    }

    #[test]
    fn test_signal_map() {
        let signal = Signal::new(5);
        let mapped = signal.map(|&x| x.to_string());
        
        assert_eq!(mapped.get(), "5");
        
        signal.set(10);
        assert_eq!(mapped.get(), "10");
    }

    #[test]
    fn test_signal_filter() {
        let signal = Signal::new(5);
        let filtered = signal.filter(|&x| x > 10);
        
        assert_eq!(filtered.get(), None);
        
        signal.set(15);
        assert_eq!(filtered.get(), Some(15));
    }
}
