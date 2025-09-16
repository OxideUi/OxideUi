//! WebAssembly platform implementation

use crate::{Platform, PlatformError, Window, WindowBuilder, WindowId, WindowInner};
use oxide_core::event::Event;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Document, HtmlCanvasElement, Window as WebWindow};

/// Web platform implementation
pub struct WebPlatform {
    canvas: Option<HtmlCanvasElement>,
    window_id: WindowId,
}

impl WebPlatform {
    /// Create a new web platform
    pub fn new() -> Self {
        Self {
            canvas: None,
            window_id: 0,
        }
    }

    /// Get the web window
    fn web_window() -> Result<WebWindow, PlatformError> {
        web_sys::window()
            .ok_or_else(|| PlatformError::Wasm("Failed to get window".to_string()))
    }

    /// Get the document
    fn document() -> Result<Document, PlatformError> {
        Self::web_window()?
            .document()
            .ok_or_else(|| PlatformError::Wasm("Failed to get document".to_string()))
    }

    /// Create a canvas element
    fn create_canvas(builder: &WindowBuilder) -> Result<HtmlCanvasElement, PlatformError> {
        let document = Self::document()?;
        
        let canvas = document
            .create_element("canvas")
            .map_err(|e| PlatformError::Wasm(format!("Failed to create canvas: {:?}", e)))?
            .dyn_into::<HtmlCanvasElement>()
            .map_err(|_| PlatformError::Wasm("Failed to cast to HtmlCanvasElement".to_string()))?;

        canvas.set_width(builder.size.width as u32);
        canvas.set_height(builder.size.height as u32);
        canvas.set_id("oxide-ui-canvas");

        // Set canvas style
        let style = canvas.style();
        style.set_property("display", "block").ok();
        style.set_property("margin", "0 auto").ok();
        
        // Append to body
        document
            .body()
            .ok_or_else(|| PlatformError::Wasm("No body element".to_string()))?
            .append_child(&canvas)
            .map_err(|e| PlatformError::Wasm(format!("Failed to append canvas: {:?}", e)))?;

        Ok(canvas)
    }

    /// Setup event listeners
    fn setup_event_listeners(canvas: &HtmlCanvasElement) -> Result<(), PlatformError> {
        let canvas_clone = canvas.clone();
        
        // Mouse move
        let mouse_move = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
            // Handle mouse move
            let _x = event.offset_x();
            let _y = event.offset_y();
            // TODO: Dispatch to event handler
        }) as Box<dyn FnMut(_)>);
        
        canvas.add_event_listener_with_callback("mousemove", mouse_move.as_ref().unchecked_ref())
            .map_err(|e| PlatformError::Wasm(format!("Failed to add mousemove listener: {:?}", e)))?;
        mouse_move.forget();

        // Mouse down
        let mouse_down = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
            // Handle mouse down
            let _button = event.button();
            // TODO: Dispatch to event handler
        }) as Box<dyn FnMut(_)>);
        
        canvas.add_event_listener_with_callback("mousedown", mouse_down.as_ref().unchecked_ref())
            .map_err(|e| PlatformError::Wasm(format!("Failed to add mousedown listener: {:?}", e)))?;
        mouse_down.forget();

        // Mouse up
        let mouse_up = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
            // Handle mouse up
            let _button = event.button();
            // TODO: Dispatch to event handler
        }) as Box<dyn FnMut(_)>);
        
        canvas.add_event_listener_with_callback("mouseup", mouse_up.as_ref().unchecked_ref())
            .map_err(|e| PlatformError::Wasm(format!("Failed to add mouseup listener: {:?}", e)))?;
        mouse_up.forget();

        // Keyboard events
        let keydown = Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
            // Handle keydown
            let _key = event.key();
            // TODO: Dispatch to event handler
        }) as Box<dyn FnMut(_)>);
        
        Self::document()?
            .add_event_listener_with_callback("keydown", keydown.as_ref().unchecked_ref())
            .map_err(|e| PlatformError::Wasm(format!("Failed to add keydown listener: {:?}", e)))?;
        keydown.forget();

        let keyup = Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
            // Handle keyup
            let _key = event.key();
            // TODO: Dispatch to event handler
        }) as Box<dyn FnMut(_)>);
        
        Self::document()?
            .add_event_listener_with_callback("keyup", keyup.as_ref().unchecked_ref())
            .map_err(|e| PlatformError::Wasm(format!("Failed to add keyup listener: {:?}", e)))?;
        keyup.forget();

        // Resize observer
        let resize = Closure::wrap(Box::new(move || {
            // Handle resize
            // TODO: Dispatch resize event
        }) as Box<dyn FnMut()>);
        
        Self::web_window()?
            .add_event_listener_with_callback("resize", resize.as_ref().unchecked_ref())
            .map_err(|e| PlatformError::Wasm(format!("Failed to add resize listener: {:?}", e)))?;
        resize.forget();

        Ok(())
    }
}

impl Platform for WebPlatform {
    fn init() -> Result<Self, PlatformError> {
        // Set panic hook for better error messages
        console_error_panic_hook::set_once();
        
        Ok(Self::new())
    }

    fn create_window(&mut self, builder: WindowBuilder) -> Result<Window, PlatformError> {
        // In web, we typically have one canvas
        if self.canvas.is_some() {
            return Err(PlatformError::Wasm("Canvas already created".to_string()));
        }

        let canvas = Self::create_canvas(&builder)?;
        Self::setup_event_listeners(&canvas)?;

        // Set document title
        Self::document()?.set_title(&builder.title);

        let window_id = self.window_id;
        self.window_id += 1;
        
        self.canvas = Some(canvas.clone());

        Ok(Window {
            id: window_id,
            inner: WindowInner::Web(canvas),
        })
    }

    fn run_event_loop<F>(&mut self, callback: F) -> Result<(), PlatformError>
    where
        F: FnMut(Event) + 'static,
    {
        // In web, the event loop is handled by the browser
        // We use requestAnimationFrame for the render loop
        
        let window = Self::web_window()?;
        let callback = std::rc::Rc::new(std::cell::RefCell::new(callback));
        
        // Animation frame loop
        let f = std::rc::Rc::new(std::cell::RefCell::new(None));
        let g = f.clone();
        
        *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
            // Request next frame
            request_animation_frame(f.borrow().as_ref().unwrap());
            
            // Handle frame update
            // TODO: Dispatch update event
        }) as Box<dyn FnMut()>));
        
        request_animation_frame(g.borrow().as_ref().unwrap());
        
        Ok(())
    }

    fn request_redraw(&self, _window_id: WindowId) {
        // In web, we use requestAnimationFrame
        // This is handled in the event loop
    }

    fn exit(&mut self) {
        // Can't really exit in web
        // Could navigate away or close tab
    }
}

/// Request animation frame helper
fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    web_sys::window()
        .unwrap()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .unwrap();
}
