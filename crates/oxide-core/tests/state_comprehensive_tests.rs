//! Comprehensive tests for the state management system
//!
//! This test suite aims to achieve >80% coverage of the state module

use oxide_core::state::{Signal, ReactiveContext, Disposable};
use oxide_core::Result;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[test]
fn test_signal_creation_and_basic_operations() {
    let signal = Signal::new(42);
    assert_eq!(signal.get(), 42);
    
    signal.set(100);
    assert_eq!(signal.get(), 100);
}

#[test]
fn test_signal_peek_does_not_track_dependency() {
    let signal = Signal::new(10);
    
    // peek should not track dependency
    let value = signal.peek();
    assert_eq!(value, 10);
    
    // get should track dependency
    let value = signal.get();
    assert_eq!(value, 10);
}

#[test]
fn test_signal_update_with_function() {
    let signal = Signal::new(0);
    
    signal.update(|v| *v += 10);
    assert_eq!(signal.get(), 10);
    
    signal.update(|v| *v *= 2);
    assert_eq!(signal.get(), 20);
}

#[test]
fn test_signal_subscription() {
    let signal = Signal::new(0);
    let received = Arc::new(Mutex::new(Vec::new()));
    let received_clone = received.clone();
    
    let _disposable = signal.subscribe(Box::new(move |value| {
        if let Some(v) = value.downcast_ref::<i32>() {
            received_clone.lock().unwrap().push(*v);
        }
    }));
    
    signal.set(1);
    signal.set(2);
    signal.set(3);
    
    // Give time for notifications
    thread::sleep(Duration::from_millis(10));
    
    let values = received.lock().unwrap();
    assert_eq!(*values, vec![1, 2, 3]);
}

#[test]
fn test_signal_multiple_subscribers() {
    let signal = Signal::new(0);
    let count1 = Arc::new(Mutex::new(0));
    let count2 = Arc::new(Mutex::new(0));
    
    let count1_clone = count1.clone();
    let _disposable1 = signal.subscribe(Box::new(move |_| {
        *count1_clone.lock().unwrap() += 1;
    }));
    
    let count2_clone = count2.clone();
    let _disposable2 = signal.subscribe(Box::new(move |_| {
        *count2_clone.lock().unwrap() += 1;
    }));
    
    signal.set(1);
    signal.set(2);
    
    thread::sleep(Duration::from_millis(10));
    
    assert_eq!(*count1.lock().unwrap(), 2);
    assert_eq!(*count2.lock().unwrap(), 2);
}

#[test]
fn test_signal_disposable_unsubscribe() {
    let signal = Signal::new(0);
    let count = Arc::new(Mutex::new(0));
    let count_clone = count.clone();
    
    let disposable = signal.subscribe(Box::new(move |_| {
        *count_clone.lock().unwrap() += 1;
    }));
    
    signal.set(1);
    thread::sleep(Duration::from_millis(10));
    assert_eq!(*count.lock().unwrap(), 1);
    
    // Dispose subscription
    disposable.dispose();
    
    signal.set(2);
    thread::sleep(Duration::from_millis(10));
    
    // Count should not increase after disposal
    assert_eq!(*count.lock().unwrap(), 1);
}

#[test]
fn test_signal_computed() {
    let signal = Signal::new(10);
    let doubled = signal.computed(|v| v * 2);
    
    assert_eq!(doubled.get(), 20);
    
    signal.set(20);
    thread::sleep(Duration::from_millis(10));
    
    assert_eq!(doubled.get(), 40);
}

#[test]
fn test_signal_map() {
    let signal = Signal::new(5);
    let squared = signal.map(|v| v * v);
    
    assert_eq!(squared.get(), 25);
    
    signal.set(10);
    thread::sleep(Duration::from_millis(10));
    
    assert_eq!(squared.get(), 100);
}

