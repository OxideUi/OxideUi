//! OxideUI WebAssembly Demo
//!
//! This example demonstrates how to build and deploy OxideUI applications
//! to the web using WebAssembly.

use wasm_bindgen::prelude::*;
use web_sys::{window, HtmlCanvasElement, Performance};
use oxide_widgets::prelude::*;
use oxide_core::{state::Signal, types::Color};
use std::sync::Arc;
use std::cell::RefCell;
use std::rc::Rc;

/// Initialize the WASM module
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    // Set up panic hook for better error messages in the browser console
    console_error_panic_hook::set_once();
    
    // Initialize logging to browser console
    console_log::init_with_level(log::Level::Info)
        .map_err(|e| JsValue::from_str(&format!("Failed to init logger: {:?}", e)))?;
    
    log::info!("OxideUI WASM Demo initialized successfully");
    
    Ok(())
}

/// Main WASM application structure
#[wasm_bindgen]
pub struct WasmApp {
    canvas_id: String,
    counter: Arc<Signal<i32>>,
    animation_frame_id: Option<i32>,
    performance: Performance,
    last_frame_time: f64,
}

#[wasm_bindgen]
impl WasmApp {
    /// Create a new WASM application instance
    #[wasm_bindgen(constructor)]
    pub fn new(canvas_id: &str) -> Result<WasmApp, JsValue> {
        log::info!("Creating WasmApp with canvas_id: {}", canvas_id);
        
        let window = window().ok_or("No window found")?;
        let document = window.document().ok_or("No document found")?;
        let canvas = document
            .get_element_by_id(canvas_id)
            .ok_or_else(|| JsValue::from_str(&format!("Canvas '{}' not found", canvas_id)))?
            .dyn_into::<HtmlCanvasElement>()?;
        
        // Verify canvas dimensions
        let width = canvas.width();
        let height = canvas.height();
        log::info!("Canvas dimensions: {}x{}", width, height);
        
        let performance = window
            .performance()
            .ok_or("Performance API not available")?;
        
        Ok(WasmApp {
            canvas_id: canvas_id.to_string(),
            counter: Arc::new(Signal::new(0)),
            animation_frame_id: None,
            performance,
            last_frame_time: performance.now(),
        })
    }
    
    /// Get the current counter value
    #[wasm_bindgen(getter)]
    pub fn counter(&self) -> i32 {
        *self.counter.get()
    }
    
    /// Increment the counter
    #[wasm_bindgen]
    pub fn increment(&mut self) {
        let current = *self.counter.get();
        self.counter.set(current + 1);
        log::info!("Counter incremented to: {}", current + 1);
    }
    
    /// Decrement the counter
    #[wasm_bindgen]
    pub fn decrement(&mut self) {
        let current = *self.counter.get();
        self.counter.set(current - 1);
        log::info!("Counter decremented to: {}", current - 1);
    }
    
    /// Reset the counter
    #[wasm_bindgen]
    pub fn reset(&mut self) {
        self.counter.set(0);
        log::info!("Counter reset to 0");
    }
    
    /// Render a single frame
    #[wasm_bindgen]
    pub fn render(&mut self) -> Result<(), JsValue> {
        let current_time = self.performance.now();
        let delta_time = current_time - self.last_frame_time;
        self.last_frame_time = current_time;
        
        // Log FPS every second
        if (current_time as i32) % 1000 < 16 {
            let fps = 1000.0 / delta_time;
            log::debug!("FPS: {:.2}", fps);
        }
        
        // Rendering logic would go here
        // For now, we'll just update the DOM
        self.update_dom()?;
        
        Ok(())
    }
    
    /// Update the DOM with current state
    fn update_dom(&self) -> Result<(), JsValue> {
        let window = window().ok_or("No window")?;
        let document = window.document().ok_or("No document")?;
        
        // Update counter display
        if let Some(counter_element) = document.get_element_by_id("counter-value") {
            counter_element.set_inner_html(&format!("{}", *self.counter.get()));
        }
        
        Ok(())
    }
    
    /// Start the render loop
    #[wasm_bindgen]
    pub fn start_render_loop(&mut self) -> Result<(), JsValue> {
        log::info!("Starting render loop");
        
        // This would typically use requestAnimationFrame
        // For simplicity, we'll just render once
        self.render()?;
        
        Ok(())
    }
    
    /// Stop the render loop
    #[wasm_bindgen]
    pub fn stop_render_loop(&mut self) {
        if let Some(id) = self.animation_frame_id.take() {
            let window = window().expect("No window");
            window.cancel_animation_frame(id).ok();
            log::info!("Render loop stopped");
        }
    }
    
    /// Get performance metrics
    #[wasm_bindgen]
    pub fn get_metrics(&self) -> JsValue {
        let metrics = serde_json::json!({
            "counter": *self.counter.get(),
            "uptime_ms": self.performance.now(),
        });
        
        serde_wasm_bindgen::to_value(&metrics).unwrap_or(JsValue::NULL)
    }
}

