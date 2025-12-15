//! Event loop management

use std::cell::RefCell;
use std::time::{Duration, Instant};
use oxide_core::event::{Event, MouseButton, MouseEvent, KeyboardEvent, KeyCode, Modifiers, WindowEvent};
use std::sync::Arc;
use std::rc::Rc;
use winit::window::Window;
use crate::Application;

/// Custom event for the event loop
#[derive(Debug)]
pub struct CustomEvent {
    pub event: Event,
}

/// Application state for managing event loop state safely
struct AppState {
    window_created: bool,
    winit_window: Option<Arc<Window>>,
    drawing_system: Option<oxide_renderer::gpu::DrawingSystem>,  // New GPU renderer
    renderer_initialized: bool,
    needs_redraw: bool,
    last_update: Instant,
    app: Option<Application>,
    cursor_position: winit::dpi::PhysicalPosition<f64>,
    scale_factor: f64,
}

impl AppState {
    fn new() -> Self {
        Self {
            window_created: false,
            winit_window: None,
            drawing_system: None,
            renderer_initialized: false,
            needs_redraw: false,
            last_update: Instant::now(),
            app: None,
            cursor_position: winit::dpi::PhysicalPosition::new(0.0, 0.0),
            scale_factor: 1.0,
        }
    }
}

/// Event loop wrapper with cross-platform support
pub struct EventLoop {
    #[cfg(not(target_arch = "wasm32"))]
    inner: winit::event_loop::EventLoop<CustomEvent>,
    #[cfg(target_arch = "wasm32")]
    _phantom: std::marker::PhantomData<()>,
}

impl EventLoop {
    /// Create a new event loop
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new() -> Result<Self, EventLoopError> {
        use winit::event_loop::EventLoopBuilder;
        
        let inner = EventLoopBuilder::with_user_event()
            .build()
            .map_err(|_| EventLoopError::CreationFailed)?;
        
        Ok(Self { inner })
    }
    
    #[cfg(target_arch = "wasm32")]
    pub fn new() -> Result<Self, EventLoopError> {
        Ok(Self {
            _phantom: std::marker::PhantomData,
        })
    }

    /// Create an event loop proxy for sending custom events
    pub fn create_proxy(&self) -> EventLoopProxy {
        #[cfg(not(target_arch = "wasm32"))]
        {
            EventLoopProxy {
                inner: self.inner.create_proxy(),
            }
        }
        
        #[cfg(target_arch = "wasm32")]
        {
            let (sender, _) = mpsc::channel();
            EventLoopProxy {
                sender,
            }
        }
    }

    /// Run the event loop with basic event handling
    #[cfg(not(target_arch = "wasm32"))]
    pub fn run<F>(self, mut handler: F) -> !
    where
        F: FnMut(Event) + 'static,
    {
        use winit::event::{Event as WinitEvent, WindowEvent as WinitWindowEvent};
        use winit::event_loop::ControlFlow;
        
        let mut last_update = Instant::now();
        let mut cursor_position = winit::dpi::PhysicalPosition::new(0.0, 0.0);
        let mut scale_factor = 1.0;
        
        self.inner.run(move |event, elwt| {
            match event {
                WinitEvent::WindowEvent { event, .. } => {
                    match event {
                        WinitWindowEvent::CloseRequested => {
                            elwt.exit();
                        }
                        WinitWindowEvent::RedrawRequested => {
                            // Handle redraw - emit a custom redraw event
                            handler(Event::Window(WindowEvent::Resize { width: 0, height: 0 }));
                        }
                        WinitWindowEvent::CursorMoved { position, device_id, .. } => {
                            cursor_position = position;
                            if let Some(oxide_event) = convert_window_event(
                                WinitWindowEvent::CursorMoved { position, device_id },
                                cursor_position,
                                scale_factor
                            ) {
                                handler(oxide_event);
                            }
                        }
                        WinitWindowEvent::ScaleFactorChanged { scale_factor: sf, inner_size_writer } => {
                            scale_factor = sf;
                            if let Some(oxide_event) = convert_window_event(
                                WinitWindowEvent::ScaleFactorChanged { scale_factor: sf, inner_size_writer },
                                cursor_position,
                                scale_factor
                            ) {
                                handler(oxide_event);
                            }
                        }
                        _ => {
                            if let Some(oxide_event) = convert_window_event(event, cursor_position, scale_factor) {
                                handler(oxide_event);
                            }
                        }
                    }
                }
                WinitEvent::AboutToWait => {
                    // Implement frame rate limiting
                    let now = Instant::now();
                    let frame_time = Duration::from_millis(16); // ~60 FPS
                    
                    if now.duration_since(last_update) >= frame_time {
                        last_update = now;
                        elwt.set_control_flow(ControlFlow::Poll);
                    } else {
                        elwt.set_control_flow(ControlFlow::WaitUntil(last_update + frame_time));
                    }
                }
                WinitEvent::UserEvent(custom) => {
                    handler(Event::Custom(Arc::new(custom.event)));
                }
                _ => {}
            }
        }).expect("Event loop failed");
        
        // This should never be reached, but if it is, exit the process
        std::process::exit(0);
    }