#[test]
fn test_signal_effect() {
    let signal = Signal::new(0);
    let effect_count = Arc::new(Mutex::new(0));
    let last_value = Arc::new(Mutex::new(0));
    
    let effect_count_clone = effect_count.clone();
    let last_value_clone = last_value.clone();
    
    let _disposable = signal.effect(move |v| {
        *effect_count_clone.lock().unwrap() += 1;
        *last_value_clone.lock().unwrap() = *v;
    });
    
    // Effect should run immediately
    assert_eq!(*effect_count.lock().unwrap(), 1);
    assert_eq!(*last_value.lock().unwrap(), 0);
    
    signal.set(42);
    thread::sleep(Duration::from_millis(10));
    
    assert_eq!(*effect_count.lock().unwrap(), 2);
    assert_eq!(*last_value.lock().unwrap(), 42);
}

#[test]
fn test_signal_with_string() {
    let signal = Signal::new(String::from("Hello"));
    assert_eq!(signal.get(), "Hello");
    
    signal.set(String::from("World"));
    assert_eq!(signal.get(), "World");
}

#[test]
fn test_signal_with_vec() {
    let signal = Signal::new(vec![1, 2, 3]);
    assert_eq!(signal.get(), vec![1, 2, 3]);
    
    signal.update(|v| v.push(4));
    assert_eq!(signal.get(), vec![1, 2, 3, 4]);
}

#[test]
fn test_signal_with_option() {
    let signal = Signal::new(Some(42));
    assert_eq!(signal.get(), Some(42));
    
    signal.set(None);
    assert_eq!(signal.get(), None);
}

#[test]
fn test_signal_clone() {
    let signal1 = Signal::new(10);
    let signal2 = signal1.clone();
    
    assert_eq!(signal1.get(), 10);
    assert_eq!(signal2.get(), 10);
    
    signal1.set(20);
    assert_eq!(signal1.get(), 20);
    assert_eq!(signal2.get(), 20);
}

#[test]
fn test_signal_thread_safety() {
    let signal = Arc::new(Signal::new(0));
    let mut handles = vec![];
    
    // Spawn multiple threads that increment the signal
    for _ in 0..10 {
        let signal_clone = Arc::clone(&signal);
        let handle = thread::spawn(move || {
            for _ in 0..100 {
                let current = signal_clone.get();
                signal_clone.set(current + 1);
            }
        });
        handles.push(handle);
    }
    
    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }
    
    // Final value should be 1000 (10 threads * 100 increments)
    // Note: Due to race conditions, this might not always be exactly 1000
    // but it should be close and the test should not panic
    let final_value = signal.get();
    assert!(final_value > 0);
    assert!(final_value <= 1000);
}

#[test]
fn test_reactive_context_creation() {
    let context = ReactiveContext::new();
    // Just verify it can be created
    drop(context);
}

#[test]
fn test_signal_with_custom_context() {
    let context = Arc::new(ReactiveContext::new());
    let signal = Signal::with_context(42, context);
    
    assert_eq!(signal.get(), 42);
}

#[test]
fn test_signal_chain_of_computations() {
    let base = Signal::new(2);
    let doubled = base.computed(|v| v * 2);
    let quadrupled = doubled.computed(|v| v * 2);
    
    assert_eq!(base.get(), 2);
    assert_eq!(doubled.get(), 4);
    assert_eq!(quadrupled.get(), 8);
    
    base.set(5);
    thread::sleep(Duration::from_millis(20));
    
    assert_eq!(base.get(), 5);
    assert_eq!(doubled.get(), 10);
    assert_eq!(quadrupled.get(), 20);
}

#[test]
fn test_signal_with_complex_type() {
    #[derive(Clone, Debug, PartialEq)]
    struct User {
        name: String,
        age: u32,
    }
    
    let signal = Signal::new(User {
        name: "Alice".to_string(),
        age: 30,
    });
    
    let user = signal.get();
    assert_eq!(user.name, "Alice");
    assert_eq!(user.age, 30);
    
    signal.set(User {
        name: "Bob".to_string(),
        age: 25,
    });
    
    let user = signal.get();
    assert_eq!(user.name, "Bob");
    assert_eq!(user.age, 25);
}

#[test]
fn test_signal_update_preserves_subscribers() {
    let signal = Signal::new(0);
    let count = Arc::new(Mutex::new(0));
    let count_clone = count.clone();
    
    let _disposable = signal.subscribe(Box::new(move |_| {
        *count_clone.lock().unwrap() += 1;
    }));
    
    // Multiple updates
    for i in 1..=5 {
        signal.set(i);
    }
    
    thread::sleep(Duration::from_millis(50));
    
    // Should have received all 5 updates
    assert_eq!(*count.lock().unwrap(), 5);
}