/// Build the UI widget tree
fn build_ui(counter: Arc<Signal<i32>>) -> Box<dyn Widget> {
    Box::new(
        Container::new()
            .background(Color::rgb(0.15, 0.15, 0.20))
            .padding(40.0)
            .child(
                Column::new()
                    .spacing(20.0)
                    .main_axis_alignment(MainAxisAlignment::Center)
                    .cross_axis_alignment(CrossAxisAlignment::Center)
                    .children(vec![
                        // Title
                        Box::new(
                            Text::new("OxideUI WASM Demo")
                                .size(36.0)
                                .color(Color::rgb(1.0, 1.0, 1.0))
                                .weight(FontWeight::Bold)
                        ),
                        
                        // Subtitle
                        Box::new(
                            Text::new("A Rust UI Framework for the Web")
                                .size(18.0)
                                .color(Color::rgb(0.7, 0.7, 0.7))
                        ),
                        
                        // Spacer
                        Box::new(Container::new().height(20.0)),
                        
                        // Counter display
                        Box::new(
                            Container::new()
                                .background(Color::rgb(0.2, 0.2, 0.25))
                                .padding(20.0)
                                .border_radius(10.0)
                                .child(
                                    Text::new(format!("Count: {}", *counter.get()))
                                        .size(48.0)
                                        .color(Color::rgb(0.3, 0.8, 1.0))
                                        .weight(FontWeight::Bold)
                                )
                        ),
                        
                        // Button row
                        Box::new(
                            Row::new()
                                .spacing(15.0)
                                .main_axis_alignment(MainAxisAlignment::Center)
                                .children(vec![
                                    Box::new(
                                        Button::new("Decrement")
                                            .style(ButtonStyle::Secondary)
                                            .on_click({
                                                let counter = counter.clone();
                                                move || {
                                                    let current = *counter.get();
                                                    counter.set(current - 1);
                                                }
                                            })
                                    ),
                                    Box::new(
                                        Button::new("Reset")
                                            .style(ButtonStyle::Danger)
                                            .on_click({
                                                let counter = counter.clone();
                                                move || {
                                                    counter.set(0);
                                                }
                                            })
                                    ),
                                    Box::new(
                                        Button::new("Increment")
                                            .style(ButtonStyle::Primary)
                                            .on_click({
                                                let counter = counter.clone();
                                                move || {
                                                    let current = *counter.get();
                                                    counter.set(current + 1);
                                                }
                                            })
                                    ),
                                ])
                        ),
                        
                        // Info text
                        Box::new(
                            Text::new("Built with Rust + WebAssembly")
                                .size(14.0)
                                .color(Color::rgb(0.5, 0.5, 0.5))
                        ),
                    ])
            )
    )
}

/// Helper function to log to browser console
#[wasm_bindgen]
pub fn log_message(message: &str) {
    web_sys::console::log_1(&message.into());
}

/// Helper function to log errors to browser console
#[wasm_bindgen]
pub fn log_error(message: &str) {
    web_sys::console::error_1(&message.into());
}

/// Get browser information
#[wasm_bindgen]
pub fn get_browser_info() -> JsValue {
    let window = match window() {
        Some(w) => w,
        None => return JsValue::NULL,
    };
    
    let navigator = window.navigator();
    let info = serde_json::json!({
        "userAgent": navigator.user_agent().unwrap_or_default(),
        "language": navigator.language().unwrap_or_default(),
        "platform": navigator.platform().unwrap_or_default(),
        "cookieEnabled": navigator.cookie_enabled(),
    });
    
    serde_wasm_bindgen::to_value(&info).unwrap_or(JsValue::NULL)
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;
    
    wasm_bindgen_test_configure!(run_in_browser);
    
    #[wasm_bindgen_test]
    fn test_counter_increment() {
        let mut app = WasmApp::new("test-canvas").expect("Failed to create app");
        
        assert_eq!(app.counter(), 0);
        
        app.increment();
        assert_eq!(app.counter(), 1);
        
        app.increment();
        assert_eq!(app.counter(), 2);
    }
    
    #[wasm_bindgen_test]
    fn test_counter_decrement() {
        let mut app = WasmApp::new("test-canvas").expect("Failed to create app");
        
        app.increment();
        app.increment();
        assert_eq!(app.counter(), 2);
        
        app.decrement();
        assert_eq!(app.counter(), 1);
    }
    
    #[wasm_bindgen_test]
    fn test_counter_reset() {
        let mut app = WasmApp::new("test-canvas").expect("Failed to create app");
        
        app.increment();
        app.increment();
        app.increment();
        assert_eq!(app.counter(), 3);
        
        app.reset();
        assert_eq!(app.counter(), 0);
    }
}