    /// Run the event loop with a window
    #[cfg(not(target_arch = "wasm32"))]
    pub fn run_with_window<F>(
        self,
        window_builder: crate::WindowBuilder,
        mut handler: F,
    ) -> Result<(), EventLoopError>
    where
        F: FnMut(Event) + 'static,
    {
        use winit::event::{Event as WinitEvent, WindowEvent as WinitWindowEvent};
        use winit::event_loop::ControlFlow;
        
        let state = Rc::new(RefCell::new(AppState::new()));
        
        self.inner.run(move |event, event_loop_window_target| {
            event_loop_window_target.set_control_flow(ControlFlow::Poll);
            
            let mut state = state.borrow_mut();
            
            match event {
                WinitEvent::Resumed => {
                    if !state.window_created {
                        let window = Arc::new(
                            window_builder
                                .build_winit(event_loop_window_target)
                                .expect("Failed to create window"),
                        );
                        
                        state.winit_window = Some(window);
                        state.window_created = true;
                        state.needs_redraw = true;
                        state.last_update = Instant::now();
                    }
                }
                WinitEvent::WindowEvent { event, .. } => {
                    match event {
                        WinitWindowEvent::CursorMoved { position, device_id, .. } => {
                            state.cursor_position = position;
                            if let Some(oxide_event) = convert_window_event(
                                WinitWindowEvent::CursorMoved { position, device_id },
                                state.cursor_position,
                                state.scale_factor
                            ) {
                                handler(oxide_event);
                            }
                        }
                        WinitWindowEvent::ScaleFactorChanged { scale_factor, inner_size_writer } => {
                            state.scale_factor = scale_factor;
                            if let Some(oxide_event) = convert_window_event(
                                WinitWindowEvent::ScaleFactorChanged { scale_factor, inner_size_writer },
                                state.cursor_position,
                                state.scale_factor
                            ) {
                                handler(oxide_event);
                            }
                        }
                        _ => {
                            if let Some(oxide_event) = convert_window_event(event, state.cursor_position, state.scale_factor) {
                                handler(oxide_event);
                            }
                        }
                    }
                }
                WinitEvent::AboutToWait => {
                    let now = Instant::now();
                    let frame_time = Duration::from_millis(16);
                    
                    if state.needs_redraw && now.duration_since(state.last_update) >= frame_time {
                        if let Some(ref window) = state.winit_window {
                            window.request_redraw();
                            state.last_update = now;
                        }
                    }
                }
                WinitEvent::UserEvent(custom) => {
                    handler(Event::Custom(Arc::new(custom.event)));
                }
                _ => {}
            }
        }).map_err(|_| EventLoopError::RunFailed)
    }