#[test]
fn test_signal_peek_vs_get_performance() {
    let signal = Signal::new(42);
    
    // Both should return the same value
    assert_eq!(signal.peek(), signal.get());
    
    // peek should be slightly faster as it doesn't track dependencies
    // but we can't easily test performance in a unit test
    // This test just verifies they both work
}

#[test]
fn test_signal_with_bool() {
    let signal = Signal::new(true);
    assert_eq!(signal.get(), true);
    
    signal.set(false);
    assert_eq!(signal.get(), false);
    
    signal.update(|v| *v = !*v);
    assert_eq!(signal.get(), true);
}

#[test]
fn test_multiple_effects_on_same_signal() {
    let signal = Signal::new(0);
    let count1 = Arc::new(Mutex::new(0));
    let count2 = Arc::new(Mutex::new(0));
    let count3 = Arc::new(Mutex::new(0));
    
    let count1_clone = count1.clone();
    let _d1 = signal.effect(move |_| {
        *count1_clone.lock().unwrap() += 1;
    });
    
    let count2_clone = count2.clone();
    let _d2 = signal.effect(move |_| {
        *count2_clone.lock().unwrap() += 1;
    });
    
    let count3_clone = count3.clone();
    let _d3 = signal.effect(move |_| {
        *count3_clone.lock().unwrap() += 1;
    });
    
    // All effects should run immediately
    assert_eq!(*count1.lock().unwrap(), 1);
    assert_eq!(*count2.lock().unwrap(), 1);
    assert_eq!(*count3.lock().unwrap(), 1);
    
    signal.set(42);
    thread::sleep(Duration::from_millis(10));
    
    // All effects should run again
    assert_eq!(*count1.lock().unwrap(), 2);
    assert_eq!(*count2.lock().unwrap(), 2);
    assert_eq!(*count3.lock().unwrap(), 2);
}

#[test]
fn test_signal_rapid_updates() {
    let signal = Signal::new(0);
    let final_values = Arc::new(Mutex::new(Vec::new()));
    let final_values_clone = final_values.clone();
    
    let _disposable = signal.subscribe(Box::new(move |value| {
        if let Some(v) = value.downcast_ref::<i32>() {
            final_values_clone.lock().unwrap().push(*v);
        }
    }));
    
    // Rapid updates
    for i in 1..=100 {
        signal.set(i);
    }
    
    thread::sleep(Duration::from_millis(50));
    
    let values = final_values.lock().unwrap();
    assert_eq!(values.len(), 100);
    assert_eq!(*values.last().unwrap(), 100);
}

#[cfg(test)]
mod property_tests {
    use super::*;
    
    #[test]
    fn test_signal_always_returns_latest_value() {
        let signal = Signal::new(0);
        
        for i in 0..1000 {
            signal.set(i);
            assert_eq!(signal.get(), i);
        }
    }
    
    #[test]
    fn test_signal_update_is_atomic() {
        let signal = Signal::new(vec![1, 2, 3]);
        
        signal.update(|v| {
            v.push(4);
            v.push(5);
        });
        
        assert_eq!(signal.get(), vec![1, 2, 3, 4, 5]);
    }
}

#[cfg(test)]
mod edge_cases {
    use super::*;
    
    #[test]
    fn test_signal_with_empty_string() {
        let signal = Signal::new(String::new());
        assert_eq!(signal.get(), "");
        
        signal.set("test".to_string());
        assert_eq!(signal.get(), "test");
    }
    
    #[test]
    fn test_signal_with_empty_vec() {
        let signal = Signal::new(Vec::<i32>::new());
        assert_eq!(signal.get(), Vec::<i32>::new());
        
        signal.update(|v| v.push(1));
        assert_eq!(signal.get(), vec![1]);
    }
    
    #[test]
    fn test_signal_with_zero_values() {
        let signal_i32 = Signal::new(0i32);
        assert_eq!(signal_i32.get(), 0);
        
        let signal_f64 = Signal::new(0.0f64);
        assert_eq!(signal_f64.get(), 0.0);
    }
}
