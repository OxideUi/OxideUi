//! Event loop management

use oxide_core::event::{Event, MouseButton, MouseEvent, KeyboardEvent, KeyCode, Modifiers, WindowEvent};
use oxide_renderer::gpu::Renderer;

/// Event loop wrapper
pub struct EventLoop {
    #[cfg(not(target_arch = "wasm32"))]
    inner: winit::event_loop::EventLoop<CustomEvent>,
    #[cfg(target_arch = "wasm32")]
    _phantom: std::marker::PhantomData<()>,
}

impl EventLoop {
    /// Create a new event loop
    pub fn new() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self {
                inner: winit::event_loop::EventLoopBuilder::with_user_event().build().expect("Failed to create event loop"),
            }
        }
        
        #[cfg(target_arch = "wasm32")]
        {
            Self {
                _phantom: std::marker::PhantomData,
            }
        }
    }

    /// Create an event loop proxy
    pub fn create_proxy(&self) -> EventLoopProxy {
        #[cfg(not(target_arch = "wasm32"))]
        {
            EventLoopProxy {
                inner: self.inner.create_proxy(),
            }
        }
        
        #[cfg(target_arch = "wasm32")]
        {
            EventLoopProxy {
                sender: mpsc::channel().0,
            }
        }
    }

    /// Run the event loop
    #[cfg(not(target_arch = "wasm32"))]
    pub fn run<F>(self, mut handler: F) -> !
    where
        F: FnMut(Event) + 'static,
    {
        use winit::event::{Event as WinitEvent, WindowEvent as WinitWindowEvent};
        use winit::event_loop::ControlFlow;
        
        self.inner.run(move |event, elwt| {
            elwt.set_control_flow(ControlFlow::Wait);
            
            match event {
                WinitEvent::WindowEvent { event, .. } => {
                    match event {
                        WinitWindowEvent::RedrawRequested => {
                            // Handle redraw
                        }
                        _ => {
                            let oxide_event = convert_window_event(event);
                            if let Some(e) = oxide_event {
                                handler(e);
                            }
                        }
                    }
                }
                WinitEvent::AboutToWait => {
                    // All events processed
                }
                WinitEvent::UserEvent(custom) => {
                    handler(custom.event);
                }
                _ => {}
            }
        }).unwrap();
        
        // This should never be reached, but we need to satisfy the ! return type
        std::process::exit(0);
    }

    /// Run the event loop with a window
    #[cfg(not(target_arch = "wasm32"))]
    pub fn run_with_window<F>(self, window_builder: crate::WindowBuilder, mut handler: F) -> !
    where
        F: FnMut(Event) + 'static,
    {
        use winit::event::{Event as WinitEvent, WindowEvent as WinitWindowEvent};
        use winit::event_loop::ControlFlow;
        
        let mut window_created = false;
        let mut winit_window: Option<winit::window::Window> = None;
        let mut needs_redraw = true; // Draw once on startup
        
        self.inner.run(move |event, elwt| {
            elwt.set_control_flow(ControlFlow::Wait);
            
            // Create window on first iteration
            if !window_created {
                if let Ok(window) = window_builder.build_winit(elwt) {
                    winit_window = Some(window);
                    window_created = true;
                    needs_redraw = true;
                }
            }
            
            match event {
                WinitEvent::WindowEvent { event, .. } => {
                    match event {
                        WinitWindowEvent::RedrawRequested => {
                            // Redraw occurred; do not schedule another automatically
                        }
                        _ => {
                            let oxide_event = convert_window_event(event);
                            if let Some(e) = oxide_event {
                                handler(e);
                                needs_redraw = true; // Input/state changed, schedule redraw
                            }
                        }
                    }
                }
                WinitEvent::AboutToWait => {
                    // Schedule redraw only if needed
                    if needs_redraw {
                        if let Some(ref window) = winit_window {
                            window.request_redraw();
                            needs_redraw = false;
                        }
                    }
                }
                WinitEvent::UserEvent(custom) => {
                    handler(custom.event);
                    needs_redraw = true;
                }
                _ => {}
            }
        }).unwrap();
        
        // This should never be reached, but we need to satisfy the ! return type
        std::process::exit(0);
    }

    /// Run the event loop with a window and application
    #[cfg(not(target_arch = "wasm32"))]
    pub fn run_with_window_and_app(self, window_builder: crate::WindowBuilder, mut app: crate::Application) -> ! {
        use winit::event::{Event as WinitEvent, WindowEvent as WinitWindowEvent};
        use winit::event_loop::ControlFlow;
        
        let mut window_created = false;
        let mut winit_window: Option<winit::window::Window> = None;
        // Do not persist a Renderer with a borrowed window across the 'static closure.
        // We'll create a fresh Renderer for each redraw to avoid lifetime issues.
        let mut needs_redraw = true; // Draw once on startup
        
        self.inner.run(move |event, elwt| {
            elwt.set_control_flow(ControlFlow::Wait);
            
            // Create window on first iteration
            if !window_created {
                if let Ok(window) = window_builder.build_winit(elwt) {
                    winit_window = Some(window);
                    window_created = true;
                    needs_redraw = true;
                }
            }
            
            match event {
                WinitEvent::WindowEvent { event, .. } => {
                    match event {
                        WinitWindowEvent::RedrawRequested => {
                            // Handle redraw - create a Renderer for this frame and render the application
                            if let Some(ref window) = winit_window {
                                let size = window.inner_size();
                                // Skip rendering if window has zero dimensions (e.g., during close)
                                if size.width > 0 && size.height > 0 {
                                    match pollster::block_on(Renderer::new(window)) {
                                        Ok(mut r) => {
                                            if let Err(e) = app.render(&mut r) {
                                                eprintln!("Render error: {}", e);
                                            }
                                        }
                                        Err(e) => {
                                            eprintln!("Failed to initialize renderer: {}", e);
                                        }
                                    }
                                }
                            }
                        }
                        _ => {
                            let oxide_event = convert_window_event(event);
                            if let Some(e) = oxide_event {
                                app.handle_event(e);
                                needs_redraw = true; // UI state may have changed
                            }
                        }
                    }
                }
                WinitEvent::AboutToWait => {
                    // Schedule redraw only if needed
                    if needs_redraw {
                        if let Some(ref window) = winit_window {
                            window.request_redraw();
                            needs_redraw = false;
                        }
                    }
                }
                WinitEvent::UserEvent(custom) => {
                    app.handle_event(custom.event);
                    needs_redraw = true;
                }
                _ => {}
            }
        }).unwrap();
        
        // This should never be reached, but we need to satisfy the ! return type
        std::process::exit(0);
    }

    /// Run the event loop (WebAssembly version)
    #[cfg(target_arch = "wasm32")]
    pub fn run<F>(self, handler: F)
    where
        F: FnMut(Event) + 'static,
    {
        // Web event loop is handled differently
        // Events are dispatched through JavaScript callbacks
    }
}