    /// Run the event loop with a window and application
    #[cfg(not(target_arch = "wasm32"))]
    pub fn run_with_window_and_app<F>(
        self,
        window_builder: crate::WindowBuilder,
        app: Application,
        handler: F,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        F: FnMut(Event) + 'static,
    {
        use winit::event::{Event as WinitEvent, WindowEvent};
        
        let app_state = Rc::new(RefCell::new(AppState::new()));
        let mut handler = handler;
        
        // Store the application in the state
        app_state.borrow_mut().app = Some(app);
        
        self.inner.run(move |event, event_loop_window_target| {
            let mut state = app_state.borrow_mut();
            
            match event {
                WinitEvent::Resumed => {
                    if !state.window_created {
                        let window = Arc::new(
                            window_builder
                                .build_winit(event_loop_window_target)
                                .expect("Failed to create window"),
                        );
                        
                        // Store the window first
                        state.winit_window = Some(window.clone());
                        
                        // Initialize GPU DrawingSystem
                        println!("=== INITIALIZING GPU DRAWING SYSTEM ===");
                        let mut drawing_system = pollster::block_on(
                            oxide_renderer::gpu::DrawingSystem::new(window.clone())
                        ).expect("Failed to create DrawingSystem");
                        
                        // Set initial scale factor
                        let scale_factor = window.scale_factor() as f32;
                        println!("Initial scale factor: {}", scale_factor);
                        drawing_system.set_scale_factor(scale_factor);
                        
                        state.drawing_system = Some(drawing_system);
                        
                        state.renderer_initialized = true;
                        state.window_created = true;
                        state.needs_redraw = true;
                        state.last_update = Instant::now();
                    }
                }
                WinitEvent::WindowEvent { event, .. } => {
                    match event {
                        WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                            println!("Scale factor changed to: {}", scale_factor);
                            state.scale_factor = scale_factor;
                            if let Some(drawing_system) = &mut state.drawing_system {
                                drawing_system.set_scale_factor(scale_factor as f32);
                            }
                        }
                        WindowEvent::CursorMoved { position, device_id, .. } => {
                            state.cursor_position = position;
                            if let Some(oxide_event) = convert_window_event(
                                WindowEvent::CursorMoved { position, device_id },
                                state.cursor_position,
                                state.scale_factor
                            ) {
                                handler(oxide_event);
                            }
                        }
                        WindowEvent::Resized(physical_size) => {
                            // Resize the drawing system when the window is resized
                            if let Some(drawing_system) = &mut state.drawing_system {
                                if let Err(e) = drawing_system.resize(physical_size.width, physical_size.height) {
                                    tracing::error!("Failed to resize drawing system: {}", e);
                                }
                            }
                            
                            handler(oxide_core::event::Event::Window(oxide_core::event::WindowEvent::Resize {
                                width: physical_size.width,
                                height: physical_size.height,
                            }));
                        }
                        WindowEvent::RedrawRequested => {
                            state.needs_redraw = false;
                            
                            // Get window size before borrowing app
                            let (physical_width, physical_height) = if let Some(window) = &state.winit_window {
                                let size = window.inner_size();
                                (size.width as f32, size.height as f32)
                            } else {
                                (800.0, 600.0) // Default fallback
                            };
                            
                            // Get scale factor
                            let scale_factor = if let Some(window) = &state.winit_window {
                                window.scale_factor() as f32
                            } else {
                                1.0
                            };
                            
                            // Use logical size for layout
                            let logical_width = physical_width / scale_factor;
                            let logical_height = physical_height / scale_factor;
                            
                            // Call the application's render method and get the render batch
                            if let Some(app) = &mut state.app {
                                if let Err(e) = app.render_simple(logical_width, logical_height) {
                                    eprintln!("Render error: {}", e);
                                } else {
                                    // Get the render batch
                                    if let Some(batch) = app.get_render_batch() {
                                        // Use GPU DrawingSystem
                                        if let Some(drawing_system) = &mut state.drawing_system {
                                            if let Err(e) = drawing_system.render(&batch) {
                                                tracing::error!("GPU render error: {}", e);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        WindowEvent::CloseRequested => {
                            handler(oxide_core::event::Event::Window(oxide_core::event::WindowEvent::Close));
                            event_loop_window_target.exit();
                        }
                        _ => {
                            if let Some(oxide_event) = convert_window_event(event, state.cursor_position, state.scale_factor) {
                                handler(oxide_event);
                            }
                        }
                    }
                }
                WinitEvent::AboutToWait => {
                    // Always request redraw to maintain continuous rendering
                    if state.renderer_initialized {
                        if let Some(window) = &state.winit_window {
                            window.request_redraw();
                        }
                        state.needs_redraw = true; // Keep requesting redraws
                    }
                }
                WinitEvent::UserEvent(custom_event) => {
                    handler(custom_event.event);
                }
                _ => {}
            }
        }).expect("Event loop failed");
        
        Ok(())
    }

    /// Run the event loop for WASM
    #[cfg(target_arch = "wasm32")]
    pub fn run_wasm<F>(self, mut handler: F)
    where
        F: FnMut(Event) + 'static,
    {
        use wasm_bindgen::prelude::*;
        use web_sys::{window, Document, Element};
        
        // Set up web event listeners
        let window = window().expect("should have a window in this context");
        let document = window.document().expect("window should have a document");
        
        // Mouse events
        let handler_clone = std::rc::Rc::new(std::cell::RefCell::new(handler));
        
        // Example: mouse click handler
        let handler_ref = handler_clone.clone();
        let closure = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
            let mouse_event = MouseEvent {
                position: glam::Vec2::new(event.client_x() as f32, event.client_y() as f32),
                button: Some(MouseButton::Left),
                modifiers: Modifiers {
                    shift: event.shift_key(),
                    control: event.ctrl_key(),
                    alt: event.alt_key(),
                    super_key: event.meta_key(),
                },
                delta: glam::Vec2::ZERO,
            };
            handler_ref.borrow_mut()(Event::MouseDown(mouse_event));
        }) as Box<dyn FnMut(_)>);
        
        document.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
            .expect("should register click handler");
        closure.forget();
        
        // Start the animation loop
        self.start_animation_loop();
    }
    
    #[cfg(target_arch = "wasm32")]
    fn start_animation_loop(&self) {
        use wasm_bindgen::prelude::*;
        use web_sys::window;
        
        let f = std::rc::Rc::new(std::cell::RefCell::new(None));
        let g = f.clone();
        
        *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
            // Animation frame logic here
            
            // Schedule next frame
            if let Some(window) = window() {
                window.request_animation_frame(f.borrow().as_ref().unwrap().as_ref().unchecked_ref())
                    .expect("should register animation frame");
            }
        }) as Box<dyn FnMut()>));
        
        if let Some(window) = window() {
            window.request_animation_frame(g.borrow().as_ref().unwrap().as_ref().unchecked_ref())
                .expect("should register animation frame");
        }
    }
}

impl Default for EventLoop {
    fn default() -> Self {
        Self::new().expect("Failed to create default event loop")
    }
}

/// Event loop proxy for sending custom events
pub struct EventLoopProxy {
    #[cfg(not(target_arch = "wasm32"))]
    inner: winit::event_loop::EventLoopProxy<CustomEvent>,
    #[cfg(target_arch = "wasm32")]
    sender: mpsc::Sender<Event>,
}

impl EventLoopProxy {
    /// Send a custom event
    pub fn send_event(&self, event: Event) -> Result<(), Box<dyn std::error::Error>> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.inner.send_event(CustomEvent { event })
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
        }
        
        #[cfg(target_arch = "wasm32")]
        {
            self.sender.send(event)
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
        }
    }
}



/// Event loop error
#[derive(Debug, thiserror::Error)]
pub enum EventLoopError {
    #[error("Failed to send event")]
    SendFailed,
    #[error("Failed to create event loop")]
    CreationFailed,
    #[error("Failed to run event loop")]
    RunFailed,
}

/// Convert winit event to OxideUI event
#[cfg(not(target_arch = "wasm32"))]
pub fn convert_window_event(
    event: winit::event::WindowEvent,
    cursor_position: winit::dpi::PhysicalPosition<f64>,
    scale_factor: f64,
) -> Option<Event> {
    use winit::event::{WindowEvent as WE, MouseButton as MB, ElementState};
    use glam::Vec2;
    
    match event {
        WE::CloseRequested => Some(Event::Window(WindowEvent::Close)),
        
        WE::Resized(size) => Some(Event::Window(WindowEvent::Resize {
            width: size.width,
            height: size.height,
        })),
        
        WE::Moved(pos) => Some(Event::Window(WindowEvent::Move {
            x: pos.x,
            y: pos.y,
        })),
        
        WE::Focused(focused) => Some(Event::Window(WindowEvent::Focus(focused))),
        
        WE::CursorMoved { position, .. } => {
            let logical_x = position.x / scale_factor;
            let logical_y = position.y / scale_factor;
            Some(Event::MouseMove(MouseEvent {
                position: Vec2::new(logical_x as f32, logical_y as f32),
                button: None,
                modifiers: Modifiers::default(),
                delta: Vec2::ZERO,
            }))
        },
        
        WE::MouseInput { state, button, .. } => {
            let button = match button {
                MB::Left => MouseButton::Left,
                MB::Right => MouseButton::Right,
                MB::Middle => MouseButton::Middle,
                MB::Back => MouseButton::Other(3),
                MB::Forward => MouseButton::Other(4),
                MB::Other(n) => MouseButton::Other(n),
            };
            
            let logical_x = cursor_position.x / scale_factor;
            let logical_y = cursor_position.y / scale_factor;
            
            match state {
                ElementState::Pressed => Some(Event::MouseDown(MouseEvent {
                    position: Vec2::new(logical_x as f32, logical_y as f32),
                    button: Some(button),
                    modifiers: Modifiers::default(),
                    delta: Vec2::ZERO,
                })),
                ElementState::Released => Some(Event::MouseUp(MouseEvent {
                    position: Vec2::new(logical_x as f32, logical_y as f32),
                    button: Some(button),
                    modifiers: Modifiers::default(),
                    delta: Vec2::ZERO,
                })),
            }
        },
        
        WE::MouseWheel { delta, .. } => {
            let delta_vec = match delta {
                winit::event::MouseScrollDelta::LineDelta(x, y) => Vec2::new(x * 20.0, y * 20.0),
                winit::event::MouseScrollDelta::PixelDelta(pos) => Vec2::new(pos.x as f32, pos.y as f32),
            };
            
            Some(Event::MouseWheel {
                delta: delta_vec,
                modifiers: Modifiers::default(),
            })
        }
        
        WE::KeyboardInput { device_id: _, event, is_synthetic: _ } => {
            if let winit::keyboard::PhysicalKey::Code(keycode) = event.physical_key {
                let key_code = convert_physical_key_code(keycode);
                
                match event.state {
                    ElementState::Pressed => Some(Event::KeyDown(KeyboardEvent {
                        key_code,
                        modifiers: Modifiers::default(),
                        is_repeat: event.repeat,
                        text: None,
                    })),
                    ElementState::Released => Some(Event::KeyUp(KeyboardEvent {
                        key_code,
                        modifiers: Modifiers::default(),
                        is_repeat: event.repeat,
                        text: None,
                    })),
                }
            } else {
                None
            }
        }
        
        WE::Ime(winit::event::Ime::Commit(text)) => {
            Some(Event::TextInput(text))
        }
        
        _ => None,
    }
}