/// Event loop proxy for sending events from other threads
pub struct EventLoopProxy {
    #[cfg(not(target_arch = "wasm32"))]
    inner: winit::event_loop::EventLoopProxy<CustomEvent>,
    #[cfg(target_arch = "wasm32")]
    sender: mpsc::Sender<Event>,
}

impl EventLoopProxy {
    /// Send an event to the event loop
    pub fn send_event(&self, event: Event) -> Result<(), EventLoopError> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.inner
                .send_event(CustomEvent { event })
                .map_err(|_| EventLoopError::SendFailed)
        }
        
        #[cfg(target_arch = "wasm32")]
        {
            self.sender
                .send(event)
                .map_err(|_| EventLoopError::SendFailed)
        }
    }
}

/// Custom event wrapper
#[cfg(not(target_arch = "wasm32"))]
pub struct CustomEvent {
    pub event: Event,
}

/// Event loop error
#[derive(Debug, thiserror::Error)]
pub enum EventLoopError {
    #[error("Failed to send event")]
    SendFailed,
}

/// Convert winit event to OxideUI event
#[cfg(not(target_arch = "wasm32"))]
pub fn convert_window_event(event: winit::event::WindowEvent) -> Option<Event> {
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
        
        WE::CursorMoved { position, .. } => Some(Event::MouseMove(MouseEvent {
            position: Vec2::new(position.x as f32, position.y as f32),
            button: None,
            modifiers: Modifiers::default(),
            delta: Vec2::ZERO,
        })),
        
        WE::MouseInput { state, button, .. } => {
            let button = match button {
                MB::Left => MouseButton::Left,
                MB::Right => MouseButton::Right,
                MB::Middle => MouseButton::Middle,
                MB::Back => MouseButton::Other(3),
                MB::Forward => MouseButton::Other(4),
                MB::Other(n) => MouseButton::Other(n),
            };
            
            match state {
                ElementState::Pressed => Some(Event::MouseDown(MouseEvent {
                    position: Vec2::ZERO, // Position should be tracked separately
                    button: Some(button),
                    modifiers: Modifiers::default(),
                    delta: Vec2::ZERO,
                })),
                ElementState::Released => Some(Event::MouseUp(MouseEvent {
                    position: Vec2::ZERO,
                    button: Some(button),
                    modifiers: Modifiers::default(),
                    delta: Vec2::ZERO,
                })),
            }
        }
        
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