/// Convert winit key code to OxideUI key code
#[cfg(not(target_arch = "wasm32"))]
fn convert_physical_key_code(keycode: winit::keyboard::KeyCode) -> KeyCode {
    use winit::keyboard::KeyCode as WK;
    
    match keycode {
        WK::KeyA => KeyCode::A, WK::KeyB => KeyCode::B, WK::KeyC => KeyCode::C,
        WK::KeyD => KeyCode::D, WK::KeyE => KeyCode::E, WK::KeyF => KeyCode::F,
        WK::KeyG => KeyCode::G, WK::KeyH => KeyCode::H, WK::KeyI => KeyCode::I,
        WK::KeyJ => KeyCode::J, WK::KeyK => KeyCode::K, WK::KeyL => KeyCode::L,
        WK::KeyM => KeyCode::M, WK::KeyN => KeyCode::N, WK::KeyO => KeyCode::O,
        WK::KeyP => KeyCode::P, WK::KeyQ => KeyCode::Q, WK::KeyR => KeyCode::R,
        WK::KeyS => KeyCode::S, WK::KeyT => KeyCode::T, WK::KeyU => KeyCode::U,
        WK::KeyV => KeyCode::V, WK::KeyW => KeyCode::W, WK::KeyX => KeyCode::X,
        WK::KeyY => KeyCode::Y, WK::KeyZ => KeyCode::Z,
        
        WK::Digit0 => KeyCode::Num0, WK::Digit1 => KeyCode::Num1,
        WK::Digit2 => KeyCode::Num2, WK::Digit3 => KeyCode::Num3,
        WK::Digit4 => KeyCode::Num4, WK::Digit5 => KeyCode::Num5,
        WK::Digit6 => KeyCode::Num6, WK::Digit7 => KeyCode::Num7,
        WK::Digit8 => KeyCode::Num8, WK::Digit9 => KeyCode::Num9,
        
        WK::F1 => KeyCode::F1, WK::F2 => KeyCode::F2, WK::F3 => KeyCode::F3,
        WK::F4 => KeyCode::F4, WK::F5 => KeyCode::F5, WK::F6 => KeyCode::F6,
        WK::F7 => KeyCode::F7, WK::F8 => KeyCode::F8, WK::F9 => KeyCode::F9,
        WK::F10 => KeyCode::F10, WK::F11 => KeyCode::F11, WK::F12 => KeyCode::F12,
        
        WK::Enter => KeyCode::Enter,
        WK::Escape => KeyCode::Escape,
        WK::Backspace => KeyCode::Backspace,
        WK::Tab => KeyCode::Tab,
        WK::Space => KeyCode::Space,
        
        WK::ArrowLeft => KeyCode::Left,
        WK::ArrowRight => KeyCode::Right,
        WK::ArrowUp => KeyCode::Up,
        WK::ArrowDown => KeyCode::Down,
        
        WK::ShiftLeft | WK::ShiftRight => KeyCode::Shift,
        WK::ControlLeft | WK::ControlRight => KeyCode::Control,
        WK::AltLeft | WK::AltRight => KeyCode::Alt,
        WK::SuperLeft | WK::SuperRight => KeyCode::Super,
        
        WK::Delete => KeyCode::Delete,
        WK::Insert => KeyCode::Insert,
        WK::Home => KeyCode::Home,
        WK::End => KeyCode::End,
        WK::PageUp => KeyCode::PageUp,
        WK::PageDown => KeyCode::PageDown,
        
        _ => KeyCode::A, // Default fallback
    }
}
